//! ScrollableContainer - A container that provides scrolling for its content.
//!
//! This container wraps any widget and provides:
//! - Vertical and/or horizontal scrolling
//! - Scrollbars with full mouse interaction
//! - Keyboard navigation (arrow keys, page up/down)
//! - Mouse wheel support
//! - CSS-configurable scrollbar styling

use std::cell::RefCell;

use crate::canvas::{Canvas, Region, Size};
use crate::scroll::{ScrollMessage, ScrollState};
use crate::scrollbar::ScrollBarRender;
use crate::widget::scrollbar_corner::ScrollBarCorner;
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent, MouseEventKind};
use tcss::types::{Overflow, ScrollbarGutter, ScrollbarStyle, ScrollbarVisibility};
use tcss::ComputedStyle;

/// Scroll amount for single scroll events (clicks, wheel, arrow keys).
const SCROLL_AMOUNT: i32 = 3;

/// Page scroll amount multiplier (relative to viewport).
const PAGE_SCROLL_RATIO: f32 = 0.9;

/// A scrollable container that wraps content and provides scrollbars.
///
/// The container manages scroll state and renders scrollbars when content
/// exceeds the viewport. It handles mouse wheel, keyboard navigation,
/// and scrollbar interactions.
pub struct ScrollableContainer<M> {
    /// The content widget to scroll
    content: Box<dyn Widget<M>>,
    /// Current scroll state (RefCell for interior mutability in render)
    scroll: RefCell<ScrollState>,
    /// Computed style from CSS
    style: ComputedStyle,
    /// Dirty flag
    dirty: bool,
    /// Scrollbar interaction state: None, Some(vertical: bool)
    scrollbar_hover: Option<bool>,
    /// Scrollbar being dragged: None, Some((vertical, grab_offset))
    scrollbar_drag: Option<(bool, i32)>,
}

impl<M> ScrollableContainer<M> {
    /// Create a new scrollable container wrapping the given content.
    pub fn new(content: Box<dyn Widget<M>>) -> Self {
        Self {
            content,
            scroll: RefCell::new(ScrollState::default()),
            style: ComputedStyle::default(),
            dirty: true,
            scrollbar_hover: None,
            scrollbar_drag: None,
        }
    }

    /// Get the scrollbar style from computed style.
    fn scrollbar_style(&self) -> &ScrollbarStyle {
        &self.style.scrollbar
    }

    /// Check if vertical scrollbar should be shown.
    ///
    /// For `Overflow::Auto`, we check if content height exceeds viewport height.
    /// Returns false if viewport is not yet initialized (height == 0).
    fn show_vertical_scrollbar(&self) -> bool {
        let style = self.scrollbar_style();
        if style.visibility == ScrollbarVisibility::Hidden || style.size.vertical == 0 {
            return false;
        }
        match self.style.overflow_y {
            Overflow::Scroll => true,
            Overflow::Auto => {
                // Only show scrollbar when viewport is initialized and content exceeds it
                let scroll = self.scroll.borrow();
                if scroll.viewport_height <= 0 {
                    return false;
                }
                let content_height = self.content.desired_size().height as i32;
                content_height > scroll.viewport_height
            }
            Overflow::Hidden => false,
        }
    }

    /// Check if horizontal scrollbar should be shown.
    ///
    /// For `Overflow::Auto`, we check if content width exceeds viewport width.
    /// Returns false if viewport is not yet initialized (width == 0).
    fn show_horizontal_scrollbar(&self) -> bool {
        let style = self.scrollbar_style();
        if style.visibility == ScrollbarVisibility::Hidden || style.size.horizontal == 0 {
            return false;
        }
        match self.style.overflow_x {
            Overflow::Scroll => true,
            Overflow::Auto => {
                // Only show scrollbar when viewport is initialized and content exceeds it
                let scroll = self.scroll.borrow();
                if scroll.viewport_width <= 0 {
                    return false;
                }
                let content_width = self.content.desired_size().width as i32;
                content_width > scroll.viewport_width
            }
            Overflow::Hidden => false,
        }
    }

