//! ScrollBar widget with full mouse interaction support.
//!
//! The ScrollBar widget handles:
//! - Click on track: Jump scroll (ScrollUp/ScrollDown/ScrollLeft/ScrollRight)
//! - Drag thumb: Smooth scroll (ScrollTo)
//! - Hover states for CSS styling

use crate::scroll::ScrollMessage;
use crate::scrollbar::ScrollBarRender;
use crate::{Canvas, MouseEvent, MouseEventKind, Region, Size, Widget};
use tcss::types::{RgbaColor, ScrollbarStyle};
use tcss::{ComputedStyle, StyleOverride, WidgetStates};

/// Scrollbar interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ScrollBarState {
    #[default]
    Normal,
    Hover,
    /// Dragging with grab position (offset within thumb where grab started)
    Grabbed {
        grab_position: i32,
    },
}

/// A scrollbar widget that emits scroll messages.
///
/// ScrollBar is typically used by containers like ScrollableContainer.
/// It handles all mouse interactions and emits ScrollMessage for the
/// container to handle.
pub struct ScrollBar<M, F>
where
    F: Fn(ScrollMessage) -> M,
{
    /// Vertical (true) or horizontal (false)
    vertical: bool,
    /// Scrollbar thickness
    thickness: u16,
    /// Current scroll position (0 to virtual_size - window_size)
    position: f32,
    /// Position when grab started
    grabbed_position: f32,
    /// Total virtual content size
    virtual_size: f32,
    /// Visible window size
    window_size: f32,
    /// Current interaction state
    state: ScrollBarState,
    /// Styling from CSS
    scrollbar_style: ScrollbarStyle,
    /// Computed style for this widget
    computed_style: ComputedStyle,
    /// Inline style override
    inline_style: StyleOverride,
    /// Callback to convert ScrollMessage to app message
    on_scroll: F,
    /// Dirty flag for style recomputation
    dirty: bool,
    /// Optional widget ID
    id: Option<String>,
}

impl<M, F> ScrollBar<M, F>
where
    F: Fn(ScrollMessage) -> M,
{
    /// Create a new scrollbar.
    ///
    /// # Arguments
    /// * `vertical` - true for vertical scrollbar, false for horizontal
    /// * `on_scroll` - callback to convert ScrollMessage to app message
    pub fn new(vertical: bool, on_scroll: F) -> Self {
        Self {
            vertical,
            thickness: 1,
            position: 0.0,
            grabbed_position: 0.0,
            virtual_size: 100.0,
            window_size: 100.0,
            state: ScrollBarState::Normal,
            scrollbar_style: ScrollbarStyle::default(),
            computed_style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
            on_scroll,
            dirty: true,
            id: None,
        }
    }

    /// Set a unique ID for this widget.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the scrollbar thickness.
    pub fn with_thickness(mut self, thickness: u16) -> Self {
        self.thickness = thickness;
        self
    }

    /// Update scroll parameters from container.
    ///
    /// Call this whenever the container's content size or scroll position changes.
    pub fn update(&mut self, virtual_size: f32, window_size: f32, position: f32) {
        if (self.virtual_size - virtual_size).abs() > f32::EPSILON
            || (self.window_size - window_size).abs() > f32::EPSILON
            || (self.position - position).abs() > f32::EPSILON
        {
            self.virtual_size = virtual_size;
            self.window_size = window_size;
            self.position = position;
            self.dirty = true;
        }
    }

    /// Set the scrollbar style directly.
    pub fn set_scrollbar_style(&mut self, style: ScrollbarStyle) {
        self.scrollbar_style = style;
        self.dirty = true;
    }

    /// Calculate thumb bounds within the track.
    fn thumb_bounds(&self, region: &Region) -> (i32, i32) {
        let track_size = if self.vertical {
            region.height
        } else {
            region.width
        };
        ScrollBarRender::thumb_bounds(
            track_size,
            self.virtual_size,
            self.window_size,
            self.position,
        )
    }

    /// Get current colors based on state.
    fn current_colors(&self) -> (RgbaColor, RgbaColor) {
        match self.state {
            ScrollBarState::Grabbed { .. } => (
                self.scrollbar_style.effective_color_active(),
                self.scrollbar_style.effective_background_active(),
            ),
            ScrollBarState::Hover => (
                self.scrollbar_style.effective_color_hover(),
                self.scrollbar_style.effective_background_hover(),
            ),
            ScrollBarState::Normal => (
                self.scrollbar_style.effective_color(),
                self.scrollbar_style.effective_background(),
            ),
        }
    }

    /// Check if scrolling is possible (content larger than viewport).
    pub fn can_scroll(&self) -> bool {
        self.virtual_size > self.window_size
    }
}

