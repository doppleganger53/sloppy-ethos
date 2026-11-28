use ethos_model::{crc16, Edit, Field, FieldEdit, Model, Value};
use std::{fs, path::Path};

fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("{name}.bin")),
    )
    .unwrap()
}
fn field<'a>(fields: &'a [Field], path: &str) -> &'a Field {
    fields
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("missing {path}"))
}
fn range_edit(value: &str) -> FieldEdit {
    FieldEdit {
        path: "rf.modules[1].channel_range".into(),
        value: value.into(),
    }
}

#[test]
fn radio_profiles_expose_the_confirmed_labels_and_preserve_downstream_telemetry() {
    for (case, slot, module, protocol) in [
        (
            "rf-internal-access",
            0,
            "X20 Pro AW internal module",
            "ACCESS/ACCST D16",
        ),
        ("rf-internal-td", 0, "X20 Pro AW internal module", "TD Mode"),
        ("rf-internal-tw", 0, "X20 Pro AW internal module", "TW Mode"),
        ("rf-external-twin", 1, "Twin Lite Pro", "ACCESS"),
        ("rf-twin-accst", 1, "Twin Lite Pro", "ACCST D16"),
        ("rf-twin-elrs", 1, "Twin Lite Pro", "ELRS"),
        ("rf-twin-tw", 1, "Twin Lite Pro", "TW Mode"),
        ("rf-external-xjt", 1, "XJT Lite", "D16"),
        ("rf-xjt-d8", 1, "XJT Lite", "D8"),
        ("rf-xjt-lr12", 1, "XJT Lite", "LR12"),
        ("rf-external-r9-lite", 1, "R9M Lite", "FCC"),
        ("rf-r9-eu", 1, "R9M Lite", "EU"),
        ("rf-r9-flex868", 1, "R9M Lite", "FLEX 868MHz"),
        ("rf-r9-flex915", 1, "R9M Lite", "FLEX 915MHz"),
        ("rf-external-r9-access", 1, "R9M Lite ACCESS", "ACCESS"),
        (
            "rf-external-r9-pro-access",
            1,
            "R9M Lite Pro ACCESS",
            "ACCESS",
        ),
        ("rf-external-ppm", 1, "PPM", "0"),
        ("rf-external-sbus", 1, "SBUS", "0"),
    ] {
        let bytes = fixture(case);
        let m = Model::parse(&bytes).unwrap();
        assert!(m.warnings().is_empty(), "{case}: {:?}", m.warnings());
        let fields = m.fields();
        let root = format!("rf.modules[{slot}]");
        assert_eq!(field(&fields, &format!("{root}.type")).value, module);
        let selected = field(&fields, &format!("{root}.protocol"));
        assert_eq!(selected.value, protocol);
        assert!(!selected.editable);
        assert_eq!(m.sensors().len(), 4);
        let ids: Vec<_> = m
            .sensors()
            .iter()
            .map(|s| (s.physical_id, s.application_id, s.sub_id))
            .collect();
        assert_eq!(
            ids,
            [
                (3, 0x6803, 0),
                (3, 0x6803, 1),
                (3, 0x6803, 2),
                (3, 0x6803, 4)
            ]
        );
        let edits: Vec<_> = m
            .sensors()
            .iter()
            .map(|s| Edit {
                slot: s.slot,
                physical_id: None,
                application_id: Some(0x6802),
            })
            .collect();
        let result = m.serialize_fields(&edits, &[]).unwrap();
        let offsets: Vec<_> = bytes
            .iter()
            .zip(&result)
            .enumerate()
            .filter_map(|(i, (old, new))| (i >= 16 && old != new).then_some(i))
            .collect();
        assert_eq!(
            offsets,
            m.sensors()
                .iter()
                .map(|s| s.range.start + 6)
                .collect::<Vec<_>>(),
            "{case}"
        );
        assert_eq!(result.len(), bytes.len());
    }
}

