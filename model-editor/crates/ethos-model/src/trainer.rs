//! The legacy trainer block stores the role and link before the role-specific model settings.
use super::{legacy::Cursor, prefix, EditFormat, Error, Field, Result, Value};

const SECTION: &str = "Trainer";
const LINKS: [&str; 4] = ["Cable", "Bluetooth", "External SBUS", "External CPPM"];

pub(crate) fn encode_link(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let link = LINKS
        .iter()
        .position(|s| s.eq_ignore_ascii_case(input.trim()))
        .or_else(|| {
            input
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|n| *n < LINKS.len())
        })
        .ok_or_else(|| Error::Invalid(format!("{path}: choose {}", LINKS.join(", "))))?;
    Ok((vec![link as u8], LINKS[link].into()))
}

pub(crate) fn encode_mode(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let labels = ["Off", "Add", "Replace"];
    let i = labels
        .iter()
        .position(|s| s.eq_ignore_ascii_case(input.trim()))
        .or_else(|| input.trim().parse::<usize>().ok().filter(|i| *i < 3))
        .ok_or_else(|| Error::Invalid(format!("{path}: choose Off, Add or Replace")))?;
    Ok((vec![i as u8], labels[i].into()))
}

const ANALOGS: [&str; 11] = [
    "Rudder",
    "Elevator",
    "Throttle",
    "Aileron",
    "Pot1",
    "Pot2",
    "Ext1",
    "Ext2",
    "Pot3",
    "Slider left",
    "Slider right",
];
fn activation_name(v: &Value) -> Option<String> {
    match v {
        Value::Source {
            category: 1,
            options: 0,
            member: 0,
        } => Some("Always on".into()),
        Value::Source {
            category: 12,
            options: 0,
            member,
        } if *member < 6 => Some(format!("FS{}", member + 1)),
        _ => super::logic::input_label(v),
    }
}
pub(crate) fn encode_activation(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let s = input.trim();
    if s.eq_ignore_ascii_case("Always on") {
        return Ok((vec![1, 0, 0, 0], "Always on".into()));
    }
    if let Some(i) = s
        .to_ascii_uppercase()
        .strip_prefix("FS")
        .and_then(|n| n.parse::<u8>().ok())
        .filter(|n| (1..=6).contains(n))
    {
        return Ok((vec![12, 0, i - 1, 0], format!("FS{i}")));
    }
    super::logic::encode_input(path, s).map_err(|_| {
        Error::Invalid(format!(
            "{path}: choose None, Always on, FS1 through FS6 or a switch position"
        ))
    })
}
pub(crate) fn encode_destination(path: &str, input: &str) -> Result<(Vec<u8>, String)> {
    let s = input.trim();
    if s.eq_ignore_ascii_case("None") {
        return Ok((vec![0], "None".into()));
    }
    let id = ANALOGS
        .iter()
        .position(|name| name.eq_ignore_ascii_case(s))
        .or_else(|| {
            s.to_ascii_lowercase()
                .strip_prefix("analog ")
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|n| *n < ANALOGS.len())
        })
        .ok_or_else(|| {
            Error::Invalid(format!(
                "{path}: choose None or a verified analog ({})",
                ANALOGS.join(", ")
            ))
        })?;
    Ok((vec![8, 0, id as u8, 0], ANALOGS[id].into()))
}
fn trainer_source(
    c: &mut Cursor<'_>,
    f: &mut Vec<Field>,
    path: &str,
    activation: bool,
) -> Result<()> {
    let (r, v, display) = c.source()?;
    let label = if activation {
        activation_name(&v)
    } else {
        match v {
            Value::Absent => Some("None".into()),
            Value::Source {
                category: 8,
                options: 0,
                member,
            } => ANALOGS.get(usize::from(member)).map(|s| s.to_string()),
            _ => None,
        }
    };
    let supported = label.is_some();
    let cell = Field::new(SECTION, path, r, v, label.unwrap_or(display), false);
    let format = if activation {
        EditFormat::TrainerActivation
    } else {
        EditFormat::TrainerDestination
    };
    f.push(if supported {
        cell.writable(format)
    } else {
        cell
    });
    Ok(())
}

pub(crate) fn decode(c: &mut Cursor<'_>, f: &mut Vec<Field>) -> Result<()> {
    c.path = "trainer.mode".into();
    let p = c.pos;
    let role = c.byte()?;
    let label = match role {
        0 => "Off",
        1 => "Master",
        2 => "Slave",
        _ => return Err(c.unsupported(&format!("trainer role {role}"))),
    };
    f.push(prefix::enumeration(
        SECTION,
        "trainer.mode",
        p..c.pos,
        role.into(),
        Some(label),
    ));
    c.path = "trainer.link".into();
    let p = c.pos;
    let link = c.byte()?;
    let name = LINKS
        .get(usize::from(link))
        .ok_or_else(|| c.unsupported(&format!("trainer link {link}")))?;
    let field = prefix::enumeration(SECTION, "trainer.link", p..c.pos, link.into(), Some(name));
    f.push(if role != 0 {
        field.writable(EditFormat::TrainerLink)
    } else {
        field
    });
    match role {
        0 => {}
        1 => {
            trainer_source(c, f, "trainer.condition", true)?;
            for i in 0..16 {
                let row = format!("trainer.channels[{i}]");
                c.path = row.clone();
                let p = c.pos;
                let mode = c.byte()?;
                let kind = match mode {
                    0 => Some("Off"),
                    1 => Some("Add"),
                    2 => Some("Replace"),
                    _ => None,
                };
                let mode_field = prefix::enumeration(
                    SECTION,
                    format!("{row}.mode"),
                    p..c.pos,
                    mode.into(),
                    kind,
                );
                f.push(if kind.is_some() {
                    mode_field.writable(EditFormat::TrainerControlMode)
                } else {
                    mode_field
                });
                let p = c.pos;
                let weight = c.byte()? as i8;
                let weight_field =
                    Field::number(SECTION, format!("{row}.weight"), p..c.pos, weight);
                f.push(if (-125..=125).contains(&weight) {
                    weight_field.writable(EditFormat::Byte {
                        minimum: -125,
                        maximum: 125,
                    })
                } else {
                    weight_field
                });
                trainer_source(c, f, &format!("{row}.destination"), false)?;
                trainer_source(c, f, &format!("{row}.active_condition"), true)?;
            }
        }
        2 => {
            c.path = "trainer.channel_range".into();
            let p = c.pos;
            let origin = prefix::byte(c, f, SECTION, "trainer.channels_start")?;
            let n = prefix::byte(c, f, SECTION, "trainer.channels_count")?;
            let end = u16::from(origin) + u16::from(n);
            if n > 0 && end <= 64 {
                let display = format!("CH{}-CH{end}", u16::from(origin) + 1);
                let field = Field::new(
                    SECTION,
                    "trainer.channel_range",
                    p..c.pos,
                    Value::Text(display.clone()),
                    display,
                    false,
                )
                .summary();
                f.push(if (4..=16).contains(&n) {
                    field.writable(EditFormat::RadioChannelRange {
                        min_count: 4,
                        max_count: 16,
                    })
                } else {
                    field
                });
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
