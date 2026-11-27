use ethos_model::{compare, crc16, DiffKind, Edit, FieldEdit, Model};
use std::{
    fs,
    path::{Path, PathBuf},
};
fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("{name}.bin")),
    )
    .unwrap()
}
fn from(name: &str) -> Model {
    Model::parse(&fixture(name)).unwrap()
}
fn change(slot: usize, app: u16) -> Edit {
    Edit {
        slot,
        physical_id: None,
        application_id: Some(app),
    }
}
fn damaged(mut b: Vec<u8>) -> Vec<u8> {
    let n = (b.len() - 16) as u32;
    b[8..12].copy_from_slice(&n.to_le_bytes());
    let crc = crc16(&b[16..]);
    b[14..16].copy_from_slice(&crc.to_le_bytes());
    b
}
fn field(path: &str, value: &str) -> FieldEdit {
    FieldEdit {
        path: path.into(),
        value: value.into(),
    }
}
#[test]
fn envelope_and_unmodified_bytes() {
    assert_eq!(crc16(b"123456789"), 0x31c3);
    for entry in fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")).unwrap()
    {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|s| s == "bin") {
            let bytes = fs::read(&path).unwrap();
            let m = Model::parse(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert_eq!(m.to_bytes(), bytes);
            assert_eq!(
                m.serialize_fields(&[], &[]).unwrap(),
                bytes,
                "{}",
                path.display()
            );
            assert!(m
                .fields()
                .iter()
                .all(|f| f.range.start <= f.range.end && f.range.end <= bytes.len()));
        }
    }
    let b = fixture("default");
    for (i, v) in [(0, 0), (4, 0), (7, 0), (8, 0), (13, 0), (15, 0)] {
        let mut bad = b.clone();
        bad[i] = v;
        assert!(Model::parse(&bad).is_err(), "header byte {i}");
    }
    for n in [0, 1, 15, 16, 18, b.len() - 1] {
        assert!(Model::parse(&b[..n]).is_err());
    }
}
#[test]
fn edit_all_throttle_ids_matches_independent_firmware_export_exactly() {
    let before = from("throttle-before");
    let after = from("throttle-after");
    assert!(before.warnings().is_empty(), "{:?}", before.warnings());
    assert_eq!(before.sensors().len(), 4);
    let differences = compare(&before, &after);
    let ids: Vec<_> = differences
        .iter()
        .filter(|d| d.path.ends_with(".application_id"))
        .collect();
    assert_eq!(ids.len(), 4);
    assert!(ids
        .iter()
        .all(|d| d.before.as_deref() == Some("6803") && d.after.as_deref() == Some("6802")));
    let edits: Vec<_> = (0..4).map(|i| change(i, 0x6802)).collect();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("copy.bin");
    before.save_copy(&out, &edits).unwrap();
    assert_eq!(fs::read(&out).unwrap(), fixture("throttle-after"));
    assert_eq!(Model::open(&out).unwrap().to_bytes(), after.to_bytes());
    assert_eq!(before.to_bytes(), fixture("throttle-before"));
    let changed: Vec<_> = before
        .to_bytes()
        .iter()
        .zip(after.to_bytes())
        .enumerate()
        .filter(|(_, (x, y))| **x != *y)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changed, [14, 15, 366, 402, 447, 487]);
}
#[test]
fn physical_only_batch_swaps_and_collision_protection() {
    let m = from("id-conflict");
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("new.bin");
    assert!(m
        .save_copy(&out, &[change(0, 0x6803)])
        .unwrap_err()
        .to_string()
        .contains("duplicate"));
    assert!(!out.exists());
    m.save_copy(&out, &[change(0, 0x6803), change(1, 0x6802)])
        .unwrap();
    let original = fs::read(&out).unwrap();
    assert!(m.save_copy(&out, &[]).is_err());
    assert_eq!(original, fs::read(&out).unwrap());
    let edit = Edit {
        slot: 0,
        physical_id: Some(0x1b),
        application_id: None,
    };
    let out2 = dir.path().join("physical.bin");
    m.save_copy(&out2, &[edit]).unwrap();
    assert_eq!(Model::open(&out2).unwrap().sensors()[0].physical_id, 0x1b);
    for id in [0u8, 0x1b] {
        assert!(m
            .preview(&[Edit {
                physical_id: Some(id),
                ..edit
            }])
            .is_ok());
    }
    assert!(m
        .preview(&[Edit {
            physical_id: Some(0x1c),
            ..edit
        }])
        .is_err());
    assert!(m.preview(&[change(80, 0x6802)]).is_err());
    assert!(m.preview(&[change(0, 0x1234)]).is_err());
    assert!(m.preview(&[change(0, 0x6802), change(0, 0x6802)]).is_err());
    assert!(m
        .save_copy(&dir.path().join("absent/none.bin"), &[])
        .is_err());
}
#[test]
fn duplicate_labels_and_moves() {
    let m = from("layout-references");
    assert_eq!(m.sensors().len(), 2);
    assert_eq!(m.sensors()[0].name, m.sensors()[1].name);
    assert_ne!(m.sensors()[0].module, m.sensors()[1].module);
    let b = from("throttle-before");
    let reordered = from("throttle-reordered");
    let d = compare(&b, &reordered);
    for kind in [DiffKind::Added, DiffKind::Removed, DiffKind::Moved] {
        assert!(d.iter().any(|v| v.kind == kind));
    }
    let twins = from("id-conflict");
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("twins.bin");
    twins
        .save_copy(&out, &[change(0, 0x6800), change(1, 0x6801)])
        .unwrap();
    assert!(compare(&twins, &Model::open(&out).unwrap())
        .iter()
        .any(|d| d.kind == DiffKind::Ambiguous));
}
#[test]
fn reject_truncation_and_unverified_variant_without_guessing_an_offset() {
    let b = from("throttle-before");
    let at = b.sensors()[0].range.start;
    let mut unknown = b.to_bytes();
    unknown[at] = 0xee;
    let m = Model::parse(&damaged(unknown)).unwrap();
    assert!(!m.warnings().is_empty());
    assert!(m.sensors().is_empty());
    assert!(m.preview(&[change(0, 0x6802)]).is_err());
    let mut malformed = b.to_bytes();
    malformed.truncate(at + 12);
    assert!(Model::parse(&damaged(malformed)).is_err());
    let mut embedded = fixture("default");
    let name = &b.to_bytes()[at..at + 10];
    embedded[19..19 + name.len()].copy_from_slice(name);
    let m = Model::parse(&damaged(embedded)).unwrap();
    assert!(m.sensors().is_empty());
    assert!(m.preview(&[change(0, 0x6802)]).is_err());
}
#[test]
fn equal_encoded_lengths_do_not_authorize_changing_the_sensor_type() {
    let m = from("sensor-variants");
    let original = m.to_bytes();
    let s = m
        .sensors()
        .iter()
        .find(|s| s.application_id == 0xF010 && s.sub_id == 0)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("unsafe.bin");
    let error = m
        .save_copy(
            &out,
            &[Edit {
                slot: s.slot,
                physical_id: Some(2),
                application_id: Some(0x0100),
            }],
        )
        .unwrap_err();
    assert!(error.to_string().contains("same-meaning"));
    assert!(!out.exists());
    assert_eq!(m.to_bytes(), original);
}

