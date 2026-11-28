//! Structures following the 0.1.37 telemetry collection. Unknown widget payloads
//! cannot be skipped unless the firmware supplied an explicit length.
use crate::legacy::Cursor;
use crate::{EditFormat, Error, Field, Model, Result, Value};
use std::ops::Range;

const SWITCH_POSITIONS: [&str; 4] = ["No check", "Up", "Middle", "Down"];
const TOGGLE: [&str; 2] = ["Off", "On"];

fn enum_field(
    section: &str,
    path: &str,
    range: Range<usize>,
    raw: i64,
    label: Option<&str>,
    editable: bool,
) -> Field {
    Field::new(
        section,
        path,
        range,
        Value::Enum {
            raw,
            label: label.map(str::to_owned),
        },
        label.map(str::to_owned).unwrap_or_else(|| raw.to_string()),
        editable && label.is_some(),
    )
}

fn read_enum(
    c: &mut Cursor<'_>,
    fields: &mut Vec<Field>,
    section: &str,
    path: &str,
    labels: &[&str],
    editable: bool,
) -> Result<u8> {
    c.path = path.into();
    let start = c.pos;
    let raw = c.byte()?;
    fields.push(enum_field(
        section,
        path,
        start..c.pos,
        raw.into(),
        labels.get(usize::from(raw)).copied(),
        editable,
    ));
    Ok(raw)
}

fn read_source(c: &mut Cursor<'_>, fields: &mut Vec<Field>, path: &str) -> Result<bool> {
    c.path = path.into();
    let (range, typed, display) = c.source()?;
    let absent = typed == Value::Absent;
    fields.push(Field::new("Screens", path, range, typed, display, false));
    Ok(absent)
}

fn bounded_byte(
    section: &str,
    path: &str,
    range: Range<usize>,
    value: i16,
    low: i16,
    high: i16,
) -> Field {
    let field = Field::number(section, path, range, value);
    if (low..=high).contains(&value) {
        field.writable(EditFormat::Byte {
            minimum: low,
            maximum: high,
        })
    } else {
        field
    }
}

fn line_chart(c: &mut Cursor<'_>, fields: &mut Vec<Field>, path: &str) -> Result<()> {
    let no_source = read_source(c, fields, &format!("{path}.source"))?;
    read_source(c, fields, &format!("{path}.pause_condition"))?;
    let start = c.pos;
    let interval = c.byte()?;
    fields.push(bounded_byte(
        "Screens",
        &format!("{path}.interval"),
        start..c.pos,
        interval.into(),
        5,
        100,
    ));
    read_enum(
        c,
        fields,
        "Screens",
        &format!("{path}.inverted"),
        &TOGGLE,
        true,
    )?;
    c.path = format!("{path}.auto_range");
    let start = c.pos;
    let automatic = c.byte()?;
    let bounds = match automatic {
        0 => Some(c.bytes(8)?),
        1 => None,
        _ => return Err(c.unsupported("line chart range mode")),
    };
    // The flag owns the optional bounds for a mode change. Overlapping leaf edits are rejected.
    fields.push(
        enum_field(
            "Screens",
            &c.path,
            start..c.pos,
            automatic.into(),
            Some(TOGGLE[usize::from(automatic)]),
            no_source,
        )
        .projected(),
    );
    if let Some(bounds) = bounds {
        for (index, leaf) in ["maximum", "minimum"].into_iter().enumerate() {
            let value = i32::from_le_bytes(bounds[index * 4..index * 4 + 4].try_into().unwrap());
            let offset = start + 1 + index * 4;
            fields.push(Field::new(
                "Screens",
                format!("{path}.{leaf}"),
                offset..offset + 4,
                Value::Integer(value.into()),
                value.to_string(),
                no_source && (-1024..=1024).contains(&value),
            ));
        }
    }
    Ok(())
}