impl<M, F> Widget<M> for ScrollBar<M, F>
where
    F: Fn(ScrollMessage) -> M,
{
    fn desired_size(&self) -> Size {
        if self.vertical {
            Size {
                width: self.thickness,
                height: 1,
            }
        } else {
            Size {
                width: 1,
                height: self.thickness,
            }
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let (thumb_color, track_color) = self.current_colors();
        let (thumb_color, track_color, draw_thumb) = ScrollBarRender::compose_colors(
            thumb_color,
            track_color,
            self.computed_style.inherited_background.clone(),
        );

        if self.vertical {
            ScrollBarRender::render_vertical(
                canvas,
                region,
                self.virtual_size,
                self.window_size,
                self.position,
                thumb_color,
                track_color,
                draw_thumb,
            );
        } else {
            ScrollBarRender::render_horizontal(
                canvas,
                region,
                self.virtual_size,
                self.window_size,
                self.position,
                thumb_color,
                track_color,
                draw_thumb,
            );
        }
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        // Check if mouse is within scrollbar region
        if !region.contains_point(mx, my) {
            // Mouse left - reset hover (but keep grabbed state if dragging)
            if self.state == ScrollBarState::Hover {
                self.state = ScrollBarState::Normal;
                self.dirty = true;
            }
            return None;
        }

        // Position along the scroll axis
        let pos_in_bar = if self.vertical {
            my - region.y
        } else {
            mx - region.x
        };

        let (thumb_start, thumb_end) = self.thumb_bounds(&region);
        let on_thumb = pos_in_bar >= thumb_start && pos_in_bar < thumb_end;

        match event.kind {
            MouseEventKind::Moved => {
                // Only update hover if not currently dragging
                if !matches!(self.state, ScrollBarState::Grabbed { .. }) {
                    if self.state != ScrollBarState::Hover {
                        self.state = ScrollBarState::Hover;
                        self.dirty = true;
                    }
                }
                None
            }

            MouseEventKind::Down(_) => {
                if on_thumb {
                    // Start drag - grab_position is where on the thumb we grabbed
                    let grab_offset = pos_in_bar - thumb_start;
                    self.state = ScrollBarState::Grabbed {
                        grab_position: grab_offset,
                    };
                    self.grabbed_position = self.position;
                    self.dirty = true;
                    None
                } else if pos_in_bar < thumb_start {
                    // Click above/left of thumb - scroll up/left
                    self.dirty = true;
                    let msg = if self.vertical {
                        ScrollMessage::ScrollUp
                    } else {
                        ScrollMessage::ScrollLeft
                    };
                    Some((self.on_scroll)(msg))
                } else {
                    // Click below/right of thumb - scroll down/right
                    self.dirty = true;
                    let msg = if self.vertical {
                        ScrollMessage::ScrollDown
                    } else {
                        ScrollMessage::ScrollRight
                    };
                    Some((self.on_scroll)(msg))
                }
            }

            MouseEventKind::Drag(_) => {
                if let ScrollBarState::Grabbed { grab_position } = self.state {
                    // Calculate new position based on drag
                    let track_size = if self.vertical {
                        region.height
                    } else {
                        region.width
                    } as f32;

                    // Where the thumb should start based on mouse position
                    let new_thumb_start = pos_in_bar - grab_position;

                    // Calculate the scroll position this corresponds to
                    let thumb_size = (self.window_size / self.virtual_size) * track_size;
                    let track_range = track_size - thumb_size;

                    let new_position = if track_range > 0.0 {
                        let ratio = new_thumb_start as f32 / track_range;
                        ratio * (self.virtual_size - self.window_size)
                    } else {
                        0.0
                    };

                    let (x, y) = if self.vertical {
                        (None, Some(new_position))
                    } else {
                        (Some(new_position), None)
                    };

                    Some((self.on_scroll)(ScrollMessage::ScrollTo {
                        x,
                        y,
                        animate: false,
                    }))
                } else {
                    None
                }
            }

            MouseEventKind::Up(_) => {
                if matches!(self.state, ScrollBarState::Grabbed { .. }) {
                    self.state = ScrollBarState::Hover;
                    self.dirty = true;
                }
                None
            }

            MouseEventKind::ScrollDown => {
                // Mouse wheel on scrollbar
                let msg = if self.vertical {
                    ScrollMessage::ScrollDown
                } else {
                    ScrollMessage::ScrollRight
                };
                Some((self.on_scroll)(msg))
            }

            MouseEventKind::ScrollUp => {
                let msg = if self.vertical {
                    ScrollMessage::ScrollUp
                } else {
                    ScrollMessage::ScrollLeft
                };
                Some((self.on_scroll)(msg))
            }

            _ => None,
        }
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        match self.state {
            ScrollBarState::Hover => states |= WidgetStates::HOVER,
            ScrollBarState::Grabbed { .. } => states |= WidgetStates::ACTIVE,
            ScrollBarState::Normal => {}
        }
        states
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.scrollbar_style = style.scrollbar.clone();
        self.computed_style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.computed_style.clone()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inline_style = style;
        self.dirty = true;
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        if self.inline_style.is_empty() {
            None
        } else {
            Some(&self.inline_style)
        }
    }

    fn clear_inline_style(&mut self) {
        self.inline_style = StyleOverride::default();
        self.dirty = true;
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

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        if !matches!(self.state, ScrollBarState::Grabbed { .. }) {
            let new_state = if is_hovered {
                ScrollBarState::Hover
            } else {
                ScrollBarState::Normal
            };
            if self.state != new_state {
                self.state = new_state;
                self.dirty = true;
                return true;
            }
        }
        false
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        if is_active {
            // Active/pressed state is internally managed via Grabbed during drag.
            // External set_active(true) is a no-op since we track drag state ourselves.
            false
        } else {
            if matches!(self.state, ScrollBarState::Grabbed { .. }) {
                self.state = ScrollBarState::Normal;
                self.dirty = true;
                return true;
            }
            false
        }
    }

    fn clear_hover(&mut self) {
        if !matches!(self.state, ScrollBarState::Grabbed { .. }) {
            if self.state != ScrollBarState::Normal {
                self.state = ScrollBarState::Normal;
                self.dirty = true;
            }
        }
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
