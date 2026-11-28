use native_windows_gui as nwg;
use serde_json::{json, Number, Value};

pub struct PointEditor {
    pub window: nwg::Window,
    _instructions: nwg::Label,
    pub input: nwg::TextBox,
    pub feedback: nwg::Label,
    pub stage: nwg::Button,
    pub cancel: nwg::Button,
}

impl PointEditor {
    pub fn build(owner: &nwg::Window) -> Result<Self, nwg::NwgError> {
        let mut window = nwg::Window::default();
        nwg::Window::builder()
            .parent(Some(owner))
            .flags(nwg::WindowFlags::WINDOW)
            .title("Curve points")
            .size((550, 495))
            .center(true)
            .build(&mut window)?;
        let mut instructions = nwg::Label::default();
        let (width, height) = window.size();
        let (width, height) = (width as i32, height as i32);
        nwg::Label::builder().parent(&window).position((14,12)).size((width-28,64))
            .text("Enter one point per line. Use X, Y percentages for a custom grid or one Y for an evenly spaced grid.")
            .build(&mut instructions)?;
        let mut input = nwg::TextBox::default();
        nwg::TextBox::builder()
            .parent(&window)
            .position((14, 82))
            .size((width - 28, height - 211))
            .flags(
                nwg::TextBoxFlags::VISIBLE
                    | nwg::TextBoxFlags::TAB_STOP
                    | nwg::TextBoxFlags::VSCROLL
                    | nwg::TextBoxFlags::AUTOVSCROLL,
            )
            .build(&mut input)?;
        let mut feedback = nwg::Label::default();
        nwg::Label::builder()
            .parent(&window)
            .position((14, height - 119))
            .size((width - 28, 68))
            .build(&mut feedback)?;
        let mut stage = nwg::Button::default();
        nwg::Button::builder()
            .parent(&window)
            .position((width - 252, height - 43))
            .size((126, 30))
            .text("Stage points")
            .build(&mut stage)?;
        let mut cancel = nwg::Button::default();
        nwg::Button::builder()
            .parent(&window)
            .position((width - 119, height - 43))
            .size((105, 30))
            .text("Cancel")
            .build(&mut cancel)?;
        Ok(Self {
            window,
            _instructions: instructions,
            input,
            feedback,
            stage,
            cancel,
        })
    }

    pub fn show(&self, owner: &nwg::Window, value: &str) -> Result<(), String> {
        let points = serde_json::from_str::<Vec<Value>>(value).map_err(|e| e.to_string())?;
        let mut rows = Vec::new();
        for point in points {
            let row = if point.is_number() {
                point.to_string()
            } else {
                format!(
                    "{}, {}",
                    point.get("x").ok_or("Missing X coordinate")?,
                    point.get("y").ok_or("Missing Y coordinate")?
                )
            };
            rows.push(row);
        }
        self.input.set_text(&rows.join("\r\n"));
        self.feedback.set_text("Use 2 to 21 points. Custom X starts at −100 and ends at 100. Y accepts tenths of a percent.");
        owner.set_enabled(false);
        self.window.set_visible(true);
        self.input.set_focus();
        Ok(())
    }
    pub fn hide(&self, owner: &nwg::Window) {
        owner.set_enabled(true);
        self.window.set_visible(false);
        owner.set_focus();
    }
}

pub fn encode_rows(text: &str) -> Result<String, String> {
    let mut points = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parse = |s: &str| {
            s.trim()
                .parse::<f64>()
                .ok()
                .and_then(Number::from_f64)
                .ok_or_else(|| format!("Line {} needs a numeric percentage.", i + 1))
        };
        let point = if let Some((x, y)) = line.split_once(',') {
            json!({"x":parse(x)?, "y":parse(y)?})
        } else {
            Value::Number(parse(line)?)
        };
        points.push(point);
    }
    serde_json::to_string(&points).map_err(|e| e.to_string())
}
