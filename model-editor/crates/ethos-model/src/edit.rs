use super::{crc16, Error, Model, Result, Sensor};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Seek, SeekFrom, Write},
    ops::Range,
    path::Path,
};
use tempfile::NamedTempFile;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Edit {
    pub slot: usize,
    pub physical_id: Option<u8>,
    pub application_id: Option<u16>,
}
#[derive(Clone, Debug, Serialize)]
pub struct FieldEdit {
    pub path: String,
    pub value: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Change {
    pub path: String,
    pub before: String,
    pub after: String,
}
struct Patch {
    range: Range<usize>,
    bytes: Vec<u8>,
    change: Change,
}

impl Patch {
    fn new(field: &super::Field, bytes: Vec<u8>, after: String) -> Self {
        Self {
            range: field.range.clone(),
            bytes,
            change: Change {
                path: field.path.clone(),
                before: field.value.clone(),
                after,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AudioAction {
    #[serde(rename = "type")]
    kind: AudioKind,
    start: u16,
    step: u16,
    haptic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
}
#[derive(Clone, Copy, Debug, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AudioKind {
    CountdownValue,
    CountdownBeep,
    PlayFile,
}
pub(crate) fn read_actions(c: &mut super::legacy::Cursor<'_>) -> Result<Vec<AudioAction>> {
    let n = c.count(10)?;
    let mut result = Vec::new();
    for _ in 0..n {
        let tag = c.byte()?;
        let start = c.word()?;
        let step = c.word()?;
        let kind = match tag {
            0 => AudioKind::CountdownValue,
            1 => AudioKind::CountdownBeep,
            2 => AudioKind::PlayFile,
            _ => return Err(c.unsupported("timer audio action")),
        };
        let filename = if tag == 2 { Some(c.text()?.1) } else { None };
        let haptic = c.byte()?;
        if haptic > 1 {
            return Err(c.unsupported("timer audio haptic flag"));
        }
        result.push(AudioAction {
            kind,
            start,
            step,
            haptic: haptic != 0,
            filename,
        });
    }
    Ok(result)
}
fn encode_actions(path: &str, text: &str) -> Result<(Vec<u8>, String)> {
    let actions: Vec<AudioAction> =
        serde_json::from_str(text).map_err(|e| Error::Invalid(format!("{path}: {e}")))?;
    if actions.len() > 10 {
        return Err(Error::Invalid(format!(
            "{path}: at most 10 timer audio actions"
        )));
    }
    let mut bytes = (actions.len() as u16).to_le_bytes().to_vec();
    for a in &actions {
        bytes.push(match a.kind {
            AudioKind::CountdownValue => 0,
            AudioKind::CountdownBeep => 1,
            AudioKind::PlayFile => 2,
        });
        bytes.extend(a.start.to_le_bytes());
        bytes.extend(a.step.to_le_bytes());
        match (&a.kind, &a.filename) {
            (AudioKind::PlayFile, Some(name))
                if name.len() <= 31 && !name.as_bytes().contains(&0) =>
            {
                bytes.push(name.len() as u8);
                bytes.extend(name.as_bytes());
            }
            (AudioKind::CountdownValue | AudioKind::CountdownBeep, None) => {}
            _ => {
                return Err(Error::Invalid(format!(
                    "{path}: only play_file accepts a filename (up to 31 bytes)"
                )))
            }
        }
        bytes.push(u8::from(a.haptic));
    }
    Ok((
        bytes,
        serde_json::to_string(&actions).expect("timer actions serialize"),
    ))
}
pub(crate) fn packed_integer(n: i32) -> Vec<u8> {
    if n == 0 {
        return vec![0x80];
    }
    if let Ok(v) = i8::try_from(n) {
        return vec![0x81, v as u8];
    }
    if let Ok(v) = i16::try_from(n) {
        let mut bytes = vec![0x82];
        bytes.extend(v.to_le_bytes());
        return bytes;
    }
    let mut bytes = vec![0x83];
    bytes.extend(n.to_le_bytes());
    bytes
}
fn percent(value: &str, path: &str, min: i16, max: i16) -> Result<(Vec<u8>, String)> {
    let display = value.trim().trim_end_matches('%');
    let n = display
        .parse::<f64>()
        .map_err(|_| Error::Invalid(format!("{path}: expected a percentage in tenths")))?;
    let scaled = n * 10.;
    let rounded = scaled.round();
    if !scaled.is_finite()
        || (scaled - rounded).abs() > 1e-8
        || !(f64::from(min)..=f64::from(max)).contains(&rounded)
    {
        return Err(Error::Invalid(format!(
            "{path}: enter a tenth-percent value between {} and {}",
            f64::from(min) / 10.,
            f64::from(max) / 10.
        )));
    }
    let raw = rounded as i32;
    Ok((packed_integer(raw), format!("{}%", f64::from(raw) / 10.0)))
}
fn encode_field(
    model: &Model,
    kind: super::EditFormat,
    path: &str,
    value: &str,
) -> Result<(Vec<u8>, String)> {
    use super::EditFormat;
    match kind {
        EditFormat::LogicInputs => super::logic::encode_inputs(path, value),
        EditFormat::VariableRates => super::variables::encode_rates(path, value),
        EditFormat::TrimTarget => super::variables::trim_target(model, path, value),
        EditFormat::VariableActions => super::variables::encode_actions(model, path, value),
        EditFormat::VariableFunction => super::variables::operator(path, value),
        EditFormat::LogicComparison => super::logic::encode_op(path, value, false),
        EditFormat::LogicCombinator => super::logic::encode_op(path, value, true),
        EditFormat::LogicInput => super::logic::encode_input(path, value),
        EditFormat::TimerInterval => super::logic::seconds(path, value, 1, 1750),
        EditFormat::CurveSelector => {
            super::curves::encode_selector(model.curve_ranges.len(), value, path)
        }
        EditFormat::CurvePoints => super::curves::encode_points(path, value),
        EditFormat::CurveFunction => super::curves::function(value, path),
        EditFormat::Operand { minimum, maximum } => {
            let v = value
                .parse::<i32>()
                .map_err(|_| Error::Invalid(format!("{path}: expected an integer")))?;
            if !(minimum..=maximum).contains(&v) {
                return Err(Error::Invalid(format!(
                    "{path}: enter a value between {minimum} and {maximum}"
                )));
            }
            Ok((packed_integer(v), v.to_string()))
        }
        EditFormat::Percent { minimum, maximum } => percent(value, path, minimum, maximum),
        EditFormat::Toggle | EditFormat::InternalRf => {
            let (v, s) = match value.to_ascii_lowercase().as_str() {
                "0" | "off" | "false" => (0, "Off"),
                "1" | "on" | "true" => (1, "On"),
                _ => return Err(Error::Invalid(format!("{path}: choose On or Off"))),
            };
            Ok((vec![v], s.into()))
        }
        EditFormat::TrimStep => {
            let v = match value.to_ascii_lowercase().as_str() {
                "2" | "fine" => 2,
                "3" | "medium" => 3,
                _ => {
                    return Err(Error::Invalid(format!(
                        "{path}: verified values are Fine (2) and Medium (3)"
                    )))
                }
            };
            Ok((vec![v], v.to_string()))
        }
        EditFormat::Label15 => {
            let label = value.as_bytes();
            if label.len() > 15 || label.contains(&0) {
                return Err(Error::Invalid(format!(
                    "{path}: names allow up to 15 UTF-8 bytes and cannot contain NUL"
                )));
            }
            let mut bytes = vec![label.len() as u8];
            bytes.extend(label);
            Ok((bytes, value.into()))
        }
        EditFormat::Boolean => {
            let v = match value.to_ascii_lowercase().as_str() {
                "0" | "up" => 0,
                "1" | "down" => 1,
                _ => return Err(Error::Invalid(format!("{path}: choose Up (0) or Down (1)"))),
            };
            Ok((vec![v], if v == 0 { "Up".into() } else { "Down".into() }))
        }
        EditFormat::TimerSeconds => {
            let v = value
                .parse::<i32>()
                .map_err(|_| Error::Invalid(format!("{path}: expected integer seconds")))?;
            Ok((v.to_le_bytes().to_vec(), v.to_string()))
        }
        EditFormat::TimerActions => encode_actions(path, value),
    }
}

pub(crate) fn hex(text: &str, width: usize, path: &str) -> Result<u16> {
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    if digits.is_empty() || digits.len() > width || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Invalid(format!(
            "{path}: expected up to {width} hexadecimal digits"
        )));
    }
    u16::from_str_radix(digits, 16).map_err(|e| Error::Invalid(format!("{path}: {e}")))
}

impl Model {
    pub fn preview(&self, edits: &[Edit]) -> Result<Vec<Change>> {
        self.preview_fields(edits, &[])
    }
    pub fn preview_fields(&self, edits: &[Edit], fields: &[FieldEdit]) -> Result<Vec<Change>> {
        Ok(self.prepare(edits, fields)?.1)
    }
    /// Serialize a verified change set without publishing a file.
    pub fn serialize_fields(&self, edits: &[Edit], fields: &[FieldEdit]) -> Result<Vec<u8>> {
        Ok(self.prepare(edits, fields)?.0)
    }
    pub fn save_copy(&self, path: &Path, edits: &[Edit]) -> Result<()> {
        self.save_fields_copy(path, edits, &[])
    }
    pub fn save_fields_copy(
        &self,
        path: &Path,
        edits: &[Edit],
        fields: &[FieldEdit],
    ) -> Result<()> {
        let (expected, _) = self.prepare(edits, fields)?;
        let directory = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let write = || -> Result<()> {
            let mut temp = NamedTempFile::new_in(directory)?;
            temp.write_all(&expected)?;
            temp.as_file().sync_all()?;
            let mut persisted = temp.reopen()?;
            persisted.seek(SeekFrom::Start(0))?;
            let mut read = Vec::new();
            persisted.read_to_end(&mut read)?;
            if read != expected {
                return Err(Error::Invalid(
                    "read-back differs from the verified edit preview".into(),
                ));
            }
            Self::parse(&read)?;
            temp.persist_noclobber(path)
                .map_err(|e| Error::Invalid(e.error.to_string()))?;
            Ok(())
        };
        write().map_err(|e| Error::Invalid(format!("{}: {e}", path.display())))
    }