#[test]
fn actual_structures_supported() {
    let cases = [
        ("sensor-variants", 47),
        ("servo-addresses", 19),
        ("layout-references", 2),
        ("diy", 0),
        ("calculated-consumption", 0),
        ("calculated-custom", 0),
        ("calculated-operation", 0),
        ("channel", 0),
        ("curve", 0),
        ("timer", 0),
        ("flight-mode", 0),
        ("free-mix", 0),
        ("special-function", 0),
        ("logic-switch", 0),
        ("variable", 0),
    ];
    for (name, count) in cases {
        let m = from(name);
        assert!(m.warnings().is_empty(), "{name}: {:?}", m.warnings());
        assert_eq!(m.sensors().len(), count, "{name}");
    }
}
#[test]
fn length_changing_names_and_timer_actions_match_firmware() {
    let b = from("default");
    let dir = tempfile::tempdir().unwrap();
    for (i, name) in ["A", "Model Test 1234", ""].into_iter().enumerate() {
        let p = dir.path().join(format!("name{i}.bin"));
        b.save_fields_copy(&p, &[], &[field("model.name", name)])
            .unwrap();
        assert_eq!(
            fs::read(p).unwrap(),
            fixture(["name-short", "name-long", "default"][i])
        );
    }
    assert!(b
        .preview_fields(&[], &[field("model.name", "0123456789012345")])
        .is_err());
    assert!(b
        .preview_fields(&[], &[field("telemetry.sensors.count", "1")])
        .is_err());
    for name in ["timer-audio-one", "timer-audio-three"] {
        let reference = from(name);
        let action = reference
            .fields()
            .into_iter()
            .find(|f| f.path == "timers[0].audio_actions")
            .unwrap();
        let p = dir.path().join(format!("{name}.bin"));
        b.save_fields_copy(&p, &[], &[field(&action.path, &action.value)])
            .unwrap();
        assert_eq!(fs::read(p).unwrap(), fixture(name));
        let back = dir.path().join(format!("{name}-removed.bin"));
        reference
            .save_fields_copy(&back, &[], &[field(&action.path, "[]")])
            .unwrap();
        assert_eq!(fs::read(back).unwrap(), fixture("default"));
    }
    let t = dir.path().join("timer.bin");
    b.save_fields_copy(
        &t,
        &[],
        &[
            field("timers[1].name", "Flight timer"),
            field("timers[1].mode", "Down"),
            field("timers[1].start_value", "173"),
            field("timers[1].alarm_value", "37"),
        ],
    )
    .unwrap();
    assert_eq!(fs::read(t).unwrap(), fixture("timer"));
}

