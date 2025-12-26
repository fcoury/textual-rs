use crate::{Canvas, KeyCode, Message, Region, Size, Widget};

pub struct Switch {
    pub id: &'static str,
    pub focused: bool,
    pub on: bool,
}

impl Switch {
    pub fn new(id: &'static str, on: bool) -> Self {
        Self {
            id,
            on,
            focused: false,
        }
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for Switch {
    fn desired_size(&self) -> Size {
        Size {
            width: 8,
            height: 3,
        } // Fixed size for the switch widget
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let style_bracket_l = if self.focused { ">[" } else { " [" };
        let style_bracket_r = if self.focused { " ]<" } else { " ] " };
        let state_text = if self.on { "  ON " } else { " OFF " };

        let display = format!("{}{}{}", style_bracket_l, state_text, style_bracket_r);

        canvas.put_str(region.x, region.y, &display);
    }

    fn on_event(&mut self, key: KeyCode) -> Option<Message> {
        match key {
            KeyCode::Char(' ') | KeyCode::Enter => Some(Message::SwitchChanged {
                id: self.id,
                on: !self.on,
            }),
            _ => None,
        }
    }
}
