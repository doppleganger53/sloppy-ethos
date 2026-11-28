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

fn children<'a>(
    edges: &'a BTreeMap<String, BTreeSet<String>>,
    object: &str,
    ancestors: &'a BTreeSet<String>,
) -> impl Iterator<Item = &'a String> {
    edges
        .get(object)
        .into_iter()
        .flatten()
        .filter(|child| !ancestors.contains(*child))
}

struct Branch {
    item: nwg::TreeItem,
    placeholder: nwg::TreeItem,
    object: String,
    ancestors: BTreeSet<String>,
}

pub struct DependencyDialog {
    pub window: nwg::Window,
    pub tree: nwg::TreeView,
    pub references: nwg::RadioButton,
    pub referenced_by: nwg::RadioButton,
    pub coverage: nwg::Button,
    pub close: nwg::Button,
    root: RefCell<String>,
    graph: RefCell<Dependencies>,
    branches: RefCell<Vec<Branch>>,
}

impl DependencyDialog {
    pub fn build(owner: &nwg::Window) -> Result<Self, nwg::NwgError> {
        let window = window(owner, "Dependencies", (650, 475))?;
        let mut references = nwg::RadioButton::default();
        let mut referenced_by = nwg::RadioButton::default();
        nwg::RadioButton::builder()
            .parent(&window)
            .text("References")
            .position((14, 12))
            .size((115, 26))
            .flags(
                nwg::RadioButtonFlags::GROUP
                    | nwg::RadioButtonFlags::TAB_STOP
                    | nwg::RadioButtonFlags::VISIBLE,
            )
            .build(&mut references)?;
        nwg::RadioButton::builder()
            .parent(&window)
            .text("Referenced by")
            .position((141, 12))
            .size((145, 26))
            .build(&mut referenced_by)?;
        let mut tree = nwg::TreeView::default();
        nwg::TreeView::builder()
            .parent(&window)
            .position((14, 45))
            .size((618, 372))
            .build(&mut tree)?;
        Ok(Self {
            coverage: button(&window, "Coverage…", (14, 429))?,
            close: button(&window, "Close", (508, 429))?,
            window,
            tree,
            references,
            referenced_by,
            root: RefCell::default(),
            graph: RefCell::default(),
            branches: RefCell::default(),
        })
    }

    pub fn open(&self, owner: &nwg::Window, root: String, graph: Dependencies) {
        self.coverage.set_visible(!graph.notes.is_empty());
        self.root.replace(root);
        self.graph.replace(graph);
        self.references
            .set_check_state(nwg::RadioButtonState::Checked);
        self.referenced_by
            .set_check_state(nwg::RadioButtonState::Unchecked);
        self.refresh();
        show(owner, &self.window);
        self.tree.set_focus();
    }

    pub fn show_coverage(&self) {
        nwg::modal_info_message(
            &self.window,
            "Dependency coverage",
            &self.graph.borrow().notes.join("\n"),
        );
    }

    fn direction(&self) -> Direction {
        if self.referenced_by.check_state() == nwg::RadioButtonState::Checked {
            Direction::Incoming
        } else {
            Direction::Outgoing
        }
    }

    pub fn refresh(&self) {
        self.tree.clear();
        self.branches.borrow_mut().clear();
        let graph = self.graph.borrow();
        let root = self.root.borrow();
        let item = self
            .tree
            .insert_item(&graph.label(&root), None, nwg::TreeInsert::Root);
        self.add_children(&graph, &item, &root, &BTreeSet::from([root.clone()]));
        self.tree.set_expand_state(&item, nwg::ExpandState::Expand);
        self.tree.select_item(&item);
    }

    fn add_children(
        &self,
        graph: &Dependencies,
        parent: &nwg::TreeItem,
        root: &str,
        ancestors: &BTreeSet<String>,
    ) {
        let edges = self.direction().edges(graph);
        for object in children(edges, root, ancestors) {
            let item =
                self.tree
                    .insert_item(&graph.label(object), Some(parent), nwg::TreeInsert::Last);
            let mut ancestors = ancestors.clone();
            ancestors.insert(object.clone());
            if children(edges, object, &ancestors).next().is_some() {
                let placeholder = self
                    .tree
                    .insert_item("…", Some(&item), nwg::TreeInsert::Last);
                self.branches.borrow_mut().push(Branch {
                    item,
                    placeholder,
                    object: object.clone(),
                    ancestors,
                });
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
            );
            self.tree.remove_item(&branch.placeholder);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_branch_omits_its_ancestors_and_keeps_other_leaves() {
        let edges = BTreeMap::from([(
            "second".to_owned(),
            BTreeSet::from(["root".to_owned(), "second".to_owned(), "third".to_owned()]),
        )]);
        let ancestors = BTreeSet::from(["root".to_owned(), "second".to_owned()]);
        assert_eq!(
            children(&edges, "second", &ancestors).collect::<Vec<_>>(),
            ["third"]
        );
    }
}