    /// Calculate the content region (excluding scrollbars).
    fn content_region(&self, region: Region) -> Region {
        let style = self.scrollbar_style();
        let v_size = if self.show_vertical_scrollbar() || style.gutter == ScrollbarGutter::Stable {
            style.size.vertical as i32
        } else {
            0
        };
        let h_size = if self.show_horizontal_scrollbar() || style.gutter == ScrollbarGutter::Stable
        {
            style.size.horizontal as i32
        } else {
            0
        };

        Region {
            x: region.x,
            y: region.y,
            width: (region.width - v_size).max(0),
            height: (region.height - h_size).max(0),
        }
    }

    /// Calculate the vertical scrollbar region.
    fn vertical_scrollbar_region(&self, region: Region) -> Region {
        let style = self.scrollbar_style();
        let h_size = if self.show_horizontal_scrollbar() {
            style.size.horizontal as i32
        } else {
            0
        };

        Region {
            x: region.x + region.width - style.size.vertical as i32,
            y: region.y,
            width: style.size.vertical as i32,
            height: (region.height - h_size).max(0),
        }
    }

    /// Calculate the horizontal scrollbar region.
    fn horizontal_scrollbar_region(&self, region: Region) -> Region {
        let style = self.scrollbar_style();
        let v_size = if self.show_vertical_scrollbar() {
            style.size.vertical as i32
        } else {
            0
        };

        Region {
            x: region.x,
            y: region.y + region.height - style.size.horizontal as i32,
            width: (region.width - v_size).max(0),
            height: style.size.horizontal as i32,
        }
    }

    /// Calculate the corner region.
    fn corner_region(&self, region: Region) -> Region {
        let style = self.scrollbar_style();
        Region {
            x: region.x + region.width - style.size.vertical as i32,
            y: region.y + region.height - style.size.horizontal as i32,
            width: style.size.vertical as i32,
            height: style.size.horizontal as i32,
        }
    }

    /// Handle a scroll message.
    fn handle_scroll(&mut self, msg: ScrollMessage) {
        match msg {
            ScrollMessage::ScrollUp => {
                self.scroll.borrow_mut().scroll_up(SCROLL_AMOUNT);
                self.dirty = true;
            }
            ScrollMessage::ScrollDown => {
                self.scroll.borrow_mut().scroll_down(SCROLL_AMOUNT);
                self.dirty = true;
            }
            ScrollMessage::ScrollLeft => {
                self.scroll.borrow_mut().scroll_left(SCROLL_AMOUNT);
                self.dirty = true;
            }
            ScrollMessage::ScrollRight => {
                self.scroll.borrow_mut().scroll_right(SCROLL_AMOUNT);
                self.dirty = true;
            }
            ScrollMessage::ScrollTo { x, y, animate: _ } => {
                self.scroll.borrow_mut().scroll_to(x, y);
                self.dirty = true;
            }
        }
    }

    /// Update scroll state dimensions from content and viewport.
    /// Uses interior mutability so it can be called from render().
    fn update_scroll_dimensions(&self, content_region: Region) {
        let content_size = self.content.desired_size();
        let mut scroll = self.scroll.borrow_mut();
        scroll.set_virtual_size(content_size.width as i32, content_size.height as i32);
        scroll.set_viewport(content_region.width, content_region.height);
    }

    /// Get colors for vertical scrollbar based on hover/drag state.
    fn vertical_colors(&self) -> (tcss::types::RgbaColor, tcss::types::RgbaColor) {
        let style = self.scrollbar_style();
        if self.scrollbar_drag.map(|(v, _)| v).unwrap_or(false) {
            (style.effective_color_active(), style.effective_background_active())
        } else if self.scrollbar_hover == Some(true) {
            (style.effective_color_hover(), style.effective_background_hover())
        } else {
            (style.effective_color(), style.effective_background())
        }
    }

    /// Get colors for horizontal scrollbar based on hover/drag state.
    fn horizontal_colors(&self) -> (tcss::types::RgbaColor, tcss::types::RgbaColor) {
        let style = self.scrollbar_style();
        if self.scrollbar_drag.map(|(v, _)| !v).unwrap_or(false) {
            (style.effective_color_active(), style.effective_background_active())
        } else if self.scrollbar_hover == Some(false) {
            (style.effective_color_hover(), style.effective_background_hover())
        } else {
            (style.effective_color(), style.effective_background())
        }
    }
}

