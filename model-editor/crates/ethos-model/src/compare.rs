use super::{Field, Model, Sensor, Value};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Changed,
    Added,
    Removed,
    Moved,
    Ambiguous,
    Opaque,
}
impl fmt::Display for DiffKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Difference {
    pub section: String,
    pub path: String,
    pub kind: DiffKind,
    pub before: Option<String>,
    pub after: Option<String>,
}
impl Difference {
    fn new(
        section: &str,
        path: impl Into<String>,
        kind: DiffKind,
        before: Option<String>,
        after: Option<String>,
    ) -> Self {
        Self {
            section: section.into(),
            path: path.into(),
            kind,
            before,
            after,
        }
    }
}
type FieldMap<'a> = BTreeMap<&'a str, &'a Field>;

struct Record<'a> {
    slot: usize,
    variant: i64,
    name: &'a str,
    raw: &'a [u8],
    sport: Option<&'a Sensor>,
    fields: FieldMap<'a>,
}

impl Record<'_> {
    fn describe(&self) -> String {
        if let Some(s) = self.sport {
            format!(
                "{} [slot {}, module {}, band {}, route {}, sub {}, Physical {:02X}, Application {:04X}]",
                self.name, self.slot, s.module, s.band, s.routing, s.sub_id, s.physical_id, s.application_id
            )
        } else {
            let kind = match self.variant {
                1 => "calculated",
                2 => "DIY",
                _ => "telemetry",
            };
            format!("{} [slot {}, {kind}]", self.name, self.slot)
        }
    }
}

fn grouped<'a, 'm: 'a, K: Ord>(
    records: impl IntoIterator<Item = &'a Record<'m>>,
    key: impl Fn(&Record<'m>) -> Option<K>,
) -> BTreeMap<K, Vec<&'a Record<'m>>> {
    let mut groups = BTreeMap::new();
    for record in records {
        if let Some(key) = key(record) {
            groups.entry(key).or_insert_with(Vec::new).push(record);
        }
    }
    groups
}

