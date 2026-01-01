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
use crate::widget::Widget;
use crate::widget::scrollbar_corner::ScrollBarCorner;
use crate::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use tcss::ComputedStyle;
use tcss::types::{Overflow, ScrollbarGutter, ScrollbarStyle, ScrollbarVisibility};

/// Scroll amount for single scroll events (arrow keys).
/// Matches Python Textual's behavior of scrolling 1 line per key press.
const SCROLL_AMOUNT: i32 = 1;

/// Page scroll amount multiplier (relative to viewport).
const PAGE_SCROLL_RATIO: f32 = 0.9;

/// A scrollable container that wraps content and provides scrollbars.
///
/// The container manages scroll state and renders scrollbars when content
/// exceeds the viewport. It handles mouse wheel, keyboard navigation,
/// and scrollbar interactions.
pub struct ScrollableContainer<M> {
    /// The content widget(s) to scroll
    children: Vec<Box<dyn Widget<M>>>,
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
    /// Create a new scrollable container with the given children.
    ///
    /// # Panics
    /// Panics if `children` does not contain exactly one child.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        assert!(
            children.len() == 1,
            "ScrollableContainer requires exactly 1 child, got {}",
            children.len()
        );
        Self {
            children,
            scroll: RefCell::new(ScrollState::default()),
            style: ComputedStyle::default(),
            dirty: true,
            scrollbar_hover: None,
            scrollbar_drag: None,
        }
    }

    /// Create a new scrollable container with a single child.
    pub fn from_child(child: Box<dyn Widget<M>>) -> Self {
        Self::new(vec![child])
    }

    fn content(&self) -> &dyn Widget<M> {
        self.children[0].as_ref()
    }

    fn content_mut(&mut self) -> &mut dyn Widget<M> {
        self.children[0].as_mut()
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
                // Use scroll state (virtual vs viewport) to handle flexible sizes (fr/%/vw/vh).
                let scroll = self.scroll.borrow();
                if scroll.viewport_height <= 0 {
                    return false;
                }
                scroll.virtual_height > scroll.viewport_height
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
                // Use scroll state (virtual vs viewport) to handle flexible sizes (fr/%/vw/vh).
                let scroll = self.scroll.borrow();
                if scroll.viewport_width <= 0 {
                    return false;
                }
                scroll.virtual_width > scroll.viewport_width
            }
            Overflow::Hidden => false,
        }
    }

    /// Check if vertical scrolling is allowed (not hidden).
    fn allow_vertical_scroll(&self) -> bool {
        self.style.overflow_y != Overflow::Hidden
    }

    /// Check if horizontal scrolling is allowed (not hidden).
    fn allow_horizontal_scroll(&self) -> bool {
        self.style.overflow_x != Overflow::Hidden
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
        let content_size = self.content().desired_size();

        // Calculate effective dimensions (same logic as render)
        let effective_width = if content_size.width == u16::MAX {
            self.content()
                .content_width_for_scroll(content_region.width as u16) as i32
        } else {
            content_size.width as i32
        };

        let effective_height = if content_size.height == u16::MAX {
            self.content()
                .content_height_for_scroll(content_region.width as u16, content_region.height as u16)
                as i32
        } else {
            content_size.height as i32
        };

        let mut scroll = self.scroll.borrow_mut();
        scroll.set_virtual_size(effective_width, effective_height);
        scroll.set_viewport(content_region.width, content_region.height);
    }

    /// Get colors for vertical scrollbar based on hover/drag state.
    fn vertical_colors(&self) -> (tcss::types::RgbaColor, tcss::types::RgbaColor) {
        let style = self.scrollbar_style();
        if self.scrollbar_drag.map(|(v, _)| v).unwrap_or(false) {
            (
                style.effective_color_active(),
                style.effective_background_active(),
            )
        } else if self.scrollbar_hover == Some(true) {
            (
                style.effective_color_hover(),
                style.effective_background_hover(),
            )
        } else {
            (style.effective_color(), style.effective_background())
        }
    }

    /// Get colors for horizontal scrollbar based on hover/drag state.
    fn horizontal_colors(&self) -> (tcss::types::RgbaColor, tcss::types::RgbaColor) {
        let style = self.scrollbar_style();
        if self.scrollbar_drag.map(|(v, _)| !v).unwrap_or(false) {
            (
                style.effective_color_active(),
                style.effective_background_active(),
            )
        } else if self.scrollbar_hover == Some(false) {
            (
                style.effective_color_hover(),
                style.effective_background_hover(),
            )
        } else {
            (style.effective_color(), style.effective_background())
        }
    }
}

