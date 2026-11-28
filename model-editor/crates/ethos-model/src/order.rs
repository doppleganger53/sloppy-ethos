//! Slot changes are available only when every possible stored telemetry reference is understood.
use super::{crc16, Change, Error, Field, Model, Result, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ORDER: &str = "telemetry.sensors.order";
pub(crate) const CURVE_ORDER: &str = "curves.order";
#[derive(Clone, Copy, PartialEq, Eq)]
enum Collection {
    Sensors,
    Curves,
}

fn parse_order(
    path: &str,
    input: &str,
    total: usize,
) -> Result<(Vec<usize>, BTreeMap<usize, usize>)> {
    let order: Vec<usize> = serde_json::from_str(input).map_err(|e| {
        Error::Invalid(format!(
            "{path}: supply an array of existing zero-based slots: {e}"
        ))
    })?;
    let mut mapping = BTreeMap::new();
    for (new, &old) in order.iter().enumerate() {
        if old >= total || mapping.insert(old, new).is_some() {
            return Err(Error::Invalid(format!(
                "{path}: unknown or repeated slot {old}"
            )));
        }
    }
    Ok((order, mapping))
}

fn ordered_bytes(
    patched: &[u8],
    count: &std::ops::Range<usize>,
    records: &[std::ops::Range<usize>],
    order: &[usize],
) -> Result<Vec<u8>> {
    let last = records.last().map_or(count.end, |r| r.end);
    let mut output = Vec::with_capacity(patched.len());
    output.extend_from_slice(&patched[..count.start]);
    output.extend_from_slice(&(order.len() as u16).to_le_bytes());
    for &old in order {
        output.extend_from_slice(&patched[records[old].clone()]);
    }
    output.extend_from_slice(&patched[last..]);
    let size = u32::try_from(output.len() - 16)
        .map_err(|_| Error::Invalid("header.payload_length: output too large".into()))?;
    output[8..12].copy_from_slice(&size.to_le_bytes());
    let crc = crc16(&output[16..]);
    output[14..16].copy_from_slice(&crc.to_le_bytes());
    Ok(output)
}

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
    fn slot_blocker(&self, collection: Collection) -> Option<String> {
        if self.bytes[16] != 0 {
            return Some(
                "model.type: collection references are not fully verified for this category".into(),
            );
        }
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
            "extra_trims.count",
            "mixes.count",
            "curves.count",
            "logic_switches.count",
            "variables.count",
            "functions.count",
            "lua.sources.count",
            "lua.tasks.count",
        ] {
            if collection == Collection::Curves && matches!(path, "mixes.count" | "curves.count") {
                continue;
            }
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
            if collection == Collection::Curves
                && f.path.starts_with("mixes[")
                && f.path.ends_with("].key")
            {
                let safe = if f.path.contains(".actions[") {
                    matches!(f.typed, Value::Enum { raw: 0 | 1, .. })
                } else {
                    matches!(f.typed, Value::Enum { raw: 1, .. })
                };
                if !safe {
                    return Some(format!(
                        "{}: this mix variant may reference curve slots",
                        f.path
                    ));
                }
            }
            if collection == Collection::Curves
                && f.path.starts_with("curves[")
                && matches!(f.typed, Value::Source { .. })
            {
                return Some(format!(
                    "{}: dynamic curve operands have unverified dependencies",
                    f.path
                ));
            }
            let curve_framing = collection == Collection::Curves
                && ((f.path.starts_with("curves[") || f.path.starts_with("mixes["))
                    && f.path.ends_with("].name.preceding_bytes")
                    && f.range.len() == 1
                    || f.path.starts_with("mixes[")
                        && f.path.ends_with("].flight_mode_mask")
                        && f.range.len() == 4);
            if f.typed == Value::Opaque && !opaque_without_slot_references(f) && !curve_framing {
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
                self.slot_blocker(Collection::Sensors).is_none(),
            ));
        }
    }

    pub(crate) fn list_curve_order(&mut self) {
        let Some(first) = self.field("curves.count") else {
            return;
        };
        let Some(last) = self.curve_ranges.last() else {
            return;
        };
        if first.typed != Value::Integer(self.curve_ranges.len() as i64) {
            return;
        }
        let value = json_slots(&(0..self.curve_ranges.len()).collect::<Vec<_>>());
        self.fields.push(
            Field::new(
                "Curves",
                CURVE_ORDER,
                first.range.start..last.end,
                Value::Text(value.clone()),
                value,
                self.slot_blocker(Collection::Curves).is_none(),
            )
            .summary(),
        );
    }

    pub(crate) fn reorder_curves(
        &self,
        input: &str,
        staged: &[Change],
    ) -> Result<(Vec<u8>, Vec<Change>)> {
        if let Some(reason) = self.slot_blocker(Collection::Curves) {
            return Err(Error::Invalid(format!("{CURVE_ORDER}: {reason}")));
        }
        let count = self
            .field("curves.count")
            .ok_or_else(|| Error::Invalid(format!("{CURVE_ORDER}: unverified library boundary")))?;
        let (order, mapping) = parse_order(CURVE_ORDER, input, self.curve_ranges.len())?;
        let previous: Vec<_> = (0..self.curve_ranges.len()).collect();
        if order == previous {
            return Ok((self.to_bytes(), vec![]));
        }
        for item in staged {
            if let Some(slot) = item
                .path
                .strip_prefix("curves[")
                .and_then(|s| s.split_once("]."))
                .and_then(|(index, _)| index.parse::<usize>().ok())
            {
                if !mapping.contains_key(&slot) {
                    return Err(Error::Invalid(format!(
                        "{}: the edited curve would be removed",
                        item.path
                    )));
                }
            }
        }
        let mut patched = self.to_bytes();
        let mut changes = vec![Change {
            path: CURVE_ORDER.into(),
            before: json_slots(&previous),
            after: json_slots(&order),
        }];
        let mut references = Vec::new();
        for field in &self.fields {
            let Value::Reference { collection, slot } = &field.typed else {
                continue;
            };
            if collection != "curves" {
                continue;
            }
            if *slot >= self.curve_ranges.len() || field.range.len() != 1 {
                return Err(Error::Invalid(format!(
                    "{}: existing curve reference is invalid",
                    field.path
                )));
            }
            if !mapping.contains_key(slot) && staged.iter().any(|edit| edit.path == field.path) {
                return Err(Error::Invalid(format!(
                    "{}: its staged target curve {slot} is removed by {CURVE_ORDER}",
                    field.path
                )));
            }
            let (stored, expected, display) = match mapping.get(slot) {
                Some(&new) => (
                    new as u8 + 2,
                    Value::Reference { collection: "curves".into(), slot: new },
                    format!("Curve slot {new}"),
                ),
                None if field.path.ends_with(".curve.default")
                    || (field.path.contains(".curve.rates[") && field.path.ends_with("].curve")) => {
                    (0, Value::Absent, "None".to_owned())
                }
                None => {
                    return Err(Error::Invalid(format!(
                        "{CURVE_ORDER}: removed curve {slot} is used by {} and its deletion behavior is unverified",
                        field.path
                    )))
                }
            };
            if patched[field.range.start] != stored {
                patched[field.range.start] = stored;
                changes.push(Change {
                    path: field.path.clone(),
                    before: field.value.clone(),
                    after: display,
                });
            }
            references.push((&field.path, expected));
        }
        let output = ordered_bytes(&patched, &count.range, &self.curve_ranges, &order)?;
        let reopened = Self::parse(&output)?;
        if !reopened.warnings.is_empty()
            || reopened.curve_ranges.len() != order.len()
            || reopened.curve_ranges.iter().zip(&order).any(|(new, &old)| {
                reopened.bytes[new.clone()] != self.bytes[self.curve_ranges[old].clone()]
            })
        {
            return Err(Error::Invalid(format!(
                "{CURVE_ORDER}: curve contents or order failed read-back"
            )));
        }
        for (path, expected) in references {
            if !reopened.field(path).is_some_and(|f| f.typed == expected) {
                return Err(Error::Invalid(format!(
                    "{path}: curve reference failed read-back"
                )));
            }
        }
        Ok((output, changes))
    }

    pub(crate) fn reorder(
        &self,
        input: &str,
        already_staged: &[Change],
    ) -> Result<(Vec<u8>, Vec<Change>)> {
        if let Some(reason) = self.slot_blocker(Collection::Sensors) {
            return Err(Error::Invalid(format!("{ORDER}: {reason}")));
        }
        let count = self
            .field("telemetry.sensors.count")
            .ok_or_else(|| Error::Invalid(format!("{ORDER}: collection boundary is unverified")))?;
        let (order, mapping) = parse_order(ORDER, input, self.sensors.len())?;
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
        let ranges: Vec<_> = self.sensors.iter().map(|s| s.range.clone()).collect();
        let output = ordered_bytes(&patched, &count.range, &ranges, &order)?;
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