impl<M> Widget<M> for ScrollableContainer<M> {
    fn desired_size(&self) -> Size {
        // ScrollableContainer fills available space
        // Return content size as hint, but container should expand
        self.content.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        // Update scroll dimensions FIRST so show_*_scrollbar() has correct viewport info
        // This fixes keyboard-only scrolling and overflow:auto decisions on first render
        let content_size = self.content.desired_size();
        {
            let mut scroll = self.scroll.borrow_mut();
            scroll.set_virtual_size(content_size.width as i32, content_size.height as i32);
            // We need to estimate content region size before calling content_region()
            // to avoid chicken-and-egg problem with scrollbar visibility
            let style = self.scrollbar_style();
            let est_v_size = match self.style.overflow_y {
                Overflow::Scroll => style.size.vertical as i32,
                Overflow::Auto => {
                    // If content is taller than region, we'll need scrollbar
                    if content_size.height as i32 > region.height {
                        style.size.vertical as i32
                    } else {
                        0
                    }
                }
                Overflow::Hidden => 0,
            };
            let est_h_size = match self.style.overflow_x {
                Overflow::Scroll => style.size.horizontal as i32,
                Overflow::Auto => {
                    if content_size.width as i32 > region.width {
                        style.size.horizontal as i32
                    } else {
                        0
                    }
                }
                Overflow::Hidden => 0,
            };
            scroll.set_viewport(
                (region.width - est_v_size).max(0),
                (region.height - est_h_size).max(0),
            );
        }

        let content_region = self.content_region(region);

        // Debug logging
        let scroll = self.scroll.borrow();
        log::debug!(
            "ScrollableContainer::render - region: ({}, {}, {}, {}), content_region: ({}, {}, {}, {})",
            region.x, region.y, region.width, region.height,
            content_region.x, content_region.y, content_region.width, content_region.height
        );
        log::debug!(
            "  scroll offset: ({}, {}), content_size: ({}, {})",
            scroll.offset_x, scroll.offset_y,
            content_size.width, content_size.height
        );
        log::debug!(
            "  show_vertical: {}, show_horizontal: {}, style.scrollbar.size: ({}, {})",
            self.show_vertical_scrollbar(),
            self.show_horizontal_scrollbar(),
            self.style.scrollbar.size.horizontal,
            self.style.scrollbar.size.vertical
        );
        log::debug!(
            "  overflow_x: {:?}, overflow_y: {:?}",
            self.style.overflow_x,
            self.style.overflow_y
        );
        let (offset_x, offset_y) = (scroll.offset_x, scroll.offset_y);
        drop(scroll); // Release borrow before calling content_region again

        // Render content with clipping and scroll offset
        canvas.push_clip(content_region);

        // Calculate content position with scroll offset
        let content_render_region = Region {
            x: content_region.x - offset_x,
            y: content_region.y - offset_y,
            width: content_size.width as i32,
            height: content_size.height as i32,
        };

        log::debug!(
            "  content_render_region: ({}, {}, {}, {})",
            content_render_region.x, content_render_region.y,
            content_render_region.width, content_render_region.height
        );

        self.content.render(canvas, content_render_region);
        canvas.pop_clip();

        // Render vertical scrollbar
        if self.show_vertical_scrollbar() {
            let v_region = self.vertical_scrollbar_region(region);
            let (thumb_color, track_color) = self.vertical_colors();

            ScrollBarRender::render_vertical(
                canvas,
                v_region,
                content_size.height as f32,
                content_region.height as f32,
                offset_y as f32,
                thumb_color,
                track_color,
            );
        }

        // Render horizontal scrollbar
        if self.show_horizontal_scrollbar() {
            let h_region = self.horizontal_scrollbar_region(region);
            let (thumb_color, track_color) = self.horizontal_colors();

            ScrollBarRender::render_horizontal(
                canvas,
                h_region,
                content_size.width as f32,
                content_region.width as f32,
                offset_x as f32,
                thumb_color,
                track_color,
            );
        }

        // Render corner if both scrollbars visible
        if self.show_vertical_scrollbar() && self.show_horizontal_scrollbar() {
            let corner_region = self.corner_region(region);
            let style = self.scrollbar_style();
            let corner = ScrollBarCorner::new(
                style.size.vertical,
                style.size.horizontal,
            );
            <ScrollBarCorner as Widget<M>>::render(&corner, canvas, corner_region);
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
        self.dirty = true;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn is_dirty(&self) -> bool {
        self.dirty || self.content.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
        self.content.mark_clean();
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        f(self.content.as_mut());
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Handle scroll keys
        match key {
            KeyCode::Up => {
                self.handle_scroll(ScrollMessage::ScrollUp);
                return None;
            }
            KeyCode::Down => {
                self.handle_scroll(ScrollMessage::ScrollDown);
                return None;
            }
            KeyCode::Left => {
                self.handle_scroll(ScrollMessage::ScrollLeft);
                return None;
            }
            KeyCode::Right => {
                self.handle_scroll(ScrollMessage::ScrollRight);
                return None;
            }
            KeyCode::PageUp => {
                let mut scroll = self.scroll.borrow_mut();
                let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
                scroll.scroll_up(amount);
                drop(scroll);
                self.dirty = true;
                return None;
            }
            KeyCode::PageDown => {
                let mut scroll = self.scroll.borrow_mut();
                let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
                scroll.scroll_down(amount);
                drop(scroll);
                self.dirty = true;
                return None;
            }
            KeyCode::Home => {
                self.scroll.borrow_mut().scroll_to(Some(0.0), Some(0.0));
                self.dirty = true;
                return None;
            }
            KeyCode::End => {
                let mut scroll = self.scroll.borrow_mut();
                let max_x = scroll.max_scroll_x() as f32;
                let max_y = scroll.max_scroll_y() as f32;
                scroll.scroll_to(Some(max_x), Some(max_y));
                drop(scroll);
                self.dirty = true;
                return None;
            }
            _ => {}
        }

        // Pass other keys to content
        self.content.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        // Update scroll dimensions
        let content_region = self.content_region(region);
        self.update_scroll_dimensions(content_region);

        // Check scrollbar regions first
        let v_region = self.vertical_scrollbar_region(region);
        let h_region = self.horizontal_scrollbar_region(region);

        let on_vertical = self.show_vertical_scrollbar() && v_region.contains_point(mx, my);
        let on_horizontal = self.show_horizontal_scrollbar() && h_region.contains_point(mx, my);

        match event.kind {
            MouseEventKind::Moved => {
                // Handle drag
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    return self.handle_scrollbar_drag(event, region, vertical, grab_offset);
                }

                // Update hover state
                let new_hover = if on_vertical {
                    Some(true)
                } else if on_horizontal {
                    Some(false)
                } else {
                    None
                };

                if self.scrollbar_hover != new_hover {
                    self.scrollbar_hover = new_hover;
                    self.dirty = true;
                }

                // Pass to content if in content area
                if content_region.contains_point(mx, my) {
                    return self.content.on_mouse(event, self.scrolled_content_region(content_region));
                }
                None
            }

            MouseEventKind::Down(_) => {
                if on_vertical {
                    return self.handle_vertical_scrollbar_click(event, v_region);
                } else if on_horizontal {
                    return self.handle_horizontal_scrollbar_click(event, h_region);
                } else if content_region.contains_point(mx, my) {
                    return self.content.on_mouse(event, self.scrolled_content_region(content_region));
                }
                None
            }

            MouseEventKind::Drag(_) => {
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    return self.handle_scrollbar_drag(event, region, vertical, grab_offset);
                }
                // Pass to content
                if content_region.contains_point(mx, my) {
                    return self.content.on_mouse(event, self.scrolled_content_region(content_region));
                }
                None
            }

            MouseEventKind::Up(_) => {
                if self.scrollbar_drag.is_some() {
                    self.scrollbar_drag = None;
                    self.dirty = true;
                }
                // Pass to content
                if content_region.contains_point(mx, my) {
                    return self.content.on_mouse(event, self.scrolled_content_region(content_region));
                }
                None
            }

            MouseEventKind::ScrollDown => {
                self.handle_scroll(ScrollMessage::ScrollDown);
                None
            }

            MouseEventKind::ScrollUp => {
                self.handle_scroll(ScrollMessage::ScrollUp);
                None
            }

            _ => {
                // Pass other events to content if in content area
                if content_region.contains_point(mx, my) {
                    return self.content.on_mouse(event, self.scrolled_content_region(content_region));
                }
                None
            }
        }
    }

