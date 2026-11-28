use super::{legacy::Cursor, prefix, EditFormat, Error, Field, Result, Value};

const SWITCHES: &str = "Logic switches";

fn switch_name(id: u16) -> Option<String> {
    if id >= 42 {
        return None;
    }
    let letter = char::from(b'A' + (id / 3) as u8);
    let position = ["Up", "Middle", "Down"][usize::from(id % 3)];
    Some(format!("S{letter} {position}"))
}
pub(crate) fn input_label(value: &Value) -> Option<String> {
    match value {
        Value::Absent => Some("None".into()),
        Value::Source {
            category: 10,
            options: 0,
            member,
        } => switch_name(*member),
        Value::Source {
            category: 12,
            options: 0,
            member: 0,
        } => Some("FS1".into()),
        _ => None,
    }
}
pub(crate) fn encode_input(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let s = input.trim();
    if s.eq_ignore_ascii_case("None") {
        return Ok((vec![0], "None".into()));
    }
    if s.eq_ignore_ascii_case("FS1") {
        return Ok((vec![12, 0, 0, 0], "FS1".into()));
    }
    let translated = s.replace('↑', " Up").replace('↓', " Down");
    let translated = if let Some(k) = translated.strip_suffix('-') {
        format!("{k} Middle")
    } else {
        translated
    };
    for id in 0..42 {
        let name = switch_name(id).expect("valid switch position");
        if name.eq_ignore_ascii_case(translated.trim()) {
            return Ok((vec![10, 0, id as u8, 0], name));
        }
    }
    Err(Error::Invalid(format!(
        "{path}: select None, FS1 or a switch position such as SA Up or SB Middle"
    )))
}
pub(crate) fn encode_op(path: &str, input: &str, boolean: bool) -> Result<(Vec<u8>, String)> {
    let choices: &[(u8, &str)] = if boolean {
        &[(10, "AND"), (11, "OR"), (12, "XOR")]
    } else {
        &[(2, "Equal"), (3, "Greater"), (4, "Less")]
    };
    let &(raw, label) = choices
        .iter()
        .find(|(n, l)| {
            l.eq_ignore_ascii_case(input.trim()) || input.trim().parse::<u8>().ok() == Some(*n)
        })
        .ok_or_else(|| {
            Error::Invalid(format!(
                "{path}: choose {}",
                choices.iter().map(|v| v.1).collect::<Vec<_>>().join(", ")
            ))
        })?;
    Ok((vec![raw], label.into()))
}

pub(crate) fn seconds(path: &str, input: &str, min: i32, max: i32) -> Result<(i32, String)> {
    let n = input
        .trim()
        .trim_end_matches('s')
        .trim()
        .parse::<f64>()
        .map_err(|_| Error::Invalid(format!("{path}: expected seconds")))?;
    let deci = n * 10.;
    let raw = deci.round();
    if !deci.is_finite()
        || (deci - raw).abs() > 1e-8
        || !(f64::from(min)..=f64::from(max)).contains(&raw)
    {
        return Err(Error::Invalid(format!(
            "{path}: enter a tenth of a second between {} and {}",
            f64::from(min) / 10.,
            f64::from(max) / 10.
        )));
    }
    let raw = raw as i32;
    Ok((raw, format!("{} s", f64::from(raw) / 10.)))
}

pub(crate) fn encode_inputs(path: &str, data: &str) -> Result<(Vec<u8>, String)> {
    let requested: Vec<Option<String>> =
        serde_json::from_str(data).map_err(|e| Error::Invalid(format!("{path}: {e}")))?;
    if !(2..=9).contains(&requested.len()) {
        return Err(Error::Invalid(format!(
            "{path}: Boolean switches retain 2 through 9 inputs"
        )));
    }
    let mut bytes = (requested.len() as u16).to_le_bytes().to_vec();
    let mut values = Vec::new();
    for (i, v) in requested.iter().enumerate() {
        let (b, s) = encode_input(&format!("{path}[{i}]"), v.as_deref().unwrap_or("None"))?;
        bytes.extend(b);
        values.push(if s == "None" { None } else { Some(s) });
    }
    Ok((
        bytes,
        serde_json::to_string(&values).expect("source labels serialize"),
    ))
}

