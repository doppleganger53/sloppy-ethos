use ethos_model::{crc16, FieldEdit, Model, Value};
use std::{fs, path::Path};

fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("{name}.bin")),
    )
    .unwrap()
}

fn read_field(model: &Model, path: &str) -> Value {
    model
        .fields()
        .into_iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("field {path} missing: {:?}", model.warnings()))
        .typed
}

#[test]
fn native_screens_checklist_and_lua_blobs_are_bounded() {
    let screens = Model::parse(&fixture("widget-reference")).unwrap();
    assert_eq!(read_field(&screens, "screens.count"), Value::Integer(2));
    assert_eq!(
        read_field(&screens, "screens[1].widgets[0].key"),
        Value::Text("FxView".into())
    );
    assert_eq!(
        read_field(&screens, "screens[0].widgets[1].source"),
        Value::Source {
            category: 21,
            options: 0,
            member: 0
        }
    );
    let lua = Model::parse(&fixture("lua-references")).unwrap();
    assert_eq!(
        read_field(&lua, "lua.sources[0].key"),
        Value::Text("FxSrc".into())
    );
    assert_eq!(
        read_field(&lua, "lua.tasks[0].key"),
        Value::Text("FxTask".into())
    );
    assert_eq!(
        read_field(&lua, "checklist.throttle_value"),
        Value::Integer(-95)
    );
    let chart = Model::parse(&fixture("chart-widget")).unwrap();
    assert_eq!(
        read_field(&chart, "screens[1].widgets[0].minimum"),
        Value::Integer(-1024)
    );
    assert_eq!(
        read_field(&chart, "screens[1].widgets[0].maximum"),
        Value::Integer(1024)
    );
    let logs = Model::parse(&fixture("timer-widget")).unwrap();
    assert_eq!(
        read_field(&logs, "screens[1].widgets[0].reversed_order"),
        Value::Enum {
            raw: 0,
            label: Some("Off".into())
        }
    );
    let gps = Model::parse(&fixture("gps-widget")).unwrap();
    assert_eq!(
        read_field(&gps, "screens[1].widgets[0].max_longitude"),
        Value::Integer(1_800_000_000)
    );
    let lipo = Model::parse(&fixture("lipo-widget")).unwrap();
    assert_eq!(
        read_field(&lipo, "screens[1].widgets[0].voltage_warning"),
        Value::Integer(300)
    );
    let channels = Model::parse(&fixture("channels-widget")).unwrap();
    assert_eq!(
        read_field(&channels, "screens[1].widgets[0].channels[3]"),
        Value::Integer(4)
    );
    let text = Model::parse(&fixture("text-widget")).unwrap();
    assert_eq!(
        read_field(&text, "screens[1].widgets[0].file"),
        Value::Text(String::new())
    );
}

