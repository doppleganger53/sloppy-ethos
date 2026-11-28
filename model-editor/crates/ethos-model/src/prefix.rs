//! Sequential X20 Pro AW 0.1.37 model sections. No field is located by searching for its text.
use super::{legacy::Cursor, EditFormat, Field, Result, Value};
type Fields = Vec<Field>;

pub(crate) fn byte(c: &mut Cursor<'_>, f: &mut Fields, section: &str, path: &str) -> Result<u8> {
    let p = c.pos;
    let n = c.byte()?;
    f.push(Field::number(section, path, p..c.pos, n));
    Ok(n)
}
pub(crate) fn label(
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
pub(crate) fn source(c: &mut Cursor<'_>, f: &mut Fields, section: &str, path: &str) -> Result<()> {
    let (r, v, s) = c.source()?;
    f.push(Field::new(section, path, r, v, s, false));
    Ok(())
}
pub(crate) fn count(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    section: &str,
    path: &str,
    max: u16,
) -> Result<usize> {
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
pub(crate) fn fixed(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    section: &str,
    path: &str,
    n: usize,
) -> Result<()> {
    let p = c.pos;
    c.bytes(n)?;
    f.push(Field::opaque(section, path, p..c.pos));
    Ok(())
}
pub(crate) fn expected(c: &mut Cursor<'_>, n: u8) -> Result<()> {
    let actual = c.byte()?;
    if actual != n {
        return Err(c.unsupported(&format!("expected record tag {n:02X}, found {actual:02X}")));
    }
    Ok(())
}
fn curve_reference(c: &mut Cursor<'_>) -> Result<()> {
    match c.byte()? {
        0 | 2..=51 => {}
        1 => {
            c.source()?;
        }
        v => return Err(c.unsupported(&format!("curve reference tag {v}"))),
    }
    Ok(())
}

pub(super) fn decode(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    curves: &mut Vec<std::ops::Range<usize>>,
) -> Result<()> {
    c.path = "model".into();
    let at = c.pos;
    let kind = c.byte()?;
    let category = match kind {
        0 => Some("Plane"),
        1 => Some("Glider"),
        2 => Some("Helicopter"),
        3 => Some("Multirotor"),
        4 => Some("Other"),
        _ => None,
    };
    f.push(enumeration(
        "Model",
        "model.type",
        at..c.pos,
        kind.into(),
        category,
    ));
    byte(c, f, "Model", "model.category_options")?;
    label(c, f, "Model", "model.name", true)?;
    let p = c.pos;
    let bitmap = c.bytes(16)?;
    let valid = &bitmap[..bitmap.iter().position(|v| *v == 0).unwrap_or(bitmap.len())];
    if let Ok(s) = std::str::from_utf8(valid) {
        let field = Field::new(
            "Model",
            "model.bitmap",
            p..c.pos,
            Value::Text(s.into()),
            s,
            false,
        );
        f.push(if valid.len() < 16 {
            field.writable(EditFormat::BitmapKey)
        } else {
            field
        });
    } else {
        f.push(Field::opaque("Model", "model.bitmap", p..c.pos));
    }
    for i in 0..2 {
        byte(c, f, "RF", &format!("rf.module_types[{i}]"))?;
    }
    for i in 0..2 {
        byte(c, f, "RF", &format!("rf.model_ids[{i}]"))?;
    }
    // Each category has different fixed model options before the independent trim count.
    let option_bytes = match kind {
        0 | 1 => 9,
        2 => 6,
        3 => 8,
        4 => 4,
        _ => {
            c.path = "model.type".into();
            return Err(c.unsupported(&format!("unverified aircraft category {kind}")));
        }
    };
    let options_at = c.pos;
    fixed(c, f, "Model", "model.airframe_options", option_bytes)?;
    if kind == 1 {
        let at = options_at + 4;
        f.push(Field::number("Model", "model.flaps_count", at..at + 1, c.data[at]).projected());
    } else if kind == 2 {
        let at = options_at + 3;
        let swash = c.data[at];
        let name = if swash == 4 {
            Some("Three servos, 135 degrees")
        } else {
            None
        };
        f.push(
            enumeration("Model", "model.swash_type", at..at + 1, swash.into(), name).projected(),
        );
    }
    super::trims::decode(c, f)?;
    let n = byte(c, f, "Trims", "cross_trims.count")?;
    for i in 0..n {
        byte(c, f, "Trims", &format!("cross_trims[{i}]"))?;
    }
    source(c, f, "Trims", "trims.instant_trim")?;
    channels(c, f)?;
    super::radio::decode(c, f)?;
    flight_modes(c, f)?;
    mixes(c, f)?;
    super::curves::decode(c, f, curves)?;
    super::logic::switches(c, f)?;
    super::variables::decode(c, f)?;
    functions(c, f)?;
    super::trainer::decode(c, f)?;
    timers(c, f)
}

pub(crate) fn enumeration(
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
pub(crate) fn on_off(
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
pub(crate) fn percent_source(
    c: &mut Cursor<'_>,
    f: &mut Fields,
    section: &str,
    path: &str,
    minimum: i16,
    maximum: i16,
) -> Result<()> {
    let (range, typed, display) = c.source()?;
    match typed {
        Value::Integer(raw) => {
            let value = format!("{}%", raw as f64 / 10.0);
            let f0 = Field::new(
                section,
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
        _ => f.push(Field::new(section, path, range, typed, display, false)),
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
                percent_source(c, f, "Outputs", &format!("{path}.limit_max"), 0, 1500)?;
                percent_source(c, f, "Outputs", &format!("{path}.limit_min"), -1500, 0)?;
                percent_source(c, f, "Outputs", &format!("{path}.center"), -1000, 1000)?;
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

fn curve_parameters(c: &mut Cursor<'_>) -> Result<()> {
    curve_reference(c)?;
    let n = c.count(120)?;
    for _ in 0..n {
        curve_reference(c)?;
        c.source()?;
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
fn mix_channels(c: &mut Cursor<'_>, f: &mut Fields, root: &str) -> Result<usize> {
    let n = byte(c, f, "Mixes", &format!("{root}.channels.count"))?;
    for j in 0..n {
        byte(c, f, "Mixes", &format!("{root}.channels[{j}].index"))?;
    }
    Ok(usize::from(n))
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
                super::curves::reference(c, f, &format!("{action}.curve.default"))?;
                let n = count(c, f, "Mixes", &format!("{action}.curve.rates"), 120)?;
                for k in 0..n {
                    super::curves::reference(c, f, &format!("{action}.curve.rates[{k}].curve"))?;
                    source(
                        c,
                        f,
                        "Mixes",
                        &format!("{action}.curve.rates[{k}].condition"),
                    )?;
                }
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
            8 => Some("Helicopter ailerons"),
            9 => Some("Helicopter elevators"),
            10 => Some("Helicopter rudders"),
            11 => Some("Helicopter pitch"),
            12 => Some("Multirotor aileron"),
            13 => Some("Multirotor elevator"),
            14 => Some("Multirotor rudder"),
            15 => Some("Bank"),
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
                curve_parameters(c)?;
                mix_channels(c, f, &path)?;
                c.source()?;
                operand_list(c)?;
                c.byte()?;
                c.source()?;
            }
            22 => {
                curve_parameters(c)?;
                mix_channels(c, f, &path)?;
                for _ in 0..3 {
                    c.source()?;
                }
            }
            7 => {
                curve_parameters(c)?;
                let n = mix_channels(c, f, &path)?;
                for j in 0..n {
                    source(c, f, "Mixes", &format!("{path}.weights[{j}]"))?;
                }
                source(c, f, "Mixes", &format!("{path}.input"))?;
                c.bytes(2)?;
            }
            20 => {
                curve_parameters(c)?;
                mix_channels(c, f, &path)?;
                for _ in 0..7 {
                    c.source()?;
                }
            }
            8 | 9 | 10 | 12 | 13 | 14 => {
                curve_parameters(c)?;
                mix_channels(c, f, &path)?;
                source(c, f, "Mixes", &format!("{path}.weight"))?;
                let n = count(c, f, "Mixes", &format!("{path}.rates"), 120)?;
                for j in 0..n {
                    source(c, f, "Mixes", &format!("{path}.rates[{j}].condition"))?;
                    source(c, f, "Mixes", &format!("{path}.rates[{j}].value"))?;
                }
                byte(c, f, "Mixes", &format!("{path}.trim"))?;
            }
            11 | 15 => {
                curve_parameters(c)?;
                let n = mix_channels(c, f, &path)?;
                for j in 0..n {
                    source(c, f, "Mixes", &format!("{path}.weights[{j}]"))?;
                }
                source(c, f, "Mixes", &format!("{path}.input"))?;
            }
            16 => {
                c.source()?;
                curve_parameters(c)?;
                mix_channels(c, f, &path)?;
                for _ in 0..3 {
                    c.source()?;
                }
                // The whole byte is persisted, but only its low bit describes the sticky option.
                // Different fresh wizard saves can contain different high bits.
                let p = c.pos;
                let flags = c.byte()?;
                f.push(
                    enumeration(
                        "Mixes",
                        format!("{path}.cut.sticky"),
                        p..c.pos,
                        (flags & 1).into(),
                        Some(if flags & 1 != 0 { "On" } else { "Off" }),
                    )
                    .projected(),
                );
                c.source()?;
                c.source()?;
                let enable = c.byte()?;
                if enable > 1 {
                    return Err(c.unsupported("throttle hold flag"));
                }
                c.source()?;
            }
            _ => return Err(c.unsupported(&format!("mix key {key}"))),
        }
        if key != 1 {
            f.push(Field::opaque(
                "Mixes",
                format!("{path}.parameters"),
                p..c.pos,
            ));
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
