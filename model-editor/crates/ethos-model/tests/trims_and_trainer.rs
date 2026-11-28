use ethos_model::{FieldEdit, Model, Value};
use std::{fs, path::Path};
fn fixture(id: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("{id}.bin")),
    )
    .unwrap()
}
fn model(id: &str) -> Model {
    let m = Model::parse(&fixture(id)).unwrap();
    assert!(m.warnings().is_empty(), "{id}: {:?}", m.warnings());
    m
}
fn field(path: &str, value: impl Into<String>) -> FieldEdit {
    FieldEdit {
        path: path.into(),
        value: value.into(),
    }
}
fn reproduce(from: &str, to: &str, paths: &[&str]) {
    let target = model(to);
    let edits: Vec<_> = paths
        .iter()
        .map(|path| field(path, &target.field(path).unwrap().value))
        .collect();
    assert_eq!(
        model(from)
            .serialize_fields(&[], &edits)
            .unwrap_or_else(|e| panic!("{from} => {to}: {e}")),
        fixture(to),
        "{from} => {to}"
    );
}
#[test]
fn independent_extra_trim_controls_preserve_the_settings_that_follow() {
    for (file, path) in [
        ("extra-trim-name", "extra_trims[0].name"),
        ("extra-trim-up", "extra_trims[0].trigger_up"),
        ("extra-trim-down", "extra_trims[0].trigger_down"),
        ("extra-trim-medium", "extra_trims[0].step"),
        ("extra-trim-range-min", "extra_trims[0].range"),
        ("extra-trim-range-max", "extra_trims[0].range"),
    ] {
        reproduce("extra-trim", file, &[path]);
        reproduce(file, "extra-trim", &[path]);
    }
    for target in ["trim-range-min", "trim-range-max"] {
        reproduce("default", target, &["trims[0].range"]);
    }
    let m = model("extra-trim");
    assert_eq!(
        m.field("extra_trims.count").unwrap().typed,
        Value::Integer(1)
    );
    assert!(m
        .preview_fields(&[], &[field("extra_trims[0].range", "101")])
        .is_err());
    assert!(m
        .preview_fields(
            &[],
            &[field("extra_trims[0].trigger_up", "unverified control")]
        )
        .is_err());
    assert!(m
        .preview_fields(&[], &[field("curves.order", "[]")])
        .unwrap_err()
        .to_string()
        .contains("extra_trims.count"));
}
#[test]
fn native_slave_ranges_and_master_controls_are_shared_by_all_stored_links() {
    for base in ["trainer", "trainer-slave-cable"] {
        for link in ["bluetooth", "sbus", "cppm"] {
            let id = if base == "trainer" {
                format!("trainer-master-{link}")
            } else {
                format!("trainer-slave-{link}")
            };
            reproduce(base, &id, &["trainer.link"]);
            reproduce(&id, base, &["trainer.link"]);
        }
    }
    for id in [
        "trainer-slave-span",
        "trainer-slave-upper-start",
        "trainer-slave-count-min",
        "trainer-slave-count-max",
    ] {
        reproduce("trainer-slave-cable", id, &["trainer.channel_range"]);
    }
    for (id, path) in [
        ("trainer-master-weight-min", "trainer.channels[0].weight"),
        ("trainer-master-weight-max", "trainer.channels[0].weight"),
        ("trainer-master-channel-off", "trainer.channels[0].mode"),
        ("trainer-master-channel-add", "trainer.channels[0].mode"),
        ("trainer-master-condition", "trainer.condition"),
        (
            "trainer-master-destination",
            "trainer.channels[0].destination",
        ),
    ] {
        reproduce("trainer", id, &[path]);
    }
    let trainer = model("trainer");
    let slave = model("trainer-slave-cable");
    for invalid in ["-126", "126"] {
        assert!(trainer
            .preview_fields(&[], &[field("trainer.channels[0].weight", invalid)])
            .is_err());
    }
    for invalid in ["CH1-CH3", "CH1-CH17", "CH63-CH66"] {
        assert!(slave
            .preview_fields(&[], &[field("trainer.channel_range", invalid)])
            .is_err());
    }
    assert!(trainer
        .preview_fields(
            &[],
            &[field("trainer.channels[0].destination", "Analog 99")]
        )
        .is_err());
    assert!(trainer
        .preview_fields(&[], &[field("trainer.link", "undiscovered protocol")])
        .is_err());
}

#[test]
fn per_flight_custom_and_off_trims_keep_the_following_telemetry_at_its_real_boundary() {
    for (id, raw) in [
        ("extra-trim-flight", 1),
        ("extra-trim-custom", 2),
        ("extra-trim-off", 3),
    ] {
        let m = model(id);
        assert!(
            matches!(m.field("extra_trims[0].mode").unwrap().typed,Value::Enum{raw:x,..} if x==raw)
        );
        assert_eq!(m.serialize_fields(&[], &[]).unwrap(), fixture(id));
    }
    let two = model("extra-trim-two");
    assert_eq!(two.field("extra_trims.count").unwrap().value, "2");
    assert_eq!(two.field("extra_trims[0].name").unwrap().value, "AZ");
    for root in ["extra-trim-sensors", "trainer-trim-sensors"] {
        let source = model(&format!("{root}-before"));
        assert_eq!(source.sensors().len(), 4);
        let props: Vec<_> = (0..4)
            .map(|slot| ethos_model::Edit {
                slot,
                physical_id: None,
                application_id: Some(0x6802),
            })
            .collect();
        assert_eq!(
            source.serialize_fields(&props, &[]).unwrap(),
            fixture(&format!("{root}-after"))
        );
    }
}
