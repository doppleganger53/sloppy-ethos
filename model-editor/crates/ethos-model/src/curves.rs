//! Point counts and grid mode are stored before the smoothing flag. The ordered
//! point data follows it. Each coordinate uses the firmware's packed operand encoding.
use super::{
    edit::packed_integer, legacy::Cursor, prefix, EditFormat, Error, Field, Result, Value,
};
use serde_json::{json, Value as Json};

const SECTION: &str = "Curves";
pub(crate) fn reference(c: &mut Cursor<'_>, fields: &mut Vec<Field>, path: &str) -> Result<()> {
    let start = c.pos;
    let selector = c.byte()?;
    let mut editable = true;
    let (value, display) = match selector {
        0 => (Value::Absent, "None".into()),
        1 => {
            let (range, value, display) = c.source()?;
            let field = Field::new(
                "Mixes",
                format!("{path}.exponent"),
                range,
                value.clone(),
                display.clone(),
                false,
            );
            editable = matches!(value, Value::Integer(-100..=100));
            fields.push(if editable {
                field.writable(EditFormat::Operand {
                    minimum: -100,
                    maximum: 100,
                })
            } else {
                field
            });
            (
                Value::Enum {
                    raw: 1,
                    label: Some("Expo".into()),
                },
                format!("Expo {display}"),
            )
        }
        2..=51 => {
            c.curve_reference(start, selector);
            let slot = usize::from(selector - 2);
            (
                Value::Reference {
                    collection: "curves".into(),
                    slot,
                },
                format!("Curve slot {slot}"),
            )
        }
        _ => return Err(c.unsupported(&format!("curve selector {selector}"))),
    };
    let field = Field::new("Mixes", path, start..c.pos, value, display, false).projected();
    fields.push(if editable {
        field.writable(EditFormat::CurveSelector)
    } else {
        field
    });
    Ok(())
}

pub(crate) fn encode_selector(total: usize, input: &str, path: &str) -> Result<(Vec<u8>, String)> {
    let tokens: Vec<_> = input.split_whitespace().collect();
    if matches!(tokens.as_slice(),[name] if name.eq_ignore_ascii_case("None")) {
        return Ok((vec![0], "None".into()));
    }
    if let [name, n] = tokens.as_slice() {
        if name.eq_ignore_ascii_case("Expo") {
            let value = n
                .parse::<i32>()
                .ok()
                .filter(|v| (-100..=100).contains(v))
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "{path}: inline Expo expects a whole value from -100 to 100"
                    ))
                })?;
            let mut bytes = vec![1];
            bytes.extend(packed_integer(value));
            return Ok((bytes, format!("Expo {value}")));
        }
    }
    let id = match tokens.as_slice() {
        [kind, slot] if kind.eq_ignore_ascii_case("Curve") => slot.parse::<usize>().ok(),
        [kind, unit, slot]
            if kind.eq_ignore_ascii_case("Curve") && unit.eq_ignore_ascii_case("slot") =>
        {
            slot.parse::<usize>().ok()
        }
        _ => {
            return Err(Error::Invalid(format!(
                "{path}: choose None, Expo N, or Curve slot N"
            )))
        }
    };
    let slot = id.filter(|&n| n < total && n < 50).ok_or_else(|| {
        Error::Invalid(format!("{path}: curve slot does not exist in this model"))
    })?;
    Ok((vec![slot as u8 + 2], format!("Curve slot {slot}")))
}

const FUNCTIONS: [&str; 7] = ["None", "x>0", "x<0", "|x|", "f>0", "f<0", "|f|"];

pub(crate) fn function(input: &str, path: &str) -> Result<(Vec<u8>, String)> {
    let choice = FUNCTIONS
        .iter()
        .position(|v| v.eq_ignore_ascii_case(input.trim()))
        .or_else(|| input.parse::<usize>().ok().filter(|v| *v < FUNCTIONS.len()));
    let i =
        choice.ok_or_else(|| Error::Invalid(format!("{path}: choose {}", FUNCTIONS.join(", "))))?;
    Ok((vec![i as u8], FUNCTIONS[i].into()))
}