#[test]
fn slot_reordering_remaps_the_target_and_rejects_a_dangling_reference() {
    let model = from("layout-references");
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("reordered.bin");
    let job = field("telemetry.sensors.order", "[1,0]");
    let edits = model
        .preview_fields(&[], std::slice::from_ref(&job))
        .unwrap();
    assert!(edits
        .iter()
        .any(|change| change.path == "timers[0].start_condition"));
    model.save_fields_copy(&p, &[], &[job]).unwrap();
    let new = Model::open(&p).unwrap();
    assert_eq!(new.sensors()[0].module, model.sensors()[1].module);
    let source = new
        .fields()
        .into_iter()
        .find(|f| f.path == "timers[0].start_condition")
        .unwrap();
    assert!(matches!(
        source.typed,
        ethos_model::Value::Source {
            category: 22,
            member: 0,
            ..
        }
    ));
    let dangling = dir.path().join("dangling.bin");
    let error = model
        .save_fields_copy(&dangling, &[], &[field("telemetry.sensors.order", "[0]")])
        .unwrap_err();
    assert!(error.to_string().contains("timers[0].start_condition"));
    assert!(!dangling.exists());
    let keep = dir.path().join("keep-referenced.bin");
    model
        .save_fields_copy(&keep, &[], &[field("telemetry.sensors.order", "[1]")])
        .unwrap();
    assert_eq!(Model::open(&keep).unwrap().sensors().len(), 1);
    let rich = from("free-mix");
    assert!(rich
        .preview_fields(&[], &[field("telemetry.sensors.order", "[]")])
        .unwrap_err()
        .to_string()
        .contains("mixes.count"));
}

#[test]
fn default_channel_expansion_and_compaction_match_firmware() {
    let model = from("default");
    let settings = [
        field("channels[1].name", "Flap left"),
        field("channels[1].center", "3%"),
        field("channels[1].limit_min", "-72%"),
        field("channels[1].limit_max", "83%"),
        field("channels[1].invert", "On"),
    ];
    let bytes = model.serialize_fields(&[], &settings).unwrap();
    assert_eq!(bytes, fixture("channel"));
    let reset = [
        field("channels[1].name", ""),
        field("channels[1].center", "0%"),
        field("channels[1].limit_min", "-100"),
        field("channels[1].limit_max", "100"),
        field("channels[1].invert", "Off"),
    ];
    assert_eq!(
        Model::parse(&bytes)
            .unwrap()
            .serialize_fields(&[], &reset)
            .unwrap(),
        fixture("default")
    );
    assert_eq!(
        model
            .serialize_fields(&[], &[field("channels[1].center", "0")])
            .unwrap(),
        fixture("default")
    );
    for (key, bad) in [
        ("limit_min", "-150.1"),
        ("limit_max", "150.1"),
        ("center", "-100.1"),
        ("center", "0.123"),
    ] {
        assert!(model
            .preview_fields(&[], &[field(&format!("channels[1].{key}"), bad)])
            .is_err());
    }
}

