//! X20 Pro AW RF module records in model format 0.1.37.
use super::legacy::Cursor;
use super::prefix::{byte, enumeration, expected, fixed, source};
use super::{EditFormat, Error, Field, Result, Value};

fn module_name(kind: u8) -> Option<&'static str> {
    Some(match kind {
        2 => "XJT Lite",
        5 => "R9M Lite",
        6 => "R9M Lite ACCESS",
        7 => "R9M Lite Pro ACCESS",
        10 => "Twin Lite Pro",
        11 => "X20 Pro AW internal module",
        13 => "PPM",
        18 => "SBUS",
        _ => return None,
    })
}

fn protocol_name(kind: u8, protocol: u8) -> Option<&'static str> {
    match kind {
        2 => ["D16", "D8", "LR12"].get(usize::from(protocol)).copied(),
        5 => ["FCC", "EU", "FLEX 868MHz", "FLEX 915MHz"]
            .get(usize::from(protocol))
            .copied(),
        6 | 7 if protocol == 0 => Some("ACCESS"),
        10 => ["ACCESS", "ACCST D16", "ELRS", "TW Mode"]
            .get(usize::from(protocol))
            .copied(),
        11 => ["ACCESS/ACCST D16", "TD Mode", "TW Mode"]
            .get(usize::from(protocol))
            .copied(),
        _ => None,
    }
}

fn fixed_label(c: &mut Cursor<'_>, fields: &mut Vec<Field>, path: &str) -> Result<bool> {
    let start = c.pos;
    let bytes = c.bytes(8)?;
    let end = bytes.iter().position(|v| *v == 0).unwrap_or(bytes.len());
    let field = match std::str::from_utf8(&bytes[..end]) {
        Ok(s) => Field::new("RF", path, start..c.pos, Value::Text(s.into()), s, false),
        Err(_) => Field::opaque("RF", path, start..c.pos),
    };
    fields.push(field);
    Ok(bytes.iter().all(|v| *v == 0))
}

fn display_range(first: u8, last: u8) -> String {
    format!("CH{first}-CH{last}")
}

pub(crate) fn encode_channel_range(
    path: &str,
    value: &str,
    min_count: u8,
    max_count: u8,
) -> Result<(Vec<u8>, String)> {
    fn channel_number(input: &str) -> Option<u8> {
        let input = input.trim().to_ascii_uppercase();
        input
            .strip_prefix("CH")
            .unwrap_or(&input)
            .trim()
            .parse()
            .ok()
    }
    let invalid = || {
        Error::Invalid(format!(
        "{path}: enter a range such as CH1-CH16, within channels 1–64 and {min_count}–{max_count} channels long"
    ))
    };
    let (first, last) = value.split_once(['-', '–']).ok_or_else(invalid)?;
    let first = channel_number(first).ok_or_else(invalid)?;
    let last = channel_number(last).ok_or_else(invalid)?;
    if first == 0 || last > 64 || first > last {
        return Err(invalid());
    }
    let count = last - first + 1;
    if !(min_count..=max_count).contains(&count) {
        return Err(invalid());
    }
    Ok((vec![first - 1, count], display_range(first, last)))
}

pub(crate) fn decode(c: &mut Cursor<'_>, fields: &mut Vec<Field>) -> Result<()> {
    for slot in 0..2 {
        let root = format!("rf.modules[{slot}]");
        c.path = root.clone();
        expected(c, 1)?;
        let enabled_at = c.pos;
        let enabled = c.byte()?;
        let type_at = c.pos;
        let kind = c.byte()?;
        fields.push(enumeration(
            "RF",
            format!("{root}.type"),
            type_at..c.pos,
            kind.into(),
            module_name(kind),
        ));
        if module_name(kind).is_none() {
            return Err(c.unsupported(&format!("RF module type {kind}")));
        }
        let protocol_at = c.pos;
        let protocol = c.byte()?;
        fields.push(enumeration(
            "RF",
            format!("{root}.protocol"),
            protocol_at..c.pos,
            protocol.into(),
            protocol_name(kind, protocol),
        ));
        byte(c, fields, "RF", &format!("{root}.power"))?;
        fixed(c, fields, "RF", &format!("{root}.flags"), 4)?;
        let channels_at = c.pos;
        let from = byte(c, fields, "RF", &format!("{root}.channels_start"))?;
        let count = byte(c, fields, "RF", &format!("{root}.channels_count"))?;
        byte(c, fields, "RF", &format!("{root}.frequency"))?;
        fixed_label(c, fields, &format!("{root}.registration_id"))?;
        let failsafe_at = c.pos;
        let failsafe = c.byte()?;
        let failsafe_label = if kind == 2 && protocol == 0 {
            ["Not set", "Hold", "Custom", "No pulses", "Receiver"]
                .get(usize::from(failsafe))
                .copied()
        } else {
            None
        };
        fields.push(enumeration(
            "RF",
            format!("{root}.failsafe_mode"),
            failsafe_at..c.pos,
            failsafe.into(),
            failsafe_label,
        ));
        let mut empty_receivers = true;
        for receiver in 0..3 {
            empty_receivers &= fixed_label(c, fields, &format!("{root}.receivers[{receiver}]"))?;
        }

        // The JSON oracle materializes three radio bands for every module.
        // The original file stores only the bands assigned to that module.
        let bands: &[u8] = match kind {
            11 => &[0, 1, 2],
            10 => &[0, 2],
            _ => &[0],
        };
        for band in bands {
            let band_path = format!("{root}.bands[{band}]");
            if bands.len() > 1 {
                byte(c, fields, "RF", &format!("{band_path}.enabled"))?;
            }
            byte(c, fields, "RF", &format!("{band_path}.antenna"))?;
            byte(c, fields, "RF", &format!("{band_path}.power"))?;
        }
        if bands.len() > 1 {
            byte(c, fields, "RF", &format!("{root}.extra_options"))?;
        }
        source(c, fields, "RF", &format!("{root}.racing_mode_condition"))?;

        let header = fields
            .iter()
            .find(|f| f.path == format!("rf.module_types[{slot}]"))
            .expect("RF header is decoded before its modules");
        let consistent = enabled <= 1
            && header.typed == Value::Integer(if enabled == 0 { 255 } else { i64::from(kind) });
        let toggle = enumeration(
            "RF",
            format!("{root}.enabled"),
            enabled_at..enabled_at + 1,
            enabled.into(),
            match enabled {
                0 => Some("Off"),
                1 => Some("On"),
                _ => None,
            },
        );
        fields.push(
            if slot == 0 && kind == 11 && empty_receivers && consistent {
                toggle.writable(EditFormat::InternalRf)
            } else {
                toggle
            },
        );

        if slot == 1 && matches!(kind, 13 | 18) {
            let min_count = if kind == 13 { 4 } else { 16 };
            if (min_count..=16).contains(&count) && u16::from(from) + u16::from(count) <= 64 {
                let span = display_range(from + 1, from + count);
                let field = Field::new(
                    "RF",
                    format!("{root}.channel_range"),
                    channels_at..channels_at + 2,
                    Value::Text(span.clone()),
                    span,
                    false,
                )
                .summary();
                fields.push(
                    if enabled == 1 && protocol == 0 && consistent && empty_receivers {
                        field.writable(EditFormat::RadioChannelRange {
                            min_count,
                            max_count: 16,
                        })
                    } else {
                        field
                    },
                );
            }
        }
    }
    Ok(())
}
