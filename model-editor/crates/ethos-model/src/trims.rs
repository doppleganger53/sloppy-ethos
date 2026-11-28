//! Physical and user-defined trim records share their range, step and per-mode value encoding.
use super::{
    legacy::Cursor,
    prefix::{byte, source},
    EditFormat, Field, Result, Value,
};

fn definition(c: &mut Cursor<'_>, f: &mut Vec<Field>, path: &str) -> Result<()> {
    c.path = path.into();
    let p = c.pos;
    let step = c.byte()?;
    f.push(
        Field::new(
            "Trims",
            format!("{path}.step"),
            p..c.pos,
            Value::Enum {
                raw: step.into(),
                label: match step {
                    2 => Some("Fine".into()),
                    3 => Some("Medium".into()),
                    _ => None,
                },
            },
            step.to_string(),
            false,
        )
        .writable(EditFormat::TrimStep),
    );
    let at = c.pos;
    let range = c.byte()?;
    let setting = Field::number("Trims", format!("{path}.range"), at..c.pos, range);
    f.push(if range <= 100 {
        setting.writable(EditFormat::Byte {
            minimum: 0,
            maximum: 100,
        })
    } else {
        setting
    });
    let p = c.pos;
    let style = c.byte()?;
    let name = match style {
        0 => Some("Shared"),
        1 => Some("Per flight mode"),
        2 => Some("Custom"),
        3 => Some("Off"),
        _ => None,
    };
    f.push(super::prefix::enumeration(
        "Trims",
        format!("{path}.mode"),
        p..c.pos,
        style.into(),
        name,
    ));
    byte(c, f, "Trims", &format!("{path}.custom_step"))?;
    byte(c, f, "Trims", &format!("{path}.play_audio"))?;
    match style {
        0 | 3 => {
            let p = c.pos;
            let n = c.word()? as i16;
            f.push(Field::number(
                "Trims",
                format!("{path}.default"),
                p..c.pos,
                n,
            ));
        }
        1 => {
            let count = byte(c, f, "Trims", &format!("{path}.lines.count"))?;
            for j in 0..count {
                let p = c.pos;
                let value = c.word()? as i16;
                f.push(Field::number(
                    "Trims",
                    format!("{path}.lines[{j}].value"),
                    p..c.pos,
                    value,
                ));
            }
        }
        2 => {
            let p = c.pos;
            let n = c.word()? as i16;
            f.push(Field::number(
                "Trims",
                format!("{path}.default"),
                p..c.pos,
                n,
            ));
            let n = c.count(20)?;
            for j in 0..n {
                source(c, f, "Trims", &format!("{path}.lines[{j}].condition"))?;
                let p = c.pos;
                let value = c.word()? as i16;
                f.push(Field::number(
                    "Trims",
                    format!("{path}.lines[{j}].value"),
                    p..c.pos,
                    value,
                ));
                byte(c, f, "Trims", &format!("{path}.lines[{j}].mode"))?;
            }
        }
        _ => return Err(c.unsupported("trim mode")),
    }
    Ok(())
}

fn trigger(c: &mut Cursor<'_>, f: &mut Vec<Field>, path: &str) -> Result<()> {
    let (range, typed, display) = c.source()?;
    let label = super::logic::input_label(&typed);
    let eligible = label.is_some();
    let field = Field::new("Trims", path, range, typed, label.unwrap_or(display), false);
    f.push(if eligible {
        field.writable(EditFormat::LogicInput)
    } else {
        field
    });
    Ok(())
}

pub(crate) fn decode(c: &mut Cursor<'_>, f: &mut Vec<Field>) -> Result<()> {
    let count = byte(c, f, "Trims", "trims.count")?;
    for i in 0..count {
        definition(c, f, &format!("trims[{i}]"))?;
    }
    let extras = super::prefix::count(c, f, "Trims", "extra_trims", 5)?;
    for i in 0..extras {
        let path = format!("extra_trims[{i}]");
        definition(c, f, &path)?;
        super::prefix::label(c, f, "Trims", &format!("{path}.name"), true)?;
        trigger(c, f, &format!("{path}.trigger_up"))?;
        trigger(c, f, &format!("{path}.trigger_down"))?;
    }
    Ok(())
}