fn coordinate(
    c: &mut Cursor<'_>,
    fields: &mut Vec<Field>,
    path: &str,
    scale: i64,
) -> Result<(Json, Option<i64>)> {
    let (range, v, display) = c.source()?;
    if let Value::Integer(raw) = v {
        let number = if raw % scale == 0 {
            json!(raw / scale)
        } else {
            json!(raw as f64 / scale as f64)
        };
        fields.push(Field::new(
            SECTION,
            path,
            range,
            Value::FixedPoint {
                raw,
                scale: scale as u32,
            },
            format!("{number}%"),
            false,
        ));
        Ok((number, Some(raw)))
    } else {
        let value = serde_json::to_value(&v).expect("field values serialize");
        fields.push(Field::new(SECTION, path, range, v, display, false));
        Ok((value, None))
    }
}
fn integer_operand(
    c: &mut Cursor<'_>,
    f: &mut Vec<Field>,
    path: &str,
    min: i32,
    max: i32,
) -> Result<()> {
    let (range, v, display) = c.source()?;
    let writable =
        matches!(&v,Value::Integer(raw) if (i64::from(min)..=i64::from(max)).contains(raw));
    let field = Field::new(SECTION, path, range, v, display, false);
    f.push(if writable {
        field.writable(EditFormat::Operand {
            minimum: min,
            maximum: max,
        })
    } else {
        field
    });
    Ok(())
}

pub(crate) fn decode(
    c: &mut Cursor<'_>,
    f: &mut Vec<Field>,
    ranges: &mut Vec<std::ops::Range<usize>>,
) -> Result<()> {
    for i in 0..prefix::count(c, f, SECTION, "curves", 50)? {
        let start = c.pos;
        let root = format!("curves[{i}]");
        c.path = root.clone();
        prefix::expected(c, 0)?;
        prefix::label(c, f, SECTION, &format!("{root}.name"), true)?;
        let at = c.pos;
        let kind = c.byte()?;
        let label = match kind {
            0 => "None",
            1 => "Expo",
            2 => "Function",
            3 => "Custom",
            _ => return Err(c.unsupported(&format!("curve type {kind}"))),
        };
        f.push(prefix::enumeration(
            SECTION,
            format!("{root}.type"),
            at..c.pos,
            kind.into(),
            Some(label),
        ));
        if kind == 0 {
            ranges.push(start..c.pos);
            continue;
        }
        if kind == 1 {
            prefix::percent_source(c, f, SECTION, &format!("{root}.offset"), -1255, 1255)?;
        } else {
            coordinate(c, f, &format!("{root}.offset"), 10)?;
        }
        match kind {
            1 => {
                integer_operand(c, f, &format!("{root}.exponent"), -125, 125)?;
                integer_operand(c, f, &format!("{root}.weight"), -125, 125)?;
            }
            2 => {
                let p = c.pos;
                let n = c.byte()?;
                let label = FUNCTIONS.get(usize::from(n)).copied();
                let field = prefix::enumeration(
                    SECTION,
                    format!("{root}.function"),
                    p..c.pos,
                    n.into(),
                    label,
                );
                f.push(if label.is_some() {
                    field.writable(EditFormat::CurveFunction)
                } else {
                    field
                });
            }
            3 => read_points(c, f, &root)?,
            _ => unreachable!(),
        }
        ranges.push(start..c.pos);
    }
    Ok(())
}

