use crate::point_editor::{encode_rows, PointEditor};
use crate::sensor_dialogs::{self, DeleteDialog, DependencyDialog, SensorForm};
use crate::session::{section_label, OpenModel, Row, Session, Target, View};
use native_windows_gui as nwg;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;

struct Controls {
    window: nwg::Window,
    open: nwg::Button,
    compare: nwg::Button,
    files: nwg::Label,
    tree: nwg::TreeView,
    title: nwg::Label,
    search_label: nwg::Label,
    search: nwg::TextInput,
    table: nwg::ListView,
    first_label: nwg::Label,
    first: nwg::TextInput,
    second_label: nwg::Label,
    second: nwg::TextInput,
    stage: nwg::Button,
    undo: nwg::Button,
    inspect: nwg::Button,
    sensor_add: nwg::Button,
    sensor_copy: nwg::Button,
    sensor_paste: nwg::Button,
    sensor_edit: nwg::Button,
    sensor_delete: nwg::Button,
    warnings: nwg::Button,
    preview: nwg::Button,
    export: nwg::Button,
    validate: nwg::Button,
    status: nwg::Label,
    notice: nwg::Notice,
}

fn button(parent: &nwg::Window, text: &str) -> Result<nwg::Button, nwg::NwgError> {
    let mut button = Default::default();
    nwg::Button::builder()
        .parent(parent)
        .text(text)
        .build(&mut button)?;
    Ok(button)
}
fn label(parent: &nwg::Window, text: &str) -> Result<nwg::Label, nwg::NwgError> {
    let mut label = Default::default();
    nwg::Label::builder()
        .parent(parent)
        .text(text)
        .build(&mut label)?;
    Ok(label)
}
fn input(parent: &nwg::Window) -> Result<nwg::TextInput, nwg::NwgError> {
    let mut input = Default::default();
    nwg::TextInput::builder().parent(parent).build(&mut input)?;
    Ok(input)
}

