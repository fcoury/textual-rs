//! Scroll-related messages and types.

/// Messages emitted by scrollbars for container handling.
#[derive(Debug, Clone)]
pub enum ScrollMessage {
    /// Scroll up by one "click" (click above thumb or wheel up)
    ScrollUp,
    /// Scroll down by one "click" (click below thumb or wheel down)
    ScrollDown,
    /// Scroll left by one "click"
    ScrollLeft,
    /// Scroll right by one "click"
    ScrollRight,
    /// Scroll to absolute position (drag or programmatic)
    ScrollTo {
        x: Option<f32>,
        y: Option<f32>,
        animate: bool,
    },
}

/// Scroll state for a scrollable container.
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Current scroll offset (pixels/cells from left)
    pub offset_x: i32,
    /// Current scroll offset (pixels/cells from top)
    pub offset_y: i32,
    /// Virtual content width
    pub virtual_width: i32,
    /// Virtual content height
    pub virtual_height: i32,
    /// Viewport width
    pub viewport_width: i32,
    /// Viewport height
    pub viewport_height: i32,
}

impl ScrollState {
    /// Create new scroll state with given dimensions.
    pub fn new(
        virtual_width: i32,
        virtual_height: i32,
        viewport_width: i32,
        viewport_height: i32,
    ) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            virtual_width,
            virtual_height,
            viewport_width,
            viewport_height,
        }
    }

    /// Maximum horizontal scroll offset.
    pub fn max_scroll_x(&self) -> i32 {
        (self.virtual_width - self.viewport_width).max(0)
    }

    /// Maximum vertical scroll offset.
    pub fn max_scroll_y(&self) -> i32 {
        (self.virtual_height - self.viewport_height).max(0)
    }

    /// Whether horizontal scrolling is possible.
    pub fn can_scroll_x(&self) -> bool {
        self.virtual_width > self.viewport_width
    }

    /// Whether vertical scrolling is possible.
    pub fn can_scroll_y(&self) -> bool {
        self.virtual_height > self.viewport_height
    }

    /// Current horizontal scroll position as 0.0-1.0 percentage.
    pub fn scroll_percent_x(&self) -> f32 {
        if self.max_scroll_x() == 0 {
            0.0
        } else {
            self.offset_x as f32 / self.max_scroll_x() as f32
        }
    }

    /// Current vertical scroll position as 0.0-1.0 percentage.
    pub fn scroll_percent_y(&self) -> f32 {
        if self.max_scroll_y() == 0 {
            0.0
        } else {
            self.offset_y as f32 / self.max_scroll_y() as f32
        }
    }

    /// Scroll up by given amount (clamped to bounds).
    pub fn scroll_up(&mut self, amount: i32) {
        self.offset_y = (self.offset_y - amount).max(0);
    }

    /// Scroll down by given amount (clamped to bounds).
    pub fn scroll_down(&mut self, amount: i32) {
        self.offset_y = (self.offset_y + amount).min(self.max_scroll_y());
    }

    /// Scroll left by given amount (clamped to bounds).
    pub fn scroll_left(&mut self, amount: i32) {
        self.offset_x = (self.offset_x - amount).max(0);
    }

    /// Scroll right by given amount (clamped to bounds).
    pub fn scroll_right(&mut self, amount: i32) {
        self.offset_x = (self.offset_x + amount).min(self.max_scroll_x());
    }

    /// Scroll to absolute position (clamped to bounds).
    pub fn scroll_to(&mut self, x: Option<f32>, y: Option<f32>) {
        if let Some(x) = x {
            let rounded = x.round() as i32;
            self.offset_x = rounded.clamp(0, self.max_scroll_x());
        }
        if let Some(y) = y {
            let rounded = y.round() as i32;
            self.offset_y = rounded.clamp(0, self.max_scroll_y());
        }
    }

    /// Update viewport dimensions.
    pub fn set_viewport(&mut self, width: i32, height: i32) {
        self.viewport_width = width;
        self.viewport_height = height;
        // Clamp current offset to new bounds
        self.offset_x = self.offset_x.min(self.max_scroll_x()).max(0);
        self.offset_y = self.offset_y.min(self.max_scroll_y()).max(0);
    }

    /// Update virtual content dimensions.
    pub fn set_virtual_size(&mut self, width: i32, height: i32) {
        self.virtual_width = width;
        self.virtual_height = height;
        // Clamp current offset to new bounds
        self.offset_x = self.offset_x.min(self.max_scroll_x()).max(0);
        self.offset_y = self.offset_y.min(self.max_scroll_y()).max(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state_new() {
        let state = ScrollState::new(200, 300, 100, 150);
        assert_eq!(state.virtual_width, 200);
        assert_eq!(state.virtual_height, 300);
        assert_eq!(state.viewport_width, 100);
        assert_eq!(state.viewport_height, 150);
        assert_eq!(state.offset_x, 0);
        assert_eq!(state.offset_y, 0);
    }

    #[test]
    fn test_max_scroll() {
        let state = ScrollState::new(200, 300, 100, 150);
        assert_eq!(state.max_scroll_x(), 100);
        assert_eq!(state.max_scroll_y(), 150);
    }

    #[test]
    fn test_max_scroll_no_overflow() {
        let state = ScrollState::new(50, 50, 100, 100);
        assert_eq!(state.max_scroll_x(), 0);
        assert_eq!(state.max_scroll_y(), 0);
    }

    #[test]
    fn test_can_scroll() {
        let state = ScrollState::new(200, 100, 100, 100);
        assert!(state.can_scroll_x());
        assert!(!state.can_scroll_y());
    }

    #[test]
    fn test_scroll_percent_zero() {
        let state = ScrollState::new(200, 200, 100, 100);
        assert_eq!(state.scroll_percent_x(), 0.0);
        assert_eq!(state.scroll_percent_y(), 0.0);
    }

    #[test]
    fn test_scroll_percent_no_overflow() {
        let state = ScrollState::new(50, 50, 100, 100);
        // No scrolling possible, should return 0
        assert_eq!(state.scroll_percent_x(), 0.0);
        assert_eq!(state.scroll_percent_y(), 0.0);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = ScrollState::new(100, 200, 100, 100);
        state.scroll_down(25);
        assert_eq!(state.offset_y, 25);
        state.scroll_down(100);
        assert_eq!(state.offset_y, 100); // Clamped to max
    }

    #[test]
    fn test_scroll_up() {
        let mut state = ScrollState::new(100, 200, 100, 100);
        state.offset_y = 50;
        state.scroll_up(25);
        assert_eq!(state.offset_y, 25);
        state.scroll_up(100);
        assert_eq!(state.offset_y, 0); // Clamped to 0
    }

    #[test]
    fn test_scroll_right() {
        let mut state = ScrollState::new(200, 100, 100, 100);
        state.scroll_right(25);
        assert_eq!(state.offset_x, 25);
        state.scroll_right(100);
        assert_eq!(state.offset_x, 100); // Clamped to max
    }

    #[test]
    fn test_scroll_left() {
        let mut state = ScrollState::new(200, 100, 100, 100);
        state.offset_x = 50;
        state.scroll_left(25);
        assert_eq!(state.offset_x, 25);
        state.scroll_left(100);
        assert_eq!(state.offset_x, 0); // Clamped to 0
    }

    #[test]
    fn test_scroll_to() {
        let mut state = ScrollState::new(200, 200, 100, 100);
        state.scroll_to(Some(50.0), Some(75.0));
        assert_eq!(state.offset_x, 50);
        assert_eq!(state.offset_y, 75);
    }

    #[test]
    fn test_scroll_to_clamped() {
        let mut state = ScrollState::new(200, 200, 100, 100);
        state.scroll_to(Some(200.0), Some(-50.0));
        assert_eq!(state.offset_x, 100); // Clamped to max
        assert_eq!(state.offset_y, 0); // Clamped to 0
    }

    #[test]
    fn test_set_viewport_clamps_offset() {
        let mut state = ScrollState::new(200, 200, 100, 100);
        state.offset_x = 100;
        state.offset_y = 100;
        state.set_viewport(150, 150);
        assert_eq!(state.offset_x, 50); // Clamped to new max (200-150=50)
        assert_eq!(state.offset_y, 50);
    }

    #[test]
    fn test_set_virtual_size_clamps_offset() {
        let mut state = ScrollState::new(200, 200, 100, 100);
        state.offset_x = 100;
        state.offset_y = 100;
        state.set_virtual_size(120, 120);
        assert_eq!(state.offset_x, 20); // Clamped to new max (120-100=20)
        assert_eq!(state.offset_y, 20);
    }
}
