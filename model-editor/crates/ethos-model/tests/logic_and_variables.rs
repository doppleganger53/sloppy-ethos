use ethos_model::{FieldEdit, Model};
use std::{fs, path::Path};
fn raw(id: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("{id}.bin")),
    )
    .unwrap()
}
fn model(id: &str) -> Model {
    let out = Model::parse(&raw(id)).unwrap();
    assert!(out.warnings().is_empty(), "{id} {:?}", out.warnings());
    out
}
fn set(path: &str, value: impl Into<String>) -> FieldEdit {
    FieldEdit {
        path: path.into(),
        value: value.into(),
    }
}
fn reproduce(src: &str, target: &str, paths: &[&str]) {
    let wanted = model(target);
    let edits: Vec<_> = paths
        .iter()
        .map(|p| set(p, wanted.field(p).unwrap().value.clone()))
        .collect();
    assert_eq!(
        model(src)
            .serialize_fields(&[], &edits)
            .unwrap_or_else(|e| panic!("{src}->{target}: {e}")),
        raw(target),
        "{src}->{target}"
    );
}
#[test]
fn native_logic_operators_sources_durations_and_boundaries() {
    for (left, right, paths) in [
        ("logic-and", "logic-or", vec!["logic_switches[0].key"]),
        ("logic-or", "logic-xor", vec!["logic_switches[0].key"]),
        (
            "logic-and",
            "logic-and-three",
            vec!["logic_switches[0].values"],
        ),
        (
            "logic-and",
            "logic-and-fs1",
            vec!["logic_switches[0].values"],
        ),
        (
            "logic-and-three",
            "logic-and-fs1",
            vec!["logic_switches[0].values[2]"],
        ),
        (
            "logic-and-fs1",
            "logic-and",
            vec!["logic_switches[0].values"],
        ),
        (
            "logic-eq",
            "logic-lt",
            vec!["logic_switches[0].key", "logic_switches[0].values[1]"],
        ),
        (
            "logic-lt",
            "logic-gt",
            vec!["logic_switches[0].key", "logic_switches[0].values[1]"],
        ),
        (
            "logic-gt",
            "logic-gt-min",
            vec!["logic_switches[0].values[1]"],
        ),
        (
            "logic-gt-min",
            "logic-gt-max",
            vec!["logic_switches[0].values[1]"],
        ),
        (
            "logic-timer",
            "logic-timer-changed",
            vec!["logic_switches[0].values[0]", "logic_switches[0].values[1]"],
        ),
        (
            "logic-timer",
            "logic-timer-bounds",
            vec!["logic_switches[0].values[0]", "logic_switches[0].values[1]"],
        ),
        (
            "logic-eq",
            "logic-inverted",
            vec!["logic_switches[0].key", "logic_switches[0].negative"],
        ),
    ] {
        reproduce(left, right, &paths);
    }
    reproduce(
        "logic-eq",
        "logic-delay-on",
        &["logic_switches[0].key", "logic_switches[0].delay_on"],
    );
    reproduce(
        "logic-delay-on",
        "logic-delay-off",
        &["logic_switches[0].delay_on", "logic_switches[0].delay_off"],
    );
    let op = model("logic-delay-on");
    for name in ["delay_on", "delay_off"] {
        let path = format!("logic_switches[0].{name}");
        let high = op.serialize_fields(&[], &[set(&path, "60.0 s")]).unwrap();
        let check = Model::parse(&high).unwrap();
        assert_eq!(check.field(&path).unwrap().value, "60 s");
        assert!(op.preview_fields(&[], &[set(&path, "60.1")]).is_err());
    }
    let timer = model("logic-timer");
    for value in ["0", "175.1", "1.24"] {
        assert!(timer
            .preview_fields(&[], &[set("logic_switches[0].values[0]", value)])
            .is_err());
    }
    let m = model("logic-and");
    for value in [
        r#"["SA Up"]"#,
        r#"["SA Up","Not a switch"]"#,
        r#"[null,null,null,null,null,null,null,null,null,null]"#,
    ] {
        assert!(m
            .preview_fields(&[], &[set("logic_switches[0].values", value)])
            .is_err());
    }
    assert!(m
        .preview_fields(&[], &[set("logic_switches[0].key", "Timer")])
        .is_err());
}
#[test]
fn native_variable_rates_actions_and_trim_targets() {
    for id in [
        "var-rate-empty",
        "var-rate-switch",
        "var-rate-two",
        "var-action-empty",
        "var-action-switch",
        "var-action-two",
        "var-combined",
        "var-assign",
        "var-subtract",
        "var-multiply",
        "var-divide",
        "var-percent",
        "var-minimum",
        "var-maximum",
        "var-repurpose",
        "var-repurpose-elevator",
        "var-step-min",
        "var-step-max",
    ] {
        reproduce(
            "variable",
            id,
            &["variables[0].rates", "variables[0].actions"],
        );
        reproduce(
            id,
            "variable",
            &["variables[0].rates", "variables[0].actions"],
        );
    }
    reproduce(
        "var-action-switch",
        "var-assign",
        &["variables[0].actions[0].function"],
    );
    reproduce(
        "var-repurpose",
        "var-repurpose-elevator",
        &[
            "variables[0].actions[0].source",
            "variables[0].actions[0].step",
        ],
    );
    let m = model("var-repurpose");
    assert!(m
        .preview_fields(&[], &[set("variables[0].actions[0].source", "Trim 999")])
        .is_err());
    assert!(m
        .preview_fields(&[], &[set("variables[0].actions[0].step", "1001")])
        .is_err());
    let src = model("variable");
    let changed = src
        .serialize_fields(
            &[],
            &[
                set("variables[0].minimum", "-2000"),
                set("variables[0].maximum", "2000"),
                set("variables[0].default", "1500"),
            ],
        )
        .unwrap();
    let updated = Model::parse(&changed).unwrap();
    assert_eq!(updated.field("variables[0].default").unwrap().value, "1500");
    assert!(src
        .preview_fields(&[], &[set("variables[0].default", "2001")])
        .is_err());
    for (key, onemax) in [("variables[0].rates", 10), ("variables[0].actions", 11)] {
        let row = if key.ends_with("rates") {
            serde_json::json!({"condition":null,"value":0})
        } else {
            serde_json::json!({"condition":null,"function":"Add","source":0})
        };
        let too_many = serde_json::to_string(&vec![row; onemax]).unwrap();
        assert!(src
            .preview_fields(&[], &[set(key, too_many)])
            .unwrap_err()
            .to_string()
            .contains(key));
    }
}
