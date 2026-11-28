use crate::session::Target;
use ethos_model::{DeletePreview, Dependencies, SensorDraft, SENSOR_FIELDS};
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

fn window(
    owner: &nwg::Window,
    title: &str,
    size: (i32, i32),
) -> Result<nwg::Window, nwg::NwgError> {
    let mut window = nwg::Window::default();
    nwg::Window::builder()
        .parent(Some(owner))
        .flags(nwg::WindowFlags::WINDOW)
        .title(title)
        .size(size)
        .center(true)
        .build(&mut window)?;
    Ok(window)
}

fn button(
    window: &nwg::Window,
    label: &str,
    position: (i32, i32),
) -> Result<nwg::Button, nwg::NwgError> {
    let mut button = nwg::Button::default();
    nwg::Button::builder()
        .parent(window)
        .position(position)
        .size((124, 30))
        .text(label)
        .build(&mut button)?;
    Ok(button)
}

pub fn show(owner: &nwg::Window, dialog: &nwg::Window) {
    owner.set_enabled(false);
    dialog.set_visible(true);
    dialog.set_focus();
}

pub fn hide(owner: &nwg::Window, dialog: &nwg::Window) {
    owner.set_enabled(true);
    dialog.set_visible(false);
    owner.set_focus();
}

pub struct SensorForm {
    pub window: nwg::Window,
    labels: Vec<nwg::Label>,
    fields: Vec<nwg::TextInput>,
    pub feedback: nwg::Label,
    pub apply: nwg::Button,
    pub close: nwg::Button,
    pub target: RefCell<Option<Target>>,
}

impl SensorForm {
    pub fn build(owner: &nwg::Window) -> Result<Self, nwg::NwgError> {
        let window = window(owner, "Sensor settings", (510, 365))?;
        let mut labels = Vec::new();
        let mut fields = Vec::new();
        for (index, title) in SENSOR_FIELDS.iter().enumerate() {
            let y = 12 + index as i32 * 34;
            let mut label = nwg::Label::default();
            nwg::Label::builder()
                .parent(&window)
                .position((14, y + 4))
                .size((165, 24))
                .text(title)
                .build(&mut label)?;
            labels.push(label);
            let mut input = nwg::TextInput::default();
            nwg::TextInput::builder()
                .parent(&window)
                .position((184, y))
                .size((308, 27))
                .build(&mut input)?;
            fields.push(input);
        }
        let mut feedback = nwg::Label::default();
        nwg::Label::builder()
            .parent(&window)
            .position((14, 252))
            .size((478, 57))
            .build(&mut feedback)?;
        Ok(Self {
            apply: button(&window, "Stage sensor", (236, 320))?,
            close: button(&window, "Cancel", (368, 320))?,
            window,
            labels,
            fields,
            feedback,
            target: RefCell::default(),
        })
    }

    pub fn open(&self, owner: &nwg::Window, target: Target, values: &SensorDraft) {
        let new_sensor = matches!(target, Target::Draft(_));
        self.window.set_text(if new_sensor {
            "Edit new S.Port sensor"
        } else {
            "Rename sensor"
        });
        for (index, field) in self.fields.iter().enumerate() {
            self.labels[index].set_visible(new_sensor || index == 0);
            field.set_visible(new_sensor || index == 0);
            field.set_text(&values.values[index]);
        }
        self.feedback.set_text(if new_sensor {
            "Use decimal for Module, Band, Routing and Sub-ID. The two IDs use hexadecimal. Incomplete entries can be staged as drafts."
        } else {
            "Use up to 15 UTF-8 bytes. For supported IDs, use the Telemetry IDs table."
        });
        self.target.replace(Some(target));
        show(owner, &self.window);
        self.fields[0].set_focus();
    }

    pub fn values(&self) -> SensorDraft {
        SensorDraft {
            values: std::array::from_fn(|index| self.fields[index].text()),
        }
    }
}

pub struct DeleteDialog {
    pub window: nwg::Window,
    pub cascade: nwg::CheckBox,
    pub apply: nwg::Button,
    pub close: nwg::Button,
    pub selection: RefCell<Vec<Target>>,
    preview: nwg::TextBox,
}

impl DeleteDialog {
    pub fn build(owner: &nwg::Window) -> Result<Self, nwg::NwgError> {
        let window = window(owner, "Delete telemetry sensors", (640, 435))?;
        let mut cascade = nwg::CheckBox::default();
        nwg::CheckBox::builder()
            .parent(&window)
            .text("Cascade delete")
            .position((14, 12))
            .size((220, 27))
            .build(&mut cascade)?;
        let mut preview = nwg::TextBox::default();
        nwg::TextBox::builder()
            .parent(&window)
            .position((14, 48))
            .size((608, 327))
            .flags(
                nwg::TextBoxFlags::VISIBLE
                    | nwg::TextBoxFlags::TAB_STOP
                    | nwg::TextBoxFlags::VSCROLL,
            )
            .readonly(true)
            .build(&mut preview)?;
        Ok(Self {
            apply: button(&window, "Stage deletion", (366, 389))?,
            close: button(&window, "Cancel", (498, 389))?,
            window,
            cascade,
            preview,
            selection: RefCell::default(),
        })
    }

