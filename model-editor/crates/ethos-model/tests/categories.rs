use ethos_model::{compare, crc16, Edit, FieldEdit, Model, Value};
use std::{fs, path::Path};
fn bytes(label: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("category-{label}.bin")),
    )
    .unwrap()
}
fn seal(b: &mut [u8]) {
    let c = crc16(&b[16..]);
    b[14..16].copy_from_slice(&c.to_le_bytes());
}
fn edit(path: &str, value: &str) -> FieldEdit {
    FieldEdit {
        path: path.into(),
        value: value.into(),
    }
}
#[test]
fn stock_and_two_independent_sensor_layouts_decode_for_each_model_category() {
    for (id, label, mixes) in [
        ("glider", "Glider", 3),
        ("heli", "Helicopter", 6),
        ("multi", "Multirotor", 5),
        ("other", "Other", 0),
    ] {
        let base = Model::parse(&bytes(id)).unwrap();
        assert!(base.warnings().is_empty(), "{id}: {:?}", base.warnings());
        assert_eq!(base.field("model.type").unwrap().value, label);
        assert_eq!(
            base.field("mixes.count").unwrap().typed,
            Value::Integer(mixes)
        );
        assert!(base.sensors().is_empty());
        assert_eq!(base.serialize_fields(&[], &[]).unwrap(), bytes(id));
        let before = bytes(&format!("{id}-before"));
        let expected = bytes(&format!("{id}-after"));
        let initial = Model::parse(&before).unwrap();
        let target = Model::parse(&expected).unwrap();
        assert!(initial.warnings().is_empty() && target.warnings().is_empty());
        assert_eq!(
            initial
                .sensors()
                .iter()
                .map(|s| s.sub_id)
                .collect::<Vec<_>>(),
            [0, 1, 2, 4]
        );
        assert_eq!(
            compare(&initial, &target)
                .iter()
                .filter(|d| d.path.ends_with(".application_id")
                    && d.before.as_deref() == Some("6803")
                    && d.after.as_deref() == Some("6802"))
                .count(),
            4
        );
        let batch: Vec<_> = (0..4)
            .map(|slot| Edit {
                slot,
                physical_id: None,
                application_id: Some(0x6802),
            })
            .collect();
        assert_eq!(
            initial.serialize_fields(&batch, &[]).unwrap(),
            expected,
            "{id}"
        );
        let physical = initial
            .serialize_fields(
                &[Edit {
                    slot: 0,
                    physical_id: Some(0x1b),
                    application_id: None,
                }],
                &[],
            )
            .unwrap();
        let reparsed = Model::parse(&physical).unwrap();
        assert!(reparsed.warnings().is_empty());
        assert_eq!(reparsed.sensors()[0].physical_id, 0x1b);
        let ids = initial.sensors()[0].range.start + 5;
        assert!(before
            .iter()
            .zip(&physical)
            .enumerate()
            .all(|(i, (x, y))| x == y || i == 14 || i == 15 || i == ids));
        assert!(initial
            .fields()
            .iter()
            .filter(|f| f.editable)
            .all(|f| f.path.ends_with(".physical_id") || f.path.ends_with(".application_id")));
        assert!(initial
            .preview_fields(&[], &[edit("model.name", "Different")])
            .unwrap_err()
            .to_string()
            .contains("model.name"));
        assert!(initial
            .preview_fields(&[], &[edit("telemetry.sensors.order", "[1,0,2,3]")])
            .unwrap_err()
            .to_string()
            .contains("model.type"));
    }
}
#[test]
fn unsupported_category_or_unverified_mixer_is_inspectable_and_cannot_lend_offsets_to_a_writer() {
    let mut unknown = bytes("heli-before");
    unknown[16] = 0xfd;
    seal(&mut unknown);
    let m = Model::parse(&unknown).unwrap();
    assert!(m.warnings().iter().any(|w| w.contains("model.type")));
    assert!(m.sensors().is_empty());
    let mut data = bytes("multi-before");
    let m = Model::parse(&data).unwrap();
    let r = m.field("mixes[0].key").unwrap().range.clone();
    data[r].copy_from_slice(&0xabcd_u32.to_le_bytes());
    seal(&mut data);
    let blocked = Model::parse(&data).unwrap();
    assert!(blocked.warnings().iter().any(|w| w.contains("mixes[0]")));
    assert!(blocked.sensors().is_empty());
    assert!(blocked.fields().iter().all(|f| !f.editable));
    assert_eq!(blocked.serialize_fields(&[], &[]).unwrap(), data);
}

#[test]
fn counted_glider_helicopter_and_multirotor_mix_settings_follow_the_native_wizard() {
    let glider = Model::parse(&bytes("glider-flaps")).unwrap();
    assert_eq!(
        glider.field("model.flaps_count").unwrap().typed,
        Value::Integer(4)
    );
    let heli = Model::parse(&bytes("heli-swash")).unwrap();
    assert_eq!(
        heli.field("model.swash_type").unwrap().typed,
        Value::Enum {
            raw: 4,
            label: Some("Three servos, 135 degrees".into())
        }
    );
    for (name, path, wanted) in [
        ("glider-flaps", "mixes[3].channels.count", 4),
        ("glider-flaps", "mixes[3].weights[3]", 30),
        ("heli-swash", "mixes[4].channels.count", 3),
        ("heli-swash", "mixes[4].weights[2]", 50),
        ("multi-channel", "mixes[0].channels[0].index", 6),
    ] {
        let m = Model::parse(&bytes(name)).unwrap();
        assert!(m.warnings().is_empty(), "{name}: {:?}", m.warnings());
        assert_eq!(m.field(path).unwrap().typed, Value::Integer(wanted));
    }
    let mut stock = bytes("multi");
    let model = Model::parse(&stock).unwrap();
    let flags = model
        .fields()
        .into_iter()
        .find(|f| f.path.ends_with(".cut.sticky"))
        .unwrap();
    stock[flags.range.start] ^= 0xfe;
    seal(&mut stock);
    let changed = Model::parse(&stock).unwrap();
    assert!(changed.warnings().is_empty());
    assert_eq!(changed.field(&flags.path).unwrap().typed, flags.typed);
    assert_eq!(changed.serialize_fields(&[], &[]).unwrap(), stock);
}