#[test]
fn scalar_and_packed_enum_edits_match_independent_firmware_outputs() {
    let model = Model::parse(&fixture("default")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cases = [
        ("screens[0].widgets[1].title", "off", "widget-title"),
        (
            "screens[0].widgets[1].source",
            "Timer 1",
            "widget-timer-source",
        ),
        ("checklist.switches.sa", "Middle", "checklist"),
        ("checklist.switches.sa", "down", "checklist-sa-down"),
        ("checklist.switches.sa", "0", "checklist-sa-none"),
        ("checklist.failsafe", "0", "checklist-failsafe"),
    ];
    for (path, value, name) in cases {
        let destination = dir.path().join(name);
        model
            .save_fields_copy(
                &destination,
                &[],
                &[FieldEdit {
                    path: path.into(),
                    value: value.into(),
                }],
            )
            .unwrap();
        assert_eq!(fs::read(destination).unwrap(), fixture(name), "{name}");
    }
    assert!(model
        .preview_fields(
            &[],
            &[FieldEdit {
                path: "checklist.switches.sa".into(),
                value: "17".into()
            }]
        )
        .is_err());
}

#[test]
fn changing_a_value_widget_source_validates_existing_slots_and_resizes_absent_refs() {
    let path = "screens[0].widgets[1].source";
    let edit = |value: &str| FieldEdit {
        path: path.into(),
        value: value.into(),
    };
    let blank = Model::parse(&fixture("default")).unwrap();
    for invalid in ["Telemetry 0", "Timer 3", "Timer -1", "Receiver 0"] {
        assert!(
            blank.preview_fields(&[], &[edit(invalid)]).is_err(),
            "{invalid}"
        );
    }
    let model = Model::parse(&fixture("throttle-after")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("telemetry.bin");
    model
        .save_fields_copy(&destination, &[], &[edit("Telemetry 1")])
        .unwrap();
    let changed = Model::open(&destination).unwrap();
    assert_eq!(
        read_field(&changed, path),
        Value::Source {
            category: 22,
            options: 0,
            member: 1
        }
    );
    let none = dir.path().join("no-source.bin");
    changed
        .save_fields_copy(&none, &[], &[edit("None")])
        .unwrap();
    let cleared = Model::open(&none).unwrap();
    assert_eq!(read_field(&cleared, path), Value::Absent);
    assert_eq!(cleared.to_bytes().len() + 3, model.to_bytes().len());
    let restored = dir.path().join("restored.bin");
    cleared
        .save_fields_copy(&restored, &[], &[edit("Timer 0")])
        .unwrap();
    assert_eq!(fs::read(restored).unwrap(), model.to_bytes());
}

#[test]
fn unknown_widget_is_opaque_without_hiding_the_verified_sensor_collection() {
    let mut raw = fixture("throttle-after");
    let model = Model::parse(&raw).unwrap();
    let field = model
        .fields()
        .into_iter()
        .find(|f| f.path == "screens[0].widgets[1].key")
        .unwrap();
    raw[field.range.start] = 0xfe;
    let crc = crc16(&raw[16..]);
    raw[14..16].copy_from_slice(&crc.to_le_bytes());
    let inspected = Model::parse(&raw).unwrap();
    assert_eq!(inspected.sensors().len(), 4);
    assert!(inspected.warnings().iter().any(|w| w.contains("widget")));
    assert!(inspected.fields().iter().any(|f| f.typed == Value::Opaque
        && f.range.start <= field.range.start
        && f.range.end > field.range.start));
    assert!(!inspected
        .fields()
        .iter()
        .any(|f| f.path == "checklist.switches.sa"));
}

#[test]
fn aliases_cannot_replace_either_input_via_copy_export() {
    let scratch = tempfile::tempdir().unwrap();
    let current = scratch.path().join("current.bin");
    let backup = scratch.path().join("backup.bin");
    let current_bytes = fixture("throttle-after");
    let backup_bytes = fixture("throttle-before");
    fs::write(&current, &current_bytes).unwrap();
    fs::write(&backup, &backup_bytes).unwrap();
    let model = Model::open(&current).unwrap();
    for (i, original) in [&current, &backup].iter().enumerate() {
        let alias = scratch.path().join(format!("alias{i}.bin"));
        fs::hard_link(original, &alias).unwrap();
        assert!(model
            .save_fields_copy(
                &alias,
                &[],
                &[FieldEdit {
                    path: "screens[0].widgets[1].title".into(),
                    value: "Off".into()
                }]
            )
            .is_err());
    }
    assert_eq!(fs::read(current).unwrap(), current_bytes);
    assert_eq!(fs::read(backup).unwrap(), backup_bytes);
}

#[test]
fn a_neighbouring_switch_bit_does_not_change_sa() {
    let mut bytes = fixture("default");
    let initial = Model::parse(&bytes).unwrap();
    let index = initial
        .fields()
        .into_iter()
        .find(|f| f.path == "checklist.switches.sa")
        .unwrap()
        .range
        .start;
    bytes[index] ^= 0x04;
    let crc = crc16(&bytes[16..]);
    bytes[14..16].copy_from_slice(&crc.to_le_bytes());
    let changed = Model::parse(&bytes).unwrap();
    let diff = ethos_model::compare(&initial, &changed);
    assert!(diff.iter().any(|value| value.path == "checklist.switches"));
    assert!(!diff
        .iter()
        .any(|value| value.path == "checklist.switches.sa"));
}

#[test]
fn checklist_thresholds_match_native_menu_limits() {
    let model = Model::parse(&fixture("default")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let edit = |value: &str| FieldEdit {
        path: "checklist.throttle_value".into(),
        value: value.into(),
    };
    for (value, expected) in [
        ("-100", "checklist-throttle-min"),
        ("100", "checklist-throttle-max"),
    ] {
        let file = dir.path().join(expected);
        model.save_fields_copy(&file, &[], &[edit(value)]).unwrap();
        assert_eq!(fs::read(file).unwrap(), fixture(expected));
    }
    assert!(model.preview_fields(&[], &[edit("-101")]).is_err());
    assert!(model.preview_fields(&[], &[edit("101")]).is_err());
}

#[test]
fn line_chart_modes_bounds_and_intervals_match_independent_native_saves() {
    let chart = Model::parse(&fixture("chart-widget")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let edit = |leaf: &str, value: &str| FieldEdit {
        path: format!("screens[1].widgets[0].{leaf}"),
        value: value.into(),
    };
    for (leaf, value, expected) in [
        ("interval", "5", "chart-period-min"),
        ("interval", "100", "chart-period-max"),
        ("inverted", "On", "chart-inverted"),
        ("auto_range", "On", "chart-auto"),
        ("minimum", "1024", "chart-min-high"),
        ("maximum", "-1024", "chart-max-low"),
        ("minimum", "-1023", "chart-min-step"),
        ("maximum", "1023", "chart-max-step"),
    ] {
        let file = dir.path().join(expected);
        chart
            .save_fields_copy(&file, &[], &[edit(leaf, value)])
            .unwrap();
        assert_eq!(fs::read(file).unwrap(), fixture(expected), "{expected}");
    }
    let automatic = Model::parse(&fixture("chart-auto")).unwrap();
    assert_eq!(
        read_field(&automatic, "checklist.throttle_value"),
        Value::Integer(-95)
    );
    assert!(automatic
        .fields()
        .iter()
        .all(|f| f.path != "screens[1].widgets[0].minimum"));
    let manual = dir.path().join("manual.bin");
    automatic
        .save_fields_copy(&manual, &[], &[edit("auto_range", "Off")])
        .unwrap();
    assert_eq!(fs::read(manual).unwrap(), fixture("chart-manual"));
    assert!(chart.preview_fields(&[], &[edit("interval", "4")]).is_err());
    assert!(chart
        .preview_fields(&[], &[edit("interval", "101")])
        .is_err());
    assert!(chart
        .preview_fields(&[], &[edit("minimum", "20"), edit("maximum", "10")])
        .is_err());
    assert!(chart
        .preview_fields(&[], &[edit("minimum", "10"), edit("maximum", "20")])
        .is_ok());
    assert!(chart
        .preview_fields(&[], &[edit("auto_range", "On"), edit("minimum", "-250")])
        .is_err());
}
