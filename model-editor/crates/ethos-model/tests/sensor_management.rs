use ethos_model::{crc16, Edit, FieldEdit, Model, SensorDraft, Value};
use std::path::Path;

fn load(name: &str) -> Model {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(format!("{name}.bin"));
    Model::open(&path).unwrap()
}

#[test]
fn builds_a_sensor_from_verified_defaults_and_copies_a_complete_record() {
    let original = load("layout-references");
    let mut draft = SensorDraft::default();
    assert_eq!(draft.missing().len(), 7);
    assert!(original
        .append_sensor(&draft)
        .unwrap_err()
        .to_string()
        .contains("Physical ID"));
    draft.values = ["Bench new", "0", "1", "0", "0", "05", "6803"].map(str::to_owned);
    let added = original.append_sensor(&draft).unwrap();
    let copy = added.copy_sensor(2).unwrap();
    let pasted = added.paste_sensor(&copy).unwrap();
    assert!(pasted.verify_sensor_keys(&original).is_err());
    let edited = Model::parse(
        &pasted
            .serialize_fields(
                &[Edit {
                    slot: 3,
                    physical_id: Some(6),
                    application_id: None,
                }],
                &[FieldEdit {
                    path: "telemetry.sensors[3].name".into(),
                    value: "Bench pasted".into(),
                }],
            )
            .unwrap(),
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("separate.bin");
    edited.save_sensor_copy(&output, &original).unwrap();
    assert!(edited.save_sensor_copy(&output, &original).is_err());
    let reopened = Model::open(&output).unwrap();
    assert_eq!(reopened.sensors()[2].name, "Bench new");
    assert_eq!(reopened.sensors()[2].physical_id, 5);
    assert_eq!(reopened.sensors()[3].name, "Bench pasted");
    assert_eq!(reopened.sensors()[3].physical_id, 6);
    let suffix = |model: &Model, slot| {
        model
            .field(&format!("telemetry.sensors[{slot}].parameters"))
            .unwrap()
            .range
            .clone()
    };
    assert_eq!(
        reopened.raw(suffix(&reopened, 2)),
        reopened.raw(suffix(&reopened, 3))
    );
}

#[test]
fn previews_indirect_deletion_and_preserves_surviving_references() {
    let model = load("layout-references");
    let graph = model.dependencies();
    assert!(graph.incoming["telemetry.sensors[1]"].contains("timers[0]"));
    assert!(graph.incoming["timers[0]"].contains("screens[0].widgets[1]"));
    assert!(graph.outgoing["screens[0].widgets[1]"].contains("timers[0]"));
    let blocked = model.preview_sensor_delete(&[1], false).unwrap();
    assert_eq!(blocked.removed, ["telemetry.sensors[1]"]);
    assert_eq!(
        blocked.blockers,
        ["timers[0] references telemetry.sensors[1]"]
    );
    let plan = model.preview_sensor_delete(&[1], true).unwrap();
    assert!(plan.blockers.is_empty());
    assert_eq!(
        plan.removed,
        ["screens[0].widgets[1]", "telemetry.sensors[1]", "timers[0]"]
    );

    let shifted = model.delete_sensors(&[0], false).unwrap();
    assert_eq!(shifted.sensors()[0].module, model.sensors()[1].module);
    assert_eq!(
        shifted.field("timers[0].start_condition").unwrap().typed,
        Value::Source {
            category: 22,
            options: 0,
            member: 0
        }
    );

    let output = model.delete_sensors(&[1], true).unwrap();
    assert_eq!(output.sensors()[0].module, model.sensors()[0].module);
    assert_eq!(output.field("timers.count").unwrap().value, "2");
    assert_eq!(
        output.field("screens[0].widgets[1].key").unwrap().value,
        "None"
    );
    assert_eq!(
        output.field("screens[0].widgets[2].source").unwrap().value,
        "Timer 0"
    );
    assert_eq!(
        output.field("screens[0].widgets[3].source").unwrap().value,
        "Timer 1"
    );
}

fn replace_sources(model: &Model, sources: &[(&str, [u8; 4])]) -> Model {
    let mut bytes = model.to_bytes();
    let mut patches: Vec<_> = sources
        .iter()
        .map(|(path, replacement)| (model.field(path).unwrap().range.clone(), replacement))
        .collect();
    patches.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, replacement) in patches {
        bytes.splice(range, replacement.iter().copied());
    }
    let size = (bytes.len() - 16) as u32;
    bytes[8..12].copy_from_slice(&size.to_le_bytes());
    let crc = crc16(&bytes[16..]);
    bytes[14..16].copy_from_slice(&crc.to_le_bytes());
    Model::parse(&bytes).unwrap()
}

#[test]
fn cycle_terminates_and_sensor_alive_references_remap() {
    let model = replace_sources(
        &load("layout-references"),
        &[
            ("timers[0].start_condition", [23, 0, 1, 0]),
            ("timers[0].reset_condition", [21, 0, 1, 0]),
            ("timers[1].reset_condition", [21, 0, 0, 0]),
        ],
    );
    let shifted = model.delete_sensors(&[0], false).unwrap();
    assert!(matches!(
        shifted.field("timers[0].start_condition").unwrap().typed,
        Value::Source {
            category: 23,
            member: 0,
            ..
        }
    ));
    let preview = model.preview_sensor_delete(&[1], true).unwrap();
    assert_eq!(preview.removed.len(), 5);
    assert!(preview.blockers.is_empty());
    assert_eq!(
        model
            .delete_sensors(&[1], true)
            .unwrap()
            .field("timers.count")
            .unwrap()
            .value,
        "1"
    );
}

#[test]
fn a_copied_source_cannot_silently_bind_to_a_different_slot() {
    let model = replace_sources(
        &load("layout-references"),
        &[("telemetry.sensors[0].reset_source", [22, 0, 1, 0])],
    );
    let copy = model.copy_sensor(0).unwrap();
    assert_eq!(
        model.paste_sensor(&copy).unwrap().telemetry_paths().len(),
        3
    );
    let shifted = model.delete_sensors(&[0], false).unwrap();
    let error = shifted.paste_sensor(&copy).unwrap_err().to_string();
    assert!(error.contains("Copy the sensor again"), "{error}");
}

#[test]
fn calculated_and_diy_records_can_be_removed_and_lua_blocks_unsafe_renumbering() {
    let mixed = load("mixed-telemetry-before");
    assert_eq!(mixed.telemetry_paths().len(), 2);
    let pasted = mixed.paste_sensor(&mixed.copy_sensor(0).unwrap()).unwrap();
    assert_eq!(pasted.telemetry_paths().len(), 3);
    let reduced = pasted.delete_sensors(&[0, 2], false).unwrap();
    assert_eq!(reduced.sensors()[0].slot, 0);
    let calculated = load("calculated-operation");
    assert_eq!(
        calculated
            .delete_sensors(&[0], false)
            .unwrap()
            .telemetry_paths()
            .len(),
        0
    );

    let lua = load("lua-references")
        .paste_sensor(&mixed.copy_sensor(0).unwrap())
        .unwrap();
    assert!(lua
        .preview_sensor_delete(&[0], true)
        .unwrap()
        .blockers
        .iter()
        .any(|text| text.contains("lua")));
    assert!(lua.delete_sensors(&[0], true).is_err());
}
