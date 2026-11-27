use super::{Field, Model, Sensor};
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
fn by_name(model: &Model) -> BTreeMap<&str, Vec<&Sensor>> {
    let mut groups = BTreeMap::new();
    for record in model.sensors() {
        groups
            .entry(record.name.as_str())
            .or_insert_with(Vec::new)
            .push(record);
    }
    groups
}
fn take_unique<'a, K: Ord>(
    left: &mut Vec<&'a Sensor>,
    right: &mut Vec<&'a Sensor>,
    key: impl Fn(&Sensor) -> K,
    pairs: &mut Vec<(&'a Sensor, &'a Sensor)>,
) {
    let mut lhs = BTreeMap::new();
    let mut rhs = BTreeMap::new();
    for &s in left.iter() {
        lhs.entry(key(s)).or_insert_with(Vec::new).push(s);
    }
    for &s in right.iter() {
        rhs.entry(key(s)).or_insert_with(Vec::new).push(s);
    }
    let mut l = BTreeSet::new();
    let mut r = BTreeSet::new();
    for (k, v) in lhs {
        if let (Some(&[b]), [a]) = (rhs.get(&k).map(Vec::as_slice), v.as_slice()) {
            pairs.push((*a, b));
            l.insert(a.slot);
            r.insert(b.slot);
        }
    }
    left.retain(|v| !l.contains(&v.slot));
    right.retain(|v| !r.contains(&v.slot));
}
fn describe(s: &Sensor) -> String {
    format!(
        "{} [slot {}, module {}, band {}, route {}, sub {}, Physical {:02X}, Application {:04X}]",
        s.name, s.slot, s.module, s.band, s.routing, s.sub_id, s.physical_id, s.application_id
    )
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
    ordinary: BTreeMap<&'a str, &'a Field>,
    sensors: BTreeMap<usize, BTreeMap<&'a str, &'a Field>>,
}
impl<'a> FieldIndex<'a> {
    fn new(model: &'a Model) -> Self {
        let mut index = Self {
            ordinary: BTreeMap::new(),
            sensors: model
                .sensors
                .iter()
                .map(|s| (s.slot, BTreeMap::new()))
                .collect(),
        };
        for f in &model.fields {
            if f.path == super::order::ORDER {
                continue;
            }
            if let Some((slot, leaf)) = f
                .path
                .strip_prefix("telemetry.sensors[")
                .and_then(|tail| tail.split_once("]."))
            {
                if let Some(slot) = slot
                    .parse::<usize>()
                    .ok()
                    .and_then(|slot| index.sensors.get_mut(&slot))
                {
                    slot.insert(leaf, f);
                    continue;
                }
            }
            index.ordinary.insert(&f.path, f);
        }
        index
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
    let left = by_name(a);
    let right = by_name(b);
    let mut pairs = Vec::new();
    let names: BTreeSet<_> = left.keys().chain(right.keys()).copied().collect();
    for name in names {
        let mut l = left.get(name).cloned().unwrap_or_default();
        let mut r = right.get(name).cloned().unwrap_or_default();
        if l.len() == r.len()
            && l.iter()
                .zip(&r)
                .all(|(x, y)| a.bytes[x.range.clone()] == b.bytes[y.range.clone()])
        {
            pairs.extend(l.into_iter().zip(r));
            continue;
        }
        if l.len() == 1 && r.len() == 1 {
            pairs.push((l[0], r[0]));
            continue;
        }
        take_unique(&mut l, &mut r, |s| s.key(), &mut pairs);
        take_unique(
            &mut l,
            &mut r,
            |s| (s.module, s.band, s.routing, s.sub_id),
            &mut pairs,
        );
        if !l.is_empty() && !r.is_empty() {
            let show = |ss: &[&Sensor]| {
                ss.iter()
                    .map(|s| describe(s))
                    .collect::<Vec<_>>()
                    .join(" / ")
            };
            result.push(Difference::new(
                "Telemetry",
                name,
                DiffKind::Ambiguous,
                Some(show(&l)),
                Some(show(&r)),
            ));
        } else {
            for s in l {
                result.push(Difference::new(
                    "Telemetry",
                    &s.name,
                    DiffKind::Removed,
                    Some(describe(s)),
                    None,
                ));
            }
            for s in r {
                result.push(Difference::new(
                    "Telemetry",
                    &s.name,
                    DiffKind::Added,
                    None,
                    Some(describe(s)),
                ));
            }
        }
    }
    let left_fields = FieldIndex::new(a);
    let right_fields = FieldIndex::new(b);
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
        if y.slot != expected {
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
            &left_fields.sensors[&x.slot],
            &right_fields.sensors[&y.slot],
            &format!("{label}."),
            &mut result,
        );
    }
    result
}
