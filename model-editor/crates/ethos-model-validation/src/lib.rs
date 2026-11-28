//! Optional firmware read-back for both frontends, with disposable inputs and a separate runtime.
use ethos_model::{compare, Difference, Error, Model, Result};
use serde::Serialize;
use serde_json::Value;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn default_adapter() -> PathBuf {
    let next = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_default()
        .join("simulator/validate.cjs");
    #[cfg(debug_assertions)]
    if !next.exists() {
        return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../simulator/validate.cjs");
    }
    next
}

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub firmware: Value,
    pub differences: Vec<Difference>,
}
impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = &self.firmware;
        writeln!(
            f,
            "{} - {} -> {}",
            v["status"].as_str().unwrap_or("unknown"),
            v["sourceFormat"].as_str().unwrap_or("?"),
            v["resultFormat"].as_str().unwrap_or("?")
        )?;
        if let Some(m) = v["message"].as_str() {
            writeln!(f, "{m}")?;
        }
        if let Some(items) = v["warnings"].as_array() {
            for item in items {
                writeln!(f, "Warning: {}", item.as_str().unwrap_or_default())?;
            }
        }
        if let Some(items) = v["changedRanges"].as_array() {
            writeln!(f, "Changed byte ranges: {}", Value::Array(items.clone()))?;
        }
        for d in &self.differences {
            writeln!(
                f,
                "{} - {}: {} -> {}",
                d.section,
                d.path,
                d.before.as_deref().unwrap_or("--"),
                d.after.as_deref().unwrap_or("--")
            )?;
        }
        Ok(())
    }
}

pub fn validate_in_ethos(input: &Path, runtime: &Path, adapter: &Path) -> Result<ValidationReport> {
    let original = Model::open(input)?;
    let work = tempfile::tempdir()?;
    // Both the adapter and comparison must observe the same input, even if the caller replaces it.
    let staged = work.path().join("input.bin");
    fs::write(&staged, original.to_bytes())?;
    let output = work.path().join("readback.bin");
    let child = Command::new("node")
        .arg(adapter)
        .arg(runtime)
        .arg(&staged)
        .arg("--readback")
        .arg(&output)
        .output()
        .map_err(|e| {
            Error::Invalid(format!(
                "Could not run Node.js for firmware validation: {e}"
            ))
        })?;
    let report: Value = serde_json::from_slice(&child.stdout).map_err(|_| {
        Error::Invalid(format!(
            "Simulator adapter did not return JSON: {}",
            String::from_utf8_lossy(&child.stderr).trim()
        ))
    })?;
    if !child.status.success() || !matches!(report["status"].as_str(), Some("ok" | "converted")) {
        return Err(Error::Invalid(format!(
            "{}: {}",
            report["status"].as_str().unwrap_or("validation failed"),
            report["message"]
                .as_str()
                .unwrap_or("firmware did not verify the copy")
        )));
    }
    let differences = if report["status"] == "ok" {
        compare(&original, &Model::open(&output)?)
    } else {
        if !output.is_file() {
            return Err(Error::Invalid("firmware read-back file is missing".into()));
        }
        Vec::new()
    };
    Ok(ValidationReport {
        firmware: report,
        differences,
    })
}
