//! Sequential X20 Pro AW 0.1.37 model sections. No field is located by searching for its text.
use super::{legacy::Cursor, EditFormat, Field, Result, Value};
type Fields = Vec<Field>;

fn byte(c: &mut Cursor<'_>, f: &mut Fields, section: &str, path: &str) -> Result<u8> {
    let p = c.pos;
    let n = c.byte()?;
    f.push(Field::number(section, path, p..c.pos, n));
    Ok(n)
}
fn label(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    section: &str,
    path: &str,
    editable: bool,
) -> Result<()> {
    let (r, s) = if editable {
        c.limited_text(15)?
    } else {
        c.text()?
    };
    let field = Field::new(section, path, r, Value::Text(s.clone()), s, false);
    f.push(if editable {
        field.writable(EditFormat::Label15)
    } else {
        field
    });
    Ok(())
}
fn source(c: &mut Cursor<'_>, f: &mut Fields, section: &str, path: &str) -> Result<()> {
    let (r, v, s) = c.source()?;
    f.push(Field::new(section, path, r, v, s, false));
    Ok(())
}
fn count(c: &mut Cursor<'_>, f: &mut Fields, section: &str, path: &str, max: u16) -> Result<usize> {
    c.path = path.into();
    let p = c.pos;
    let n = c.count(max)?;
    f.push(Field::number(
        section,
        format!("{path}.count"),
        p..c.pos,
        n as u32,
    ));
    Ok(n)
}
fn fixed(c: &mut Cursor<'_>, f: &mut Fields, section: &str, path: &str, n: usize) -> Result<()> {
    let p = c.pos;
    c.bytes(n)?;
    f.push(Field::opaque(section, path, p..c.pos));
    Ok(())
}
fn expected(c: &mut Cursor<'_>, n: u8) -> Result<()> {
    let actual = c.byte()?;
    if actual != n {
        return Err(c.unsupported(&format!("expected record tag {n:02X}, found {actual:02X}")));
    }
    Ok(())
}
fn curve_reference(c: &mut Cursor<'_>) -> Result<()> {
    match c.byte()? {
        0 | 2..=5 => {}
        1 => {
            c.source()?;
        }
        v => return Err(c.unsupported(&format!("curve reference tag {v}"))),
    }
    Ok(())
}