impl Controls {
    fn build() -> Result<Self, nwg::NwgError> {
        let mut window = nwg::Window::default();
        nwg::Window::builder()
            .flags(nwg::WindowFlags::MAIN_WINDOW)
            .title("ETHOS Model Editor")
            .size((1060, 660))
            .center(true)
            .build(&mut window)?;
        let mut tree = nwg::TreeView::default();
        nwg::TreeView::builder().parent(&window).build(&mut tree)?;
        let mut table = nwg::ListView::default();
        nwg::ListView::builder()
            .parent(&window)
            .list_style(nwg::ListViewStyle::Detailed)
            .flags(
                nwg::ListViewFlags::VISIBLE
                    | nwg::ListViewFlags::TAB_STOP
                    | nwg::ListViewFlags::ALWAYS_SHOW_SELECTION,
            )
            .ex_flags(nwg::ListViewExFlags::FULL_ROW_SELECT)
            .double_buffer(true)
            .build(&mut table)?;
        for (i, heading) in View::Settings(None).columns().iter().enumerate() {
            table.insert_column(nwg::InsertListViewColumn {
                index: Some(i as i32),
                text: Some((*heading).to_owned()),
                ..Default::default()
            });
        }
        let mut notice = Default::default();
        nwg::Notice::builder().parent(&window).build(&mut notice)?;
        Ok(Self {
            open: button(&window, "Open current…")?,
            compare: button(&window, "Compare backup…")?,
            files: label(&window, "")?,
            title: label(&window, "Model settings")?,
            search_label: label(&window, "Search")?,
            search: input(&window)?,
            first_label: label(&window, "Physical ID (hex)")?,
            first: input(&window)?,
            second_label: label(&window, "Application ID (hex)")?,
            second: input(&window)?,
            stage: button(&window, "Stage change")?,
            undo: button(&window, "Undo staged")?,
            inspect: button(&window, "Inspect bytes")?,
            sensor_add: button(&window, "Add sensor…")?,
            sensor_copy: button(&window, "Copy")?,
            sensor_paste: button(&window, "Paste")?,
            sensor_edit: button(&window, "Edit sensor…")?,
            sensor_delete: button(&window, "Delete selected…")?,
            warnings: button(&window, "Format notes…")?,
            preview: button(&window, "Preview edits")?,
            export: button(&window, "Save copy…")?,
            validate: button(&window, "Validate in ETHOS…")?,
            status: label(&window, "Open a current model to browse its settings.")?,
            window,
            tree,
            table,
            notice,
        })
    }
    fn layout(&self, view: &View) {
        macro_rules! place {
            ($widget:ident, $x:expr, $y:expr, $w:expr, $h:expr) => {{
                self.$widget.set_position($x, $y);
                self.$widget
                    .set_size(($w).max(1) as u32, ($h).max(1) as u32);
            }};
        }
        let (w, h) = self.window.size();
        let (w, h) = (w as i32, h as i32);
        let nav = if w >= 970 { 208 } else { 165 };
        let x = nav + 26;
        let area = w - x - 14;
        place!(open, 12, 12, 122_i32, 32_i32);
        place!(compare, 142, 12, 152_i32, 32_i32);
        place!(preview, w - 433, 12, 120_i32, 32_i32);
        place!(export, w - 305, 12, 105_i32, 32_i32);
        place!(validate, w - 192, 12, 179_i32, 32_i32);
        place!(files, 14, 53, w - 28, 21_i32);
        place!(tree, 12, 86, nav, h - 142);
        place!(title, x, 86, area - 260, 24_i32);
        place!(search_label, w - 262, 88, 54_i32, 23_i32);
        place!(search, w - 205, 82, 192_i32, 28_i32);
        place!(table, x, 118, area, h - 258);
        place!(first_label, x, h - 129, 138_i32, 19_i32);
        place!(second_label, x + 150, h - 129, 150_i32, 19_i32);
        place!(
            first,
            x,
            h - 107,
            if self.first_label.text() == "Proposed value" {
                288_i32
            } else {
                140_i32
            },
            27_i32
        );
        place!(second, x + 150, h - 107, 140_i32, 27_i32);
        place!(stage, x + 300, h - 107, 115_i32, 28_i32);
        place!(undo, x + 424, h - 107, 110_i32, 28_i32);
        place!(inspect, x, h - 69, 125_i32, 29_i32);
        place!(warnings, x + 136, h - 69, 140_i32, 29_i32);
        place!(sensor_add, x, h - 69, 100_i32, 29_i32);
        place!(sensor_copy, x + 106, h - 69, 60_i32, 29_i32);
        place!(sensor_paste, x + 172, h - 69, 60_i32, 29_i32);
        place!(sensor_edit, x + 238, h - 69, 100_i32, 29_i32);
        place!(sensor_delete, x + 344, h - 69, 124_i32, 29_i32);
        if *view == View::Telemetry {
            place!(warnings, x + 474, h - 69, 105_i32, 29_i32);
        }
        place!(status, 14, h - 32, w - 28, 27_i32);
        self.widths(view);
    }
    fn widths(&self, view: &View) {
        let w = self.table.size().0 as i32 - 18;
        let fractions = match view {
            View::Bytes(_) => [0.17, 0.57, 0.26, 0.0],
            View::Settings(_) => [0.41, 0.29, 0.16, 0.14],
            View::Telemetry => [0.32, 0.16, 0.19, 0.33],
            _ => [0.40, 0.23, 0.23, 0.14],
        };
        for (i, frac) in fractions.iter().enumerate() {
            self.table.set_column_width(i, (w as f64 * frac) as isize);
        }
    }
}

pub struct App {
    c: Controls,
    points: PointEditor,
    point_path: RefCell<Option<String>>,
    sensor_form: SensorForm,
    delete_dialog: DeleteDialog,
    dependency_dialog: DependencyDialog,
    dependencies_item: nwg::MenuItem,
    object_menu: nwg::Menu,
    menu_target: RefCell<Option<Target>>,
    dialog_handlers: RefCell<Vec<nwg::EventHandler>>,
    point_handler: RefCell<Option<nwg::EventHandler>>,
    session: RefCell<Session>,
    view: RefCell<View>,
    nav_views: RefCell<Vec<(nwg::TreeItem, View)>>,
    rows: RefCell<Vec<Row>>,
    busy: Cell<bool>,
    validation: RefCell<Option<mpsc::Receiver<Result<String, String>>>>,
    handler: RefCell<Option<nwg::EventHandler>>,
}

impl Drop for App {
    fn drop(&mut self) {
        for handler in self.dialog_handlers.get_mut().drain(..) {
            nwg::unbind_event_handler(&handler);
        }
        if let Some(handler) = self.point_handler.get_mut().take() {
            nwg::unbind_event_handler(&handler);
        }
        if let Some(handler) = self.handler.get_mut().take() {
            nwg::unbind_event_handler(&handler);
        }
    }
}

