use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(format!("{name}.bin"))
}
fn cli(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ethos-model"))
        .args(args)
        .output()
        .unwrap()
}
fn json_output(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn commands_agree_on_fixture_comparison_preview_and_file() {
    let before = fixture("throttle-before");
    let after = fixture("throttle-after");
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("backup file.bin");
    fs::copy(&before, &input).unwrap();
    let output = tmp.path().join("separate repaired.bin");
    let view = json_output(cli(&[
        "inspect".as_ref(),
        input.as_ref(),
        "--json".as_ref(),
    ]));
    assert_eq!(view["sensors"].as_array().unwrap().len(), 4);
    let diffs = json_output(cli(&[
        "diff".as_ref(),
        input.as_ref(),
        after.as_ref(),
        "--json".as_ref(),
    ]));
    assert_eq!(
        diffs["differences"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["path"].as_str().unwrap().ends_with(".application_id"))
            .count(),
        4
    );
    let edits = [
        "--set", "0:-:6802", "--set", "1:-:6802", "--set", "2:-:6802", "--set", "3:-:6802",
        "--json",
    ];
    let mut args = vec!["preview".as_ref(), input.as_ref()];
    args.extend(edits.iter().map(std::ffi::OsStr::new));
    let preview = json_output(cli(&args));
    assert_eq!(preview["changes"].as_array().unwrap().len(), 4);
    let mut args = vec![
        "repair".as_ref(),
        input.as_ref(),
        "--output".as_ref(),
        output.as_ref(),
    ];
    args.extend(edits.iter().map(std::ffi::OsStr::new));
    assert_eq!(json_output(cli(&args))["changes"], preview["changes"]);
    assert_eq!(fs::read(&output).unwrap(), fs::read(&after).unwrap());
    assert!(!cli(&args).status.success());
    assert_eq!(fs::read(&input).unwrap(), fs::read(&before).unwrap());
    let bytes = json_output(cli(&[
        "bytes".as_ref(),
        input.as_ref(),
        "telemetry.sensors[0].application_id".as_ref(),
        "--json".as_ref(),
    ]));
    assert_eq!(bytes["bytes"], serde_json::json!([3, 104]));
}
#[test]
fn bad_version_and_existing_source_are_protected() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("bad format.bin");
    let mut bad = fs::read(fixture("default")).unwrap();
    bad[7] = 29;
    fs::write(&file, bad).unwrap();
    let out = cli(&["inspect".as_ref(), file.as_ref(), "--json".as_ref()]);
    assert!(!out.status.success());
    let d: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert!(d["error"].as_str().unwrap().contains("bad format.bin"));
    assert!(d["error"].as_str().unwrap().contains("header.format"));
    let src = tmp.path().join("original.bin");
    fs::copy(fixture("throttle-before"), &src).unwrap();
    let out = cli(&[
        "repair".as_ref(),
        src.as_ref(),
        "--output".as_ref(),
        src.as_ref(),
        "--set".as_ref(),
        "0:-:6802".as_ref(),
    ]);
    assert!(!out.status.success());
    assert_eq!(
        fs::read(&src).unwrap(),
        fs::read(fixture("throttle-before")).unwrap()
    );
}
