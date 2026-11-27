use super::{Error, Field, Model, Result, Sensor, Value};
use std::ops::Range;
pub(crate) struct Cursor<'a> {
    pub data: &'a [u8],
    pub pos: usize,
    pub path: String,
}
impl<'a> Cursor<'a> {
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|end| *end <= self.data.len())
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "{}: truncated field at 0x{:X} (needs {n} bytes)",
                    self.path, self.pos
                ))
            })?;
        let s = &self.data[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    pub fn byte(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    pub fn word(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    pub fn dword(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    pub fn text(&mut self) -> Result<(Range<usize>, String)> {
        self.limited_text(u8::MAX)
    }
    pub fn limited_text(&mut self, limit: u8) -> Result<(Range<usize>, String)> {
        let p = self.pos;
        let n = self.byte()?;
        if n > limit {
            return Err(Error::Invalid(format!(
                "{}: string length {n} at 0x{p:X} exceeds the {limit}-byte field",
                self.path
            )));
        }
        let bytes = self.bytes(n.into())?;
        let s =
            std::str::from_utf8(bytes).map_err(|_| self.unsupported("non-UTF-8 display text"))?;
        Ok((p..self.pos, s.into()))
    }
    pub fn unsupported(&self, why: &str) -> Error {
        Error::Unsupported(format!(
            "unsupported {} at 0x{:X}: {why}",
            self.path, self.pos
        ))
    }
    pub fn source(&mut self) -> Result<(Range<usize>, Value, String)> {
        let p = self.pos;
        let cat = self.byte()?;
        let (v, s) = match cat {
            0 => (Value::Absent, "None".into()),
            0x80 => (Value::Integer(0), "0".into()),
            0x81 => {
                let n = i8::from_le_bytes([self.byte()?]);
                (Value::Integer(n.into()), n.to_string())
            }
            0x82 => {
                let n = i16::from_le_bytes(self.bytes(2)?.try_into().unwrap());
                (Value::Integer(n.into()), n.to_string())
            }
            0x83 => {
                let n = i32::from_le_bytes(self.bytes(4)?.try_into().unwrap());
                (Value::Integer(n.into()), n.to_string())
            }
            1..=0x1d => {
                let options = self.byte()?;
                let member = self.word()?;
                let prefix = match cat {
                    1 if options == 0 && member == 0 => Some("Always on"),
                    8 => Some("Analog"),
                    10 => Some("Switch position"),
                    16 => Some("Output channel"),
                    21 => Some("Timer slot"),
                    22 => Some("Telemetry slot"),
                    27 => Some("System event"),
                    _ => None,
                };
                let s = match prefix {
                    Some("Always on") => "Always on".into(),
                    Some(name) => format!("{name} {member} (options {options})"),
                    None => format!("Category {cat}, member {member}, options {options}"),
                };
                (
                    Value::Source {
                        category: cat,
                        options,
                        member,
                    },
                    s,
                )
            }
            _ => return Err(self.unsupported(&format!("source encoding {cat:02X}"))),
        };
        Ok((p..self.pos, v, s))
    }
    pub fn count(&mut self, max: u16) -> Result<usize> {
        let n = self.word()?;
        if n > max {
            return Err(self.unsupported(&format!("collection count {n} exceeds {max}")));
        }
        Ok(n.into())
    }
}
pub(crate) fn sport_suffix(app: u16, sub: u8) -> Option<usize> {
    match (app, sub) {
        (0x6800..=0x6803, 0 | 4 | 5 | 6)
        | (0x6804 | 0x6805 | 0x6808 | 0x6809, 0 | 4)
        | (0x6807, 0)
        | (0xf101, 1)
        | (0xf107 | 0x0e20 | 0x0dc0 | 0x0df0, 0)
        | (0x0d90 | 0x0dd0, 1)
        | (0x0d80 | 0x0da0 | 0x0db0, 0 | 1) => Some(2),
        (0x6800..=0x6805 | 0x6807..=0x6809, 1)
        | (0xf108, 0..=2)
        | (0x0de0, 0 | 1)
        | (0xf103, 0) => Some(10),
        (0x6800..=0x6805 | 0x6807..=0x6809, 2) => Some(6),
        (0xf010 | 0xf011 | 0x0100, 0) => Some(3),
        (0xf101, 0) => Some(18),
        _ => None,
    }
}

fn read_telemetry(
    cur: &mut Cursor<'_>,
    fields: &mut Vec<Field>,
    sensors: &mut Vec<Sensor>,
) -> Result<()> {
    cur.path = "telemetry.sensors".into();
    let p = cur.pos;
    let count = cur.word()?;
    fields.push(Field::number(
        "Telemetry",
        "telemetry.sensors.count",
        p..cur.pos,
        count,
    ));
    for slot in 0..usize::from(count) {
        let start = cur.pos;
        let root = format!("telemetry.sensors[{slot}]");
        cur.path = root.clone();
        let key = cur.bytes(8)?;
        let tag = key[0];
        let subtype = key[1];
        if tag > 2 {
            return Err(cur.unsupported(&format!("unknown sensor variant {tag}")));
        }
        let (name_range, name) = cur.limited_text(15)?;
        let common_start = cur.pos;
        let unit = cur.byte()?;
        let decimals = cur.byte()?;
        let flags = cur.byte()?;
        let limits = cur.bytes(8)?;
        let (reset, v, display) = cur.source()?;
        fields.push(Field::number(
            "Telemetry",
            format!("{root}.variant"),
            start..start + 1,
            tag,
        ));
        fields.push(Field::new(
            "Telemetry",
            format!("{root}.name"),
            name_range,
            Value::Text(name.clone()),
            name.clone(),
            false,
        ));
        fields.push(Field::number(
            "Telemetry",
            format!("{root}.display_unit"),
            common_start..common_start + 1,
            unit,
        ));
        fields.push(Field::number(
            "Telemetry",
            format!("{root}.display_decimals"),
            common_start + 1..common_start + 2,
            decimals,
        ));
        fields.push(Field::number(
            "Telemetry",
            format!("{root}.flags"),
            common_start + 2..common_start + 3,
            flags,
        ));
        for (index, label) in ["minimum", "maximum"].into_iter().enumerate() {
            let at = common_start + 3 + index * 4;
            let val = i32::from_le_bytes(limits[index * 4..index * 4 + 4].try_into().unwrap());
            fields.push(Field::number(
                "Telemetry",
                format!("{root}.{label}"),
                at..at + 4,
                val,
            ));
        }
        fields.push(Field::new(
            "Telemetry",
            format!("{root}.reset_source"),
            reset,
            v,
            display,
            false,
        ));
        let suffix_start = cur.pos;
        match (tag, subtype) {
            (0, module) => {
                let app = u16::from_le_bytes([key[6], key[7]]);
                let n = sport_suffix(app, key[4]).ok_or_else(|| {
                    cur.unsupported(&format!(
                        "S.Port Application ID {app:04X}, sub-ID {}",
                        key[4]
                    ))
                })?;
                cur.bytes(n)?;
                for (i, label) in ["module", "band", "routing", "sub_id"]
                    .into_iter()
                    .enumerate()
                {
                    fields.push(Field::number(
                        "Telemetry",
                        format!("{root}.{label}"),
                        start + i + 1..start + i + 2,
                        key[i + 1],
                    ));
                }
                let phys = key[5];
                if phys > 0x1b {
                    return Err(cur.unsupported(&format!("Physical ID {phys:02X} outside 00..1B")));
                }
                fields.push(Field::new(
                    "Telemetry",
                    format!("{root}.physical_id"),
                    start + 5..start + 6,
                    Value::Integer(phys.into()),
                    format!("{phys:02X}"),
                    true,
                ));
                fields.push(Field::new(
                    "Telemetry",
                    format!("{root}.application_id"),
                    start + 6..start + 8,
                    Value::Integer(app.into()),
                    format!("{app:04X}"),
                    (0x6800..=0x6803).contains(&app),
                ));
                sensors.push(Sensor {
                    slot,
                    name,
                    module,
                    band: key[2],
                    routing: key[3],
                    sub_id: key[4],
                    physical_id: phys,
                    application_id: app,
                    range: start..cur.pos,
                });
            }
            (1, 0) => {
                let (r, v, s) = cur.source()?;
                fields.push(Field::new(
                    "Telemetry",
                    format!("{root}.source"),
                    r,
                    v,
                    s,
                    false,
                ));
                let flag = cur.byte()?;
                if flag > 1 {
                    return Err(cur.unsupported("consumption persistence flag"));
                }
                if flag == 1 {
                    cur.bytes(4)?;
                }
            }
            (1, 6) => {
                let (r, v, s) = cur.source()?;
                fields.push(Field::new(
                    "Telemetry",
                    format!("{root}.first_source"),
                    r,
                    v,
                    s,
                    false,
                ));
                let n = cur.word()?;
                for _ in 0..n {
                    cur.source()?;
                    cur.byte()?;
                }
            }
            (2, _) => {
                cur.bytes(12)?;
            }
            _ => return Err(cur.unsupported(&format!("calculated sensor subtype {subtype}"))),
        }
        if tag != 0 {
            fields.push(Field::opaque(
                "Telemetry",
                format!("{root}.key"),
                start + 1..start + 8,
            ));
        }
        if suffix_start < cur.pos {
            fields.push(Field::opaque(
                "Telemetry",
                format!("{root}.parameters"),
                suffix_start..cur.pos,
            ));
        }
    }
    Ok(())
}

pub(crate) fn decode(model: &mut Model) -> Result<()> {
    // The firmware writes the model runtime last, outside every variable-length collection.
    let end = model
        .bytes
        .len()
        .checked_sub(4)
        .filter(|&n| n >= 19)
        .ok_or_else(|| Error::Invalid("model: missing model settings or runtime counter".into()))?;
    let mut cur = Cursor {
        data: &model.bytes[..end],
        pos: 16,
        path: "model".into(),
    };
    let mut fields = Vec::new();
    let mut sensors = Vec::new();
    let result = super::prefix::decode(&mut cur, &mut fields)
        .and_then(|()| read_telemetry(&mut cur, &mut fields, &mut sensors));
    match result {
        Ok(()) => {
            match super::sections_after::decode(&mut cur, &mut fields) {
                Ok(()) => {}
                Err(e @ Error::Unsupported(_)) => model.warnings.push(e.to_string()),
                Err(e) => return Err(e),
            }
            let through = fields.iter().map(|f| f.range.end).max().unwrap_or(cur.pos);
            if through < end {
                fields.push(Field::opaque(
                    "Other settings",
                    "model.after_telemetry",
                    through..end,
                ));
            }
            model.sensors = sensors;
        }
        Err(e @ Error::Unsupported(_)) => {
            // Incomplete record coverage cannot certify any slot or cross-record key conflicts.
            model.warnings.push(e.to_string());
            fields.retain(|f| f.section != "Telemetry");
            for f in &mut fields {
                f.editable = false;
            }
            let through = fields.iter().map(|f| f.range.end).max().unwrap_or(16);
            if through < end {
                fields.push(Field::opaque(
                    "Other settings",
                    "model.unverified",
                    through..end,
                ));
            }
        }
        Err(e) => return Err(e),
    }
    let runtime = u32::from_le_bytes(model.bytes[end..end + 4].try_into().unwrap());
    fields.push(Field::number(
        "Model",
        "model.runtime",
        end..end + 4,
        runtime,
    ));
    model.fields.extend(fields);
    Ok(())
}
