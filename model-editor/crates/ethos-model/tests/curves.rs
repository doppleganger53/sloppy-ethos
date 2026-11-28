use ethos_model::{compare, crc16, FieldEdit, Model, Value};
use std::{fs, path::Path};
fn bytes(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(format!("{name}.bin")),
    )
    .unwrap()
}
fn open(name: &str) -> Model {
    let m = Model::parse(&bytes(name)).unwrap();
    assert!(m.warnings().is_empty(), "{name}: {:?}", m.warnings());
    m
}
fn field(m: &Model, path: &str) -> ethos_model::Field {
    m.fields()
        .into_iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("missing {path}"))
}
fn set(path: &str, value: &str) -> FieldEdit {
    FieldEdit {
        path: path.into(),
        value: value.into(),
    }
}
const POINTS: &str = "curves[0].points";

#[test]
fn switch_grid_resize_and_smooth_in_one_batch_matches_native_files() {
    for (left, right) in [
        ("curve-easy", "curve-maximum"),
        ("curve-maximum", "curve-easy"),
        ("curve-custom", "curve-smooth"),
        ("curve-smooth", "curve-custom"),
        ("curve-referenced", "curve-ref-five"),
        ("curve-ref-five", "curve-ref-two"),
        ("curve-ref-two", "curve-referenced"),
    ] {
        let target = open(right);
        let old = open(left);
        let edits = [
            set(POINTS, &field(&target, POINTS).value),
            set(
                "curves[0].smooth",
                &field(&target, "curves[0].smooth").value,
            ),
        ];
        let data = old
            .serialize_fields(&[], &edits)
            .unwrap_or_else(|e| panic!("{left} -> {right}: {e}"));
        assert_eq!(data, bytes(right), "{left} -> {right}");
        let diff = compare(&old, &target);
        assert!(diff.iter().any(|d| d.path.starts_with("curves[0].points[")));
        assert!(
            diff.iter().all(|d| d.path != POINTS),
            "summary duplicated individual points"
        );
    }
}

#[test]
fn points_validate_bounds_precision_endpoints_and_step_order_before_publishing() {
    let model = open("curve-custom");
    let scratch = tempfile::tempdir().unwrap();
    for invalid in [
        "[]",
        "[0]",
        "[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21]",
        "[0,100.1]",
        "[0,-100.1]",
        "[0,2.01]",
        "[0,{}]",
        r#"[{"x":-99,"y":0},{"x":100,"y":1}]"#,
        r#"[{"x":-100,"y":0},{"x":99.5,"y":1},{"x":100,"y":2}]"#,
        r#"[{"x":-100,"y":0},{"x":30,"y":0},{"x":-30,"y":0},{"x":100,"y":2}]"#,
    ] {
        let out = scratch.path().join("invalid.bin");
        let error = model
            .save_fields_copy(&out, &[], &[set(POINTS, invalid)])
            .unwrap_err();
        assert!(error.to_string().contains(POINTS), "{error}");
        assert!(!out.exists());
    }
    let mut irregular = bytes("curve-custom");
    let x = field(&model, "curves[0].points[0].x").range;
    assert_eq!(x.len(), 2);
    irregular[x.end - 1] = (-77i8) as u8;
    let crc = crc16(&irregular[16..]);
    irregular[14..16].copy_from_slice(&crc.to_le_bytes());
    let model = Model::parse(&irregular).unwrap();
    assert!(!field(&model, POINTS).editable);
    assert_eq!(model.serialize_fields(&[], &[]).unwrap(), irregular);
}

#[test]
fn expo_and_function_show_raw_enums_and_exact_numeric_scales() {
    let model = open("curve-expo");
    assert_eq!(
        field(&model, "curves[2].offset").typed,
        Value::FixedPoint {
            raw: -125,
            scale: 10
        }
    );
    let data = model
        .serialize_fields(
            &[],
            &[
                set("curves[2].offset", "125.5%"),
                set("curves[2].exponent", "125"),
                set("curves[2].weight", "-125"),
            ],
        )
        .unwrap();
    let reopened = Model::parse(&data).unwrap();
    assert_eq!(field(&reopened, "curves[2].offset").value, "125.5%");
    assert!(!field(&open("curve-functions"), "curves[0].offset").editable);
    let modes = open("curve-functions");
    for i in 0..7 {
        let name = field(&modes, &format!("curves[{i}].function")).value;
        let data = modes
            .serialize_fields(&[], &[set("curves[0].function", &name)])
            .unwrap();
        let m = Model::parse(&data).unwrap();
        assert_eq!(field(&m, "curves[0].function").value, name);
    }
}

#[test]
fn library_moves_and_deletion_match_native_default_and_conditional_pointers() {
    let ref_path = "mixes[0].actions[1].curve.default";
    let rate_path = "mixes[0].actions[1].curve.rates[0].curve";
    for (before, order, after) in [
        ("curve-library", "[1,0,2,3,4]", "curve-library-moved-one"),
        ("curve-library", "[1,2,3,4,0]", "curve-library-moved"),
        ("curve-library-moved", "[0,1,2,3]", "curve-library-deleted"),
    ] {
        let model = open(before);
        let expected = open(after);
        let edit = set("curves.order", order);
        let plan = model
            .preview_fields(&[], std::slice::from_ref(&edit))
            .unwrap();
        for path in [ref_path, rate_path] {
            assert!(
                plan.iter()
                    .any(|ch| ch.path == path && ch.after == field(&expected, path).value),
                "{path}: {plan:?}"
            );
        }
        assert_eq!(
            model.serialize_fields(&[], &[edit]).unwrap(),
            bytes(after),
            "{before} -> {after}"
        );
    }
    let m = open("curve-library");
    assert!(m
        .preview_fields(&[], &[set("curves.order", "[4,4]")])
        .is_err());
    assert!(m
        .preview_fields(
            &[],
            &[
                set("curves[0].name", "Discarded"),
                set("curves.order", "[1,2]")
            ]
        )
        .unwrap_err()
        .to_string()
        .contains("curves[0].name"));
    let conflict = m
        .preview_fields(
            &[],
            &[set(ref_path, "Curve slot 1"), set("curves.order", "[0]")],
        )
        .unwrap_err();
    assert!(conflict.to_string().contains(ref_path));
    let src = field(&m, ref_path);
    assert_eq!(
        src.typed,
        Value::Reference {
            collection: "curves".into(),
            slot: 0
        }
    );
    assert!(m
        .preview_fields(&[], &[set(ref_path, "Curve slot 900")])
        .is_err());
    assert_eq!(
        open("curve-referenced")
            .serialize_fields(&[], &[set(ref_path, "Expo 0")])
            .unwrap(),
        bytes("curve-mix-inline")
    );
    assert_eq!(
        open("curve-mix-inline")
            .serialize_fields(&[], &[set(ref_path, "Curve slot 0")])
            .unwrap(),
        bytes("curve-referenced")
    );
}