impl<M> Widget<M> for ScrollableContainer<M> {
    fn desired_size(&self) -> Size {
        // ScrollableContainer fills available space
        // Return content size as hint, but container should expand
        self.content().desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // 1. Render background/border and get inner region
        let inner_region = crate::containers::render_container_chrome(canvas, region, &self.style);

        // Update scroll dimensions FIRST so show_*_scrollbar() has correct viewport info
        // This fixes keyboard-only scrolling and overflow:auto decisions on first render
        let content_size = self.content().desired_size();

        // Handle u16::MAX (signal for "fill available space") - use actual content size
        // When widgets have flexible sizes (fr, %, etc), desired_size returns u16::MAX
        // which would incorrectly allow scrolling. Instead, use the widget's intrinsic size.
        //
        // We need to handle the chicken-and-egg problem:
        // - Content width determines if we need horizontal scrollbar
        // - Horizontal scrollbar reduces available height for content
        // - So we calculate width first, then height with scrollbar accounted for

        let style = self.scrollbar_style();

        // Determine scrollbar sizes with a small fixed-point iteration
        let mut h_scrollbar_size: i32 = 0;
        let mut v_scrollbar_size: i32 = 0;
        for _ in 0..2 {
            let available_width = (inner_region.width - v_scrollbar_size).max(0);
            let available_height = (inner_region.height - h_scrollbar_size).max(0);

            // Effective content width
            let effective_width = if content_size.width == u16::MAX {
                self.content()
                    .content_width_for_scroll(available_width as u16) as i32
            } else {
                content_size.width as i32
            };

            let needs_h_scrollbar = match self.style.overflow_x {
                Overflow::Scroll => true,
                Overflow::Auto => effective_width > available_width,
                Overflow::Hidden => false,
            };
            let next_h_scrollbar_size = if needs_h_scrollbar {
                style.size.horizontal as i32
            } else {
                0
            };

            // Effective content height (uses available width for wrapping)
            let available_width_u16 =
                available_width.clamp(0, u16::MAX as i32) as u16;
            let should_measure_height =
                content_size.height == u16::MAX || content_size.width > available_width_u16;
            let effective_height = if should_measure_height {
                self.content()
                    .content_height_for_scroll(available_width_u16, available_height as u16)
                    as i32
            } else {
                content_size.height as i32
            };

            let needs_v_scrollbar = match self.style.overflow_y {
                Overflow::Scroll => true,
                Overflow::Auto => effective_height > available_height as i32,
                Overflow::Hidden => false,
            };
            let next_v_scrollbar_size = if needs_v_scrollbar {
                style.size.vertical as i32
            } else {
                0
            };

            if next_h_scrollbar_size == h_scrollbar_size
                && next_v_scrollbar_size == v_scrollbar_size
            {
                break;
            }
            h_scrollbar_size = next_h_scrollbar_size;
            v_scrollbar_size = next_v_scrollbar_size;
        }

        {
            let mut scroll = self.scroll.borrow_mut();
            let available_width = (inner_region.width - v_scrollbar_size).max(0);
            let available_height = (inner_region.height - h_scrollbar_size).max(0);
            let effective_width = if content_size.width == u16::MAX {
                self.content()
                    .content_width_for_scroll(available_width as u16) as i32
            } else {
                content_size.width as i32
            };
            let available_width_u16 =
                available_width.clamp(0, u16::MAX as i32) as u16;
            let should_measure_height =
                content_size.height == u16::MAX || content_size.width > available_width_u16;
            let effective_height = if should_measure_height {
                self.content()
                    .content_height_for_scroll(available_width_u16, available_height as u16)
                    as i32
            } else {
                content_size.height as i32
            };
            scroll.set_virtual_size(effective_width, effective_height);
            scroll.set_viewport(
                available_width,
                available_height,
            );
        }

        let content_region = self.content_region(inner_region);

        // Verbose render diagnostics (use RUST_LOG=trace to enable)
        let scroll = self.scroll.borrow();
        log::trace!(
            "ScrollableContainer::render - inner_region: ({}, {}, {}, {}), content_region: ({}, {}, {}, {})",
            inner_region.x,
            inner_region.y,
            inner_region.width,
            inner_region.height,
            content_region.x,
            content_region.y,
            content_region.width,
            content_region.height
        );
        log::trace!(
            "  scroll offset: ({}, {}), content_size: ({}, {})",
            scroll.offset_x,
            scroll.offset_y,
            content_size.width,
            content_size.height
        );
        log::trace!(
            "  show_vertical: {}, show_horizontal: {}, style.scrollbar.size: ({}, {})",
            self.show_vertical_scrollbar(),
            self.show_horizontal_scrollbar(),
            self.style.scrollbar.size.horizontal,
            self.style.scrollbar.size.vertical
        );
        log::trace!(
            "  overflow_x: {:?}, overflow_y: {:?}",
            self.style.overflow_x,
            self.style.overflow_y
        );
        let (offset_x, offset_y) = (scroll.offset_x, scroll.offset_y);
        drop(scroll); // Release borrow before calling content_region again

        // Render content with clipping and scroll offset
        canvas.push_clip(content_region);

        // Calculate content position with scroll offset
        // Use the full virtual content size for layout so all children are positioned correctly.
        // The clipping will show only the visible portion.
        // NOTE: We use effective_height/effective_width calculated earlier for scroll purposes.
        let scroll_ref = self.scroll.borrow();
        let virtual_height = scroll_ref.virtual_height;
        let virtual_width = scroll_ref.virtual_width;
        drop(scroll_ref);

        // For layout, use viewport width for percentage calculations (e.g., width: 50% means 50% of viewport).
        // Individual elements with min-width can extend beyond this, but percentages should be relative to viewport.
        // Use virtual_height for height since vertical scrolling uses the full content height.
        let content_render_region = Region {
            x: content_region.x - offset_x,
            y: content_region.y - offset_y,
            width: content_region.width, // Use viewport width for percentage-based layouts
            height: virtual_height.max(content_region.height), // Full content height for vertical layout
        };

        log::trace!(
            "  content_render_region: ({}, {}, {}, {})",
            content_render_region.x,
            content_render_region.y,
            content_render_region.width,
            content_render_region.height
        );

        self.content().render(canvas, content_render_region);
        canvas.pop_clip();

        // Render vertical scrollbar ON TOP of chrome
        if self.show_vertical_scrollbar() {
            let v_region = self.vertical_scrollbar_region(inner_region);
            let (thumb_color, track_color) = self.vertical_colors();

            // Use virtual_height (which equals effective_height) for correct scrollbar rendering
            ScrollBarRender::render_vertical(
                canvas,
                v_region,
                virtual_height as f32,
                content_region.height as f32,
                offset_y as f32,
                thumb_color,
                track_color,
            );
        }

        // Render horizontal scrollbar ON TOP of chrome
        if self.show_horizontal_scrollbar() {
            let h_region = self.horizontal_scrollbar_region(inner_region);
            let (thumb_color, track_color) = self.horizontal_colors();

            // Use virtual_width (which equals effective_width) for correct scrollbar rendering
            ScrollBarRender::render_horizontal(
                canvas,
                h_region,
                virtual_width as f32,
                content_region.width as f32,
                offset_x as f32,
                thumb_color,
                track_color,
            );
        }

        // Render corner if both scrollbars visible
        if self.show_vertical_scrollbar() && self.show_horizontal_scrollbar() {
            let corner_region = self.corner_region(inner_region);
            let style = self.scrollbar_style();
            let corner = ScrollBarCorner::new(style.size.vertical, style.size.horizontal);
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
        self.dirty || self.content().is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
        self.content_mut().mark_clean();
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn on_resize(&mut self, size: Size) {
        self.content_mut().on_resize(size);
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Handle scroll keys (only if scrolling is allowed for that direction)
        match key {
            KeyCode::Up if self.allow_vertical_scroll() => {
                self.handle_scroll(ScrollMessage::ScrollUp);
                return None;
            }
            KeyCode::Down if self.allow_vertical_scroll() => {
                self.handle_scroll(ScrollMessage::ScrollDown);
                return None;
            }
            KeyCode::Left if self.allow_horizontal_scroll() => {
                self.handle_scroll(ScrollMessage::ScrollLeft);
                return None;
            }
            KeyCode::Right if self.allow_horizontal_scroll() => {
                self.handle_scroll(ScrollMessage::ScrollRight);
                return None;
            }
            KeyCode::PageUp if self.allow_vertical_scroll() => {
                let mut scroll = self.scroll.borrow_mut();
                let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
                scroll.scroll_up(amount);
                drop(scroll);
                self.dirty = true;
                return None;
            }
            KeyCode::PageDown if self.allow_vertical_scroll() => {
                let mut scroll = self.scroll.borrow_mut();
                let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
                scroll.scroll_down(amount);
                drop(scroll);
                self.dirty = true;
                return None;
            }
            KeyCode::Home if self.allow_vertical_scroll() || self.allow_horizontal_scroll() => {
                let x = if self.allow_horizontal_scroll() { Some(0.0) } else { None };
                let y = if self.allow_vertical_scroll() { Some(0.0) } else { None };
                self.scroll.borrow_mut().scroll_to(x, y);
                self.dirty = true;
                return None;
            }
            KeyCode::End if self.allow_vertical_scroll() || self.allow_horizontal_scroll() => {
                let mut scroll = self.scroll.borrow_mut();
                let x = if self.allow_horizontal_scroll() { Some(scroll.max_scroll_x() as f32) } else { None };
                let y = if self.allow_vertical_scroll() { Some(scroll.max_scroll_y() as f32) } else { None };
                scroll.scroll_to(x, y);
                drop(scroll);
                self.dirty = true;
                return None;
            }
            _ => {}
        }

        // Pass other keys to content
        self.content_mut().on_event(key)
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

        // Debug mouse events
        if matches!(event.kind, MouseEventKind::Down(_)) {
            log::info!("MOUSE DOWN: mx={}, my={}", mx, my);
            log::info!(
                "  h_region: x={}, y={}, w={}, h={}",
                h_region.x,
                h_region.y,
                h_region.width,
                h_region.height
            );
            log::info!(
                "  show_horizontal={}, contains_point={}, on_horizontal={}",
                self.show_horizontal_scrollbar(),
                h_region.contains_point(mx, my),
                on_horizontal
            );
        }

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
                    let scrolled = self.scrolled_content_region(content_region);
                    return self.content_mut().on_mouse(event, scrolled);
                }
                None
            }

            MouseEventKind::Down(_) => {
                if on_vertical {
                    return self.handle_vertical_scrollbar_click(event, v_region);
                } else if on_horizontal {
                    return self.handle_horizontal_scrollbar_click(event, h_region);
                } else if content_region.contains_point(mx, my) {
                    let scrolled = self.scrolled_content_region(content_region);
                    return self.content_mut().on_mouse(event, scrolled);
                }
                None
            }

            MouseEventKind::Drag(_) => {
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    return self.handle_scrollbar_drag(event, region, vertical, grab_offset);
                }
                // Pass to content
                if content_region.contains_point(mx, my) {
                    let scrolled = self.scrolled_content_region(content_region);
                    return self.content_mut().on_mouse(event, scrolled);
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
                    let scrolled = self.scrolled_content_region(content_region);
                    return self.content_mut().on_mouse(event, scrolled);
                }
                None
            }

