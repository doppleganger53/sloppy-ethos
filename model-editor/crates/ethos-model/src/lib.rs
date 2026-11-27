//! Same-format editing is enabled only for fields with a verified structural location.
mod channel_edit;
mod compare;
mod edit;
mod legacy;
mod order;
mod prefix;
mod sections_after;

pub use compare::{compare, DiffKind, Difference};
pub use edit::{Change, Edit, FieldEdit};
use serde::Serialize;
use std::{fmt, fs, io::Read, ops::Range, path::Path};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
    Unsupported(String),
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Unsupported(message) | Self::Io(message) => {
                f.write_str(message)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", content = "stored", rename_all = "snake_case")]
pub enum Value {
    Integer(i64),
    FixedPoint {
        raw: i64,
        scale: u32,
    },
    Text(String),
    Enum {
        raw: i64,
        label: Option<String>,
    },
    Source {
        category: u8,
        options: u8,
        member: u16,
    },
    Absent,
    Opaque,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub section: String,
    pub path: String,
    pub value: String,
    pub typed: Value,
    pub range: Range<usize>,
    pub editable: bool,
    pub defaulted: bool,
    #[serde(skip)]
    projected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<EditFormat>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditFormat {
    InternalRf,
    Operand { minimum: i32, maximum: i32 },
    Percent { minimum: i16, maximum: i16 },
    Toggle,
    TrimStep,
    Label15,
    Boolean,
    TimerSeconds,
    TimerActions,
}

impl Field {
    pub(crate) fn new(
        section: &str,
        path: impl Into<String>,
        range: Range<usize>,
        typed: Value,
        value: impl Into<String>,
        editable: bool,
    ) -> Self {
        Self {
            section: section.into(),
            path: path.into(),
            typed,
            value: value.into(),
            range,
            editable,
            defaulted: false,
            projected: false,
            encoding: None,
        }
    }
    pub(crate) fn writable(mut self, encoding: EditFormat) -> Self {
        self.editable = true;
        self.encoding = Some(encoding);
        self
    }
    pub(crate) fn defaulted(mut self) -> Self {
        self.defaulted = true;
        self.projected = true;
        self
    }
    pub(crate) fn projected(mut self) -> Self {
        self.projected = true;
        self
    }
    pub(crate) fn opaque(section: &str, path: impl Into<String>, range: Range<usize>) -> Self {
        let label = format!("{} bytes", range.len());
        Self::new(section, path, range, Value::Opaque, label, false)
    }
    pub(crate) fn number(
        section: &str,
        path: impl Into<String>,
        range: Range<usize>,
        v: impl Into<i64>,
    ) -> Self {
        let v = v.into();
        Self::new(
            section,
            path,
            range,
            Value::Integer(v),
            v.to_string(),
            false,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Sensor {
    /// Zero-based slot in the stored collection, including any calculated entries.
    pub slot: usize,
    pub name: String,
    pub module: u8,
    pub band: u8,
    pub routing: u8,
    pub sub_id: u8,
    pub physical_id: u8,
    pub application_id: u16,
    pub range: Range<usize>,
}

impl Sensor {
    fn key(&self) -> (u8, u8, u8, u8, u8, u16) {
        (
            self.module,
            self.band,
            self.routing,
            self.sub_id,
            self.physical_id,
            self.application_id,
        )
    }

    pub fn path(&self) -> String {
        format!("telemetry.sensors[{}]", self.slot)
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    bytes: Vec<u8>,
    sensors: Vec<Sensor>,
    fields: Vec<Field>,
    warnings: Vec<String>,
}

pub fn crc16(payload: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in payload {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

impl Model {
    pub fn open(path: &Path) -> Result<Self> {
        let load = || -> Result<Self> {
            let mut file = fs::File::open(path)?;
            let actual = file.metadata()?.len();
            let mut header = [0; 16];
            file.read_exact(&mut header)
                .map_err(|e| Error::Invalid(format!("header: could not read 16 bytes: {e}")))?;
            let payload = u32::from_le_bytes(header[8..12].try_into().unwrap());
            if 16 + u64::from(payload) != actual {
                return Err(Error::Invalid(format!("header.payload_length: declares {payload} bytes, file has {actual} including header")));
            }
            let size = usize::try_from(actual).map_err(|_| {
                Error::Invalid("model: file exceeds available address space".into())
            })?;
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(size)
                .map_err(|e| Error::Invalid(format!("model: cannot reserve {size} bytes: {e}")))?;
            bytes.extend(header);
            file.take(u64::from(payload) + 1).read_to_end(&mut bytes)?;
            Self::parse_owned(bytes)
        };
        load().map_err(|e| Error::Invalid(format!("{}: {e}", path.display())))
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Self::parse_owned(bytes.to_vec())
    }

    fn parse_owned(bytes: Vec<u8>) -> Result<Self> {
        let header = bytes
            .get(..16)
            .ok_or_else(|| Error::Invalid("header: truncated, expected 16 bytes".into()))?;
        if &header[..4] != b"FRSK" {
            return Err(Error::Invalid("header.signature: expected FRSK".into()));
        }
        if header[4] != 1 {
            return Err(Error::Invalid(format!(
                "header.marker: unsupported {:02X} (expected 01)",
                header[4]
            )));
        }
        if header[5..8] != [0, 1, 37] {
            return Err(Error::Invalid(format!(
                "header.format: unsupported {}.{}.{} (expected 0.1.37)",
                header[5], header[6], header[7]
            )));
        }
        let size = u32::from_le_bytes(header[8..12].try_into().unwrap());
        if size as usize != bytes.len() - 16 {
            return Err(Error::Invalid(format!(
                "header.payload_length: declares {size} bytes, found {}",
                bytes.len() - 16
            )));
        }
        let board = u16::from_le_bytes([header[12], header[13]]);
        if board != 0x1381 {
            return Err(Error::Invalid(format!(
                "header.board: unsupported {board:04X} (expected X20 Pro AW / 1381)"
            )));
        }
        let check = u16::from_le_bytes([header[14], header[15]]);
        let computed = crc16(&bytes[16..]);
        if check != computed {
            return Err(Error::Invalid(format!(
                "header.crc: stored {check:04X}, computed {computed:04X}"
            )));
        }
        let mut model = Self {
            bytes,
            sensors: Vec::new(),
            warnings: Vec::new(),
            fields: vec![
                Field::new(
                    "File",
                    "header.signature",
                    0..4,
                    Value::Text("FRSK".into()),
                    "FRSK",
                    false,
                ),
                Field::number("File", "header.marker", 4..5, 1),
                Field::new(
                    "File",
                    "header.format",
                    5..8,
                    Value::Text("0.1.37".into()),
                    "0.1.37",
                    false,
                ),
                Field::number("File", "header.payload_length", 8..12, size),
                Field::new(
                    "File",
                    "header.board",
                    12..14,
                    Value::Enum {
                        raw: board.into(),
                        label: Some("X20 Pro AW".into()),
                    },
                    "X20 Pro AW (1381)",
                    false,
                ),
                Field::new(
                    "File",
                    "header.crc",
                    14..16,
                    Value::Integer(check.into()),
                    format!("{check:04X}"),
                    false,
                ),
            ],
        };
        legacy::decode(&mut model)?;
        model.describe_gaps();
        model.list_order();
        Ok(model)
    }

    fn describe_gaps(&mut self) {
        // Record tags, reserved bytes, and unlabelled framing are inspectable too.
        let mut fields: Vec<_> = self.fields.iter().collect();
        fields.sort_by_key(|f| (f.range.start, f.range.end));
        let mut gaps = Vec::new();
        let mut cursor = 0;
        for field in fields {
            if cursor < field.range.start {
                gaps.push(Field::opaque(
                    &field.section,
                    format!("{}.preceding_bytes", field.path),
                    cursor..field.range.start,
                ));
            }
            cursor = cursor.max(field.range.end);
        }
        if cursor < self.bytes.len() {
            gaps.push(Field::opaque(
                "Other settings",
                "model.trailing_bytes",
                cursor..self.bytes.len(),
            ));
        }
        self.fields.extend(gaps);
        self.fields.sort_by_key(|f| (f.range.start, f.range.end));
    }

    pub fn fields(&self) -> Vec<Field> {
        self.fields.clone()
    }
    pub fn sensors(&self) -> &[Sensor] {
        &self.sensors
    }
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
    pub fn raw(&self, range: Range<usize>) -> Option<&[u8]> {
        self.bytes.get(range)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}