fn describe_group(records: &[&Record<'_>]) -> String {
    records
        .iter()
        .map(|r| r.describe())
        .collect::<Vec<_>>()
        .join(" / ")
}

fn take_unique<'a, 'm, K: Ord>(
    left: &mut Vec<&'a Record<'m>>,
    right: &mut Vec<&'a Record<'m>>,
    key: impl Fn(&Record<'m>) -> Option<K>,
    pairs: &mut Vec<(&'a Record<'m>, &'a Record<'m>)>,
) {
    let lhs = grouped(left.iter().copied(), &key);
    let rhs = grouped(right.iter().copied(), key);
    let mut paired_left = BTreeSet::new();
    let mut paired_right = BTreeSet::new();
    for (key, old) in lhs {
        if let ([a], Some(&[b])) = (old.as_slice(), rhs.get(&key).map(Vec::as_slice)) {
            pairs.push((*a, b));
            paired_left.insert(a.slot);
            paired_right.insert(b.slot);
        }
    }
    left.retain(|r| !paired_left.contains(&r.slot));
    right.retain(|r| !paired_right.contains(&r.slot));
}

fn field_diff(a: &Model, b: &Model, old: &Field, new: &Field, path: &str) -> Option<Difference> {
    if old.typed == new.typed {
        if old.projected || new.projected {
            return None;
        }
        let l = &a.bytes[old.range.clone()];
        let r = &b.bytes[new.range.clone()];
        if l == r {
            return None;
        }
        let at = l
            .iter()
            .zip(r)
            .position(|(x, y)| x != y)
            .unwrap_or(l.len().min(r.len()));
        let display = |data: &[u8], base: usize| {
            format!(
                "{} bytes; @0x{:X} {}",
                data.len(),
                base + at,
                data.get(at..)
                    .unwrap_or(&[])
                    .iter()
                    .take(8)
                    .map(|v| format!("{v:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        return Some(Difference::new(
            &old.section,
            path,
            DiffKind::Opaque,
            Some(display(l, old.range.start)),
            Some(display(r, new.range.start)),
        ));
    }
    Some(Difference::new(
        &old.section,
        path,
        DiffKind::Changed,
        Some(old.value.clone()),
        Some(new.value.clone()),
    ))
}

struct FieldIndex<'a> {
    ordinary: FieldMap<'a>,
    records: Vec<Record<'a>>,
}

impl<'a> FieldIndex<'a> {
    fn new(model: &'a Model) -> Self {
        let mut ordinary = BTreeMap::new();
        let mut slots: BTreeMap<usize, FieldMap<'a>> = BTreeMap::new();
        for f in &model.fields {
            if f.summary || f.path == super::order::ORDER {
                continue;
            }
            if let Some((slot, leaf)) = f
                .path
                .strip_prefix("telemetry.sensors[")
                .and_then(|tail| tail.split_once("]."))
                .and_then(|(slot, leaf)| Some((slot.parse::<usize>().ok()?, leaf)))
            {
                slots.entry(slot).or_default().insert(leaf, f);
            } else {
                ordinary.insert(f.path.as_str(), f);
            }
        }
        let sport: BTreeMap<_, _> = model.sensors().iter().map(|s| (s.slot, s)).collect();
        let mut records = Vec::with_capacity(slots.len());
        for (slot, fields) in slots {
            // Compare every decoded telemetry variant. The S.Port editing index is a subset.
            if let (Some(tag), Some(label)) = (fields.get("variant"), fields.get("name")) {
                if let (Value::Integer(variant), Value::Text(name)) = (&tag.typed, &label.typed) {
                    let end = fields
                        .values()
                        .map(|f| f.range.end)
                        .max()
                        .unwrap_or(tag.range.end);
                    records.push(Record {
                        slot,
                        variant: *variant,
                        name,
                        raw: &model.bytes[tag.range.start..end],
                        sport: sport.get(&slot).copied(),
                        fields,
                    });
                    continue;
                }
            }
            // Future rows with no verified identity still get an ordinary path-based diff.
            ordinary.extend(fields.into_values().map(|f| (f.path.as_str(), f)));
        }
        Self { ordinary, records }
    }
}

// A globally unique unchanged S.Port key can identify a renamed row after named
// matches have been removed. Count every record, including previously matched ones.
fn pair_renamed_sensors<'a, 'm>(
    old: &'a FieldIndex<'m>,
    new: &'a FieldIndex<'m>,
    removed: &mut BTreeMap<usize, &'a Record<'m>>,
    added: &mut BTreeMap<usize, &'a Record<'m>>,
    pairs: &mut Vec<(&'a Record<'m>, &'a Record<'m>)>,
    differences: &mut Vec<Difference>,
) {
    if removed.is_empty() || added.is_empty() {
        return;
    }
    let by_key = |r: &Record<'_>| r.sport.map(Sensor::key);
    let old_keys = grouped(&old.records, by_key);
    let mut new_keys = grouped(&new.records, by_key);
    for (key, old_group) in old_keys {
        let Some(new_group) = new_keys.remove(&key) else {
            continue;
        };
        let old_candidates: Vec<_> = old_group
            .iter()
            .copied()
            .filter(|r| removed.contains_key(&r.slot))
            .collect();
        let new_candidates: Vec<_> = new_group
            .iter()
            .copied()
            .filter(|r| added.contains_key(&r.slot))
            .collect();
        if old_candidates.is_empty() || new_candidates.is_empty() {
            continue;
        }
        for r in &old_candidates {
            removed.remove(&r.slot);
        }
        for r in &new_candidates {
            added.remove(&r.slot);
        }
        if let ([a], [b]) = (old_group.as_slice(), new_group.as_slice()) {
            pairs.push((*a, *b));
        } else {
            let (module, band, route, sub, physical, application) = key;
            differences.push(Difference::new(
                "Telemetry", format!("S.Port key [module {module}, band {band}, route {route}, sub {sub}, Physical {physical:02X}, Application {application:04X}]"),
                DiffKind::Ambiguous, Some(describe_group(&old_group)), Some(describe_group(&new_group)),
            ));
        }
    }
}

fn diff_fields(
    a: &Model,
    b: &Model,
    left: &BTreeMap<&str, &Field>,
    right: &BTreeMap<&str, &Field>,
    prefix: &str,
    out: &mut Vec<Difference>,
) {
    for leaf in left
        .keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let path = format!("{prefix}{leaf}");
        match (left.get(leaf), right.get(leaf)) {
            (Some(x), Some(y)) => {
                if let Some(diff) = field_diff(a, b, x, y, &path) {
                    out.push(diff);
                }
            }
            (Some(x), None) => out.push(Difference::new(
                &x.section,
                path,
                DiffKind::Removed,
                Some(x.value.clone()),
                None,
            )),
            (None, Some(y)) => out.push(Difference::new(
                &y.section,
                path,
                DiffKind::Added,
                None,
                Some(y.value.clone()),
            )),
            _ => {}
        }
    }
}

/// Compare stored values. Sensor identity matching is independent of the file offsets.
pub fn compare(a: &Model, b: &Model) -> Vec<Difference> {
    if a.bytes == b.bytes {
        return Vec::new();
    }
    let mut result = Vec::new();
    let left_fields = FieldIndex::new(a);
    let right_fields = FieldIndex::new(b);
    let left = grouped(&left_fields.records, |r| Some((r.variant, r.name)));
    let right = grouped(&right_fields.records, |r| Some((r.variant, r.name)));
    let mut removed = BTreeMap::new();
    let mut added = BTreeMap::new();
    let mut pairs = Vec::new();
    let names: BTreeSet<_> = left.keys().chain(right.keys()).copied().collect();
    for group in names {
        let mut l = left.get(&group).cloned().unwrap_or_default();
        let mut r = right.get(&group).cloned().unwrap_or_default();
        if l.len() == r.len() && l.iter().zip(&r).all(|(x, y)| x.raw == y.raw) {
            pairs.extend(l.into_iter().zip(r));
            continue;
        }
        if l.len() == 1 && r.len() == 1 {
            pairs.push((l[0], r[0]));
            continue;
        }
        take_unique(&mut l, &mut r, |s| Some(s.raw), &mut pairs);
        take_unique(&mut l, &mut r, |s| s.sport.map(Sensor::key), &mut pairs);
        take_unique(
            &mut l,
            &mut r,
            |s| s.sport.map(|v| (v.module, v.band, v.routing, v.sub_id)),
            &mut pairs,
        );
        // The first byte of a calculated record's parsed key is its formula.
        // Other calculated key bytes and DIY keys remain opaque.
        take_unique(
            &mut l,
            &mut r,
            |s| (s.variant == 1).then(|| s.raw[1]),
            &mut pairs,
        );
        if !l.is_empty() && !r.is_empty() {
            result.push(Difference::new(
                "Telemetry",
                group.1,
                DiffKind::Ambiguous,
                Some(describe_group(&l)),
                Some(describe_group(&r)),
            ));
        } else {
            removed.extend(l.into_iter().map(|r| (r.slot, r)));
            added.extend(r.into_iter().map(|r| (r.slot, r)));
        }
    }
    pair_renamed_sensors(
        &left_fields,
        &right_fields,
        &mut removed,
        &mut added,
        &mut pairs,
        &mut result,
    );
    for r in removed.into_values() {
        result.push(Difference::new(
            "Telemetry",
            r.name,
            DiffKind::Removed,
            Some(r.describe()),
            None,
        ));
    }
    for r in added.into_values() {
        result.push(Difference::new(
            "Telemetry",
            r.name,
            DiffKind::Added,
            None,
            Some(r.describe()),
        ));
    }
    diff_fields(
        a,
        b,
        &left_fields.ordinary,
        &right_fields.ordinary,
        "",
        &mut result,
    );
    pairs.sort_by_key(|(x, _)| x.slot);
    let mut new_order: Vec<_> = pairs.iter().map(|(_, y)| y.slot).collect();
    new_order.sort_unstable();
    for ((x, y), expected) in pairs.into_iter().zip(new_order) {
        let label = format!("{} [slot {}]", x.name, x.slot);
        if y.slot != expected && x.slot != y.slot {
            result.push(Difference::new(
                "Telemetry",
                format!("{label}.order"),
                DiffKind::Moved,
                Some(x.slot.to_string()),
                Some(y.slot.to_string()),
            ));
        }
        diff_fields(
            a,
            b,
            &x.fields,
            &y.fields,
            &format!("{label}."),
            &mut result,
        );
    }
    result
}