            MouseEventKind::ScrollDown => {
                // Shift or Ctrl + scroll converts vertical scroll to horizontal
                if event.modifiers.contains(KeyModifiers::SHIFT)
                    || event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if self.allow_horizontal_scroll() {
                        self.handle_scroll(ScrollMessage::ScrollRight);
                    }
                } else if self.allow_vertical_scroll() {
                    self.handle_scroll(ScrollMessage::ScrollDown);
                }
                None
            }

            MouseEventKind::ScrollUp => {
                // Shift or Ctrl + scroll converts vertical scroll to horizontal
                if event.modifiers.contains(KeyModifiers::SHIFT)
                    || event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if self.allow_horizontal_scroll() {
                        self.handle_scroll(ScrollMessage::ScrollLeft);
                    }
                } else if self.allow_vertical_scroll() {
                    self.handle_scroll(ScrollMessage::ScrollUp);
                }
                None
            }

            MouseEventKind::ScrollLeft => {
                if self.allow_horizontal_scroll() {
                    self.handle_scroll(ScrollMessage::ScrollLeft);
                }
                None
            }

            MouseEventKind::ScrollRight => {
                if self.allow_horizontal_scroll() {
                    self.handle_scroll(ScrollMessage::ScrollRight);
                }
                None
            }
        }
    }

    fn count_focusable(&self) -> usize {
        self.content().count_focusable()
    }

    fn clear_focus(&mut self) {
        self.content_mut().clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.content_mut().focus_nth(n)
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        self.children
            .get_mut(index)
            .map(|c| c.as_mut() as &mut dyn Widget<M>)
    }

    fn clear_hover(&mut self) {
        self.scrollbar_hover = None;
        self.content_mut().clear_hover();
    }
}

