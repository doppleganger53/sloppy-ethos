use super::dependencies::{patched_model, SENSORS};
use super::{Error, Model, Result, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

pub const SENSOR_FIELDS: [&str; 7] = [
    "Name",
    "Module",
    "Band",
    "Routing",
    "Sub-ID",
    "Physical ID (hex)",
    "Application ID (hex)",
];

#[derive(Clone, Default)]
pub struct SensorDraft {
    pub values: [String; SENSOR_FIELDS.len()],
}

impl SensorDraft {
    pub fn missing(&self) -> Vec<&'static str> {
        SENSOR_FIELDS
            .into_iter()
            .zip(&self.values)
            .filter_map(|(label, input)| input.trim().is_empty().then_some(label))
            .collect()
    }

    fn encode(&self) -> Result<Vec<u8>> {
        let missing = self.missing();
        if !missing.is_empty() {
            return Err(Error::Invalid(format!(
                "Missing required sensor fields: {}",
                missing.join(", ")
            )));
        }
        let [name, module, band, routing, sub_id, physical, application] = &self.values;
        if name.len() > 15 || name.chars().any(char::is_control) {
            return Err(Error::Invalid(
                "Name must use 1–15 UTF-8 bytes without control characters.".into(),
            ));
        }
        let decimal = |input: &str, field: &str, max: u8| {
            input
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|value| *value <= max)
                .ok_or_else(|| {
                    Error::Invalid(format!("{field} requires a number from 0 to {max}."))
                })
        };
        let module = decimal(module, "Module", 1)?;
        let band = decimal(band, "Band", 1)?;
        let routing = decimal(routing, "Routing", 0)?;
        let sub_id = decimal(sub_id, "Sub-ID", u8::MAX)?;
        let physical = super::edit::hex(physical, 2, "Physical ID")? as u8;
        if physical > 0x1b {
            return Err(Error::Invalid(
                "Physical ID must be between 00 and 1B.".into(),
            ));
        }
        let application = super::edit::hex(application, 4, "Application ID")?;
        let template = default_parameters(application, sub_id).ok_or_else(|| {
            Error::Invalid(format!(
                "Application ID {application:04X}, sub-ID {sub_id}: no verified S.Port defaults."
            ))
        })?;
        let mut record = vec![0, module, band, routing, sub_id, physical];
        record.extend_from_slice(&application.to_le_bytes());
        record.push(name.len() as u8);
        record.extend_from_slice(name.as_bytes());
        record.extend_from_slice(template);
        Ok(record)
    }
}

// Keep the unit, precision, flags, limits, reset source and variant payload from
// a firmware-discovered sensor with exactly the same application and sub-ID.
fn default_parameters(app: u16, sub: u8) -> Option<&'static [u8]> {
    static MODELS: OnceLock<[Model; 4]> = OnceLock::new();
    let models = MODELS.get_or_init(|| {
        [
            include_bytes!("../../../fixtures/throttle-before.bin").as_slice(),
            include_bytes!("../../../fixtures/throttle-after.bin").as_slice(),
            include_bytes!("../../../fixtures/sensor-variants.bin").as_slice(),
            include_bytes!("../../../fixtures/servo-addresses.bin").as_slice(),
        ]
        .map(|bytes| Model::parse(bytes).expect("verified sensor default fixture"))
    });
    models.iter().find_map(|model| {
        let sensor = model
            .sensors
            .iter()
            .find(|s| s.application_id == app && s.sub_id == sub)?;
        let start = model.field(&format!("{}.name", sensor.path()))?.range.end;
        // The fixture's reset source must be unset so append cannot import a foreign slot.
        (model.bytes[start + 11] == 0).then(|| &model.bytes[start..sensor.range.end])
    })
}

#[derive(Clone)]
pub struct SensorCopy {
    bytes: Vec<u8>,
    targets: BTreeMap<String, Vec<u8>>,
}

impl Model {
    fn copied_target(&self, path: &str) -> Option<&[u8]> {
        let range = if let Some(slot) = path
            .strip_prefix("curves[")
            .and_then(|s| s.strip_suffix(']'))
        {
            self.curve_ranges.get(slot.parse::<usize>().ok()?)?
        } else {
            &self
                .records
                .iter()
                .find(|record| record.path == path)?
                .range
        };
        self.raw(range.clone())
    }

