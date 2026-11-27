use super::{
    edit::packed_integer,
    legacy::Cursor,
    logic::{encode_input, input_label},
    prefix, EditFormat, Error, Field, Model, Result, Value,
};
use serde::Deserialize;
use serde_json::{json, Value as Json};

const SECTION: &str = "Variables";
const FUNCTIONS: [&str; 9] = [
    "Assign",
    "Add",
    "Subtract",
    "Multiply",
    "Divide",
    "Percent",
    "Min",
    "Max",
    "Repurpose",
];
const LIMIT: i32 = 2000; // Independently retained by both legacy firmware and a converted JSON read-back.

fn number(c: &mut Cursor<'_>, f: &mut Vec<Field>, path: String) -> Result<(Json, Option<i64>)> {
    let (r, v, s) = c.source()?;
    let stored = if let Value::Integer(n) = v {
        Some(n)
    } else {
        None
    };
    let json = stored
        .map(|n| json!(n))
        .unwrap_or_else(|| serde_json::to_value(&v).expect("operand serializes"));
    let cell = Field::new(SECTION, path, r, v, s, false);
    let eligible = stored.is_some_and(|n| (-i64::from(LIMIT)..=i64::from(LIMIT)).contains(&n));
    f.push(if eligible {
        cell.writable(EditFormat::Operand {
            minimum: -LIMIT,
            maximum: LIMIT,
        })
    } else {
        cell
    });
    Ok((json, if eligible { stored } else { None }))
}
fn condition(c: &mut Cursor<'_>, f: &mut Vec<Field>, path: String) -> Result<(Json, bool)> {
    let (r, v, s) = c.source()?;
    let label = input_label(&v);
    let eligible = label.is_some();
    let json = match label.as_deref() {
        Some("None") => Json::Null,
        Some(s) => json!(s),
        None => serde_json::to_value(&v).expect("source serializes"),
    };
    let cell = Field::new(SECTION, path, r, v, label.unwrap_or(s), false);
    f.push(if eligible {
        cell.writable(EditFormat::LogicInput)
    } else {
        cell
    });
    Ok((json, eligible))
}
fn aggregate(
    f: &mut Vec<Field>,
    path: String,
    range: std::ops::Range<usize>,
    rows: Vec<Json>,
    writable: bool,
    encoding: EditFormat,
) {
    let display = serde_json::to_string(&rows).expect("condition table serializes");
    let field = Field::new(SECTION, path, range, Value::Array(rows), display, false).summary();
    f.push(if writable {
        field.writable(encoding)
    } else {
        field
    });
}
pub(crate) fn decode(c: &mut Cursor<'_>, f: &mut Vec<Field>) -> Result<()> {
    for i in 0..prefix::count(c, f, SECTION, "variables", 64)? {
        let root = format!("variables[{i}]");
        c.path = root.clone();
        prefix::expected(c, 0)?;
        prefix::label(c, f, SECTION, &format!("{root}.name"), true)?;
        prefix::label(c, f, SECTION, &format!("{root}.comment"), false)?;
        number(c, f, format!("{root}.default"))?;
        let rates = format!("{root}.rates");
        let start = c.pos;
        let n = prefix::count(c, f, SECTION, &rates, 9)?;
        let mut rows = Vec::new();
        let mut writable = true;
        for j in 0..n {
            c.path = format!("{rates}[{j}]");
            let (test, known) = condition(c, f, format!("{rates}[{j}].condition"))?;
            let (v, stored) = number(c, f, format!("{rates}[{j}].value"))?;
            writable &= known && stored.is_some();
            rows.push(json!({"condition":test,"value":v}));
        }
        aggregate(
            f,
            rates,
            start..c.pos,
            rows,
            writable,
            EditFormat::VariableRates,
        );
        number(c, f, format!("{root}.minimum"))?;
        number(c, f, format!("{root}.maximum"))?;
        let actions = format!("{root}.actions");
        let start = c.pos;
        let n = prefix::count(c, f, SECTION, &actions, 10)?;
        let mut rows = Vec::new();
        let mut writable = true;
        for j in 0..n {
            let row = format!("{actions}[{j}]");
            c.path = row.clone();
            let (test, known) = condition(c, f, format!("{row}.condition"))?;
            let pos = c.pos;
            let op = c.byte()?;
            let label = FUNCTIONS
                .get(usize::from(op))
                .ok_or_else(|| c.unsupported(&format!("variable action {op}")))?;
            let kind = prefix::enumeration(
                SECTION,
                format!("{row}.function"),
                pos..c.pos,
                op.into(),
                Some(label),
            );
            f.push(if op < 8 {
                kind.writable(EditFormat::VariableFunction)
            } else {
                kind
            });
            if op == 8 {
                let (r, v, display) = c.source()?;
                let actual_trims = f
                    .iter()
                    .find(|entry| entry.path == "trims.count")
                    .and_then(|entry| {
                        if let Value::Integer(count) = entry.typed {
                            Some(count)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                let target = if let Value::Source {
                    category: 14,
                    options: 0,
                    member,
                } = v
                {
                    (i64::from(member) < actual_trims).then(|| format!("Trim {member}"))
                } else {
                    None
                };
                let field = Field::new(
                    SECTION,
                    format!("{row}.source"),
                    r,
                    v.clone(),
                    target.clone().unwrap_or(display),
                    false,
                );
                f.push(if target.is_some() {
                    field.writable(EditFormat::TrimTarget)
                } else {
                    field
                });
                let (r, v, display) = c.source()?;
                let raw = if let Value::Integer(n) = v {
                    Some(n)
                } else {
                    None
                };
                let valid = raw.is_some_and(|n| (-1000..=1000).contains(&n));
                let step = raw
                    .map(|n| json!(n))
                    .unwrap_or_else(|| serde_json::to_value(&v).expect("operand serializes"));
                let field = Field::new(SECTION, format!("{row}.step"), r, v, display, false);
                f.push(if valid {
                    field.writable(EditFormat::Operand {
                        minimum: -1000,
                        maximum: 1000,
                    })
                } else {
                    field
                });
                writable &= known && target.is_some() && valid;
                let src = target
                    .map(|s| json!(s))
                    .unwrap_or_else(|| json!({"unverified_trim":true}));
                rows.push(json!({"condition":test,"function":label,"source":src,"step":step}));
            } else {
                let (v, stored) = number(c, f, format!("{row}.source"))?;
                writable &= known && stored.is_some();
                rows.push(json!({"condition":test,"function":label,"source":v}));
            }
        }
        aggregate(
            f,
            actions,
            start..c.pos,
            rows,
            writable,
            EditFormat::VariableActions,
        );
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rate {
    condition: Option<String>,
    value: i32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    condition: Option<String>,
    function: String,
    source: Json,
    step: Option<i32>,
}

pub(crate) fn operator(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let id = FUNCTIONS[..8]
        .iter()
        .position(|name| name.eq_ignore_ascii_case(input.trim()))
        .or_else(|| input.trim().parse::<usize>().ok().filter(|i| *i < 8))
        .ok_or_else(|| Error::Invalid(format!("{path}: choose {}", FUNCTIONS[..8].join(", "))))?;
    Ok((vec![id as u8], FUNCTIONS[id].into()))
}
fn integer(path: &str, value: i32) -> Result<Vec<u8>> {
    if !(-LIMIT..=LIMIT).contains(&value) {
        return Err(Error::Invalid(format!(
            "{path}: number exceeds the verified range -{LIMIT} through {LIMIT}"
        )));
    }
    Ok(packed_integer(value))
}
fn source_bytes(path: &str, s: Option<&str>) -> Result<(Vec<u8>, Json)> {
    let (b, s) = encode_input(path, s.unwrap_or("None"))?;
    Ok((b, if s == "None" { Json::Null } else { json!(s) }))
}
pub(crate) fn encode_rates(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let values: Vec<Rate> =
        serde_json::from_str(input).map_err(|e| Error::Invalid(format!("{path}: {e}")))?;
    if values.len() > 9 {
        return Err(Error::Invalid(format!(
            "{path}: at most nine conditional values"
        )));
    }
    let mut b = (values.len() as u16).to_le_bytes().to_vec();
    let mut rows = Vec::new();
    for (i, row) in values.into_iter().enumerate() {
        let p = format!("{path}[{i}]");
        let (raw, s) = source_bytes(&format!("{p}.condition"), row.condition.as_deref())?;
        b.extend(raw);
        b.extend(integer(&format!("{p}.value"), row.value)?);
        rows.push(json!({"condition":s,"value":row.value}));
    }
    Ok((
        b,
        serde_json::to_string(&rows).expect("rate values serialize"),
    ))
}
pub(crate) fn trim_target(model: &Model, path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let words: Vec<_> = input.split_whitespace().collect();
    let id = if let [label, id] = words.as_slice() {
        if label.eq_ignore_ascii_case("Trim") {
            id.parse::<u16>().ok()
        } else {
            None
        }
    } else {
        None
    };
    let limit = model
        .field("trims.count")
        .and_then(|f| {
            if let Value::Integer(n) = f.typed {
                Some(n)
            } else {
                None
            }
        })
        .unwrap_or(0);
    let id = id.filter(|id| i64::from(*id) < limit).ok_or_else(|| {
        Error::Invalid(format!(
            "{path}: choose Trim N for an existing zero-based trim slot"
        ))
    })?;
    let data = id.to_le_bytes();
    Ok((vec![14, 0, data[0], data[1]], format!("Trim {id}")))
}
pub(crate) fn encode_actions(model: &Model, path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let values: Vec<Action> =
        serde_json::from_str(input).map_err(|e| Error::Invalid(format!("{path}: {e}")))?;
    if values.len() > 10 {
        return Err(Error::Invalid(format!("{path}: at most ten actions")));
    }
    let mut b = (values.len() as u16).to_le_bytes().to_vec();
    let mut rows = Vec::new();
    for (i, row) in values.into_iter().enumerate() {
        let p = format!("{path}[{i}]");
        let (raw, condition) = source_bytes(&format!("{p}.condition"), row.condition.as_deref())?;
        b.extend(raw);
        if row.function.eq_ignore_ascii_case("Repurpose") || row.function == "8" {
            let target = row
                .source
                .as_str()
                .ok_or_else(|| Error::Invalid(format!("{p}.source: choose Trim N")))?;
            let (raw, label) = trim_target(model, &format!("{p}.source"), target)?;
            let step = row
                .step
                .filter(|n| (-1000..=1000).contains(n))
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "{p}.step: expected -1000 through 1000 in stored tenths"
                    ))
                })?;
            b.push(8);
            b.extend(raw);
            b.extend(packed_integer(step));
            rows.push(
                json!({"condition":condition,"function":"Repurpose","source":label,"step":step}),
            );
        } else {
            if row.step.is_some() {
                return Err(Error::Invalid(format!(
                    "{p}.step: only Repurpose encodes a step"
                )));
            }
            let n = row
                .source
                .as_i64()
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| Error::Invalid(format!("{p}.source: expected a numeric operand")))?;
            let (tag, label) = operator(&format!("{p}.function"), &row.function)?;
            b.extend(tag);
            b.extend(integer(&format!("{p}.source"), n)?);
            rows.push(json!({"condition":condition,"function":label,"source":n}));
        }
    }
    Ok((b, serde_json::to_string(&rows).expect("actions serialize")))
}