impl<M> ScrollableContainer<M> {
    /// Calculate content region adjusted for scroll offset (for mouse routing).
    /// Must match the region used in render() for proper hit detection.
    fn scrolled_content_region(&self, content_region: Region) -> Region {
        let scroll = self.scroll.borrow();
        Region {
            x: content_region.x - scroll.offset_x,
            y: content_region.y - scroll.offset_y,
            // Use viewport dimensions to match render region
            width: content_region.width,
            height: content_region.height,
        }
    }

    /// Handle click on vertical scrollbar.
    fn handle_vertical_scrollbar_click(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let my = event.row as i32;
        let pos_in_bar = my - region.y;

        // Use virtual_height from scroll state (set during render from effective_height)
        let scroll = self.scroll.borrow();
        let (thumb_start, thumb_end) = ScrollBarRender::thumb_bounds(
            region.height,
            scroll.virtual_height as f32,
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
    fn handle_horizontal_scrollbar_click(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<M> {
        let mx = event.column as i32;
        let pos_in_bar = mx - region.x;

        // Use virtual_width from scroll state (set during render from effective_width)
        let scroll = self.scroll.borrow();
        let (thumb_start, thumb_end) = ScrollBarRender::thumb_bounds(
            region.width,
            scroll.virtual_width as f32,
            scroll.viewport_width as f32,
            scroll.offset_x as f32,
        );
        let offset_x = scroll.offset_x;
        let virtual_width = scroll.virtual_width;
        let viewport_width = scroll.viewport_width;
        drop(scroll);

        log::info!(
            "H-SCROLLBAR CLICK: mx={}, region.x={}, pos_in_bar={}",
            mx,
            region.x,
            pos_in_bar
        );
        log::info!("  thumb_bounds: start={}, end={}", thumb_start, thumb_end);
        log::info!(
            "  scroll state: offset_x={}, virtual_width={}, viewport_width={}",
            offset_x,
            virtual_width,
            viewport_width
        );

        if pos_in_bar >= thumb_start && pos_in_bar < thumb_end {
            // Start drag
            log::info!("  -> Starting DRAG");
            self.scrollbar_drag = Some((false, pos_in_bar - thumb_start));
            self.dirty = true;
        } else if pos_in_bar < thumb_start {
            // Click left of thumb - page scroll left
            log::info!("  -> Scroll LEFT");
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_width as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_left(amount);
            drop(scroll);
            self.dirty = true;
        } else {
            // Click right of thumb - page scroll right
            log::info!("  -> Scroll RIGHT");
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
        // Use virtual sizes from scroll state (set during render from effective heights/widths)
        if vertical {
            let v_region = self.vertical_scrollbar_region(region);
            let my = event.row as i32;
            let pos_in_bar = my - v_region.y;
            let new_thumb_start = pos_in_bar - grab_offset;

            let track_size = v_region.height as f32;
            let scroll = self.scroll.borrow();
            let virtual_size = scroll.virtual_height as f32;
            let window_size = scroll.viewport_height as f32;
            drop(scroll);
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
            let scroll = self.scroll.borrow();
            let virtual_size = scroll.virtual_width as f32;
            let window_size = scroll.viewport_width as f32;
            drop(scroll);
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