impl App {
    fn build() -> Result<Rc<Self>, nwg::NwgError> {
        let c = Controls::build()?;
        let points = PointEditor::build(&c.window)?;
        let mut object_menu = nwg::Menu::default();
        nwg::Menu::builder()
            .parent(&c.window)
            .popup(true)
            .build(&mut object_menu)?;
        let mut dependencies_item = nwg::MenuItem::default();
        nwg::MenuItem::builder()
            .parent(&object_menu)
            .text("&Dependencies")
            .build(&mut dependencies_item)?;
        let app = Rc::new(Self {
            sensor_form: SensorForm::build(&c.window)?,
            delete_dialog: DeleteDialog::build(&c.window)?,
            dependency_dialog: DependencyDialog::build(&c.window)?,
            dependencies_item,
            object_menu,
            menu_target: Default::default(),
            dialog_handlers: Default::default(),
            c,
            points,
            point_path: Default::default(),
            point_handler: Default::default(),
            session: Default::default(),
            view: RefCell::new(View::Settings(None)),
            rows: Default::default(),
            nav_views: Default::default(),
            busy: Cell::new(false),
            validation: Default::default(),
            handler: Default::default(),
        });
        let weak = Rc::downgrade(&app);
        let handler =
            nwg::full_bind_event_handler(&app.c.window.handle, move |event, data, source| {
                if let Some(app) = weak.upgrade() {
                    app.event(event, data, source);
                }
            });
        app.handler.replace(Some(handler));
        let weak = Rc::downgrade(&app);
        let handler =
            nwg::full_bind_event_handler(&app.points.window.handle, move |event, data, source| {
                if let Some(app) = weak.upgrade() {
                    app.point_event(event, data, source);
                }
            });
        app.point_handler.replace(Some(handler));
        for dialog in [
            &app.sensor_form.window,
            &app.delete_dialog.window,
            &app.dependency_dialog.window,
        ] {
            let weak = Rc::downgrade(&app);
            let handler =
                nwg::full_bind_event_handler(&dialog.handle, move |event, data, source| {
                    if let Some(app) = weak.upgrade() {
                        app.sensor_dialog_event(event, data, source);
                    }
                });
            app.dialog_handlers.borrow_mut().push(handler);
        }
        app.c.layout(&app.view.borrow());
        app.navigation(View::Settings(None));
        Ok(app)
    }

    fn point_event(&self, event: nwg::Event, data: nwg::EventData, source: nwg::ControlHandle) {
        let d = &self.points;
        let close = match event {
            nwg::Event::OnWindowClose if source == d.window.handle => {
                if let nwg::EventData::OnWindowClose(action) = data {
                    action.close(false);
                }
                true
            }
            nwg::Event::OnButtonClick if source == d.cancel.handle => true,
            nwg::Event::OnButtonClick if source == d.stage.handle => {
                let Some(path) = self.point_path.borrow().clone() else {
                    return;
                };
                let result = encode_rows(&d.input.text())
                    .and_then(|v| self.session.borrow_mut().stage_points(&path, v));
                if let Err(error) = result {
                    d.feedback.set_text(&error);
                    return;
                }
                self.refresh_staging();
                true
            }
            _ => false,
        };
        if close {
            self.point_path.take();
            d.hide(&self.c.window);
        }
    }

