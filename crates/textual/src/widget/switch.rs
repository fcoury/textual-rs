use tcss::{ComputedStyle, WidgetStates};

use crate::{Canvas, KeyCode, MouseEvent, MouseEventKind, Region, Size, Widget};

/// A toggle switch widget that produces messages via a callback.
///
/// Supports focus, hover, and active pseudo-class states for CSS styling.
pub struct Switch<M, F>
where
    F: Fn(bool) -> M,
{
    /// Whether the widget has keyboard focus
    pub focused: bool,
    /// Whether the mouse is hovering over the widget
    pub hovered: bool,
    /// Whether the widget is being actively pressed
    pub active: bool,
    /// The current on/off value
    pub value: bool,
    /// Computed CSS styles
    pub style: ComputedStyle,
    /// Whether styles need to be recomputed
    dirty: bool,
    /// Optional widget ID for message tracking
    id: Option<String>,
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
            hovered: false,
            active: false,
            dirty: true, // Start dirty so initial styles are computed
            id: None,
            on_change,
            style: ComputedStyle::default(),
        }
    }

    /// Set a unique ID for this widget.
    ///
    /// The ID is included in `MessageEnvelope.sender_id` when this widget
    /// produces a message, allowing you to identify which widget sent it.
    ///
    /// # Example
    /// ```ignore
    /// Switch::new(false, Message::WifiToggled).with_id("wifi-switch")
    /// ```
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        if self.focused != focused {
            self.focused = focused;
            self.dirty = true;
        }
        self
    }

    pub fn with_hover(mut self, hovered: bool) -> Self {
        if self.hovered != hovered {
            self.hovered = hovered;
            self.dirty = true;
        }
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        if self.active != active {
            self.active = active;
            self.dirty = true;
        }
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
            KeyCode::Char(' ') | KeyCode::Enter => {
                // Toggle our own value (persistent widget owns its state)
                self.value = !self.value;
                self.dirty = true;
                // Notify the app of the change
                Some((self.on_change)(self.value))
            }
            _ => None,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, is_focused: bool) {
        if self.focused != is_focused {
            self.focused = is_focused;
            self.dirty = true; // Reactive: mark dirty when state changes
        }
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        if self.focused {
            states |= WidgetStates::FOCUS;
        }
        if self.hovered {
            states |= WidgetStates::HOVER;
        }
        if self.active {
            states |= WidgetStates::ACTIVE;
        }
        states
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if n == 0 {
            self.set_focus(true);
            true
        } else {
            false
        }
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let x = event.column;
        let y = event.row;

        // Hit test: is mouse within this widget's region?
        let in_bounds = x >= region.x
            && x < region.x + region.width
            && y >= region.y
            && y < region.y + region.height;

        match event.kind {
            MouseEventKind::Moved => {
                // Update hover state
                if in_bounds != self.hovered {
                    self.hovered = in_bounds;
                    self.dirty = true;
                }
                None
            }
            MouseEventKind::Down(_button) if in_bounds => {
                // Start press (active state)
                if !self.active {
                    self.active = true;
                    self.dirty = true;
                }
                None
            }
            MouseEventKind::Up(_button) if in_bounds && self.active => {
                // Complete click: toggle value and send message
                self.active = false;
                self.value = !self.value;
                self.dirty = true;
                Some((self.on_change)(self.value))
            }
            MouseEventKind::Up(_) => {
                // Mouse released outside - cancel active state
                if self.active {
                    self.active = false;
                    self.dirty = true;
                }
                None
            }
            _ => None,
        }
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        if self.hovered != is_hovered {
            self.hovered = is_hovered;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        if self.active != is_active {
            self.active = is_active;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn clear_hover(&mut self) {
        if self.hovered {
            self.hovered = false;
            self.dirty = true;
        }
        if self.active {
            self.active = false;
            self.dirty = true;
        }
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