pub(super) fn decode(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    c.path = "model".into();
    let kind = byte(c, f, "Model", "model.type")?;
    byte(c, f, "Model", "model.category_options")?;
    label(c, f, "Model", "model.name", true)?;
    let p = c.pos;
    let bitmap = c.bytes(16)?;
    let valid = &bitmap[..bitmap.iter().position(|v| *v == 0).unwrap_or(bitmap.len())];
    if let Ok(s) = std::str::from_utf8(valid) {
        f.push(Field::new(
            "Model",
            "model.bitmap",
            p..c.pos,
            Value::Text(s.into()),
            s,
            false,
        ));
    } else {
        f.push(Field::opaque("Model", "model.bitmap", p..c.pos));
    }
    for i in 0..2 {
        byte(c, f, "RF", &format!("rf.module_types[{i}]"))?;
    }
    for i in 0..2 {
        byte(c, f, "RF", &format!("rf.model_ids[{i}]"))?;
    }
    if kind != 0 {
        return Err(c.unsupported("aircraft category has a different parameter layout"));
    }
    fixed(c, f, "Model", "model.airframe_options", 9)?;
    let n = byte(c, f, "Trims", "trims.count")?;
    for i in 0..n {
        let path = format!("trims[{i}]");
        c.path = path.clone();
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
        byte(c, f, "Trims", &format!("{path}.range"))?;
        let style = byte(c, f, "Trims", &format!("{path}.mode"))?;
        byte(c, f, "Trims", &format!("{path}.custom_step"))?;
        byte(c, f, "Trims", &format!("{path}.play_audio"))?;
        match style {
            0 => {
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
    }
    if count(c, f, "Trims", "extra_trims", 5)? != 0 {
        return Err(c.unsupported("extra trim records"));
    }
    let n = byte(c, f, "Trims", "cross_trims.count")?;
    for i in 0..n {
        byte(c, f, "Trims", &format!("cross_trims[{i}]"))?;
    }
    source(c, f, "Trims", "trims.instant_trim")?;
    channels(c, f)?;
    rf(c, f)?;
    flight_modes(c, f)?;
    mixes(c, f)?;
    curves(c, f)?;
    logic(c, f)?;
    variables(c, f)?;
    functions(c, f)?;
    c.path = "trainer".into();
    let v = byte(c, f, "Trainer", "trainer.mode")?;
    byte(c, f, "Trainer", "trainer.options")?;
    if v == 1 {
        source(c, f, "Trainer", "trainer.condition")?;
        for i in 0..16 {
            let path = format!("trainer.channels[{i}]");
            byte(c, f, "Trainer", &format!("{path}.mode"))?;
            byte(c, f, "Trainer", &format!("{path}.weight"))?;
            source(c, f, "Trainer", &format!("{path}.destination"))?;
            source(c, f, "Trainer", &format!("{path}.active_condition"))?;
        }
    } else if v != 0 {
        return Err(c.unsupported("trainer mode"));
    }
    timers(c, f)
}

fn enumeration(
    section: &str,
    path: impl Into<String>,
    range: std::ops::Range<usize>,
    raw: i64,
    label: Option<&str>,
) -> Field {
    let display = label.map_or_else(|| raw.to_string(), str::to_string);
    Field::new(
        section,
        path,
        range,
        Value::Enum {
            raw,
            label: label.map(str::to_string),
        },
        display,
        false,
    )
}
fn on_off(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    section: &str,
    path: &str,
    editable: bool,
) -> Result<()> {
    let start = c.pos;
    let n = c.byte()?;
    let f0 = enumeration(
        section,
        path,
        start..c.pos,
        n.into(),
        match n {
            0 => Some("Off"),
            1 => Some("On"),
            _ => None,
        },
    );
    f.push(if editable && n <= 1 {
        f0.writable(EditFormat::Toggle)
    } else {
        f0
    });
    Ok(())
}
fn channel_percent(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    path: &str,
    minimum: i16,
    maximum: i16,
) -> Result<()> {
    let (range, typed, display) = c.source()?;
    match typed {
        Value::Integer(raw) => {
            let value = format!("{}%", raw as f64 / 10.0);
            let f0 = Field::new(
                "Outputs",
                path,
                range,
                Value::FixedPoint { raw, scale: 10 },
                value,
                false,
            );
            f.push(
                if (i64::from(minimum)..=i64::from(maximum)).contains(&raw) {
                    f0.writable(EditFormat::Percent { minimum, maximum })
                } else {
                    f0
                },
            );
        }
        _ => f.push(Field::new("Outputs", path, range, typed, display, false)),
    }
    Ok(())
}
fn channels(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..64 {
        let path = format!("channels[{i}]");
        c.path = path.clone();
        let p = c.pos;
        let tag = c.byte()?;
        match tag {
            1 => {
                f.push(Field::new(
                    "Outputs",
                    format!("{path}.default"),
                    p..c.pos,
                    Value::Enum {
                        raw: 1,
                        label: Some("Default".into()),
                    },
                    "Default",
                    false,
                ));
                super::channel_edit::default_fields(f, &path, p);
            }
            0 => {
                f.push(Field::number(
                    "Outputs",
                    format!("{path}.default"),
                    p..c.pos,
                    0,
                ));
                on_off(c, f, "Outputs", &format!("{path}.invert"), true)?;
                for name in ["offset_1", "offset_2"] {
                    source(c, f, "Outputs", &format!("{path}.{name}"))?;
                }
                channel_percent(c, f, &format!("{path}.limit_max"), 0, 1500)?;
                channel_percent(c, f, &format!("{path}.limit_min"), -1500, 0)?;
                channel_percent(c, f, &format!("{path}.center"), -1000, 1000)?;
                let p = c.pos;
                c.bytes(3)?;
                curve_reference(c)?;
                let n = c.byte()?;
                if n > 0 {
                    c.byte()?;
                    for _ in 0..n {
                        c.source()?;
                    }
                }
                c.bytes(2)?;
                f.push(Field::opaque(
                    "Outputs",
                    format!("{path}.parameters"),
                    p..c.pos,
                ));
                label(c, f, "Outputs", &format!("{path}.name"), true)?;
            }
            _ => return Err(c.unsupported("channel encoding")),
        }
    }
    Ok(())
}

fn fixed_label(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    section: &str,
    path: &str,
    width: usize,
) -> Result<bool> {
    let start = c.pos;
    let raw = c.bytes(width)?;
    let cut = raw.iter().position(|v| *v == 0).unwrap_or(width);
    match std::str::from_utf8(&raw[..cut]) {
        Ok(s) => f.push(Field::new(
            section,
            path,
            start..c.pos,
            Value::Text(s.into()),
            s,
            false,
        )),
        Err(_) => f.push(Field::opaque(section, path, start..c.pos)),
    }
    Ok(raw.iter().all(|&v| v == 0))
}
fn rf(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..2 {
        let root = format!("rf.modules[{i}]");
        c.path = root.clone();
        expected(c, 1)?;
        let enabled_at = c.pos;
        let enabled = c.byte()?;
        let protocol = byte(c, f, "RF", &format!("{root}.type"))?;
        if !matches!(protocol, 10 | 11) {
            return Err(c.unsupported(&format!("RF protocol {protocol}")));
        }
        byte(c, f, "RF", &format!("{root}.protocol"))?;
        byte(c, f, "RF", &format!("{root}.power"))?;
        fixed(c, f, "RF", &format!("{root}.flags"), 4)?;
        byte(c, f, "RF", &format!("{root}.channels_start"))?;
        byte(c, f, "RF", &format!("{root}.channels_count"))?;
        byte(c, f, "RF", &format!("{root}.frequency"))?;
        fixed_label(c, f, "RF", &format!("{root}.registration_id"), 8)?;
        byte(c, f, "RF", &format!("{root}.failsafe_mode"))?;
        let mut empty_receivers = true;
        for j in 0..3 {
            empty_receivers &= fixed_label(c, f, "RF", &format!("{root}.receivers[{j}]"), 8)?;
        }
        for j in 0..3 {
            if protocol == 11 {
                byte(c, f, "RF", &format!("{root}.bands[{j}].enabled"))?;
            }
            byte(c, f, "RF", &format!("{root}.bands[{j}].antenna"))?;
            byte(c, f, "RF", &format!("{root}.bands[{j}].power"))?;
        }
        byte(c, f, "RF", &format!("{root}.extra_options"))?;
        source(c, f, "RF", &format!("{root}.racing_mode_condition"))?;
        let field = enumeration(
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
        let header = f
            .iter()
            .find(|item| item.path == format!("rf.module_types[{i}]"))
            .expect("header fields are decoded first");
        let proven = i == 0
            && protocol == 11
            && empty_receivers
            && enabled <= 1
            && header.typed == Value::Integer(if enabled == 0 { 255 } else { 11 });
        f.push(if proven {
            field.writable(EditFormat::InternalRf)
        } else {
            field
        });
    }
    Ok(())
}
fn flight_modes(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Flight modes", "flight_modes", 20)? {
        let path = format!("flight_modes[{i}]");
        c.path = path.clone();
        expected(c, 0)?;
        source(c, f, "Flight modes", &format!("{path}.active_condition"))?;
        label(c, f, "Flight modes", &format!("{path}.name"), true)?;
        label(c, f, "Flight modes", &format!("{path}.audio"), false)?;
        source(c, f, "Flight modes", &format!("{path}.fade_in"))?;
        source(c, f, "Flight modes", &format!("{path}.fade_out"))?;
    }
    Ok(())
}

fn operand_list(c: &mut Cursor<'_>) -> Result<()> {
    let n = c.count(120)?;
    for _ in 0..n {
        c.source()?;
        c.source()?;
    }
    Ok(())
}
fn mix_channels(c: &mut Cursor<'_>) -> Result<()> {
    let n = c.byte()?;
    c.bytes(n.into())?;
    Ok(())
}
fn free_mix(c: &mut Cursor<'_>, f: &mut Fields, path: &str) -> Result<()> {
    let n = byte(c, f, "Mixes", &format!("{path}.channels.count"))?;
    for j in 0..n {
        byte(c, f, "Mixes", &format!("{path}.channels[{j}].index"))?;
    }
    source(c, f, "Mixes", &format!("{path}.input"))?;
    byte(c, f, "Mixes", &format!("{path}.operation"))?;
    byte(c, f, "Mixes", &format!("{path}.inverted_outputs"))?;
    let n = count(c, f, "Mixes", &format!("{path}.actions"), 120)?;
    for j in 0..n {
        let action = format!("{path}.actions[{j}]");
        let p = c.pos;
        let variant = c.byte()?;
        f.push(enumeration(
            "Mixes",
            format!("{action}.key"),
            p..c.pos,
            variant.into(),
            match variant {
                0 => Some("Curve"),
                1 => Some("Weight"),
                _ => None,
            },
        ));
        source(c, f, "Mixes", &format!("{action}.active_condition"))?;
        match variant {
            0 => {
                let p = c.pos;
                curve_reference(c)?;
                operand_list(c)?;
                f.push(Field::opaque(
                    "Mixes",
                    format!("{action}.curve_parameters"),
                    p..c.pos,
                ));
            }
            1 => {
                let (r, v, s) = c.source()?;
                let field = Field::new("Mixes", format!("{action}.weight"), r, v.clone(), s, false);
                f.push(if matches!(v, Value::Integer(-100..=100)) {
                    field.writable(EditFormat::Operand {
                        minimum: -100,
                        maximum: 100,
                    })
                } else {
                    field
                });
                let n = count(c, f, "Mixes", &format!("{action}.rates"), 120)?;
                for k in 0..n {
                    source(c, f, "Mixes", &format!("{action}.rates[{k}].condition"))?;
                    source(c, f, "Mixes", &format!("{action}.rates[{k}].value"))?;
                }
            }
            5 => {}
            _ => return Err(c.unsupported("free mix action")),
        }
    }
    Ok(())
}
fn mixes(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Mixes", "mixes", 120)? {
        let path = format!("mixes[{i}]");
        c.path = path.clone();
        let p = c.pos;
        let key = c.dword()?;
        let kind = match key {
            1 => Some("Free"),
            4 => Some("Ailerons"),
            5 => Some("Elevators"),
            6 => Some("Rudders"),
            7 => Some("Flaps"),
            16 => Some("Throttle"),
            17 => Some("Ailerons to flaps"),
            20 => Some("Butterfly"),
            22 => Some("Flaps to elevator"),
            _ => None,
        };
        f.push(enumeration(
            "Mixes",
            format!("{path}.key"),
            p..c.pos,
            key.into(),
            kind,
        ));
        expected(c, 1)?;
        label(c, f, "Mixes", &format!("{path}.name"), true)?;
        fixed(c, f, "Mixes", &format!("{path}.flight_mode_mask"), 4)?;
        source(c, f, "Mixes", &format!("{path}.active_condition"))?;
        let p = c.pos;
        match key {
            1 => free_mix(c, f, &path)?,
            4 | 5 | 6 | 17 => {
                curve_reference(c)?;
                operand_list(c)?;
                mix_channels(c)?;
                c.source()?;
                operand_list(c)?;
                c.byte()?;
                c.source()?;
            }
            22 => {
                curve_reference(c)?;
                operand_list(c)?;
                mix_channels(c)?;
                for _ in 0..3 {
                    c.source()?;
                }
            }
            7 => {
                curve_reference(c)?;
                operand_list(c)?;
                mix_channels(c)?;
                for _ in 0..4 {
                    c.source()?;
                }
                c.bytes(2)?;
            }
            20 => {
                curve_reference(c)?;
                operand_list(c)?;
                mix_channels(c)?;
                for _ in 0..7 {
                    c.source()?;
                }
            }
            16 => {
                c.source()?;
                curve_reference(c)?;
                operand_list(c)?;
                mix_channels(c)?;
                for _ in 0..3 {
                    c.source()?;
                }
                // Version 1 gyro configuration with a numeric base and a conditional second setting.
                let flags = c.byte()?;
                if flags != 0x93 {
                    return Err(c.unsupported("gyro mix settings"));
                }
                c.source()?;
                c.source()?;
                expected(c, 1)?;
                c.source()?;
            }
            _ => return Err(c.unsupported(&format!("mix key {key}"))),
        }
        f.push(Field::opaque(
            "Mixes",
            format!("{path}.parameters"),
            p..c.pos,
        ));
    }
    Ok(())
}
fn curves(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Curves", "curves", 50)? {
        let path = format!("curves[{i}]");
        c.path = path.clone();
        expected(c, 0)?;
        label(c, f, "Curves", &format!("{path}.name"), true)?;
        let kind = byte(c, f, "Curves", &format!("{path}.type"))?;
        if kind == 0 {
            continue;
        }
        if kind != 3 {
            return Err(c.unsupported("curve shape"));
        }
        source(c, f, "Curves", &format!("{path}.offset"))?;
        let n = byte(c, f, "Curves", &format!("{path}.points.count"))?;
        byte(c, f, "Curves", &format!("{path}.smooth"))?;
        let custom = byte(c, f, "Curves", &format!("{path}.custom_x"))?;
        if custom > 1 {
            return Err(c.unsupported("curve coordinates"));
        }
        for j in 0..n {
            if custom == 1 {
                source(c, f, "Curves", &format!("{path}.points[{j}].x"))?;
            }
            source(c, f, "Curves", &format!("{path}.points[{j}].y"))?;
        }
    }
    Ok(())
}
fn logic(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Logic switches", "logic_switches", 100)? {
        let path = format!("logic_switches[{i}]");
        c.path = path.clone();
        let p = c.pos;
        let k = c.byte()?;
        f.push(enumeration(
            "Logic switches",
            format!("{path}.key"),
            p..c.pos,
            k.into(),
            match k {
                1 => Some("Around"),
                9 => Some("Range"),
                _ => None,
            },
        ));
        if !matches!(k, 1 | 9) {
            return Err(c.unsupported("logic switch key"));
        }
        expected(c, 1)?;
        fixed(c, f, "Logic switches", &format!("{path}.timing"), 8)?;
        let n = c.count(64)?;
        for j in 0..n {
            source(c, f, "Logic switches", &format!("{path}.values[{j}]"))?;
        }
        source(c, f, "Logic switches", &format!("{path}.active_condition"))?;
        label(c, f, "Logic switches", &format!("{path}.name"), true)?;
        fixed(c, f, "Logic switches", &format!("{path}.flags"), 3)?;
        label(c, f, "Logic switches", &format!("{path}.comment"), false)?;
    }
    Ok(())
}
fn variables(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Variables", "variables", 64)? {
        let path = format!("variables[{i}]");
        c.path = path.clone();
        expected(c, 0)?;
        label(c, f, "Variables", &format!("{path}.name"), true)?;
        label(c, f, "Variables", &format!("{path}.comment"), false)?;
        source(c, f, "Variables", &format!("{path}.default"))?;
        if c.word()? != 0 {
            return Err(c.unsupported("variable conditional values"));
        }
        source(c, f, "Variables", &format!("{path}.minimum"))?;
        source(c, f, "Variables", &format!("{path}.maximum"))?;
        if c.word()? != 0 {
            return Err(c.unsupported("variable actions"));
        }
    }
    Ok(())
}
fn functions(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Special functions", "functions", 100)? {
        let path = format!("functions[{i}]");
        c.path = path.clone();
        let p = c.pos;
        let k = c.byte()?;
        f.push(enumeration(
            "Special functions",
            format!("{path}.key"),
            p..c.pos,
            k.into(),
            match k {
                0 => Some("Reset"),
                3 => Some("Play track or value"),
                _ => None,
            },
        ));
        expected(c, 1)?;
        byte(c, f, "Special functions", &format!("{path}.enabled"))?;
        source(
            c,
            f,
            "Special functions",
            &format!("{path}.active_condition"),
        )?;
        if k == 0 {
            source(c, f, "Special functions", &format!("{path}.target"))?;
            continue;
        }
        if k != 3 {
            return Err(c.unsupported("special function variant"));
        }
        source(
            c,
            f,
            "Special functions",
            &format!("{path}.reset_condition"),
        )?;
        byte(c, f, "Special functions", &format!("{path}.mode"))?;
        let n = c.count(64)?;
        for j in 0..n {
            match c.byte()? {
                0 => {
                    label(
                        c,
                        f,
                        "Special functions",
                        &format!("{path}.elements[{j}].audio"),
                        false,
                    )?;
                    c.byte()?;
                }
                1 => {
                    source(
                        c,
                        f,
                        "Special functions",
                        &format!("{path}.elements[{j}].source"),
                    )?;
                    c.bytes(2)?;
                }
                _ => return Err(c.unsupported("function element")),
            }
        }
        fixed(c, f, "Special functions", &format!("{path}.timing"), 8)?;
    }
    Ok(())
}
fn timers(c: &mut Cursor<'_>, f: &mut Fields) -> Result<()> {
    for i in 0..count(c, f, "Timers", "timers", 8)? {
        let path = format!("timers[{i}]");
        c.path = path.clone();
        expected(c, 0)?;
        label(c, f, "Timers", &format!("{path}.name"), true)?;
        let p = c.pos;
        let mode = c.byte()?;
        let mode_field = Field::new(
            "Timers",
            format!("{path}.mode"),
            p..c.pos,
            Value::Enum {
                raw: mode.into(),
                label: match mode {
                    0 => Some("Up".into()),
                    1 => Some("Down".into()),
                    _ => None,
                },
            },
            match mode {
                0 => "Up".into(),
                1 => "Down".into(),
                _ => mode.to_string(),
            },
            false,
        );
        f.push(if mode <= 1 {
            mode_field.writable(EditFormat::Boolean)
        } else {
            mode_field
        });
        for key in ["start_value", "alarm_value"] {
            let p = c.pos;
            let n = c.dword()?;
            f.push(
                Field::number("Timers", format!("{path}.{key}"), p..c.pos, n as i32)
                    .writable(EditFormat::TimerSeconds),
            );
        }
        byte(c, f, "Timers", &format!("{path}.flags"))?;
        let persist = byte(c, f, "Timers", &format!("{path}.persistent"))?;
        if persist != 0 {
            fixed(c, f, "Timers", &format!("{path}.stored_value"), 4)?;
        }
        source(c, f, "Timers", &format!("{path}.start_condition"))?;
        source(c, f, "Timers", &format!("{path}.reset_condition"))?;
        let p = c.pos;
        let actions = super::edit::read_actions(c)?;
        let display = serde_json::to_string(&actions).expect("timer actions serialize");
        f.push(
            Field::new(
                "Timers",
                format!("{path}.audio_actions"),
                p..c.pos,
                Value::Text(display.clone()),
                display,
                false,
            )
            .writable(EditFormat::TimerActions),
        );
        let custom = byte(c, f, "Timers", &format!("{path}.custom_stop"))?;
        if custom != 0 {
            source(c, f, "Timers", &format!("{path}.stop_condition"))?;
        }
        source(c, f, "Timers", &format!("{path}.counting_source"))?;
    }
    Ok(())
}