    fn event(&self, evt: nwg::Event, data: nwg::EventData, source: nwg::ControlHandle) {
        use nwg::Event as E;
        let c = &self.c;
        if self.busy.get() {
            return;
        }
        match evt {
            E::OnWindowClose if source == c.window.handle => {
                if self.confirm_discard() {
                    nwg::stop_thread_dispatch();
                } else if let nwg::EventData::OnWindowClose(close) = data {
                    close.close(false);
                }
            }
            E::OnMinMaxInfo if source == c.window.handle => {
                data.on_min_max().set_min_size(790, 490)
            }
            E::OnResize if source == c.window.handle => c.layout(&self.view.borrow()),
            E::OnTreeItemSelectionChanged if source == c.tree.handle => {
                let (_, item) = data.on_tree_item_selection_changed();
                let view = self
                    .nav_views
                    .borrow()
                    .iter()
                    .find(|(node, _)| node == item)
                    .map(|(_, v)| v.clone());
                if let Some(view) = view {
                    self.set_view(view);
                }
            }
            E::OnTextInput if source == c.search.handle => self.render(),
            E::OnListViewItemChanged if source == c.table.handle => self.edit_controls(),
            E::OnListViewDoubleClick if source == c.table.handle => {
                if let Some(target @ Target::Draft(_)) = self.target() {
                    self.open_sensor_form(target);
                } else {
                    self.inspect_selected();
                }
            }
            E::OnListViewRightClick if source == c.table.handle => {
                let (index, _) = data.on_list_view_item_index();
                let target = self
                    .rows
                    .borrow()
                    .get(index)
                    .and_then(|row| row.target.clone());
                if target.is_some() {
                    self.menu_target.replace(target);
                    let (x, y) = nwg::GlobalCursor::position();
                    self.object_menu.popup(x, y);
                }
            }
            E::OnMenuItemSelected if source == self.dependencies_item.handle => {
                self.show_dependencies()
            }
            E::OnButtonClick if source == c.sensor_add.handle => {
                let id = self.session.borrow_mut().add_draft();
                self.refresh_staging();
                self.open_sensor_form(Target::Draft(id));
            }
            E::OnButtonClick if source == c.sensor_edit.handle => {
                if let Some(target) = self.target() {
                    self.open_sensor_form(target);
                }
            }
            E::OnButtonClick if source == c.sensor_copy.handle => self.copy_sensor(),
            E::OnButtonClick if source == c.sensor_paste.handle => self.paste_sensor(),
            E::OnButtonClick if source == c.sensor_delete.handle => self.preview_deletion(),
            E::OnNotice if source == c.notice.handle => self.validation_done(),
            E::OnButtonClick if source == c.open.handle => self.pick_open(false),
            E::OnButtonClick if source == c.compare.handle => self.pick_open(true),
            E::OnButtonClick if source == c.stage.handle => self.stage_selected(),
            E::OnButtonClick if source == c.undo.handle => self.undo_selected(),
            E::OnButtonClick if source == c.inspect.handle => self.inspect_selected(),
            E::OnButtonClick if source == c.warnings.handle => {
                nwg::modal_info_message(
                    &c.window,
                    "Format notes",
                    &self.session.borrow().warnings().join("\n\n"),
                );
            }
            E::OnButtonClick if source == c.preview.handle => self.navigation(View::Staged),
            E::OnButtonClick if source == c.export.handle => self.pick_export(),
            E::OnButtonClick if source == c.validate.handle => self.validate_export(),
            _ => {}
        }
    }

    fn selected_sensors(&self) -> Vec<Target> {
        let rows = self.rows.borrow();
        self.c
            .table
            .selected_items()
            .into_iter()
            .filter_map(|index| rows.get(index)?.target.clone())
            .filter(|target| matches!(target, Target::Sensor(_) | Target::Draft(_)))
            .collect()
    }

    fn show_dependencies(&self) {
        let Some(target) = self.menu_target.take() else {
            return;
        };
        let result = self.session.borrow().dependencies(&target);
        match result {
            Ok((root, graph)) => self.dependency_dialog.open(&self.c.window, root, graph),
            Err(error) => self.error(&error),
        }
    }

    fn copy_sensor(&self) {
        let Some(Target::Sensor(slot)) = self.target() else {
            return;
        };
        let result = self.session.borrow_mut().copy_sensor(slot);
        match result {
            Ok(()) => {
                self.c.sensor_paste.set_enabled(true);
                self.c
                    .status
                    .set_text("Sensor copied. Choose Paste to create a separate entry.");
            }
            Err(error) => self.error(&error),
        }
    }

    fn paste_sensor(&self) {
        let result = self.session.borrow_mut().paste_sensor();
        match result {
            Ok(()) => {
                self.refresh_staging();
                self.c.status.set_text("Sensor pasted. Change its S.Port key before saving if it matches another entry.");
            }
            Err(error) => self.error(&error),
        }
    }

    fn preview_deletion(&self) {
        let selection = self.selected_sensors();
        if !selection.is_empty() {
            let dialog = &self.delete_dialog;
            dialog.selection.replace(selection);
            dialog
                .cascade
                .set_check_state(nwg::CheckBoxState::Unchecked);
            self.refresh_delete_preview();
            sensor_dialogs::show(&self.c.window, &dialog.window);
        }
    }

    fn open_sensor_form(&self, target: Target) {
        let s = self.session.borrow();
        let values = match target {
            Target::Draft(id) => s.draft(id).cloned().unwrap_or_default(),
            Target::Sensor(slot) => {
                let mut values = ethos_model::SensorDraft::default();
                values.values[0] = s
                    .field_value(&format!("telemetry.sensors[{slot}].name"))
                    .unwrap_or_default();
                values
            }
            Target::Field(_) => return,
        };
        self.sensor_form.open(&self.c.window, target, &values);
    }

