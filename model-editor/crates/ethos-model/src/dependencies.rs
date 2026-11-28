use super::{crc16, Error, Model, Result, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

pub(crate) const SENSORS: &str = "telemetry.sensors";

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub path: String,
    pub range: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Reference {
    pub offset: usize,
    pub collection: &'static str,
    pub slot: usize,
    category: u8,
}

impl Reference {
    pub(crate) fn source(offset: usize, category: u8, member: u16) -> Option<Self> {
        let collection = match category {
            13 => "logic_switches",
            14 | 15 => "trims",
            16 => "channels",
            17 => "variables",
            21 => "timers",
            22 | 23 => SENSORS,
            24 => "flight_modes",
            29 => "lua.sources",
            _ => return None,
        };
        Some(Self {
            offset,
            collection,
            slot: member.into(),
            category,
        })
    }

    pub(crate) fn curve(offset: usize, slot: u8) -> Self {
        Self {
            offset,
            collection: "curves",
            slot: slot.into(),
            category: 0,
        }
    }

    pub(crate) fn target(&self) -> String {
        format!("{}[{}]", self.collection, self.slot)
    }

    fn patch(&self, slot: usize) -> (Range<usize>, Vec<u8>) {
        if self.category == 0 {
            (self.offset..self.offset + 1, vec![slot as u8 + 2])
        } else {
            (
                self.offset + 2..self.offset + 4,
                (slot as u16).to_le_bytes().to_vec(),
            )
        }
    }
}

#[derive(Default)]
pub struct Dependencies {
    pub names: BTreeMap<String, String>,
    pub outgoing: BTreeMap<String, BTreeSet<String>>,
    pub incoming: BTreeMap<String, BTreeSet<String>>,
    pub notes: Vec<String>,
}

impl Dependencies {
    pub fn label(&self, path: &str) -> String {
        match self.names.get(path) {
            Some(name) if !name.is_empty() => format!("{name} ({path})"),
            Some(_) => path.to_owned(),
            None => format!("{path} (unresolved)"),
        }
    }
}

pub struct DeletePreview {
    pub removed: Vec<String>,
    pub blockers: Vec<String>,
}

fn split_slot(path: &str) -> Option<(&str, usize)> {
    let (collection, slot) = path.strip_suffix(']')?.rsplit_once('[')?;
    Some((collection, slot.parse().ok()?))
}

fn object_path(path: &str) -> &str {
    if let Some((index, _)) = path
        .match_indices(']')
        .nth(usize::from(path.contains("].widgets[")))
    {
        &path[..=index]
    } else {
        path.split('.').next().unwrap_or(path)
    }
}

/// Apply non-overlapping patches in original file coordinates and rebuild the envelope.
pub(crate) fn patched_model(
    bytes: &[u8],
    mut patches: Vec<(Range<usize>, Vec<u8>)>,
) -> Result<Model> {
    patches.sort_by_key(|(range, _)| (range.start, range.end));
    let mut output = Vec::with_capacity(bytes.len());
    let mut end = 0;
    for (range, replacement) in patches {
        if range.start < end || range.end > bytes.len() {
            return Err(Error::Invalid(
                "Overlapping or invalid model patches".into(),
            ));
        }
        output.extend_from_slice(&bytes[end..range.start]);
        output.extend_from_slice(&replacement);
        end = range.end;
    }
    output.extend_from_slice(&bytes[end..]);
    let size = u32::try_from(output.len() - 16)
        .map_err(|_| Error::Invalid("header.payload_length: output too large".into()))?;
    output[8..12].copy_from_slice(&size.to_le_bytes());
    let crc = crc16(&output[16..]);
    output[14..16].copy_from_slice(&crc.to_le_bytes());
    let reopened = Model::parse(&output)?;
    if let Some(warning) = reopened.warnings.first() {
        return Err(Error::Invalid(format!(
            "Edited model failed read-back: {warning}"
        )));
    }
    Ok(reopened)
}

impl Model {
    pub fn dependency_object(&self, path: &str) -> String {
        object_path(path).to_owned()
    }

    fn reference_owner(&self, offset: usize) -> String {
        if let Some(record) = self
            .records
            .iter()
            .find(|record| record.range.contains(&offset))
        {
            return record.path.clone();
        }
        if let Some(index) = self
            .curve_ranges
            .iter()
            .position(|range| range.contains(&offset))
        {
            return format!("curves[{index}]");
        }
        self.fields
            .iter()
            .filter(|field| field.range.contains(&offset) && !field.summary)
            .min_by_key(|field| field.range.len())
            .map_or_else(
                || "model".into(),
                |field| object_path(&field.path).to_owned(),
            )
    }

    pub(crate) fn opaque_dependency_notes(&self) -> Vec<String> {
        let mut notes = self.warnings.clone();
        for field in &self.fields {
            let path = &field.path;
            if path == "model.after_telemetry"
                || path.ends_with(".configuration") && field.typed == Value::Opaque
                || path.contains("].widgets[")
                    && path.ends_with("].key")
                    && matches!(&field.typed, Value::Text(_) | Value::Opaque)
                || path == "lua.sources.count" && field.typed != Value::Integer(0)
                || path == "lua.tasks.count" && field.typed != Value::Integer(0)
            {
                notes.push(format!("{path}: dependency coverage is incomplete"));
            }
        }
        notes
    }

    pub fn dependencies(&self) -> Dependencies {
        let mut graph = Dependencies {
            notes: self.opaque_dependency_notes(),
            ..Dependencies::default()
        };
        for field in &self.fields {
            let root = object_path(&field.path);
            let name = graph.names.entry(root.to_owned()).or_default();
            if field.path.strip_prefix(root) == Some(".name") {
                *name = field.value.clone();
            }
        }
        for reference in &self.references {
            let owner = self.reference_owner(reference.offset);
            let target = reference.target();
            graph
                .outgoing
                .entry(owner.clone())
                .or_default()
                .insert(target.clone());
            graph.incoming.entry(target).or_default().insert(owner);
        }
        graph
    }

    pub fn telemetry_paths(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.path.starts_with("telemetry.sensors["))
            .map(|r| r.path.clone())
            .collect()
    }

    fn deletion_range(&self, path: &str) -> Option<Range<usize>> {
        let (collection, slot) = split_slot(path)?;
        if collection == "curves" {
            return self.curve_ranges.get(slot).cloned();
        }
        if !matches!(
            collection,
            SENSORS | "timers" | "logic_switches" | "variables" | "functions" | "mixes"
        ) && !collection.ends_with("].widgets")
        {
            return None;
        }
        self.records
            .iter()
            .find(|r| r.path == path)
            .map(|r| r.range.clone())
    }

    pub fn preview_sensor_delete(&self, slots: &[usize], cascade: bool) -> Result<DeletePreview> {
        let mut removal = BTreeSet::new();
        for slot in slots {
            let path = format!("{SENSORS}[{slot}]");
            if self.deletion_range(&path).is_none() {
                return Err(Error::Invalid(format!("{path}: sensor is unavailable")));
            }
            removal.insert(path);
        }
        let graph = self.dependencies();
        let mut blockers = graph.notes;
        let mut queue: Vec<_> = removal.iter().cloned().collect();
        while let Some(target) = queue.pop() {
            if let Some(incoming) = graph.incoming.get(&target) {
                for owner in incoming {
                    if removal.contains(owner) {
                        continue;
                    }
                    if cascade && self.deletion_range(owner).is_some() {
                        removal.insert(owner.clone());
                        queue.push(owner.clone());
                    } else {
                        blockers.push(format!(
                            "{owner} references {target}{}",
                            if cascade {
                                " and cannot be removed safely"
                            } else {
                                ""
                            }
                        ));
                    }
                }
            }
        }
        blockers.sort();
        blockers.dedup();
        Ok(DeletePreview {
            removed: removal.into_iter().collect(),
            blockers,
        })
    }

    pub fn delete_sensors(&self, slots: &[usize], cascade: bool) -> Result<Self> {
        if slots.is_empty() {
            return Ok(self.clone());
        }
        let preview = self.preview_sensor_delete(slots, cascade)?;
        if !preview.blockers.is_empty() {
            return Err(Error::Invalid(preview.blockers.join("\n")));
        }

        let mut removed_slots: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
        let mut removals = Vec::new();
        for path in &preview.removed {
            let (collection, slot) = split_slot(path).expect("deletable record has a slot");
            let range = self
                .deletion_range(path)
                .expect("preview verified a record boundary");
            let replacement = if collection.ends_with("].widgets") {
                // Screen geometry keeps its widget positions. An empty widget is an eight-byte zero key.
                vec![0; 8]
            } else {
                removed_slots.entry(collection).or_default().insert(slot);
                Vec::new()
            };
            removals.push((range, replacement));
        }
        removals.sort_by_key(|(range, _)| range.start);
        let mut patches = removals.clone();
        let mut expected_counts = Vec::new();
        for (collection, slots) in &removed_slots {
            let path = format!("{collection}.count");
            let field = self
                .field(&path)
                .ok_or_else(|| Error::Invalid(format!("{path}: missing count")))?;
            let count = u16::from_le_bytes(
                self.bytes[field.range.clone()]
                    .try_into()
                    .map_err(|_| Error::Invalid(format!("{path}: unsupported count")))?,
            );
            let expected = count
                .checked_sub(slots.len() as u16)
                .ok_or_else(|| Error::Invalid(format!("{path}: invalid count")))?;
            patches.push((field.range.clone(), expected.to_le_bytes().to_vec()));
            expected_counts.push((path, expected));
        }

        let mut expected = Vec::new();
        for reference in &self.references {
            if removals
                .iter()
                .any(|(range, _)| range.contains(&reference.offset))
            {
                continue;
            }
            let mut retained = reference.clone();
            if let Some(removed) = removed_slots.get(reference.collection) {
                if removed.contains(&reference.slot) {
                    return Err(Error::Invalid(format!(
                        "{} still references {}",
                        self.reference_owner(reference.offset),
                        reference.target()
                    )));
                }
                retained.slot -= removed.range(..reference.slot).count();
                if retained.slot != reference.slot {
                    patches.push(reference.patch(retained.slot));
                }
            }
            retained.offset -= removals
                .iter()
                .take_while(|(range, _)| range.end <= reference.offset)
                .map(|(range, replacement)| range.len() - replacement.len())
                .sum::<usize>();
            expected.push(retained);
        }
        let reopened = patched_model(&self.bytes, patches)?;
        if reopened.references != expected
            || expected_counts.iter().any(|(path, count)| {
                reopened
                    .field(path)
                    .is_none_or(|field| field.typed != Value::Integer((*count).into()))
            })
        {
            return Err(Error::Invalid(
                "Sensor deletion failed reference or count read-back".into(),
            ));
        }
        Ok(reopened)
    }
}