    fn count_focusable(&self) -> usize {
        self.content.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.content.clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.content.focus_nth(n)
    }

    fn child_count(&self) -> usize {
        1
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index == 0 {
            Some(self.content.as_mut())
        } else {
            None
        }
    }

    fn clear_hover(&mut self) {
        self.scrollbar_hover = None;
        self.content.clear_hover();
    }
}

impl<M> ScrollableContainer<M> {
    /// Calculate content region adjusted for scroll offset (for mouse routing).
    fn scrolled_content_region(&self, content_region: Region) -> Region {
        let scroll = self.scroll.borrow();
        Region {
            x: content_region.x - scroll.offset_x,
            y: content_region.y - scroll.offset_y,
            width: scroll.virtual_width,
            height: scroll.virtual_height,
        }
    }

    /// Handle click on vertical scrollbar.
    fn handle_vertical_scrollbar_click(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let my = event.row as i32;
        let pos_in_bar = my - region.y;

        let content_size = self.content.desired_size();
        let scroll = self.scroll.borrow();
        let (thumb_start, thumb_end) = ScrollBarRender::thumb_bounds(
            region.height,
            content_size.height as f32,
            scroll.viewport_height as f32,
            scroll.offset_y as f32,
        );
        drop(scroll);

        if pos_in_bar >= thumb_start && pos_in_bar < thumb_end {
            // Start drag
            self.scrollbar_drag = Some((true, pos_in_bar - thumb_start));
            self.dirty = true;
        } else if pos_in_bar < thumb_start {
            // Click above thumb - page scroll up
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_up(amount);
            drop(scroll);
            self.dirty = true;
        } else {
            // Click below thumb - page scroll down
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_down(amount);
            drop(scroll);
            self.dirty = true;
        }
        None
    }