fn value_source(c: &mut Cursor<'_>, fields: &mut Vec<Field>, path: &str) -> Result<()> {
    c.path = path.into();
    let (range, typed, original) = c.source()?;
    let label = match &typed {
        Value::Absent => Some("None".to_owned()),
        Value::Source {
            category: 21,
            options: 0,
            member,
        } => Some(format!("Timer {member}")),
        Value::Source {
            category: 22,
            options: 0,
            member,
        } => Some(format!("Telemetry {member}")),
        _ => None,
    };
    let editable = label.is_some();
    fields.push(Field::new(
        "Screens",
        path,
        range,
        typed,
        label.unwrap_or(original),
        editable,
    ));
    Ok(())
}

fn collection_count(
    c: &mut Cursor<'_>,
    fields: &mut Vec<Field>,
    section: &str,
    path: &str,
) -> Result<u16> {
    c.path = path.into();
    let start = c.pos;
    let count = c.word()?;
    fields.push(Field::number(
        section,
        format!("{path}.count"),
        start..c.pos,
        count,
    ));
    Ok(count)
}

fn read_blob(c: &mut Cursor<'_>, fields: &mut Vec<Field>, section: &str, path: &str) -> Result<()> {
    c.path = path.into();
    let length_offset = c.pos;
    let length = c.word()?;
    fields.push(Field::number(
        section,
        format!("{path}.length"),
        length_offset..c.pos,
        length,
    ));
    let start = c.pos;
    c.bytes(usize::from(length))?;
    if length != 0 {
        fields.push(Field::opaque(section, path, start..c.pos));
    }
    Ok(())
}

fn stored_key(bytes: &[u8]) -> Option<&str> {
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    if end == 0 || bytes[end..].iter().any(|&byte| byte != 0) {
        return None;
    }
    std::str::from_utf8(&bytes[..end])
        .ok()
        .filter(|key| key.chars().all(|ch| !ch.is_control()))
}