pub(crate) fn switches(c: &mut Cursor<'_>, f: &mut Vec<Field>) -> Result<()> {
    for i in 0..prefix::count(c, f, SWITCHES, "logic_switches", 100)? {
        let root = format!("logic_switches[{i}]");
        c.path = root.clone();
        let at = c.pos;
        let key = c.byte()?;
        let label = match key {
            1 => "Around",
            2 => "Equal",
            3 => "Greater",
            4 => "Less",
            9 => "Range",
            10 => "AND",
            11 => "OR",
            12 => "XOR",
            13 => "Timer",
            _ => return Err(c.unsupported(&format!("logic switch key {key}"))),
        };
        let kind = prefix::enumeration(
            SWITCHES,
            format!("{root}.key"),
            at..c.pos,
            key.into(),
            Some(label),
        );
        f.push(match key {
            2..=4 => kind.writable(EditFormat::LogicComparison),
            10..=12 => kind.writable(EditFormat::LogicCombinator),
            _ => kind,
        });
        prefix::expected(c, 1)?;
        for name in ["delay_on", "delay_off"] {
            let p = c.pos;
            let n = c.word()?;
            let field = Field::new(
                SWITCHES,
                format!("{root}.{name}"),
                p..c.pos,
                Value::FixedPoint {
                    raw: n.into(),
                    scale: 10,
                },
                format!("{} s", f64::from(n) / 10.),
                false,
            );
            f.push(if n <= 600 {
                field.writable(EditFormat::LogicDelay)
            } else {
                field
            });
        }
        prefix::fixed(c, f, SWITCHES, &format!("{root}.other_timing"), 4)?;
        let table_start = c.pos;
        let n = prefix::count(
            c,
            f,
            SWITCHES,
            &format!("{root}.values"),
            if matches!(key, 10..=12) { 9 } else { 64 },
        )?;
        if matches!(key, 10..=12) && n < 2 {
            return Err(c.unsupported("Boolean switch requires at least two inputs"));
        }
        let mut labels = Vec::new();
        let mut all_known = true;
        let mut analog = false;
        for j in 0..n {
            let path = format!("{root}.values[{j}]");
            let (r, v, s) = c.source()?;
            if j == 0 {
                analog = matches!(
                    v,
                    Value::Source {
                        category: 8,
                        options: 0,
                        ..
                    }
                );
            }
            let plain = Field::new(SWITCHES, path.clone(), r.clone(), v.clone(), s, false);
            let cell = match key {
                1..=4 if j == 1 && analog && matches!(v, Value::Integer(-100..=100)) => plain
                    .writable(EditFormat::Operand {
                        minimum: -100,
                        maximum: 100,
                    }),
                10..=12 => {
                    if let Some(label) = input_label(&v) {
                        Field::new(SWITCHES, path, r, v, label, false)
                            .writable(EditFormat::LogicInput)
                    } else {
                        plain
                    }
                }
                13 if j < 2 => {
                    if let Value::Integer(raw) = v {
                        let d = Field::new(
                            SWITCHES,
                            path,
                            r,
                            Value::FixedPoint { raw, scale: 10 },
                            format!("{} s", raw as f64 / 10.),
                            false,
                        );
                        if (1..=1750).contains(&raw) {
                            d.writable(EditFormat::TimerInterval)
                        } else {
                            d
                        }
                    } else {
                        plain
                    }
                }
                _ => plain,
            };
            if matches!(key, 10..=12) {
                let v = input_label(&cell.typed);
                all_known &= v.is_some();
                labels.push(match v.as_deref() {
                    Some("None") => serde_json::Value::Null,
                    Some(value) => serde_json::json!(value),
                    None => serde_json::to_value(&cell.typed).expect("input serializes"),
                });
            }
            f.push(cell);
        }
        if matches!(key, 10..=12) {
            let display = serde_json::to_string(&labels).expect("input table serializes");
            let table = Field::new(
                SWITCHES,
                format!("{root}.values"),
                table_start..c.pos,
                Value::Array(labels),
                display,
                false,
            )
            .summary();
            f.push(if all_known {
                table.writable(EditFormat::LogicInputs)
            } else {
                table
            });
        }
        prefix::source(c, f, SWITCHES, &format!("{root}.active_condition"))?;
        prefix::label(c, f, SWITCHES, &format!("{root}.name"), true)?;
        prefix::on_off(c, f, SWITCHES, &format!("{root}.negative"), true)?;
        prefix::fixed(c, f, SWITCHES, &format!("{root}.additional_flags"), 2)?;
        prefix::label(c, f, SWITCHES, &format!("{root}.comment"), false)?;
    }
    Ok(())
}
