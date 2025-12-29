use tcss::{ComputedStyle, WidgetStates};

use crate::canvas::TextAttributes;
use crate::{Canvas, KeyCode, MouseEvent, MouseEventKind, Region, Size, Widget};

/// Braille spinner animation frames for loading state.
const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// A toggle switch widget that produces messages via a callback.
///
/// Supports focus, hover, and active pseudo-class states for CSS styling.
/// Also supports reactive visibility, loading, and disabled states.
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
    /// Whether the widget is visible
    visible: bool,
    /// Whether the widget is in loading state
    loading: bool,
    /// Whether the widget is disabled
    disabled: bool,
    /// Current frame of the loading spinner animation
    spinner_frame: usize,
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
            visible: true,
            loading: false,
            disabled: false,
            spinner_frame: 0,
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

    /// Set initial visibility state.
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set initial loading state.
    pub fn with_loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set the spinner animation frame (0-9).
    ///
    /// Use this when recomposing to maintain animation continuity.
    pub fn with_spinner_frame(mut self, frame: usize) -> Self {
        self.spinner_frame = frame % SPINNER_FRAMES.len();
        self
    }

    /// Set initial disabled state.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the switch value externally (e.g., from API response).
    ///
    /// Unlike toggling via keyboard/mouse, this doesn't produce a message.
    pub fn set_value(&mut self, value: bool) {
        if self.value != value {
            self.value = value;
            self.dirty = true;
        }
    }

    /// Advance the spinner animation frame.
    ///
    /// Call this from a timer (e.g., every 100ms) for smooth animation.
    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        if self.loading {
            self.dirty = true;
        }
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
            "SWITCH RENDER: fg={:?} bg={:?} loading={} disabled={}",
            self.style.color,
            self.style.background,
            self.loading,
            self.disabled
        );

        // Show loading spinner instead of normal content
        if self.loading {
            let frame = SPINNER_FRAMES[self.spinner_frame];
            let style_bracket_l = if self.focused { ">[" } else { " [" };
            let style_bracket_r = if self.focused { " ]<" } else { " ] " };
            let display = format!("{}  {}  {}", style_bracket_l, frame, style_bracket_r);
            canvas.put_str(
                region.x,
                region.y,
                &display,
                self.style.color.clone(),
                self.style.background.clone(),
                TextAttributes::default(),
            );
            return;
        }

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
            TextAttributes::default(),
        );
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Ignore input when disabled, loading, or not focused
        if self.disabled || self.loading || !self.focused {
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
        if self.disabled {
            states |= WidgetStates::DISABLED;
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
        // Can't focus if invisible, disabled, or loading
        self.visible && !self.disabled && !self.loading
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
        // Disabled widgets still track hover for styling, but don't respond to clicks
        let mx = event.column as i32;
        let my = event.row as i32;

        // Hit test: is mouse within this widget's region?
        let in_bounds = region.contains_point(mx, my);

        match event.kind {
            MouseEventKind::Moved => {
                // Update hover state (even when disabled, for CSS :hover:disabled)
                if in_bounds != self.hovered {
                    self.hovered = in_bounds;
                    self.dirty = true;
                }
                None
            }
            MouseEventKind::Down(_button) if in_bounds && !self.disabled && !self.loading => {
                // Start press (active state) - only if not disabled or loading
                if !self.active {
                    self.active = true;
                    self.dirty = true;
                }
                None
            }
            MouseEventKind::Up(_button) if in_bounds && self.active && !self.disabled && !self.loading => {
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

    // =========================================================================
    // Reactive Attributes
    // =========================================================================

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.dirty = true;
        }
    }

    fn is_loading(&self) -> bool {
        self.loading
    }

    fn set_loading(&mut self, loading: bool) {
        if self.loading != loading {
            self.loading = loading;
            self.dirty = true;
        }
    }

    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn set_disabled(&mut self, disabled: bool) {
        if self.disabled != disabled {
            self.disabled = disabled;
            self.dirty = true;
        }
    }
}