    /// Handle click on horizontal scrollbar.
    fn handle_horizontal_scrollbar_click(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let pos_in_bar = mx - region.x;

        let content_size = self.content.desired_size();
        let scroll = self.scroll.borrow();
        let (thumb_start, thumb_end) = ScrollBarRender::thumb_bounds(
            region.width,
            content_size.width as f32,
            scroll.viewport_width as f32,
            scroll.offset_x as f32,
        );
        drop(scroll);

        if pos_in_bar >= thumb_start && pos_in_bar < thumb_end {
            // Start drag
            self.scrollbar_drag = Some((false, pos_in_bar - thumb_start));
            self.dirty = true;
        } else if pos_in_bar < thumb_start {
            // Click left of thumb - page scroll left
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_width as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_left(amount);
            drop(scroll);
            self.dirty = true;
        } else {
            // Click right of thumb - page scroll right
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_width as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_right(amount);
            drop(scroll);
            self.dirty = true;
        }
        None
    }

    /// Handle scrollbar drag.
    fn handle_scrollbar_drag(
        &mut self,
        event: MouseEvent,
        region: Region,
        vertical: bool,
        grab_offset: i32,
    ) -> Option<M> {
        let content_size = self.content.desired_size();

        if vertical {
            let v_region = self.vertical_scrollbar_region(region);
            let my = event.row as i32;
            let pos_in_bar = my - v_region.y;
            let new_thumb_start = pos_in_bar - grab_offset;

            let track_size = v_region.height as f32;
            let virtual_size = content_size.height as f32;
            let window_size = self.scroll.borrow().viewport_height as f32;
            let thumb_size = (window_size / virtual_size) * track_size;
            let track_range = track_size - thumb_size;

            if track_range > 0.0 {
                let ratio = new_thumb_start as f32 / track_range;
                let new_position = ratio * (virtual_size - window_size);
                self.scroll.borrow_mut().scroll_to(None, Some(new_position));
                self.dirty = true;
            }
        } else {
            let h_region = self.horizontal_scrollbar_region(region);
            let mx = event.column as i32;
            let pos_in_bar = mx - h_region.x;
            let new_thumb_start = pos_in_bar - grab_offset;

            let track_size = h_region.width as f32;
            let virtual_size = content_size.width as f32;
            let window_size = self.scroll.borrow().viewport_width as f32;
            let thumb_size = (window_size / virtual_size) * track_size;
            let track_range = track_size - thumb_size;

            if track_range > 0.0 {
                let ratio = new_thumb_start as f32 / track_range;
                let new_position = ratio * (virtual_size - window_size);
                self.scroll.borrow_mut().scroll_to(Some(new_position), None);
                self.dirty = true;
            }
        }
        None
    }
}