    pub fn cascade_enabled(&self) -> bool {
        self.cascade.check_state() == nwg::CheckBoxState::Checked
    }

    pub fn preview(&self, preview: Result<DeletePreview, String>) {
        match preview {
            Ok(preview) => {
                let list = preview.removed.join("\r\n");
                let message = if preview.blockers.is_empty() {
                    format!(
                        "Will remove {} object(s)\r\n\r\n{list}",
                        preview.removed.len()
                    )
                } else {
                    format!("Resolve these references before staging\r\n\r\n{}\r\n\r\nSelected for removal\r\n\r\n{list}", preview.blockers.join("\r\n"))
                };
                self.preview.set_text(&message);
                self.apply
                    .set_enabled(preview.blockers.is_empty() && !preview.removed.is_empty());
            }
            Err(error) => {
                self.preview.set_text(&error);
                self.apply.set_enabled(false);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Outgoing,
    Incoming,
}

impl Direction {
    fn edges(self, graph: &Dependencies) -> &BTreeMap<String, BTreeSet<String>> {
        match self {
            Self::Outgoing => &graph.outgoing,
            Self::Incoming => &graph.incoming,
        }
    }
}

struct Branch {
    item: nwg::TreeItem,
    placeholder: nwg::TreeItem,
    object: String,
    ancestors: BTreeSet<String>,
    direction: Direction,
}

pub struct DependencyDialog {
    pub window: nwg::Window,
    pub tree: nwg::TreeView,
    pub close: nwg::Button,
    graph: RefCell<Dependencies>,
    branches: RefCell<Vec<Branch>>,
}

impl DependencyDialog {
    pub fn build(owner: &nwg::Window) -> Result<Self, nwg::NwgError> {
        let window = window(owner, "Dependencies", (650, 475))?;
        let mut tree = nwg::TreeView::default();
        nwg::TreeView::builder()
            .parent(&window)
            .position((14, 12))
            .size((618, 405))
            .build(&mut tree)?;
        Ok(Self {
            close: button(&window, "Close", (508, 429))?,
            window,
            tree,
            graph: RefCell::default(),
            branches: RefCell::default(),
        })
    }

    pub fn open(&self, owner: &nwg::Window, root: String, graph: Dependencies) {
        self.tree.clear();
        self.branches.borrow_mut().clear();
        let item = self
            .tree
            .insert_item(&graph.label(&root), None, nwg::TreeInsert::Root);
        for (title, direction) in [
            ("References", Direction::Outgoing),
            ("Referenced by", Direction::Incoming),
        ] {
            let group = self
                .tree
                .insert_item(title, Some(&item), nwg::TreeInsert::Last);
            self.add_children(
                &graph,
                &group,
                &root,
                &BTreeSet::from([root.clone()]),
                direction,
            );
            self.tree.set_expand_state(&group, nwg::ExpandState::Expand);
        }
        if !graph.notes.is_empty() {
            let notes =
                self.tree
                    .insert_item("Dependency coverage", Some(&item), nwg::TreeInsert::Last);
            for note in &graph.notes {
                self.tree
                    .insert_item(note, Some(&notes), nwg::TreeInsert::Last);
            }
        }
        self.graph.replace(graph);
        self.tree.set_expand_state(&item, nwg::ExpandState::Expand);
        self.tree.select_item(&item);
        show(owner, &self.window);
        self.tree.set_focus();
    }

    fn add_children(
        &self,
        graph: &Dependencies,
        parent: &nwg::TreeItem,
        root: &str,
        ancestors: &BTreeSet<String>,
        direction: Direction,
    ) {
        let edges = direction.edges(graph);
        let Some(children) = edges.get(root) else {
            self.tree
                .insert_item("None", Some(parent), nwg::TreeInsert::Last);
            return;
        };
        for object in children {
            let mut text = graph.label(object);
            if ancestors.contains(object) {
                text.push_str(" — cycle");
                self.tree
                    .insert_item(&text, Some(parent), nwg::TreeInsert::Last);
            } else {
                let item = self
                    .tree
                    .insert_item(&text, Some(parent), nwg::TreeInsert::Last);
                if edges.contains_key(object) {
                    let placeholder =
                        self.tree
                            .insert_item("…", Some(&item), nwg::TreeInsert::Last);
                    let mut ancestors = ancestors.clone();
                    ancestors.insert(object.clone());
                    self.branches.borrow_mut().push(Branch {
                        item,
                        placeholder,
                        object: object.clone(),
                        ancestors,
                        direction,
                    });
                }
            }
        }
    }

    pub fn expand(&self, item: &nwg::TreeItem) {
        let branch = {
            let mut branches = self.branches.borrow_mut();
            branches
                .iter()
                .position(|branch| &branch.item == item)
                .map(|index| branches.swap_remove(index))
        };
        if let Some(branch) = branch {
            self.add_children(
                &self.graph.borrow(),
                item,
                &branch.object,
                &branch.ancestors,
                branch.direction,
            );
            self.tree.remove_item(&branch.placeholder);
        }
    }
}