    fn prepare(&self, edits: &[Edit], fields: &[FieldEdit]) -> Result<(Vec<u8>, Vec<Change>)> {
        if let Some(expanded) = self.expand_channels(fields)? {
            return expanded.prepare(edits, fields);
        }
        if let Some(order) = fields.iter().find(|f| {
            matches!(
                f.path.as_str(),
                super::order::ORDER | super::order::CURVE_ORDER
            )
        }) {
            let scalar: Vec<_> = fields
                .iter()
                .filter(|f| f.path != order.path)
                .cloned()
                .collect();
            if scalar.len() + 1 != fields.len() {
                return Err(Error::Invalid(format!(
                    "{}: order staged twice",
                    order.path
                )));
            }
            let (bytes, mut changes) = self.prepare(edits, &scalar)?;
            let staged = Self::parse(&bytes)?;
            let (next, mut order_changes) = if order.path == super::order::CURVE_ORDER {
                staged.reorder_curves(&order.value, &changes)?
            } else {
                staged.reorder(&order.value, &changes)?
            };
            changes.append(&mut order_changes);
            return Ok((next, changes));
        }
        let mut staged = fields.to_vec();
        for edit in edits {
            let root = format!("telemetry.sensors[{}]", edit.slot);
            if edit.physical_id.is_none() && edit.application_id.is_none() {
                return Err(Error::Invalid(format!(
                    "{root}: supply a Physical or Application ID"
                )));
            }
            if let Some(v) = edit.physical_id {
                staged.push(FieldEdit {
                    path: format!("{root}.physical_id"),
                    value: format!("{v:02X}"),
                });
            }
            if let Some(v) = edit.application_id {
                staged.push(FieldEdit {
                    path: format!("{root}.application_id"),
                    value: format!("{v:04X}"),
                });
            }
        }
        let mut final_sensors = self.sensors.clone();
        let mut seen = BTreeSet::new();
        let mut patches = Vec::new();
        for proposal in &staged {
            let path = &proposal.path;
            if !seen.insert(path) {
                return Err(Error::Invalid(format!(
                    "{path}: field staged more than once"
                )));
            }
            let field = self
                .fields
                .iter()
                .find(|f| &f.path == path)
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "{path}: this field has no verified same-format edit for this layout"
                    ))
                })?;
            if !field.editable {
                if path.ends_with(".application_id")
                    && self
                        .sensors
                        .iter()
                        .any(|s| s.range.start + 6 == field.range.start)
                {
                    let requested = hex(&proposal.value, 4, path)?;
                    if field.typed == super::Value::Integer(requested.into()) {
                        continue;
                    }
                    return Err(Error::Invalid(format!(
                        "{path}: this Application ID has no verified same-meaning replacement"
                    )));
                }
                return Err(Error::Invalid(format!(
                    "{path}: this field has no verified same-format edit for this layout"
                )));
            }
            let (mut bytes, display) = if let Some(kind) = field.encoding {
                encode_field(self, kind, path, &proposal.value)?
            } else if let Some(proposed) = super::sections_after::encode(
                self,
                field,
                &proposal.value,
                &self.bytes[field.range.clone()],
            )? {
                proposed
            } else if let Some(sensor) = final_sensors.iter_mut().find(|s| {
                field.range.start == s.range.start + 5 || field.range.start == s.range.start + 6
            }) {
                if path.ends_with(".physical_id") {
                    let v = hex(&proposal.value, 2, path)?;
                    if v > 0x1b {
                        return Err(Error::Invalid(format!(
                            "{path}: Physical ID must be in 00..1B"
                        )));
                    }
                    sensor.physical_id = v as u8;
                    (vec![v as u8], format!("{v:02X}"))
                } else if path.ends_with(".application_id") {
                    let v = hex(&proposal.value, 4, path)?;
                    let servo_family = 0x6800..=0x6803;
                    if v != sensor.application_id
                        && !(servo_family.contains(&sensor.application_id)
                            && servo_family.contains(&v))
                    {
                        return Err(Error::Invalid(format!("{path}: changing {:04X} to {v:04X} has no verified same-meaning sensor encoding",sensor.application_id)));
                    }
                    sensor.application_id = v;
                    (v.to_le_bytes().to_vec(), format!("{v:04X}"))
                } else {
                    return Err(Error::Invalid(format!("{path}: unsupported edit")));
                }
            } else {
                return Err(Error::Invalid(format!("{path}: unsupported edit")));
            };
            if field.encoding == Some(super::EditFormat::CurvePoints) {
                let root = field
                    .path
                    .strip_suffix(".points")
                    .expect("point table path");
                let number = bytes[0];
                let easy = bytes[1];
                let count_field = self
                    .fields
                    .iter()
                    .find(|f| f.path == format!("{root}.points.count"))
                    .expect("parsed curve count");
                let grid_field = self
                    .fields
                    .iter()
                    .find(|f| f.path == format!("{root}.easy_mode"))
                    .expect("parsed curve grid mode");
                for (target, v, after) in [
                    (count_field, number, number.to_string()),
                    (
                        grid_field,
                        easy,
                        if easy == 1 { "On" } else { "Off" }.into(),
                    ),
                ] {
                    if self.bytes[target.range.clone()] != [v] {
                        patches.push(Patch::new(target, vec![v], after));
                    }
                }
                bytes.drain(..2);
            }
            if field.encoding == Some(super::EditFormat::InternalRf)
                && self.bytes[field.range.clone()] != bytes
            {
                let header = self
                    .fields
                    .iter()
                    .find(|f| f.path == "rf.module_types[0]")
                    .expect("module type checked during decode");
                let stored = if bytes[0] == 1 { 11u8 } else { 255 };
                patches.push(Patch::new(header, vec![stored], stored.to_string()));
            }
            if self.bytes[field.range.clone()] != bytes
                || (field.encoding == Some(super::EditFormat::CurvePoints)
                    && field.value != display)
            {
                patches.push(Patch::new(field, bytes, display));
            }
        }
        check_keys(&self.sensors, &final_sensors)?;
        patches.sort_by_key(|p| p.range.start);
        let mut result = Vec::with_capacity(self.bytes.len());
        let mut offset = 0;
        for patch in &patches {
            if patch.range.start < offset {
                return Err(Error::Invalid(format!(
                    "{}: overlapping edit",
                    patch.change.path
                )));
            }
            result.extend_from_slice(&self.bytes[offset..patch.range.start]);
            result.extend_from_slice(&patch.bytes);
            offset = patch.range.end;
        }
        result.extend_from_slice(&self.bytes[offset..]);
        let len = u32::try_from(result.len() - 16)
            .map_err(|_| Error::Invalid("header.payload_length: result exceeds u32".into()))?;
        result[8..12].copy_from_slice(&len.to_le_bytes());
        let crc = crc16(&result[16..]);
        result[14..16].copy_from_slice(&crc.to_le_bytes());
        let mut reopened = Self::parse(&result)?;
        if let Some(compacted) = reopened.compact_channels(fields)? {
            reopened = compacted;
            result = reopened.to_bytes();
        }
        match (&self.unverified, &reopened.unverified) {
            (None, None) => {}
            (Some(old), Some(new))
                if old.path == new.path
                    && old.reason == new.reason
                    && old.offset as i64 - old.bytes.start as i64
                        == new.offset as i64 - new.bytes.start as i64
                    && self.bytes[old.bytes.clone()] == reopened.bytes[new.bytes.clone()] => {}
            (old, new) => {
                let path = old
                    .as_ref()
                    .or(new.as_ref())
                    .expect("at least one unknown region")
                    .path
                    .as_str();
                return Err(Error::Invalid(format!(
                    "{path}: editing would change the unverified structure or its contents"
                )));
            }
        }
        if reopened.sensors.len() != final_sensors.len()
            || reopened.sensors.iter().zip(&final_sensors).any(|(a, b)| {
                a.slot != b.slot
                    || a.name != b.name
                    || a.key() != b.key()
                    || a.range.len() != b.range.len()
            })
        {
            return Err(Error::Invalid("telemetry: reopening the edited copy changed the verified record layout or identities".into()));
        }
        for patch in &patches {
            if !reopened
                .fields
                .iter()
                .any(|f| f.path == patch.change.path && f.value == patch.change.after)
            {
                return Err(Error::Invalid(format!(
                    "{}: requested value did not survive reopening",
                    patch.change.path
                )));
            }
        }
        Ok((result, patches.into_iter().map(|p| p.change).collect()))
    }
}

fn check_keys(before: &[Sensor], after: &[Sensor]) -> Result<()> {
    let mut keys = BTreeMap::new();
    for (old, new) in before.iter().zip(after) {
        keys.entry(new.key())
            .or_insert_with(Vec::new)
            .push((old, new));
    }
    for (key, rows) in keys {
        if rows.len() > 1 && rows.iter().any(|(s, _)| s.key() != key) {
            let slots = rows
                .iter()
                .map(|(_, s)| s.slot.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Error::Invalid(format!("telemetry.sensors: slots {slots} would share a duplicate S.Port key (Physical {:02X}, Application {:04X})",key.4,key.5)));
        }
    }
    Ok(())
}
