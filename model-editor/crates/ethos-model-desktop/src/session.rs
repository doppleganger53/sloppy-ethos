use ethos_model::{compare, Change, Difference, Edit, EditFormat, Field, FieldEdit, Model, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// FrSky's Model Setup menu. File framing and opaque regions follow the radio's sections.
const MENU_ORDER: &[&str] = &[
    "Model",
    "Flight modes",
    "Mixes",
    "Outputs",
    "Timers",
    "Trims",
    "RF",
    "Telemetry",
    "Checklist",
    "Logic switches",
    "Special functions",
    "Curves",
    "Variables",
    "Trainer",
    "Lua",
    "Screens",
    "File",
    "Other settings",
];

fn menu_rank(section: &str) -> usize {
    MENU_ORDER
        .iter()
        .position(|&s| s == section)
        .unwrap_or(MENU_ORDER.len())
}

pub fn section_label(section: &str) -> &str {
    match section {
        "Model" => "Edit model",
        "Outputs" => "Channels",
        "RF" => "RF system",
        other => other,
    }
}

fn ordered_diff(backup: &Model, current: &Model) -> Vec<Difference> {
    let mut differences = compare(backup, current);
    differences.sort_by_key(|d| menu_rank(&d.section));
    differences
}

fn sensor_id_path(path: &str) -> Option<(usize, bool)> {
    let (slot, field) = path.strip_prefix("telemetry.sensors[")?.split_once("].")?;
    let is_physical = match field {
        "physical_id" => true,
        "application_id" => false,
        _ => return None,
    };
    Some((slot.parse().ok()?, is_physical))
}

pub struct OpenModel {
    pub path: PathBuf,
    pub model: Model,
    pub fields: Vec<Field>,
}

impl OpenModel {
    fn load(path: &Path) -> Result<Self, String> {
        let model = Model::open(path).map_err(|e| e.to_string())?;
        let mut fields = model.fields();
        fields.sort_by_key(|f| menu_rank(&f.section));
        Ok(Self {
            path: path.into(),
            model,
            fields,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum View {
    Settings(Option<String>),
    Differences(Option<String>),
    Telemetry,
    Bytes(String),
    Staged,
}

impl View {
    pub fn title(&self) -> &str {
        match self {
            Self::Settings(None) => "Model settings",
            Self::Differences(None) => "Differences",
            Self::Settings(Some(section)) | Self::Differences(Some(section)) => {
                section_label(section)
            }
            Self::Bytes(path) => path,
            Self::Telemetry => "Telemetry IDs",
            Self::Staged => "Staged changes",
        }
    }

    pub fn columns(&self) -> [&'static str; 4] {
        match self {
            Self::Settings(_) => ["Field", "Stored value", "File bytes", "Access"],
            Self::Differences(_) => ["Field", "Backup", "Current", "Change"],
            Self::Bytes(_) => ["File offset", "Hex bytes", "ASCII", ""],
            Self::Telemetry => ["Sensor", "Physical ID", "Application ID", "Route / sub-ID"],
            Self::Staged => ["Field", "Current", "Proposed", "Slot"],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    Field(String),
    Sensor(usize),
}

pub struct Row {
    pub cells: [String; 4],
    pub target: Option<Target>,
    pub searchable: String,
}

impl Row {
    fn new(cells: [String; 4], target: Option<Target>, extra: &str) -> Self {
        let searchable = format!("{} {extra}", cells.join(" ")).to_lowercase();
        Self {
            cells,
            target,
            searchable,
        }
    }
}

#[derive(Default)]
pub struct Session {
    pub current: Option<OpenModel>,
    pub backup: Option<OpenModel>,
    differences: Vec<Difference>,
    pending: BTreeMap<usize, Edit>,
    fields: BTreeMap<String, FieldEdit>,
    pub unsaved: bool,
    pub last_export: Option<PathBuf>,
}

impl Session {
    pub fn warnings(&self) -> Vec<String> {
        [
            ("Current", self.current.as_ref()),
            ("Backup", self.backup.as_ref()),
        ]
        .into_iter()
        .flat_map(|(side, opened)| {
            opened.into_iter().flat_map(move |m| {
                m.model
                    .warnings()
                    .iter()
                    .map(move |w| format!("{side} — {w}"))
            })
        })
        .collect()
    }
    pub fn open(&mut self, current: &Path, backup: Option<&Path>) -> Result<(), String> {
        let current = OpenModel::load(current)?;
        let backup = backup.map(OpenModel::load).transpose()?;
        let differences = backup
            .as_ref()
            .map(|old| ordered_diff(&old.model, &current.model))
            .unwrap_or_default();
        *self = Self {
            current: Some(current),
            backup,
            differences,
            ..Self::default()
        };
        Ok(())
    }

    pub fn compare_backup(&mut self, path: &Path) -> Result<(), String> {
        let current = self.current.as_ref().ok_or("Open a current model first.")?;
        let backup = OpenModel::load(path)?;
        self.differences = ordered_diff(&backup.model, &current.model);
        self.backup = Some(backup);
        Ok(())
    }

    pub fn sections(&self, diff: bool) -> Vec<String> {
        let mut sections = Vec::new();
        let mut add = |section: &str| {
            if !sections.iter().any(|s| s == section) {
                sections.push(section.to_owned());
            }
        };
        if diff {
            for d in &self.differences {
                add(&d.section);
            }
        } else if let Some(current) = &self.current {
            for f in &current.fields {
                add(&f.section);
            }
        }
        sections
    }

    pub fn rows(&self, view: &View, search: &str) -> Vec<Row> {
        let Some(current) = &self.current else {
            return vec![];
        };
        let rows: Vec<_> = match view {
            View::Settings(section) => current
                .fields
                .iter()
                .filter(|f| section.as_ref().is_none_or(|s| s == &f.section))
                .map(|f| {
                    Row::new(
                        [
                            f.path.clone(),
                            f.value.clone(),
                            format!("{}..{}", f.range.start, f.range.end),
                            if f.editable { "Editable" } else { "Read only" }.into(),
                        ],
                        Some(Target::Field(f.path.clone())),
                        &format!("{} {}", f.section, section_label(&f.section)),
                    )
                })
                .collect(),
            View::Differences(section) => self
                .differences
                .iter()
                .filter(|d| section.as_ref().is_none_or(|s| s == &d.section))
                .map(|d| {
                    Row::new(
                        [
                            d.path.clone(),
                            d.before.clone().unwrap_or_default(),
                            d.after.clone().unwrap_or_default(),
                            d.kind.to_string(),
                        ],
                        None,
                        &format!("{} {}", d.section, section_label(&d.section)),
                    )
                })
                .collect(),
            View::Telemetry => current
                .model
                .sensors()
                .iter()
                .map(|s| {
                    let pending = self.pending.contains_key(&s.slot);
                    Row::new(
                        [
                            format!(
                                "{} [{}]{}",
                                s.name,
                                s.slot,
                                if pending { " • staged" } else { "" }
                            ),
                            format!("{:02X}", s.physical_id),
                            format!("{:04X}", s.application_id),
                            format!(
                                "module {}, band {}, route {:02X}, sub {}",
                                s.module, s.band, s.routing, s.sub_id
                            ),
                        ],
                        Some(Target::Sensor(s.slot)),
                        "",
                    )
                })
                .collect(),
            View::Bytes(path) => {
                let Some(field) = current.fields.iter().find(|f| &f.path == path) else {
                    return vec![];
                };
                let bytes = current.model.to_bytes();
                let Some(slice) = bytes.get(field.range.clone()) else {
                    return vec![];
                };
                slice
                    .chunks(16)
                    .enumerate()
                    .map(|(i, chunk)| {
                        let hex = chunk
                            .iter()
                            .map(|b| format!("{b:02X}"))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let ascii = chunk
                            .iter()
                            .map(|&b| {
                                if (32..=126).contains(&b) {
                                    b as char
                                } else {
                                    '.'
                                }
                            })
                            .collect();
                        Row::new(
                            [
                                format!("{:08X}", field.range.start + i * 16),
                                hex,
                                ascii,
                                String::new(),
                            ],
                            None,
                            path,
                        )
                    })
                    .collect()
            }
            View::Staged => {
                if let Ok(changes) = self.preview() {
                    let needle = search.trim().to_lowercase();
                    return changes
                        .into_iter()
                        .map(|change| {
                            let mut name = change.path.clone();
                            let mut slot_label = String::new();
                            if let Some((slot, physical)) = sensor_id_path(&change.path) {
                                if let Some(sensor) =
                                    current.model.sensors().iter().find(|s| s.slot == slot)
                                {
                                    name = format!(
                                        "{} / {}",
                                        sensor.name,
                                        if physical {
                                            "Physical ID"
                                        } else {
                                            "Application ID"
                                        }
                                    );
                                    slot_label = slot.to_string();
                                }
                            }
                            Row::new(
                                [name, change.before, change.after, slot_label],
                                Some(Target::Field(change.path.clone())),
                                &change.path,
                            )
                        })
                        .filter(|r| r.searchable.contains(&needle))
                        .collect();
                }
                let mut rows = vec![];
                for s in current.model.sensors() {
                    if let Some(e) = self.pending.get(&s.slot) {
                        if let Some(id) = e.physical_id {
                            rows.push(Row::new(
                                [
                                    format!("{} / Physical ID", s.name),
                                    format!("{:02X}", s.physical_id),
                                    format!("{id:02X}"),
                                    s.slot.to_string(),
                                ],
                                Some(Target::Sensor(s.slot)),
                                "",
                            ));
                        }
                        if let Some(id) = e.application_id {
                            rows.push(Row::new(
                                [
                                    format!("{} / Application ID", s.name),
                                    format!("{:04X}", s.application_id),
                                    format!("{id:04X}"),
                                    s.slot.to_string(),
                                ],
                                Some(Target::Sensor(s.slot)),
                                "",
                            ));
                        }
                    }
                }
                for edit in self.fields.values() {
                    if let Some(field) = current.fields.iter().find(|f| f.path == edit.path) {
                        rows.push(Row::new(
                            [
                                field.path.clone(),
                                field.value.clone(),
                                edit.value.clone(),
                                String::new(),
                            ],
                            Some(Target::Field(field.path.clone())),
                            &field.section,
                        ));
                    }
                }
                rows
            }
        };
        let search = search.trim().to_lowercase();
        let search = if *view == View::Telemetry {
            search.strip_prefix("0x").unwrap_or(&search)
        } else {
            &search
        };
        rows.into_iter()
            .filter(|r| r.searchable.contains(search))
            .collect()
    }

    pub fn ids(&self, slot: usize) -> Option<(u8, u16)> {
        let sensor = self
            .current
            .as_ref()?
            .model
            .sensors()
            .iter()
            .find(|s| s.slot == slot)?;
        let edit = self.pending.get(&slot);
        Some((
            edit.and_then(|e| e.physical_id)
                .unwrap_or(sensor.physical_id),
            edit.and_then(|e| e.application_id)
                .unwrap_or(sensor.application_id),
        ))
    }

    pub fn id_permissions(&self, slot: usize) -> (bool, bool) {
        let root = format!("telemetry.sensors[{slot}]");
        let editable = |name| {
            self.field(&format!("{root}.{name}"))
                .is_some_and(|f| f.editable)
        };
        (editable("physical_id"), editable("application_id"))
    }

    pub fn stage(&mut self, slot: usize, physical: &str, application: &str) -> Result<(), String> {
        let model = &self.current.as_ref().ok_or("Open a model first.")?.model;
        let sensor = model
            .sensors()
            .iter()
            .find(|s| s.slot == slot)
            .ok_or("Select a sensor in Telemetry IDs.")?;
        let (can_physical, can_application) = self.id_permissions(slot);
        let physical_id = if can_physical {
            parse_hex(physical, 2, "Physical ID")?
        } else {
            u16::from(sensor.physical_id)
        };
        if physical_id > 0x1b {
            return Err("Physical ID must be between 00 and 1B.".into());
        }
        let application_id = if can_application {
            parse_hex(application, 4, "Application ID")?
        } else {
            sensor.application_id
        };
        let edit = Edit {
            slot,
            physical_id: (physical_id as u8 != sensor.physical_id).then_some(physical_id as u8),
            application_id: (application_id != sensor.application_id).then_some(application_id),
        };
        if edit.physical_id.is_none() && edit.application_id.is_none() {
            self.pending.remove(&slot);
        } else {
            self.pending.insert(slot, edit);
        }
        self.unsaved = self.has_edits();
        Ok(())
    }

    pub fn field(&self, path: &str) -> Option<&Field> {
        self.current
            .as_ref()?
            .fields
            .iter()
            .find(|f| f.path == path)
    }
    pub fn field_help(field: &Field) -> String {
        let mut parts = Vec::new();
        if let Value::Enum { raw, label } = &field.typed {
            parts.push(format!(
                "Stored enum {raw} ({})",
                label.as_deref().unwrap_or("unknown")
            ));
        }
        if field.defaulted {
            parts.push("Firmware default".to_owned());
        }
        if field.editable {
            let hint = match field.encoding {
                Some(EditFormat::Percent { minimum, maximum }) => format!(
                    "Percentage {} to {} in 0.1% steps",
                    f64::from(minimum) / 10.0,
                    f64::from(maximum) / 10.0
                ),
                Some(EditFormat::Operand { minimum, maximum }) => {
                    format!("Whole number from {minimum} to {maximum}")
                }
                Some(EditFormat::Toggle | EditFormat::InternalRf) => "Choose On or Off".to_owned(),
                Some(EditFormat::TrimStep) => "Choose Fine or Medium".to_owned(),
                Some(EditFormat::Label15) => "Name of up to 15 UTF-8 bytes".to_owned(),
                Some(EditFormat::Boolean) => "Choose Up or Down".to_owned(),
                Some(EditFormat::TimerSeconds) => "Whole seconds".to_owned(),
                Some(EditFormat::TimerActions) => {
                    "Edit the ordered list of timer audio actions. An empty list [] clears the actions.".to_owned()
                }
                Some(EditFormat::LogicInputs) => "Enter 2–9 switch names as a JSON array, using null for an empty input".to_owned(),
                Some(EditFormat::VariableRates) => r#"Use up to 9 rows, for example [{"condition":"SA Up","value":3}]. Stored value 3 is 0.3%."#.to_owned(),
                Some(EditFormat::VariableActions) => r#"Use up to 10 rows, for example [{"condition":"FS1","function":"Add","source":3}]."#.to_owned(),
                Some(EditFormat::TrimTarget) => "Enter Trim N using an existing trim slot, starting at 0".to_owned(),
                Some(EditFormat::VariableFunction) => "Choose Assign, Add, Subtract, Multiply, Divide, Percent, Min, or Max".to_owned(),
                Some(EditFormat::LogicComparison) => "Choose Equal, Greater or Less".to_owned(),
                Some(EditFormat::LogicCombinator) => "Choose AND, OR or XOR".to_owned(),
                Some(EditFormat::LogicInput) => "Choose a switch position such as SA Up, FS1, or None".to_owned(),
                Some(EditFormat::TimerInterval) => "Enter 0.1 to 175.0 seconds in tenths".to_owned(),
                Some(EditFormat::CurveSelector) => "Choose None, Expo N or Curve slot N".to_owned(),
                Some(EditFormat::CurvePoints) => {
                    "Choose Edit points to change the coordinates or the point count".to_owned()
                }
                Some(EditFormat::CurveFunction) => {
                    "Choose None, x>0, x<0, |x|, f>0, f<0, or |f|".to_owned()
                }
                None if field.path == "curves.order" => concat!("Arrange zero-based curve slots, for example [1,0]. ",
                    "Omitting a curve removes it and clears its references.").to_owned(),
                None if field.path == "telemetry.sensors.order" => {
                    "Arrange zero-based sensor slots, for example [1,0]. Referenced sensors must stay in the list.".to_owned()
                }
                None if field.path.starts_with("screens[") && field.path.ends_with(".source") => {
                    "Choose None, Timer N, or Telemetry N using a zero-based slot".to_owned()
                }
                None => "Enter a proposed value and choose Stage change".to_owned(),
            };
            parts.push(hint);
        } else if let Some((_, false)) = sensor_id_path(&field.path) {
            parts.push(
                "No replacement Application ID has been verified for this sensor family".to_owned(),
            );
        } else {
            parts.push("Read only".to_owned());
        }
        parts.join("   •   ")
    }
    pub fn field_value(&self, path: &str) -> Option<String> {
        if let Some((slot, physical)) = sensor_id_path(path) {
            let (physical_id, application_id) = self.ids(slot)?;
            return Some(if physical {
                format!("{physical_id:02X}")
            } else {
                format!("{application_id:04X}")
            });
        }
        self.fields
            .get(path)
            .map(|f| f.value.clone())
            .or_else(|| self.field(path).map(|f| f.value.clone()))
    }
    pub fn point_field(&self, selected: &str) -> Option<&Field> {
        let field = if let Some((curve, _)) = selected.split_once(".points[") {
            self.field(&format!("{curve}.points"))
        } else {
            self.field(selected)
        }?;
        (field.editable && field.encoding == Some(EditFormat::CurvePoints)).then_some(field)
    }

    pub fn stage_points(&mut self, path: &str, value: String) -> Result<(), String> {
        let current = self.current.as_ref().ok_or("Open a model first.")?;
        let edit = FieldEdit {
            path: path.into(),
            value,
        };
        let changes = current
            .model
            .preview_fields(&[], std::slice::from_ref(&edit))
            .map_err(|e| e.to_string())?;
        if changes.is_empty() {
            self.fields.remove(path);
            self.unsaved = self.has_edits();
            return Ok(());
        }
        // A grid/count change can reuse the exact stored coordinate bytes.
        // Stage the validated request even when the point payload has no diff.
        self.stage_field(path, &edit.value)
    }
    pub fn stage_field(&mut self, path: &str, value: &str) -> Result<(), String> {
        let field = self
            .field(path)
            .ok_or_else(|| format!("Unknown field {path}"))?;
        if !field.editable {
            return Err(format!("{path} is read only."));
        }
        if let Some((slot, physical)) = sensor_id_path(path) {
            let (p, a) = self
                .ids(slot)
                .ok_or_else(|| format!("Missing sensor at {path}"))?;
            return if physical {
                self.stage(slot, value, &format!("{a:04X}"))
            } else {
                self.stage(slot, &format!("{p:02X}"), value)
            };
        }
        if value == field.value {
            self.fields.remove(path);
        } else {
            self.fields.insert(
                path.into(),
                FieldEdit {
                    path: path.into(),
                    value: value.into(),
                },
            );
        }
        self.unsaved = self.has_edits();
        Ok(())
    }
    pub fn remove(&mut self, target: &Target) {
        match target {
            Target::Sensor(slot) => {
                self.pending.remove(slot);
            }
            Target::Field(path) => {
                if let Some((slot, physical)) = sensor_id_path(path) {
                    if let Some(edit) = self.pending.get_mut(&slot) {
                        if physical {
                            edit.physical_id = None;
                        } else {
                            edit.application_id = None;
                        }
                        if edit.physical_id.is_none() && edit.application_id.is_none() {
                            self.pending.remove(&slot);
                        }
                    }
                } else {
                    self.fields.remove(path);
                }
            }
        }
        self.unsaved = self.has_edits();
    }
    pub fn has_edit(&self, target: &Target) -> bool {
        match target {
            Target::Sensor(slot) => self.pending.contains_key(slot),
            Target::Field(path) => match sensor_id_path(path) {
                Some((slot, physical)) => self.pending.get(&slot).is_some_and(|e| {
                    if physical {
                        e.physical_id.is_some()
                    } else {
                        e.application_id.is_some()
                    }
                }),
                None => self.fields.contains_key(path),
            },
        }
    }
    pub fn has_edits(&self) -> bool {
        !self.pending.is_empty() || !self.fields.is_empty()
    }
    pub fn staged_count(&self) -> usize {
        self.pending.len() + self.fields.len()
    }

    pub fn preview(&self) -> Result<Vec<Change>, String> {
        let model = &self.current.as_ref().ok_or("Open a model first.")?.model;
        model
            .preview_fields(
                &self.pending.values().copied().collect::<Vec<_>>(),
                &self.fields.values().cloned().collect::<Vec<_>>(),
            )
            .map_err(|e| e.to_string())
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        if !self.has_edits() {
            return Err("Stage a change before exporting.".into());
        }
        let current = self.current.as_ref().ok_or("Open a model first.")?;
        if path == current.path || self.backup.as_ref().is_some_and(|m| path == m.path) {
            return Err("Choose a new filename to preserve the opened models.".into());
        }
        current
            .model
            .save_fields_copy(
                path,
                &self.pending.values().copied().collect::<Vec<_>>(),
                &self.fields.values().cloned().collect::<Vec<_>>(),
            )
            .map_err(|e| e.to_string())?;
        self.last_export = Some(path.into());
        self.unsaved = false;
        Ok(())
    }
}

fn parse_hex(text: &str, width: usize, label: &str) -> Result<u16, String> {
    let text = text.trim();
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    if digits.is_empty() || digits.len() > width || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{label} requires 1–{width} hexadecimal digits."));
    }
    u16::from_str_radix(digits, 16).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(name)
    }

    #[test]
    fn failed_open_preserves_current_comparison_and_pending_edits() {
        let mut session = Session::default();
        let current = fixture("throttle-after.bin");
        let backup = fixture("throttle-before.bin");
        session.open(&current, Some(&backup)).unwrap();
        let (slot, phys) = session
            .current
            .as_ref()
            .unwrap()
            .model
            .sensors()
            .iter()
            .find(|s| s.application_id == 0x6802)
            .map(|s| (s.slot, s.physical_id))
            .unwrap();
        session.stage(slot, &format!("{phys:02X}"), "6803").unwrap();
        assert!(session.open(&fixture("absent-file.bin"), None).is_err());
        assert_eq!(session.current.as_ref().unwrap().path, current);
        assert_eq!(session.backup.as_ref().unwrap().path, backup);
        assert_eq!(session.preview().unwrap().len(), 1);
        assert_eq!(session.rows(&View::Staged, "6803").len(), 1);
        session.stage(slot, &format!("{phys:02X}"), "6802").unwrap();
        assert!(!session.has_edits());
    }

    #[test]
    fn a_physical_only_row_preserves_the_locked_application_id() {
        let mut session = Session::default();
        session.open(&fixture("sensor-variants.bin"), None).unwrap();
        let (slot, old_physical, old_application) = session
            .current
            .as_ref()
            .unwrap()
            .model
            .sensors()
            .iter()
            .find(|sensor| session.id_permissions(sensor.slot) == (true, false))
            .map(|s| (s.slot, s.physical_id, s.application_id))
            .unwrap();
        let replacement = (old_physical + 1) % 0x1C;
        session
            .stage(slot, &format!("{replacement:02X}"), "")
            .unwrap();
        assert_eq!(session.ids(slot), Some((replacement, old_application)));
        let edit = session.pending[&slot];
        assert_eq!(edit.application_id, None);
        assert_eq!(session.preview().unwrap().len(), 1);
        assert!(session
            .stage_field(&format!("telemetry.sensors[{slot}].application_id"), "6803")
            .is_err());
    }

    #[test]
    fn export_refuses_both_inputs_and_existing_hard_links() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("editor-export-{}-{nonce}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let current = directory.join("current.bin");
        let backup = directory.join("backup.bin");
        fs::copy(fixture("throttle-after.bin"), &current).unwrap();
        fs::copy(fixture("throttle-before.bin"), &backup).unwrap();
        let baseline = [fs::read(&current).unwrap(), fs::read(&backup).unwrap()];
        let alias = directory.join("existing link.bin");
        fs::hard_link(&backup, &alias).unwrap();
        let mut session = Session::default();
        session.open(&current, Some(&backup)).unwrap();
        session.stage(0, "03", "6803").unwrap();
        for dest in [&current, &backup, &alias] {
            assert!(session.save(dest).is_err());
        }
        assert_eq!(fs::read(&current).unwrap(), baseline[0]);
        assert_eq!(fs::read(&backup).unwrap(), baseline[1]);
        assert_eq!(fs::read(&alias).unwrap(), baseline[1]);
        for path in [alias, backup, current] {
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn switching_point_grid_can_reuse_the_entire_coordinate_payload() {
        let mut session = Session::default();
        session
            .open(&fixture("curve-referenced.bin"), None)
            .unwrap();
        let field = "curves[0].points";
        let old: serde_json::Value =
            serde_json::from_str(&session.field(field).unwrap().value).unwrap();
        let ordinates: Vec<f64> = old
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|p| [p["x"].as_f64().unwrap() / 10.0, p["y"].as_f64().unwrap()])
            .collect();
        session
            .stage_points(field, serde_json::to_string(&ordinates).unwrap())
            .unwrap();
        assert!(session.has_edit(&Target::Field(field.into())));
        let changes = session.preview().unwrap();
        assert!(changes
            .iter()
            .any(|c| c.path == "curves[0].easy_mode" && c.after == "On"));
        assert!(changes.iter().any(|c| c.path == "curves[0].points.count"));
    }

    #[test]
    fn invalid_hex_never_stages_an_edit() {
        let mut session = Session::default();
        session.open(&fixture("throttle-after.bin"), None).unwrap();
        let slot = session.current.as_ref().unwrap().model.sensors()[0].slot;
        for (physical, application) in [
            ("1C", "6802"),
            ("03", "-1"),
            ("03", "10000"),
            ("", "6802"),
            ("03", "zz"),
        ] {
            assert!(session.stage(slot, physical, application).is_err());
            assert!(!session.has_edits());
        }
    }
}