    fn sensor_count_field(&self) -> Result<&super::Field> {
        self.field("telemetry.sensors.count")
            .ok_or_else(|| Error::Invalid("Telemetry sensor collection is unavailable.".into()))
    }

    pub fn copy_sensor(&self, slot: usize) -> Result<SensorCopy> {
        let path = format!("{SENSORS}[{slot}]");
        let record = self
            .records
            .iter()
            .find(|record| record.path == path)
            .ok_or_else(|| Error::Invalid(format!("{path}: sensor is unavailable")))?;
        let mut targets = BTreeMap::new();
        for reference in self.references.iter().filter(|r| {
            record.range.contains(&r.offset)
                && matches!(
                    r.collection,
                    SENSORS | "logic_switches" | "variables" | "timers" | "curves" | "flight_modes"
                )
        }) {
            let target = reference.target();
            let bytes = self
                .copied_target(&target)
                .ok_or_else(|| Error::Invalid(format!("{path} references unavailable {target}")))?;
            targets.insert(target, bytes.to_vec());
        }
        Ok(SensorCopy {
            bytes: self.bytes[record.range.clone()].to_vec(),
            targets,
        })
    }

    pub fn append_sensor(&self, draft: &SensorDraft) -> Result<Self> {
        self.append_sensor_record(&draft.encode()?)
    }

    pub fn paste_sensor(&self, sensor: &SensorCopy) -> Result<Self> {
        for (path, expected) in &sensor.targets {
            if self.copied_target(path) != Some(expected) {
                return Err(Error::Invalid(format!(
                    "Copied sensor references {path}, which changed. Copy the sensor again."
                )));
            }
        }
        self.append_sensor_record(&sensor.bytes)
    }

    fn append_sensor_record(&self, record: &[u8]) -> Result<Self> {
        if let Some(warning) = self.warnings.first() {
            return Err(Error::Invalid(warning.clone()));
        }
        if self
            .field("telemetry.competition")
            .is_some_and(|field| matches!(field.typed, Value::Enum { raw: 1, .. }))
        {
            return Err(Error::Invalid(
                "Disable Competition mode on the radio before adding sensors.".into(),
            ));
        }
        let count = self.sensor_count_field()?;
        let old = u16::from_le_bytes(self.bytes[count.range.clone()].try_into().unwrap());
        let next = old
            .checked_add(1)
            .ok_or_else(|| Error::Invalid("Telemetry sensor collection is full.".into()))?;
        let end = self
            .records
            .iter()
            .rev()
            .find(|r| r.path.starts_with("telemetry.sensors["))
            .map_or(count.range.end, |r| r.range.end);
        let reopened = patched_model(
            &self.bytes,
            vec![
                (count.range.clone(), next.to_le_bytes().to_vec()),
                (end..end, record.to_vec()),
            ],
        )?;
        if reopened.sensor_count_field()?.typed != Value::Integer(next.into())
            || !reopened.records.iter().any(|r| {
                r.path == format!("{SENSORS}[{old}]")
                    && reopened.raw(r.range.clone()) == Some(record)
            })
        {
            return Err(Error::Invalid(
                "Appended sensor failed record read-back.".into(),
            ));
        }
        Ok(reopened)
    }

    // A pasted S.Port entry is editable in the session even while it shares the source key.
    // Preserve older model files which already had conflicts without introducing more.
    pub fn verify_sensor_keys(&self, original: &Model) -> Result<()> {
        let mut before = BTreeMap::new();
        for sensor in original.sensors() {
            *before.entry(sensor.key()).or_insert(0) += 1;
        }
        let mut after = BTreeMap::new();
        for sensor in self.sensors() {
            after
                .entry(sensor.key())
                .or_insert_with(Vec::new)
                .push(sensor.slot);
        }
        for (key, slots) in after {
            if slots.len() > before.get(&key).copied().unwrap_or(1) {
                return Err(Error::Invalid(format!(
                    "Sensor slots {slots:?} share a S.Port key (Physical {:02X}, Application {:04X}). Change the key of the new entry.",
                    key.4, key.5
                )));
            }
        }
        Ok(())
    }

    pub fn save_sensor_copy(&self, path: &Path, original: &Model) -> Result<()> {
        self.verify_sensor_keys(original)?;
        Self::persist_copy(path, &self.bytes)
    }
}