#[test]
fn external_band_layout_uses_the_firmware_band_indices() {
    for (name, active0, active2) in [
        ("rf-external-twin", 1, 0),
        ("rf-twin-elrs", 0, 1),
        ("rf-twin-tw", 1, 1),
    ] {
        let m = Model::parse(&fixture(name)).unwrap();
        let fields = m.fields();
        assert_eq!(
            field(&fields, "rf.modules[1].bands[0].enabled").typed,
            Value::Integer(active0)
        );
        assert_eq!(
            field(&fields, "rf.modules[1].bands[2].enabled").typed,
            Value::Integer(active2)
        );
        assert_eq!(
            field(&fields, "rf.modules[1].bands[0].power").typed,
            Value::Integer(10)
        );
        assert!(!fields
            .iter()
            .any(|f| f.path.starts_with("rf.modules[1].bands[1]")));
    }
    let single = Model::parse(&fixture("rf-external-xjt")).unwrap().fields();
    assert!(!single
        .iter()
        .any(|f| f.path == "rf.modules[1].bands[0].enabled"));
    assert_eq!(
        field(&single, "rf.modules[1].bands[0].antenna").typed,
        Value::Integer(0)
    );
    assert_eq!(
        field(&single, "rf.modules[1].bands[0].power").typed,
        Value::Integer(10)
    );
}

#[test]
fn offline_channel_ranges_match_native_extremes_and_reject_invalid_spans() {
    for (original, edit, expected) in [
        ("rf-external-ppm", "CH1-CH4", "rf-ppm-min"),
        ("rf-external-ppm", "1-16", "rf-ppm-max"),
        ("rf-ppm-min", "ch61-ch64", "rf-ppm-last-four"),
        ("rf-external-sbus", "CH49–CH64", "rf-sbus-last"),
        ("rf-sbus-last", "1-16", "rf-external-sbus"),
    ] {
        let m = Model::parse(&fixture(original)).unwrap();
        assert_eq!(
            m.serialize_fields(&[], &[range_edit(edit)]).unwrap(),
            fixture(expected),
            "{original}: {edit}"
        );
    }
    let ppm = Model::parse(&fixture("rf-external-ppm")).unwrap();
    for bad in ["CH0-CH4", "CH1-CH3", "60-65", "16-7", "1-17", "9"] {
        assert!(ppm
            .preview_fields(&[], &[range_edit(bad)])
            .unwrap_err()
            .to_string()
            .contains("rf.modules[1].channel_range"));
    }
    let sbus = Model::parse(&fixture("rf-external-sbus")).unwrap();
    assert!(sbus.preview_fields(&[], &[range_edit("1-15")]).is_err());
}

#[test]
fn receiver_slots_preserve_stored_names_and_are_read_only() {
    let mut bytes = fixture("rf-twin-tw");
    let fields = Model::parse(&bytes).unwrap().fields();
    let first = field(&fields, "rf.modules[1].receivers[0]").range.clone();
    let third = field(&fields, "rf.modules[1].receivers[2]").range.clone();
    // Controlled synthetic names, independently reopened by 1.6.8 and decoded by 26.2.0.
    bytes[first.clone()].copy_from_slice(b"BenchRX0");
    bytes[third.clone()].copy_from_slice(b"RX 2\0\0\0\0");
    let crc = crc16(&bytes[16..]);
    bytes[14..16].copy_from_slice(&crc.to_le_bytes());
    let m = Model::parse(&bytes).unwrap();
    let fields = m.fields();
    for (slot, name) in ["BenchRX0", "", "RX 2"].iter().enumerate() {
        let f = field(&fields, &format!("rf.modules[1].receivers[{slot}]"));
        assert_eq!(f.value, *name);
        assert!(!f.editable);
    }
    let result = m
        .serialize_fields(
            &[Edit {
                slot: 0,
                physical_id: Some(4),
                application_id: None,
            }],
            &[],
        )
        .unwrap();
    assert_eq!(result[first], *b"BenchRX0");
    assert_eq!(result[third], *b"RX 2\0\0\0\0");
}

#[test]
fn xjt_failsafe_modes_remain_read_only_and_preserve_the_collection() {
    for (file, label) in [
        ("rf-external-xjt", "Not set"),
        ("rf-xjt-failsafe-hold", "Hold"),
        ("rf-xjt-failsafe-custom", "Custom"),
        ("rf-xjt-failsafe-no-pulses", "No pulses"),
        ("rf-xjt-failsafe-receiver", "Receiver"),
    ] {
        let model = Model::parse(&fixture(file)).unwrap();
        let fields = model.fields();
        let mode = field(&fields, "rf.modules[1].failsafe_mode");
        assert_eq!(mode.value, label);
        assert!(!mode.editable);
        assert_eq!(model.sensors().len(), 4);
        assert!(model.warnings().is_empty());
    }
}
