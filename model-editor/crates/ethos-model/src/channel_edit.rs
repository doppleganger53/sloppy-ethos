//! The radio stores an untouched output channel in a single default marker.
//! This full default record was measured by naming a single stock channel in the 1.6.8 simulator.
use super::{crc16, EditFormat, Field, FieldEdit, Model, Result, Value};
use std::collections::BTreeSet;

const EXPANDED: &[u8] = &[
    0, 0, 0x80, 0x80, 0x82, 0xe8, 3, 0x82, 0x18, 0xfc, 0x80, 1, 0, 0, 0, 0, 0, 0, 0,
];

pub(crate) fn default_fields(f: &mut Vec<Field>, root: &str, at: usize) {
    let span = at..at + 1;
    let name = Field::new(
        "Outputs",
        format!("{root}.name"),
        span.clone(),
        Value::Text(String::new()),
        "",
        false,
    );
    f.push(name.writable(EditFormat::Label15).defaulted());
    let invert = Field::new(
        "Outputs",
        format!("{root}.invert"),
        span.clone(),
        Value::Enum {
            raw: 0,
            label: Some("Off".into()),
        },
        "Off",
        false,
    );
    f.push(invert.writable(EditFormat::Toggle).defaulted());
    for (key, v, minimum, maximum) in [
        ("limit_min", -1000, -1500, 0),
        ("limit_max", 1000, 0, 1500),
        ("center", 0, -1000, 1000),
    ] {
        let field = Field::new(
            "Outputs",
            format!("{root}.{key}"),
            span.clone(),
            Value::FixedPoint { raw: v, scale: 10 },
            format!("{}%", v / 10),
            false,
        );
        f.push(
            field
                .writable(EditFormat::Percent { minimum, maximum })
                .defaulted(),
        );
    }
}
fn envelope(b: &mut [u8]) {
    let size = (b.len() - 16) as u32;
    b[8..12].copy_from_slice(&size.to_le_bytes());
    let crc = crc16(&b[16..]);
    b[14..16].copy_from_slice(&crc.to_le_bytes());
}
impl Model {
    pub(crate) fn expand_channels(&self, edits: &[FieldEdit]) -> Result<Option<Model>> {
        let mut slots = BTreeSet::new();
        for e in edits {
            if !e.path.starts_with("channels[") {
                continue;
            }
            if let Some(f) = self
                .fields
                .iter()
                .find(|f| f.path == e.path && f.defaulted && f.editable)
            {
                slots.insert(f.range.start);
            }
        }
        if slots.is_empty() {
            return Ok(None);
        }
        let mut b = self.to_bytes();
        for p in slots.into_iter().rev() {
            b.splice(p..p + 1, EXPANDED.iter().copied());
        }
        envelope(&mut b);
        Self::parse_owned(b).map(Some)
    }
    pub(crate) fn compact_channels(&self, edits: &[FieldEdit]) -> Result<Option<Model>> {
        let candidates: BTreeSet<_> = edits
            .iter()
            .filter_map(|f| {
                f.path
                    .strip_prefix("channels[")
                    .and_then(|p| p.split_once("]."))
                    .map(|(slot, _)| format!("channels[{slot}]"))
            })
            .collect();
        let mut spans = Vec::new();
        for root in candidates {
            let (Some(flag), Some(name)) = (
                self.fields
                    .iter()
                    .find(|f| f.path == format!("{root}.default")),
                self.fields
                    .iter()
                    .find(|f| f.path == format!("{root}.name")),
            ) else {
                continue;
            };
            let span = flag.range.start..name.range.end;
            if self.bytes[span.clone()] == *EXPANDED {
                spans.push(span);
            }
        }
        if spans.is_empty() {
            return Ok(None);
        }
        spans.sort_by_key(|r| r.start);
        let mut b = self.to_bytes();
        for span in spans.into_iter().rev() {
            b.splice(span, [1]);
        }
        envelope(&mut b);
        Self::parse_owned(b).map(Some)
    }
}