#[test]
fn additional_verified_controls_match_firmware_bytes() {
    let model = from("default");
    for (name, changes) in [
        ("rf", vec![field("rf.modules[0].enabled", "On")]),
        ("trim", vec![field("trims[0].step", "Medium")]),
        (
            "timer-limits",
            vec![
                field("timers[1].start_value", "-2147483648"),
                field("timers[1].alarm_value", "2147483647"),
            ],
        ),
        (
            "output-limits",
            vec![
                field("channels[1].invert", "On"),
                field("channels[1].center", "-7.5%"),
                field("channels[1].limit_min", "-150%"),
                field("channels[1].limit_max", "150%"),
            ],
        ),
    ] {
        assert_eq!(
            model.serialize_fields(&[], &changes).unwrap(),
            fixture(name),
            "{name}"
        );
    }
    assert_eq!(
        from("rf")
            .serialize_fields(&[], &[field("rf.modules[0].enabled", "Off")])
            .unwrap(),
        fixture("default")
    );
    let mixed = from("free-mix");
    let paths = [
        field("mixes[0].name", "Bench free"),
        field("mixes[0].actions[0].weight", "-100"),
    ];
    let reopened = Model::parse(&mixed.serialize_fields(&[], &paths).unwrap()).unwrap();
    for edit in paths {
        assert!(reopened
            .fields()
            .iter()
            .any(|f| f.path == edit.path && f.value == edit.value));
    }
}

#[test]
fn unchanged_indistinguishable_rows_do_not_create_a_false_difference() {
    let source = from("id-conflict");
    let mut bytes = source.to_bytes();
    let offset = source.sensors()[0].range.start + 6;
    bytes[offset..offset + 2].copy_from_slice(&0x6803u16.to_le_bytes());
    let same = Model::parse(&damaged(bytes)).unwrap();
    assert!(compare(&same, &same).is_empty());
    let named = Model::parse(
        &same
            .serialize_fields(&[], &[field("model.name", "Changed")])
            .unwrap(),
    )
    .unwrap();
    assert!(compare(&same, &named)
        .iter()
        .all(|change| change.kind != DiffKind::Ambiguous));
}

#[test]
fn longer_label_keeps_unknown_widget_bytes_and_moves_its_warning() {
    let mut raw = fixture("throttle-after");
    let valid = Model::parse(&raw).unwrap();
    let widget = valid
        .fields()
        .into_iter()
        .find(|f| f.path == "screens[0].widgets[1].key")
        .unwrap();
    raw[widget.range.start] = 0xfe;
    let model = Model::parse(&damaged(raw)).unwrap();
    let before = model
        .fields()
        .into_iter()
        .find(|f| f.path == "model.after_telemetry")
        .unwrap();
    let data = model
        .serialize_fields(&[], &[field("model.name", "Bench longer")])
        .unwrap();
    let updated = Model::parse(&data).unwrap();
    let after = updated
        .fields()
        .into_iter()
        .find(|f| f.path == "model.after_telemetry")
        .unwrap();
    assert!(after.range.start > before.range.start);
    assert_eq!(updated.raw(after.range), model.raw(before.range));
    assert_eq!(updated.warnings().len(), 1);
    assert!(updated.warnings()[0].contains("screens[0].widgets[1].key"));
    assert!(updated
        .fields()
        .iter()
        .any(|f| f.path == "model.name" && f.value == "Bench longer"));
}

#[test]
fn concurrent_publish_never_replaces_output() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("one.bin");
    let results: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = [0x6802, 0x6801]
            .into_iter()
            .map(|id| {
                let dst: &PathBuf = &path;
                scope.spawn(move || from("throttle-before").save_copy(dst, &[change(0, id)]))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(results.iter().filter(|v| v.is_ok()).count(), 1);
    assert!(Model::open(&path).is_ok());
}
