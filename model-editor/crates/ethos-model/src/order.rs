//! Slot changes are available only when every possible stored telemetry reference is understood.
use super::{crc16, Change, Error, Field, Model, Result, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ORDER: &str = "telemetry.sensors.order";
fn json_slots(slots: &[usize]) -> String {
    serde_json::to_string(slots).expect("slot array is serializable")
}

// New opaque structures are read-only for slot renumbering until their dependency
// semantics are established. These paths contain framing tags or verified scalar data.
fn opaque_without_slot_references(f: &Field) -> bool {
    let p = f.path.as_str();
    matches!(
        p,
        "model.bitmap"
            | "model.airframe_options"
            | "model.additional_controls"
            | "screens.selection"
            | "telemetry.additional_flags"
    ) || (p.starts_with("rf.modules[")
        && (p.ends_with("].flags") || p.ends_with("].enabled.preceding_bytes")))
        || (p.starts_with("flight_modes[")
            && p.ends_with("].active_condition.preceding_bytes")
            && f.range.len() == 1)
        || (p.starts_with("timers[") && p.ends_with("].name.preceding_bytes") && f.range.len() == 1)
        || (p.starts_with("telemetry.sensors[") && p.ends_with("].parameters"))
}

impl Model {
    fn field(&self, path: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.path == path)
    }
    fn slot_blocker(&self) -> Option<String> {
        if let Some(w) = self.warnings.first() {
            return Some(w.clone());
        }
        let total = self.field("telemetry.sensors.count")?;
        if total.typed != Value::Integer(self.sensors.len() as i64) {
            return Some(
                "telemetry.sensors contains a non-S.Port record with unverified dependencies"
                    .into(),
            );
        }
        for path in [
            "mixes.count",
            "curves.count",
            "logic_switches.count",
            "variables.count",
            "functions.count",
            "lua.sources.count",
            "lua.tasks.count",
        ] {
            if self
                .field(path)
                .is_some_and(|f| f.typed != Value::Integer(0))
            {
                return Some(format!("{path}: reference coverage is incomplete"));
            }
        }
        for f in &self.fields {
            if f.path.starts_with("channels[")
                && f.path.ends_with(".default")
                && f.typed == Value::Integer(0)
            {
                return Some(format!(
                    "{}: custom output references are not fully mapped",
                    f.path
                ));
            }
            if f.path.contains(".widgets[")
                && f.path.ends_with(".key")
                && !matches!(&f.typed, Value::Enum { raw: 0..=3, .. })
            {
                return Some(format!(
                    "{}: widget references are not fully mapped",
                    f.path
                ));
            }
            if f.typed == Value::Opaque && !opaque_without_slot_references(f) {
                return Some(format!(
                    "{}: bytes may contain unresolved slot references",
                    f.path
                ));
            }
        }
        None
    }

    pub(crate) fn list_order(&mut self) {
        if let Some(count) = self.field("telemetry.sensors.count") {
            let start = count.range.start;
            let end = self.sensors.last().map_or(count.range.end, |s| s.range.end);
            if self.sensors.is_empty() {
                return;
            }
            let ids: Vec<_> = self.sensors.iter().map(|s| s.slot).collect();
            let value = json_slots(&ids);
            self.fields.push(Field::new(
                "Telemetry",
                ORDER,
                start..end,
                Value::Text(value.clone()),
                value,
                self.slot_blocker().is_none(),
            ));
        }
    }

    pub(crate) fn reorder(
        &self,
        input: &str,
        already_staged: &[Change],
    ) -> Result<(Vec<u8>, Vec<Change>)> {
        if let Some(reason) = self.slot_blocker() {
            return Err(Error::Invalid(format!("{ORDER}: {reason}")));
        }
        let count = self
            .field("telemetry.sensors.count")
            .ok_or_else(|| Error::Invalid(format!("{ORDER}: collection boundary is unverified")))?;
        let order: Vec<usize> = serde_json::from_str(input).map_err(|e| {
            Error::Invalid(format!(
                "{ORDER}: supply an array of existing slot numbers: {e}"
            ))
        })?;
        let mut mapping = BTreeMap::new();
        for (new, &old) in order.iter().enumerate() {
            if old >= self.sensors.len() || mapping.insert(old, new).is_some() {
                return Err(Error::Invalid(format!(
                    "{ORDER}: unknown or repeated slot {old}"
                )));
            }
        }
        let previous: Vec<_> = (0..self.sensors.len()).collect();
        if order == previous {
            return Ok((self.to_bytes(), vec![]));
        }
        for change in already_staged {
            if let Some((old, _)) = split_sensor(&change.path) {
                if !mapping.contains_key(&old) {
                    return Err(Error::Invalid(format!(
                        "{}: edited slot is removed by {ORDER}",
                        change.path
                    )));
                }
            }
        }
        let mut patched = self.to_bytes();
        let mut changes = Vec::new();
        let mut expected_refs = Vec::new();
        let mut checked_ranges = BTreeSet::new();
        for f in &self.fields {
            let Value::Source {
                category: 22,
                options,
                member,
            } = f.typed
            else {
                continue;
            };
            if !checked_ranges.insert((f.range.start, f.range.end)) {
                continue;
            }
            let target_path = if let Some((old, leaf)) = split_sensor(&f.path) {
                let Some(new) = mapping.get(&old) else {
                    continue;
                };
                format!("telemetry.sensors[{new}].{leaf}")
            } else {
                f.path.clone()
            };
            let new = *mapping.get(&usize::from(member)).ok_or_else(|| {
                Error::Invalid(format!(
                    "{ORDER}: slot {member} is referenced by {}",
                    f.path
                ))
            })?;
            if f.range.len() != 4 {
                return Err(Error::Invalid(format!(
                    "{}: unsupported reference encoding",
                    f.path
                )));
            }
            if new != usize::from(member) {
                patched[f.range.start + 2..f.range.end]
                    .copy_from_slice(&(new as u16).to_le_bytes());
                changes.push(Change {
                    path: f.path.clone(),
                    before: format!("Telemetry slot {member}"),
                    after: format!("Telemetry slot {new}"),
                });
            }
            expected_refs.push((
                target_path,
                Value::Source {
                    category: 22,
                    options,
                    member: new as u16,
                },
            ));
        }
        let last = self.sensors.last().map_or(count.range.end, |s| s.range.end);
        let mut output = Vec::with_capacity(patched.len());
        output.extend_from_slice(&patched[..count.range.start]);
        output.extend_from_slice(&(order.len() as u16).to_le_bytes());
        for &old in &order {
            output.extend_from_slice(&patched[self.sensors[old].range.clone()]);
        }
        output.extend_from_slice(&patched[last..]);
        let size = u32::try_from(output.len() - 16)
            .map_err(|_| Error::Invalid("header.payload_length: output too large".into()))?;
        output[8..12].copy_from_slice(&size.to_le_bytes());
        let crc = crc16(&output[16..]);
        output[14..16].copy_from_slice(&crc.to_le_bytes());
        let reopened = Model::parse(&output)?;
        if !reopened.warnings.is_empty()
            || reopened.sensors.len() != order.len()
            || !reopened
                .sensors
                .iter()
                .zip(&order)
                .all(|(n, &o)| n.name == self.sensors[o].name && n.key() == self.sensors[o].key())
        {
            return Err(Error::Invalid(format!(
                "{ORDER}: slot order failed read-back"
            )));
        }
        for (path, expected) in expected_refs {
            if !reopened.field(&path).is_some_and(|f| f.typed == expected) {
                return Err(Error::Invalid(format!(
                    "{path}: reference remapping failed read-back"
                )));
            }
        }
        changes.insert(
            0,
            Change {
                path: ORDER.into(),
                before: json_slots(&previous),
                after: json_slots(&order),
            },
        );
        Ok((output, changes))
    }
}

fn split_sensor(path: &str) -> Option<(usize, &str)> {
    let (i, leaf) = path.strip_prefix("telemetry.sensors[")?.split_once("].")?;
    Some((i.parse().ok()?, leaf))
}