    fn refresh_delete_preview(&self) {
        let dialog = &self.delete_dialog;
        dialog.preview(
            self.session
                .borrow()
                .deletion_preview(&dialog.selection.borrow(), dialog.cascade_enabled()),
        );
    }

    fn sensor_dialog_event(
        &self,
        event: nwg::Event,
        data: nwg::EventData,
        source: nwg::ControlHandle,
    ) {
        use nwg::Event as E;
        let form = &self.sensor_form;
        let deletion = &self.delete_dialog;
        let deps = &self.dependency_dialog;
        let close = match event {
            E::OnWindowClose => {
                if let nwg::EventData::OnWindowClose(action) = data {
                    action.close(false);
                }
                if source == form.window.handle {
                    &form.window
                } else if source == deletion.window.handle {
                    &deletion.window
                } else if source == deps.window.handle {
                    &deps.window
                } else {
                    return;
                }
            }
            E::OnButtonClick if source == form.close.handle => &form.window,
            E::OnButtonClick if source == deletion.close.handle => &deletion.window,
            E::OnButtonClick if source == deps.close.handle => &deps.window,
            E::OnButtonClick if source == form.apply.handle => {
                let result = match form.target.borrow().as_ref() {
                    Some(Target::Draft(id)) => {
                        self.session.borrow_mut().update_draft(*id, form.values())
                    }
                    Some(Target::Sensor(slot)) => self.session.borrow_mut().stage_field(
                        &format!("telemetry.sensors[{slot}].name"),
                        &form.values().values[0],
                    ),
                    _ => return,
                };
                if let Err(error) = result {
                    form.feedback.set_text(&error);
                    return;
                }
                self.refresh_staging();
                &form.window
            }
            E::OnButtonClick if source == deletion.cascade.handle => {
                self.refresh_delete_preview();
                return;
            }
            E::OnButtonClick if source == deletion.apply.handle => {
                let result = self
                    .session
                    .borrow_mut()
                    .delete_sensors(&deletion.selection.borrow(), deletion.cascade_enabled());
                if let Err(error) = result {
                    deletion.preview(Err(error));
                    return;
                }
                self.refresh_staging();
                &deletion.window
            }
            E::OnTreeItemExpanded if source == deps.tree.handle => {
                let (item, action) = data.on_tree_item_update();
                if matches!(
                    action,
                    nwg::TreeItemAction::Expand(nwg::ExpandState::Expand)
                ) {
                    deps.expand(item);
                }
                return;
            }
            _ => return,
        };
        sensor_dialogs::hide(&self.c.window, close);
    }

