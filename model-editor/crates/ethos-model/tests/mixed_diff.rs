use ethos_model::{compare, crc16, DiffKind, Model};
use std::{fs, path::Path};

fn fixture(name: &str) -> Model {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(format!("{name}.bin"));
    Model::parse(&fs::read(path).unwrap()).unwrap()
}

fn field_range(model: &Model, path: &str) -> std::ops::Range<usize> {
    model
        .fields()
        .into_iter()
        .find(|f| f.path == path)
        .unwrap()
        .range
}

fn first_record(model: &Model) -> Vec<u8> {
    let start = field_range(model, "telemetry.sensors[0].variant").start;
    let next = model
        .fields()
        .iter()
        .find(|f| f.path == "telemetry.sensors[1].variant")
        .map(|f| f.range.start)
        .unwrap_or_else(|| field_range(model, "telemetry.competition").start);
    model.raw(start..next).unwrap().to_vec()
}

// Assemble a valid mixed collection from complete records emitted by the native
// firmware. These fixtures preserve the original record encodings and vary their order.
fn mixed(records: &[&[u8]]) -> Model {
    let model = fixture("default");
    let count = field_range(&model, "telemetry.sensors.count");
    let source = model.to_bytes();
    let mut bytes = source[..count.start].to_vec();
    bytes.extend((records.len() as u16).to_le_bytes());
    for record in records {
        bytes.extend_from_slice(record);
    }
    bytes.extend_from_slice(&source[count.end..]);
    let size = (bytes.len() - 16) as u32;
    bytes[8..12].copy_from_slice(&size.to_le_bytes());
    let crc = crc16(&bytes[16..]);
    bytes[14..16].copy_from_slice(&crc.to_le_bytes());
    let result = Model::parse(&bytes).unwrap();
    assert!(result.warnings().is_empty());
    result
}

#[test]
fn all_decoded_telemetry_types_participate_in_relative_order() {
    let sport = first_record(&fixture("throttle-before"));
    let calculated = first_record(&fixture("calculated-consumption"));
    let diy = first_record(&fixture("diy"));
    let before = mixed(&[&sport, &calculated, &diy]);
    let after = mixed(&[&diy, &sport, &calculated]);
    let delta: Vec<_> = compare(&before, &after)
        .into_iter()
        .filter(|d| d.section == "Telemetry")
        .collect();
    assert_eq!(delta.len(), 3, "{delta:?}");
    for (d, (old, new)) in delta.iter().zip([("0", "1"), ("1", "2"), ("2", "0")]) {
        assert_eq!(d.kind, DiffKind::Moved);
        assert_eq!(
            (d.before.as_deref(), d.after.as_deref()),
            (Some(old), Some(new))
        );
    }
    // An insertion shifts absolute slots but preserves the relative order of existing rows.
    let appended = compare(&mixed(&[&sport, &calculated]), &after);
    let rows: Vec<_> = appended
        .iter()
        .filter(|d| d.section == "Telemetry" && d.path != "telemetry.sensors.count")
        .collect();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].kind, DiffKind::Added);
    assert_eq!(rows[0].path, "Bench DIY");

    // A new sensor can cross a stationary record; that record did not move from slot 1 to itself.
    let changed = compare(
        &mixed(&[&sport, &calculated]),
        &mixed(&[&diy, &calculated, &sport]),
    );
    let moved: Vec<_> = changed
        .iter()
        .filter(|d| d.kind == DiffKind::Moved)
        .collect();
    assert_eq!(moved.len(), 1, "{moved:?}");
    assert_eq!(moved[0].before.as_deref(), Some("0"));
    assert_eq!(moved[0].after.as_deref(), Some("2"));
}

#[test]
fn differences_in_indistinguishable_calculated_rows_report_ambiguity() {
    let custom = fixture("calculated-custom");
    let operation = fixture("calculated-operation");
    let original_a = first_record(&custom);
    let original_b = first_record(&operation);
    let mut changed_a = original_a.clone();
    let mut changed_b = original_b.clone();
    for (model, bytes) in [(&custom, &mut changed_a), (&operation, &mut changed_b)] {
        let base = field_range(model, "telemetry.sensors[0].variant").start;
        let decimals = field_range(model, "telemetry.sensors[0].display_decimals").start;
        bytes[decimals - base] = 2;
    }
    let before = mixed(&[&original_a, &original_b]);
    let after = mixed(&[&changed_b, &changed_a]);
    let delta: Vec<_> = compare(&before, &after)
        .into_iter()
        .filter(|d| d.section == "Telemetry")
        .collect();
    assert_eq!(delta.len(), 1, "{delta:?}");
    assert_eq!(delta[0].kind, DiffKind::Ambiguous);
    assert_eq!(delta[0].path, "Custom");
}

fn renamed_record(model: &Model, name: &str) -> Vec<u8> {
    let mut record = first_record(model);
    let start = field_range(model, "telemetry.sensors[0].variant").start;
    let label = field_range(model, "telemetry.sensors[0].name");
    let mut encoded = vec![u8::try_from(name.len()).unwrap()];
    encoded.extend(name.as_bytes());
    record.splice(label.start - start..label.end - start, encoded);
    record
}

#[test]
fn a_rename_requires_a_unique_unchanged_key_across_both_complete_models() {
    let source = fixture("throttle-before");
    let old = first_record(&source);
    let new = renamed_record(&source, "Current probe");
    let result = compare(&mixed(&[&old]), &mixed(&[&new]));
    let rows: Vec<_> = result.iter().filter(|d| d.section == "Telemetry").collect();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].kind, DiffKind::Changed);
    assert!(rows[0].path.ends_with(".name"));
    assert_eq!(rows[0].before.as_deref(), Some("Throttle amps"));
    assert_eq!(rows[0].after.as_deref(), Some("Current probe"));

    // The untouched label has already been paired. It still makes this full key ambiguous.
    let other = renamed_record(&source, "Existing twin");
    for (before, after) in [
        (mixed(&[&old, &other]), mixed(&[&new, &other])),
        (mixed(&[&old, &other]), mixed(&[&new])),
        (mixed(&[&old]), mixed(&[&new, &other])),
    ] {
        let delta = compare(&before, &after);
        assert!(
            delta.iter().any(|d| d.kind == DiffKind::Ambiguous),
            "{delta:?}"
        );
        assert!(!delta
            .iter()
            .any(|d| d.path.ends_with(".name") && d.kind == DiffKind::Changed));
    }
}