fn read_widget(c: &mut Cursor<'_>, fields: &mut Vec<Field>, path: &str) -> Result<()> {
    let key_path = format!("{path}.key");
    c.path = key_path.clone();
    let start = c.pos;
    let key = c.bytes(8)?;
    let kind = key[0];
    if kind == 8 {
        if let Some(name) = stored_key(&key[1..]) {
            fields.push(Field::new(
                "Screens",
                &key_path,
                start..c.pos,
                Value::Text(name.into()),
                name,
                false,
            ));
        } else {
            fields.push(Field::opaque("Screens", &key_path, start..c.pos));
        }
        read_enum(
            c,
            fields,
            "Screens",
            &format!("{path}.title"),
            &TOGGLE,
            true,
        )?;
        return read_blob(c, fields, "Screens", &format!("{path}.configuration"));
    }
    if key[1..].iter().any(|&byte| byte != 0) {
        c.pos = start;
        return Err(c.unsupported("unverified widget key encoding"));
    }
    let label = match kind {
        0 => "None",
        1 => "Bitmap",
        2 => "Value",
        3 => "Timer logs",
        4 => "GPS map",
        5 => "LiPo",
        6 => "Channels",
        7 => "Line chart",
        9 => "Text",
        _ => {
            c.pos = start;
            return Err(c.unsupported(&format!("widget type {kind}")));
        }
    };
    fields.push(enum_field(
        "Screens",
        &key_path,
        start..c.pos,
        kind.into(),
        Some(label),
        false,
    ));
    match kind {
        0 => {}
        1 => {
            let value = read_enum(
                c,
                fields,
                "Screens",
                &format!("{path}.bitmap_type"),
                &["Model bitmap"],
                false,
            )?;
            if value != 0 {
                return Err(c.unsupported("bitmap widget payload variant"));
            }
            c.path = format!("{path}.file");
            let (range, name) = c.text()?;
            fields.push(Field::new(
                "Screens",
                &c.path,
                range,
                Value::Text(name.clone()),
                name,
                false,
            ));
        }
        2 | 3 => {
            if kind == 2 {
                value_source(c, fields, &format!("{path}.source"))?;
            } else {
                read_source(c, fields, &format!("{path}.source"))?;
            }
            read_enum(
                c,
                fields,
                "Screens",
                &format!("{path}.title"),
                &TOGGLE,
                true,
            )?;
            if kind == 3 {
                read_enum(
                    c,
                    fields,
                    "Screens",
                    &format!("{path}.reversed_order"),
                    &TOGGLE,
                    false,
                )?;
            }
        }
        4 => {
            read_source(c, fields, &format!("{path}.source"))?;
            for leaf in [
                "min_latitude",
                "max_latitude",
                "min_longitude",
                "max_longitude",
            ] {
                c.path = format!("{path}.{leaf}");
                let start = c.pos;
                let value = i32::from_le_bytes(c.bytes(4)?.try_into().unwrap());
                fields.push(Field::number("Screens", &c.path, start..c.pos, value));
            }
            c.path = format!("{path}.background");
            let (range, name) = c.text()?;
            fields.push(Field::new(
                "Screens",
                &c.path,
                range,
                Value::Text(name.clone()),
                name,
                false,
            ));
        }
        5 => {
            read_source(c, fields, &format!("{path}.source"))?;
            let start = c.pos;
            let warning = c.word()?;
            fields.push(Field::number(
                "Screens",
                format!("{path}.voltage_warning"),
                start..c.pos,
                warning,
            ));
            read_enum(
                c,
                fields,
                "Screens",
                &format!("{path}.title"),
                &TOGGLE,
                true,
            )?;
        }
        6 => {
            read_enum(
                c,
                fields,
                "Screens",
                &format!("{path}.vertical"),
                &TOGGLE,
                false,
            )?;
            let start = c.pos;
            let count = c.byte()?;
            if count > 8 {
                return Err(c.unsupported("channel widget count exceeds its eight slots"));
            }
            fields.push(Field::number(
                "Screens",
                format!("{path}.channels.count"),
                start..c.pos,
                count,
            ));
            for index in 0..8 {
                let start = c.pos;
                let channel = c.byte()?;
                fields.push(Field::number(
                    "Screens",
                    format!("{path}.channels[{index}]"),
                    start..c.pos,
                    channel,
                ));
            }
        }
        7 => line_chart(c, fields, path)?,
        9 => {
            c.path = format!("{path}.file");
            let (range, name) = c.text()?;
            fields.push(Field::new(
                "Screens",
                &c.path,
                range,
                Value::Text(name.clone()),
                name,
                false,
            ));
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn screens(c: &mut Cursor<'_>, fields: &mut Vec<Field>) -> Result<()> {
    for index in 0..collection_count(c, fields, "Screens", "screens")? {
        let path = format!("screens[{index}]");
        c.path = format!("{path}.layout");
        let start = c.pos;
        let raw = i64::from_le_bytes(c.bytes(8)?.try_into().unwrap());
        let label = match raw {
            0 => Some("1+3 with trims"),
            1 => Some("2x2"),
            3 => Some("3x3"),
            10 => Some("Full screen, borderless"),
            14 => Some("2x3 with trims"),
            _ => None,
        };
        fields.push(enum_field(
            "Screens",
            &c.path,
            start..c.pos,
            raw,
            label,
            false,
        ));
        let widgets = format!("{path}.widgets");
        for slot in 0..collection_count(c, fields, "Screens", &widgets)? {
            read_widget(c, fields, &format!("{widgets}[{slot}]"))?;
        }
    }
    Ok(())
}

fn checklist(c: &mut Cursor<'_>, fields: &mut Vec<Field>) -> Result<()> {
    c.path = "checklist.switches".into();
    let start = c.pos;
    let switches = c.dword()?;
    fields.push(Field::number("Checklist", &c.path, start..c.pos, switches));
    let sa = (switches & 3) as usize;
    fields.push(
        enum_field(
            "Checklist",
            "checklist.switches.sa",
            start..start + 1,
            sa as i64,
            Some(SWITCH_POSITIONS[sa]),
            true,
        )
        .projected(),
    );
    read_enum(c, fields, "Checklist", "checklist.failsafe", &TOGGLE, true)?;
    let start = c.pos;
    let analogs = c.dword()?;
    fields.push(Field::number(
        "Checklist",
        "checklist.analog_checks",
        start..c.pos,
        analogs,
    ));
    let start = c.pos;
    let count = c.byte()?;
    fields.push(Field::number(
        "Checklist",
        "checklist.pot_values.count",
        start..c.pos,
        count,
    ));
    for index in 0..count {
        let start = c.pos;
        let value = c.byte()? as i8;
        fields.push(Field::number(
            "Checklist",
            format!("checklist.pot_values[{index}]"),
            start..c.pos,
            value,
        ));
    }
    let start = c.pos;
    let throttle = c.byte()? as i8;
    fields.push(bounded_byte(
        "Checklist",
        "checklist.throttle_value",
        start..c.pos,
        throttle.into(),
        -100,
        100,
    ));
    let start = c.pos;
    let function_switches = c.word()?;
    fields.push(Field::number(
        "Checklist",
        "checklist.function_switches",
        start..c.pos,
        function_switches,
    ));
    Ok(())
}

fn lua_registrations(c: &mut Cursor<'_>, fields: &mut Vec<Field>, group: &str) -> Result<()> {
    for index in 0..collection_count(c, fields, "Lua", group)? {
        let path = format!("{group}[{index}]");
        c.path = format!("{path}.key");
        let start = c.pos;
        let bytes = c.bytes(8)?;
        if let Some(name) = stored_key(bytes) {
            fields.push(Field::new(
                "Lua",
                &c.path,
                start..c.pos,
                Value::Text(name.into()),
                name,
                false,
            ));
        } else {
            fields.push(Field::opaque("Lua", &c.path, start..c.pos));
        }
        read_blob(c, fields, "Lua", &format!("{path}.configuration"))?;
    }
    Ok(())
}

pub(crate) fn decode(c: &mut Cursor<'_>, fields: &mut Vec<Field>) -> Result<()> {
    // Enabling Competition in ETHOS removes existing sensors. The editor exposes it for inspection.
    read_enum(
        c,
        fields,
        "Telemetry",
        "telemetry.competition",
        &TOGGLE,
        false,
    )?;
    read_enum(
        c,
        fields,
        "Telemetry",
        "telemetry.bluetooth",
        &["None", "Telemetry"],
        false,
    )?;
    let start = c.pos;
    c.byte()?;
    fields.push(Field::opaque(
        "Telemetry",
        "telemetry.additional_flags",
        start..c.pos,
    ));
    screens(c, fields)?;
    let start = c.pos;
    c.byte()?;
    fields.push(Field::opaque("Screens", "screens.selection", start..c.pos));
    checklist(c, fields)?;
    let start = c.pos;
    c.bytes(2)?;
    fields.push(Field::opaque(
        "Model",
        "model.additional_controls",
        start..c.pos,
    ));
    lua_registrations(c, fields, "lua.sources")?;
    lua_registrations(c, fields, "lua.tasks")?;
    if c.pos != c.data.len() {
        return Err(c.unsupported("unverified trailing model bytes"));
    }
    Ok(())
}

pub(crate) fn encode(
    model: &Model,
    field: &Field,
    input: &str,
    original: &[u8],
) -> Result<Option<(Vec<u8>, String)>> {
    let path = field.path.as_str();
    if field.editable && field.section == "Screens" && path.ends_with(".source") {
        let value = input.trim();
        if value.eq_ignore_ascii_case("none") {
            return Ok(Some((vec![0], "None".into())));
        }
        let target = value.split_once(' ').and_then(|(group, slot)| {
            let (category, count_path, name) = if group.eq_ignore_ascii_case("timer") {
                (21, "timers.count", "Timer")
            } else if group.eq_ignore_ascii_case("telemetry") {
                (22, "telemetry.sensors.count", "Telemetry")
            } else {
                return None;
            };
            Some((category, count_path, name, slot.trim().parse::<u16>().ok()?))
        });
        let (category, count_path, name, slot) = target.ok_or_else(|| {
            Error::Invalid(format!(
                "{path}: use Timer N, Telemetry N (zero-based slot), or None"
            ))
        })?;
        let available = model
            .fields
            .iter()
            .find(|f| f.path == count_path)
            .and_then(|f| match f.typed {
                Value::Integer(n) => Some(n),
                _ => None,
            });
        if !available.is_some_and(|count| i64::from(slot) < count) {
            return Err(Error::Invalid(format!(
                "{path}: {name} slot {slot} does not exist in this model"
            )));
        }
        let [lo, hi] = slot.to_le_bytes();
        return Ok(Some((vec![category, 0, lo, hi], format!("{name} {slot}"))));
    }
    let screen = field.section == "Screens" && path.starts_with("screens[");
    if screen && field.editable && (path.ends_with(".minimum") || path.ends_with(".maximum")) {
        let value = input
            .trim()
            .parse::<i32>()
            .ok()
            .filter(|v| (-1024..=1024).contains(v))
            .ok_or_else(|| {
                Error::Invalid(format!("{path}: enter a whole value from -1024 to 1024"))
            })?;
        return Ok(Some((value.to_le_bytes().to_vec(), value.to_string())));
    }
    let switch = path == "checklist.switches.sa";
    let auto_range = screen && path.ends_with(".auto_range");
    let boolean = path == "checklist.failsafe"
        || auto_range
        || (screen && (path.ends_with(".title") || path.ends_with(".inverted")));
    if !switch && !boolean {
        return Ok(None);
    }
    if !field.editable || !(original.len() == 1 || (auto_range && original.len() == 9)) {
        return Err(Error::Invalid(format!(
            "{path}: the stored value is outside the verified edit range"
        )));
    }
    let labels: &[&str] = if switch { &SWITCH_POSITIONS } else { &TOGGLE };
    let input = input.trim();
    let choice = labels
        .iter()
        .position(|label| label.eq_ignore_ascii_case(input))
        .or_else(|| {
            if boolean && input.eq_ignore_ascii_case("true") {
                Some(1)
            } else if boolean && input.eq_ignore_ascii_case("false") {
                Some(0)
            } else {
                input
                    .parse::<usize>()
                    .ok()
                    .filter(|value| *value < labels.len())
            }
        });
    let choice =
        choice.ok_or_else(|| Error::Invalid(format!("{path}: choose {}", labels.join(", "))))?;
    if auto_range {
        let bytes = if choice as u8 == original[0] {
            original.to_vec()
        } else if choice == 1 {
            vec![1]
        } else {
            let mut manual = vec![0];
            manual.extend(1024_i32.to_le_bytes());
            manual.extend((-1024_i32).to_le_bytes());
            manual
        };
        return Ok(Some((bytes, labels[choice].to_owned())));
    }
    let byte = if switch {
        (original[0] & !3) | choice as u8
    } else {
        choice as u8
    };
    Ok(Some((vec![byte], labels[choice].to_string())))
}

/// Native manual chart bounds permit equality and constrain the minimum to the final maximum.
pub(crate) fn check_chart_bounds(model: &Model, path: &str) -> Result<()> {
    let Some((parent, leaf)) = path.rsplit_once('.') else {
        return Ok(());
    };
    if !parent.starts_with("screens[") || !matches!(leaf, "minimum" | "maximum") {
        return Ok(());
    }
    let bound = |name: &str| {
        let target = format!("{parent}.{name}");
        model
            .fields
            .iter()
            .find(|f| f.path == target)
            .and_then(|f| match f.typed {
                Value::Integer(value) => Some(value),
                _ => None,
            })
    };
    if let (Some(minimum), Some(maximum)) = (bound("minimum"), bound("maximum")) {
        if minimum > maximum {
            return Err(Error::Invalid(format!(
                "{path}: chart minimum must be at most its maximum"
            )));
        }
    }
    Ok(())
}