    fn navigation(&self, wanted: View) {
        let s = self.session.borrow();
        let mut rows = Vec::new();
        self.busy.set(true);
        self.c.tree.clear();
        for is_diff in [false, true] {
            if is_diff && s.backup.is_none() {
                continue;
            }
            let group = if is_diff {
                View::Differences(None)
            } else {
                View::Settings(None)
            };
            let root = self
                .c
                .tree
                .insert_item(group.title(), None, nwg::TreeInsert::Last);
            for section in s.sections(is_diff) {
                let child = self.c.tree.insert_item(
                    section_label(&section),
                    Some(&root),
                    nwg::TreeInsert::Last,
                );
                rows.push((
                    child,
                    if is_diff {
                        View::Differences(Some(section))
                    } else {
                        View::Settings(Some(section))
                    },
                ));
            }
            self.c
                .tree
                .set_expand_state(&root, nwg::ExpandState::Expand);
            rows.push((root, group));
        }
        for v in [View::Telemetry, View::Staged] {
            let name = if v == View::Staged && s.has_edits() {
                format!("Staged changes ({})", s.staged_count())
            } else {
                v.title().into()
            };
            rows.push((
                self.c.tree.insert_item(&name, None, nwg::TreeInsert::Last),
                v,
            ));
        }
        if let View::Bytes(path) = &wanted {
            let name = format!("Bytes / {path}");
            rows.push((
                self.c.tree.insert_item(&name, None, nwg::TreeInsert::Last),
                wanted.clone(),
            ));
        }
        if let Some((node, _)) = rows.iter().find(|(_, v)| *v == wanted) {
            self.c.tree.select_item(node);
        }
        self.nav_views.replace(rows);
        drop(s);
        self.busy.set(false);
        self.set_view(wanted);
    }
    fn set_view(&self, view: View) {
        self.busy.set(true);
        self.view.replace(view);
        self.c.search.set_text("");
        self.busy.set(false);
        self.render();
    }
    fn render(&self) {
        self.busy.set(true);
        let view = self.view.borrow();
        let s = self.session.borrow();
        let c = &self.c;
        let rows = s.rows(&view, &c.search.text());
        c.title.set_text(view.title());
        c.table.set_redraw(false);
        c.table.clear();
        for (i, name) in view.columns().iter().enumerate() {
            c.table.update_column(
                i,
                nwg::InsertListViewColumn {
                    text: Some((*name).into()),
                    ..Default::default()
                },
            );
        }
        for row in &rows {
            c.table.insert_items_row(None, &row.cells);
        }
        self.rows.replace(rows);
        c.widths(&view);
        c.table.set_redraw(true);
        c.table.invalidate();
        let name = |opened: &OpenModel| {
            let path = &opened.path;
            let filename = path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy();
            match opened.model.field("model.type") {
                Some(category) => format!("{filename} ({})", category.value),
                None => filename.into_owned(),
            }
        };
        let current = s.current.as_ref().map(name);
        let backup = s.backup.as_ref().map(name);
        let files = match (current, backup) {
            (Some(current), Some(backup)) => format!("Current   {current}       Backup   {backup}"),
            (Some(current), None) => format!("Current   {current}"),
            _ => "".into(),
        };
        c.files.set_text(&files);
        c.compare.set_enabled(s.current.is_some());
        let telemetry = *view == View::Telemetry;
        for button in [
            &c.sensor_add,
            &c.sensor_copy,
            &c.sensor_paste,
            &c.sensor_edit,
            &c.sensor_delete,
        ] {
            button.set_visible(telemetry);
        }
        c.inspect.set_visible(!telemetry);
        c.sensor_add.set_enabled(s.current.is_some());
        c.sensor_paste.set_enabled(s.can_paste());
        c.layout(&view);
        c.preview.set_enabled(s.has_edits());
        let warning_count = s.warnings().len();
        c.warnings.set_visible(warning_count > 0);
        c.warnings
            .set_text(&format!("Format notes ({warning_count})…"));
        let staged = *view == View::Staged && s.has_edits();
        if staged {
            match s.preview() {
                Ok(changes) => {
                    c.status.set_text(&format!(
                        "{} verified change(s). Choose Save copy to export.",
                        changes.len()
                    ));
                }
                Err(error) => c.status.set_text(&error.replace('\n', " • ")),
            }
        }
        c.export.set_enabled(staged);
        c.validate
            .set_enabled(s.last_export.is_some() && self.validation.borrow().is_none());
        if !staged {
            let n = self.rows.borrow().len();
            let status = if s.current.is_none() {
                "Open a current ETHOS .bin file to browse its settings.".into()
            } else if *view == View::Staged {
                "Select a sensor or editable setting, then choose Stage change.".into()
            } else if n == 0 && matches!(*view, View::Differences(_)) && c.search.text().is_empty()
            {
                "No differences in this section.".into()
            } else {
                let note = if warning_count > 0 {
                    "     •     See format notes for decoding limits"
                } else {
                    ""
                };
                format!("{n} rows     •     {} staged{note}", s.staged_count())
            };
            c.status.set_text(&status);
        }
        drop(s);
        drop(view);
        self.busy.set(false);
        self.edit_controls();
    }
    fn target(&self) -> Option<Target> {
        let selected = self.c.table.selected_items();
        let [index] = selected.as_slice() else {
            return None;
        };
        let index = *index;
        self.rows.borrow().get(index)?.target.clone()
    }
    fn edit_controls(&self) {
        let c = &self.c;
        let s = self.session.borrow();
        let target = self.target();
        if *self.view.borrow() == View::Telemetry {
            c.sensor_copy
                .set_enabled(matches!(target, Some(Target::Sensor(_))));
            c.sensor_edit.set_enabled(match &target {
                Some(Target::Draft(_)) => true,
                Some(Target::Sensor(slot)) => s
                    .field(&format!("telemetry.sensors[{slot}].name"))
                    .is_some_and(|field| field.editable),
                _ => false,
            });
            c.sensor_delete
                .set_enabled(!self.selected_sensors().is_empty());
        }
        let is_draft = matches!(target, Some(Target::Draft(_)));
        let is_field = matches!(target, Some(Target::Field(_)));
        let point_field = match &target {
            Some(Target::Field(path)) => s.point_field(path),
            _ => None,
        };
        c.stage.set_text(if is_draft {
            "Edit sensor…"
        } else if point_field.is_some() {
            "Edit points…"
        } else {
            "Stage change"
        });
        c.first_label.set_text(if point_field.is_some() {
            "Curve points"
        } else if is_field {
            "Proposed value"
        } else {
            "Physical ID (hex)"
        });
        c.second_label.set_visible(!is_field);
        c.second.set_visible(!is_field);
        let (first_editable, second_editable) = match &target {
            Some(Target::Sensor(slot)) => {
                if let Some((physical, app)) = s.ids(*slot) {
                    c.first.set_text(&format!("{physical:02X}"));
                    c.second.set_text(&format!("{app:04X}"));
                    let permissions = s.id_permissions(*slot);
                    let help = match permissions {
                        (true, false) => concat!("Physical ID is editable. ",
                            "No replacement Application ID has been verified for this sensor family."),
                        (false, true) => "Application ID is editable. Physical ID is read only.",
                        (true, true) => "Edit the sensor IDs in hexadecimal, then choose Stage change.",
                        (false, false) => "Both sensor IDs are read only.",
                    };
                    c.status.set_text(help);
                    permissions
                } else {
                    (false, false)
                }
            }
            Some(Target::Field(_)) if point_field.is_some() => {
                c.first.set_text("Ordered point list");
                c.status
                    .set_text("Choose Edit points to change coordinates or the point count.");
                (false, false)
            }
            Some(Target::Field(path)) => {
                c.first.set_text(&s.field_value(path).unwrap_or_default());
                if let Some(field) = s.field(path) {
                    c.status.set_text(&Session::field_help(field));
                    (field.editable, false)
                } else {
                    (false, false)
                }
            }
            Some(Target::Draft(_)) | None => {
                c.first.set_text("");
                c.second.set_text("");
                (false, false)
            }
        };
        c.first.set_enabled(first_editable);
        c.second.set_enabled(second_editable);
        c.stage
            .set_enabled(first_editable || second_editable || point_field.is_some() || is_draft);
        c.inspect.set_enabled(is_field);
        c.undo
            .set_enabled(target.as_ref().is_some_and(|t| s.has_edit(t)));
        c.first.set_size(if is_field { 288 } else { 140 }, 27);
    }
    fn confirm_discard(&self) -> bool {
        !self.session.borrow().unsaved
            || nwg::modal_message(
                &self.c.window,
                &nwg::MessageParams {
                    title: "Discard staged changes?",
                    content: "The current model has staged edits. Discard them and continue?",
                    buttons: nwg::MessageButtons::YesNo,
                    icons: nwg::MessageIcons::Warning,
                },
            ) == nwg::MessageChoice::Yes
    }
    fn choose(
        &self,
        title: &str,
        action: nwg::FileDialogAction,
    ) -> Result<Option<PathBuf>, String> {
        let mut dlg = nwg::FileDialog::default();
        let files = !matches!(action, nwg::FileDialogAction::OpenDirectory);
        let mut builder = nwg::FileDialog::builder().title(title).action(action);
        if files {
            builder = builder.filters("ETHOS model (*.bin)|All files (*.*)");
        }
        builder.build(&mut dlg).map_err(|e| e.to_string())?;
        if !dlg.run(Some(&self.c.window)) {
            return Ok(None);
        }
        dlg.get_selected_item()
            .map(|p| Some(PathBuf::from(p)))
            .map_err(|e| e.to_string())
    }
    fn error(&self, msg: &str) {
        self.c.status.set_text(msg);
        nwg::modal_error_message(&self.c.window, "ETHOS Model Editor", msg);
    }
    fn open_paths(&self, current: &Path, backup: Option<&Path>) -> Result<(), String> {
        self.session.borrow_mut().open(current, backup)?;
        self.navigation(if backup.is_some() {
            View::Differences(None)
        } else {
            View::Settings(None)
        });
        Ok(())
    }
    fn pick_open(&self, backup: bool) {
        let result = (|| {
            let title = if backup {
                "Choose the earlier backup to compare"
            } else {
                "Choose the current model"
            };
            let Some(path) = self.choose(title, nwg::FileDialogAction::Open)? else {
                return Ok(());
            };
            if backup {
                self.session.borrow_mut().compare_backup(&path)?;
                self.navigation(View::Differences(None));
            } else if self.confirm_discard() {
                self.open_paths(&path, None)?;
            }
            Ok::<(), String>(())
        })();
        if let Err(e) = result {
            self.error(&e);
        }
    }
    fn stage_selected(&self) {
        if let Some(Target::Field(path)) = self.target() {
            let s = self.session.borrow();
            if let Some(field) = s.point_field(&path) {
                if let Err(error) = self.points.show(
                    &self.c.window,
                    &s.field_value(&field.path).unwrap_or_default(),
                ) {
                    self.error(&error);
                } else {
                    self.point_path.replace(Some(field.path.clone()));
                }
                return;
            }
        }
        let result = match self.target() {
            Some(target @ Target::Draft(_)) => {
                self.open_sensor_form(target);
                return;
            }
            Some(Target::Sensor(slot)) => {
                self.session
                    .borrow_mut()
                    .stage(slot, &self.c.first.text(), &self.c.second.text())
            }
            Some(Target::Field(path)) => self
                .session
                .borrow_mut()
                .stage_field(&path, &self.c.first.text()),
            None => return,
        };
        if let Err(e) = result {
            self.error(&e);
            return;
        }
        self.refresh_staging();
    }
    fn refresh_staging(&self) {
        let view = self.view.borrow().clone();
        let query = self.c.search.text();
        self.navigation(view);
        self.c.search.set_text(&query);
    }
    fn undo_selected(&self) {
        if let Some(t) = self.target() {
            self.session.borrow_mut().remove(&t);
            let view = self.view.borrow().clone();
            self.navigation(view);
        }
    }
    fn inspect_selected(&self) {
        if let Some(Target::Field(path)) = self.target() {
            self.navigation(View::Bytes(path));
        }
    }
    fn save_path(&self, path: &Path) -> Result<(), String> {
        self.session.borrow_mut().save(path)?;
        self.render();
        self.c
            .status
            .set_text(&format!("Verified copy saved to {}", path.display()));
        Ok(())
    }
    fn pick_export(&self) {
        let result = (|| {
            self.session.borrow().preview()?;
            let Some(mut path) = self.choose(
                "Save repaired model as a new file",
                nwg::FileDialogAction::Save,
            )?
            else {
                return Ok(());
            };
            if path.extension().is_none() {
                path.set_extension("bin");
            }
            self.save_path(&path)
        })();
        if let Err(e) = result {
            self.error(&e);
        }
    }
    fn validate_export(&self) {
        let Some(path) = self.session.borrow().last_export.clone() else {
            return;
        };
        let runtime = match self.choose(
            "Choose the extracted X20 Pro AW FCC simulator",
            nwg::FileDialogAction::OpenDirectory,
        ) {
            Ok(Some(runtime)) => runtime,
            Ok(None) => return,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let (tx, rx) = mpsc::channel();
        self.validation.replace(Some(rx));
        self.c.validate.set_enabled(false);
        self.c.status.set_text(&format!(
            "Checking {} in the isolated simulator…",
            path.display()
        ));
        let notice = self.c.notice.sender();
        std::thread::spawn(move || {
            let result = ethos_model_validation::validate_in_ethos(
                &path,
                &runtime,
                &ethos_model_validation::default_adapter(),
            )
            .map(|report| format!("{}\n\n{report}", path.display()))
            .map_err(|e| e.to_string());
            if tx.send(result).is_ok() {
                notice.notice();
            }
        });
    }
    fn validation_done(&self) {
        let Some(rx) = self.validation.take() else {
            return;
        };
        self.c
            .validate
            .set_enabled(self.session.borrow().last_export.is_some());
        match rx.try_recv() {
            Ok(Ok(msg)) => {
                self.c.status.set_text("Simulator validation finished.");
                nwg::modal_info_message(&self.c.window, "ETHOS validation", &msg);
            }
            Ok(Err(e)) => self.error(&e),
            Err(e) => self.error(&e.to_string()),
        }
    }
}

pub fn run(paths: Vec<PathBuf>) -> Result<(), String> {
    nwg::init().map_err(|e| e.to_string())?;
    nwg::Font::set_global_family("Segoe UI").map_err(|e| e.to_string())?;
    let app = App::build().map_err(|e| e.to_string())?;
    app.c.window.set_visible(true);
    let result = match paths.as_slice() {
        [] => Ok(()),
        [current] => app.open_paths(current, None),
        [backup, current] => app.open_paths(current, Some(backup)),
        _ => Err("Open one current file or an earlier backup followed by the current file.".into()),
    };
    if let Err(e) = result {
        app.error(&e);
    }
    nwg::dispatch_thread_events();
    Ok(())
}
