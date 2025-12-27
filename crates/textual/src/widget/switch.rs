use tcss::ComputedStyle;

use crate::{Canvas, KeyCode, Region, Size, Widget};

/// A toggle switch widget that produces messages via a callback.
pub struct Switch<M, F>
where
    F: Fn(bool) -> M,
{
    pub focused: bool,
    pub value: bool,
    pub style: ComputedStyle,
    on_change: F,
}

impl<M, F> Switch<M, F>
where
    F: Fn(bool) -> M,
{
    pub fn new(value: bool, on_change: F) -> Self {
        Self {
            value,
            focused: false,
            on_change,
            style: ComputedStyle::default(),
        }
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<M, F> Widget<M> for Switch<M, F>
where
    F: Fn(bool) -> M,
{
    fn desired_size(&self) -> Size {
        Size {
            width: 10,
            height: 3,
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        // Log to verify what colors are actually in the struct right now
        log::debug!(
            "SWITCH RENDER: fg={:?} bg={:?}",
            self.style.color,
            self.style.background
        );

        let style_bracket_l = if self.focused { ">[" } else { " [" };
        let style_bracket_r = if self.focused { " ]<" } else { " ] " };
        let state_text = if self.value { "  ON " } else { " OFF " };

        let display = format!("{}{}{}", style_bracket_l, state_text, style_bracket_r);

        // This call sends the colors to the Canvas
        canvas.put_str(
            region.x,
            region.y,
            &display,
            self.style.color.clone(),
            self.style.background.clone(),
        );
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        if !self.focused {
            return None;
        }

        match key {
            KeyCode::Char(' ') | KeyCode::Enter => Some((self.on_change)(!self.value)),
            _ => None,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, is_focused: bool) {
        self.focused = is_focused;
    }
}