fn read_points(c: &mut Cursor<'_>, f: &mut Vec<Field>, root: &str) -> Result<()> {
    let points = format!("{root}.points");
    c.path = points.clone();
    let n = prefix::byte(c, f, SECTION, &format!("{points}.count"))?;
    if !(2..=21).contains(&n) {
        return Err(c.unsupported(&format!("point count {n} outside 2..21")));
    }
    let p = c.pos;
    let easy = c.byte()?;
    if easy > 1 {
        return Err(c.unsupported("curve grid mode"));
    }
    f.push(prefix::enumeration(
        SECTION,
        format!("{root}.easy_mode"),
        p..c.pos,
        easy.into(),
        Some(if easy == 1 { "On" } else { "Off" }),
    ));
    prefix::on_off(c, f, SECTION, &format!("{root}.smooth"), true)?;
    let start = c.pos;
    let mut points_json = Vec::with_capacity(n.into());
    let mut writable = true;
    let mut last_x = -100;
    for j in 0..n {
        let path = format!("{points}[{j}]");
        let x = if easy == 0 {
            let (x, stored) = coordinate(c, f, &format!("{path}.x"), 1)?;
            match stored {
                Some(x)
                    if (-100..=100).contains(&x)
                        && x >= last_x
                        && (j != 0 || x == -100)
                        && (j + 1 != n || x == 100) =>
                {
                    last_x = x
                }
                _ => writable = false,
            }
            Some(x)
        } else {
            None
        };
        let (y, stored) = coordinate(c, f, &format!("{path}.y"), 10)?;
        writable &= stored.is_some_and(|y| (-1000..=1000).contains(&y));
        points_json.push(if let Some(x) = x {
            json!({"x":x,"y":y})
        } else {
            y
        });
    }
    let display = serde_json::to_string(&points_json).expect("point values serialize");
    let field = Field::new(
        SECTION,
        points,
        start..c.pos,
        Value::Array(points_json),
        display,
        false,
    )
    .summary();
    f.push(if writable {
        field.writable(EditFormat::CurvePoints)
    } else {
        field
    });
    Ok(())
}

fn numeric(value: &Json, scale: i32, path: &str) -> Result<i32> {
    let n = value
        .as_f64()
        .ok_or_else(|| Error::Invalid(format!("{path}: expected a numeric percentage")))?;
    let adjusted = n * f64::from(scale);
    let round = adjusted.round();
    if !n.is_finite() || !(-100.0..=100.0).contains(&n) || (round - adjusted).abs() > 1e-8 {
        let precision = if scale == 1 { "whole" } else { "tenth" };
        return Err(Error::Invalid(format!(
            "{path}: expected a {precision}-percent value between -100 and 100"
        )));
    }
    Ok(round as i32)
}
fn display_percent(raw: i32) -> Json {
    if raw % 10 == 0 {
        json!(raw / 10)
    } else {
        json!(f64::from(raw) / 10.0)
    }
}

/// Returns count, grid mode and packed coordinates; smoothing stays in its own field.
pub(crate) fn encode_points(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let values: Vec<Json> =
        serde_json::from_str(input).map_err(|e| Error::Invalid(format!("{path}: {e}")))?;
    if !(2..=21).contains(&values.len()) {
        return Err(Error::Invalid(format!(
            "{path}: supply between 2 and 21 points"
        )));
    }
    let easy = values[0].is_number();
    let end = values.len() - 1;
    let mut bytes = vec![values.len() as u8, u8::from(easy)];
    let mut canonical = Vec::new();
    let mut last = -100;
    for (i, v) in values.iter().enumerate() {
        let row = format!("{path}[{i}]");
        if easy {
            let y = numeric(v, 10, &row)?;
            bytes.extend(packed_integer(y));
            canonical.push(display_percent(y));
        } else {
            let obj = v
                .as_object()
                .filter(|o| o.len() == 2 && o.contains_key("x") && o.contains_key("y"))
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "{row}: each custom point requires only numeric x and y"
                    ))
                })?;
            let x = numeric(&obj["x"], 1, &format!("{row}.x"))?;
            let y = numeric(&obj["y"], 10, &format!("{row}.y"))?;
            if (i == 0 && x != -100) || (i == end && x != 100) || x < last {
                return Err(Error::Invalid(format!(
                    "{row}.x: X starts at -100, ends at 100 and cannot decrease"
                )));
            }
            last = x;
            bytes.extend(packed_integer(x));
            bytes.extend(packed_integer(y));
            canonical.push(json!({"x":x,"y":display_percent(y)}));
        }
    }
    Ok((
        bytes,
        serde_json::to_string(&canonical).expect("point values serialize"),
    ))
}
