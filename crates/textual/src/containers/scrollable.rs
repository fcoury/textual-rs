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
use crate::layouts::{self, Viewport};
use crate::render_cache::RenderCache;
use crate::scroll::{ScrollMessage, ScrollState};
use crate::scrollbar::ScrollBarRender;
use crate::widget::Widget;
use crate::widget::scrollbar_corner::ScrollBarCorner;
use crate::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use tcss::types::{
    Layout as LayoutDirection, Overflow, ScrollbarGutter, ScrollbarStyle, ScrollbarVisibility,
    Visibility,
};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

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
    /// Widget ID for CSS targeting
    id: Option<String>,
    /// CSS classes for styling
    classes: Vec<String>,
}

struct ScrollLayout {
    content_region: Region,
    placements: Vec<layouts::WidgetPlacement>,
    virtual_width: i32,
    virtual_height: i32,
    show_vertical: bool,
    show_horizontal: bool,
}

impl<M> ScrollableContainer<M> {
    /// Create a new scrollable container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            scroll: RefCell::new(ScrollState::default()),
            style: ComputedStyle::default(),
            dirty: true,
            scrollbar_hover: None,
            scrollbar_drag: None,
            id: None,
            classes: Vec::new(),
        }
    }

    /// Create a new scrollable container with a single child.
    pub fn from_child(child: Box<dyn Widget<M>>) -> Self {
        Self::new(vec![child])
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set CSS classes (space-separated).
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.classes = classes
            .into()
            .split_whitespace()
            .map(String::from)
            .collect();
        self
    }

    fn compute_inner_region(&self, region: Region) -> Region {
        if region.width <= 0 || region.height <= 0 {
            return region;
        }

        let width = region.width as usize;
        let height = region.height as usize;
        let cache = RenderCache::new(&self.style);
        let (inner_width, inner_height) = cache.inner_size(width, height);

        let border_offset = if cache.has_border() { 1 } else { 0 };
        let padding_left = cache.padding_left() as i32;
        let padding_top = cache.padding_top() as i32;

        Region::new(
            region.x + border_offset + padding_left,
            region.y + border_offset + padding_top,
            inner_width as i32,
            inner_height as i32,
        )
    }

    /// Get the scrollbar style from computed style.
    fn scrollbar_style(&self) -> &ScrollbarStyle {
        &self.style.scrollbar
    }

    /// Check if vertical scrollbar is needed (layout), ignoring visibility.
    ///
    /// For `Overflow::Auto`, we check if content height exceeds viewport height.
    /// Returns false if viewport is not yet initialized (height == 0).
    fn show_vertical_scrollbar(&self) -> bool {
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

    /// Check if horizontal scrollbar is needed (layout), ignoring visibility.
    ///
    /// For `Overflow::Auto`, we check if content width exceeds viewport width.
    /// Returns false if viewport is not yet initialized (width == 0).
    fn show_horizontal_scrollbar(&self) -> bool {
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

    fn render_vertical_scrollbar(&self, show_vertical: bool) -> bool {
        let style = self.scrollbar_style();
        show_vertical && style.visibility == ScrollbarVisibility::Visible && style.size.vertical > 0
    }

    fn render_horizontal_scrollbar(&self, show_horizontal: bool) -> bool {
        let style = self.scrollbar_style();
        show_horizontal
            && style.visibility == ScrollbarVisibility::Visible
            && style.size.horizontal > 0
    }

    fn content_region_with_flags(
        &self,
        region: Region,
        show_vertical: bool,
        show_horizontal: bool,
    ) -> Region {
        let style = self.scrollbar_style();
        // Vertical gutter: apply when showing scrollbar OR when stable gutter with overflow-y: auto
        // (matches Python Textual behavior where scrollbar-gutter only affects vertical scrollbar)
        let v_size = if show_vertical
            || (style.gutter == ScrollbarGutter::Stable && self.style.overflow_y == Overflow::Auto)
        {
            style.size.vertical as i32
        } else {
            0
        };
        // Horizontal gutter: only when actually showing scrollbar (no stable gutter support)
        let h_size = if show_horizontal {
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
        let show_horizontal = self.show_horizontal_scrollbar();
        self.vertical_scrollbar_region_with_flags(region, show_horizontal)
    }

    fn vertical_scrollbar_region_with_flags(
        &self,
        region: Region,
        show_horizontal: bool,
    ) -> Region {
        let style = self.scrollbar_style();
        let h_size = if show_horizontal {
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
        let show_vertical = self.show_vertical_scrollbar();
        self.horizontal_scrollbar_region_with_flags(region, show_vertical)
    }

    fn horizontal_scrollbar_region_with_flags(
        &self,
        region: Region,
        show_vertical: bool,
    ) -> Region {
        let style = self.scrollbar_style();
        let v_size = if show_vertical {
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

    fn compute_child_placements(
        &self,
        region: Region,
        viewport: Viewport,
    ) -> Vec<layouts::WidgetPlacement> {
        let children_with_styles: Vec<layouts::LayoutChild> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.participates_in_layout())
            .map(|(i, c)| layouts::LayoutChild {
                index: i,
                style: c.get_style(),
                desired_size: c.desired_size(),
                node: c,
            })
            .collect();

        layouts::arrange_children_with_layers(&self.style, &children_with_styles, region, viewport)
    }

    fn compute_virtual_size(
        &self,
        placements: &[layouts::WidgetPlacement],
        base_region: Region,
    ) -> (i32, i32) {
        let mut virtual_width = 0;
        let mut virtual_height = 0;

        for placement in placements {
            let child_style = self
                .children
                .get(placement.child_index)
                .map(|child| child.get_style());
            let margin_right = child_style
                .as_ref()
                .map(|style| style.margin.right.value as i32)
                .unwrap_or(0);
            let margin_bottom = child_style
                .as_ref()
                .map(|style| style.margin.bottom.value as i32)
                .unwrap_or(0);
            let width =
                (placement.region.x - base_region.x) + placement.region.width + margin_right;
            let height =
                (placement.region.y - base_region.y) + placement.region.height + margin_bottom;
            if width > virtual_width {
                virtual_width = width;
            }
            if height > virtual_height {
                virtual_height = height;
            }
        }

        (virtual_width, virtual_height)
    }

    fn calculate_intrinsic_size(&self) -> Size {
        if self.children.is_empty() {
            let border_size = if self.style.border.is_none() { 0 } else { 2 };
            let padding_h =
                self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
            let padding_v =
                self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
            return Size::new(border_size + padding_h, border_size + padding_v);
        }

        let mut total_width: u16 = 0;
        let mut total_height: u16 = 0;
        let mut max_width: u16 = 0;
        let mut max_height: u16 = 0;
        let mut any_child_wants_fill_width = false;
        let mut any_child_wants_fill_height = false;

        let mut prev_margin_bottom: u16 = 0;
        let mut first_child_margin_top: u16 = 0;
        let mut last_child_margin_bottom: u16 = 0;
        let mut is_first_child = true;

        let mut prev_margin_right: u16 = 0;
        let mut first_child_margin_left: u16 = 0;
        let mut last_child_margin_right: u16 = 0;
        let mut is_first_h_child = true;

        for child in &self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let child_size = child.desired_size();
            let child_style = child.get_style();

            if child_size.width == u16::MAX {
                any_child_wants_fill_width = true;
            }
            if child_size.height == u16::MAX {
                any_child_wants_fill_height = true;
            }

            let capped_width = if child_size.width == u16::MAX {
                1000
            } else {
                child_size.width
            };
            let capped_height = if child_size.height == u16::MAX {
                1000
            } else {
                child_size.height
            };

            let margin_left = child_style.margin.left.value as u16;
            let margin_right = child_style.margin.right.value as u16;
            let margin_top = child_style.margin.top.value as u16;
            let margin_bottom = child_style.margin.bottom.value as u16;

            let child_width_with_margins = capped_width + margin_left + margin_right;
            let child_height_with_margins = capped_height + margin_top + margin_bottom;

            match self.style.layout {
                LayoutDirection::Vertical => {
                    if is_first_child {
                        first_child_margin_top = margin_top;
                        is_first_child = false;
                    }
                    last_child_margin_bottom = margin_bottom;

                    let margin_overlap = prev_margin_bottom.max(margin_top);
                    let effective_margin = margin_overlap;
                    total_height += capped_height + effective_margin;
                    prev_margin_bottom = margin_bottom;

                    max_width = max_width.max(child_width_with_margins);
                }
                LayoutDirection::Horizontal => {
                    if is_first_h_child {
                        first_child_margin_left = margin_left;
                        is_first_h_child = false;
                    }
                    last_child_margin_right = margin_right;

                    let margin_overlap = prev_margin_right.max(margin_left);
                    let effective_margin = margin_overlap;
                    total_width += capped_width + effective_margin;
                    prev_margin_right = margin_right;

                    max_height = max_height.max(child_height_with_margins);
                }
                LayoutDirection::Grid => {
                    max_width = max_width.max(child_width_with_margins);
                    max_height = max_height.max(child_height_with_margins);
                }
            }
        }

        match self.style.layout {
            LayoutDirection::Vertical => {
                total_height += last_child_margin_bottom;
                total_width = max_width;
                if any_child_wants_fill_width {
                    total_width = u16::MAX;
                }
                if any_child_wants_fill_height {
                    total_height = u16::MAX;
                }
                let border_size = if self.style.border.is_none() { 0 } else { 2 };
                let padding_h =
                    self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
                let padding_v =
                    self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
                Size::new(
                    total_width.saturating_add(border_size + padding_h),
                    total_height.saturating_add(border_size + padding_v + first_child_margin_top),
                )
            }
            LayoutDirection::Horizontal => {
                total_width += last_child_margin_right;
                total_height = max_height;
                if any_child_wants_fill_width {
                    total_width = u16::MAX;
                }
                if any_child_wants_fill_height {
                    total_height = u16::MAX;
                }
                let border_size = if self.style.border.is_none() { 0 } else { 2 };
                let padding_h =
                    self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
                let padding_v =
                    self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
                Size::new(
                    total_width.saturating_add(border_size + padding_h + first_child_margin_left),
                    total_height.saturating_add(border_size + padding_v),
                )
            }
            LayoutDirection::Grid => {
                let border_size = if self.style.border.is_none() { 0 } else { 2 };
                let padding_h =
                    self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
                let padding_v =
                    self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
                Size::new(
                    max_width.saturating_add(border_size + padding_h),
                    max_height.saturating_add(border_size + padding_v),
                )
            }
        }
    }

    fn compute_scroll_layout(&self, inner_region: Region, viewport: Viewport) -> ScrollLayout {
        let mut show_vertical = false;
        let mut show_horizontal = false;
        let mut content_region = inner_region;
        let mut placements = Vec::new();
        let mut virtual_width = 0;
        let mut virtual_height = 0;

        for _ in 0..3 {
            content_region =
                self.content_region_with_flags(inner_region, show_vertical, show_horizontal);
            placements = self.compute_child_placements(content_region, viewport);
            let (next_virtual_width, next_virtual_height) =
                self.compute_virtual_size(&placements, content_region);
            virtual_width = next_virtual_width;
            virtual_height = next_virtual_height;

            let needs_horizontal = match self.style.overflow_x {
                Overflow::Scroll => true,
                Overflow::Auto => virtual_width > content_region.width,
                Overflow::Hidden => false,
            };
            let needs_vertical = match self.style.overflow_y {
                Overflow::Scroll => true,
                Overflow::Auto => virtual_height > content_region.height,
                Overflow::Hidden => false,
            };

            let next_show_horizontal = needs_horizontal;
            let next_show_vertical = needs_vertical;

            if next_show_horizontal == show_horizontal && next_show_vertical == show_vertical {
                break;
            }

            show_horizontal = next_show_horizontal;
            show_vertical = next_show_vertical;
        }

        {
            let mut scroll = self.scroll.borrow_mut();
            scroll.set_virtual_size(virtual_width, virtual_height);
            scroll.set_viewport(content_region.width, content_region.height);
        }

        ScrollLayout {
            content_region,
            placements,
            virtual_width,
            virtual_height,
            show_vertical,
            show_horizontal,
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
    fn default_css(&self) -> &'static str {
        // Match Python Textual's ScrollableContainer DEFAULT_CSS
        r#"
ScrollableContainer {
    width: 1fr;
    height: 1fr;
    layout: vertical;
    overflow: auto auto;
}
"#
    }

    fn desired_size(&self) -> Size {
        use tcss::types::Unit;

        let intrinsic_size = self.calculate_intrinsic_size();

        let width = if let Some(w) = &self.style.width {
            match w.unit {
                Unit::Cells => w.value as u16,
                Unit::Percent
                | Unit::ViewWidth
                | Unit::ViewHeight
                | Unit::Fraction
                | Unit::Width
                | Unit::Height => u16::MAX,
                Unit::Auto => intrinsic_size.width,
            }
        } else {
            intrinsic_size.width
        };

        let height = if let Some(h) = &self.style.height {
            match h.unit {
                Unit::Cells => h.value as u16,
                Unit::Percent
                | Unit::ViewWidth
                | Unit::ViewHeight
                | Unit::Fraction
                | Unit::Width
                | Unit::Height => u16::MAX,
                Unit::Auto => intrinsic_size.height,
            }
        } else {
            intrinsic_size.height
        };

        Size::new(width, height)
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // 1. Render background/border and get inner region
        let inner_region = crate::containers::render_container_chrome(canvas, region, &self.style);

        let viewport = canvas.viewport();
        let layout = self.compute_scroll_layout(inner_region, viewport);
        let content_region = layout.content_region;

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
            "  scroll offset: ({}, {}), virtual_size: ({}, {})",
            scroll.offset_x,
            scroll.offset_y,
            layout.virtual_width,
            layout.virtual_height
        );
        log::trace!(
            "  show_vertical: {}, show_horizontal: {}, style.scrollbar.size: ({}, {})",
            layout.show_vertical,
            layout.show_horizontal,
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

        for placement in &layout.placements {
            let child = &self.children[placement.child_index];
            if child.get_style().visibility == Visibility::Hidden {
                continue;
            }

            let scrolled_region = Region {
                x: placement.region.x - offset_x,
                y: placement.region.y - offset_y,
                width: placement.region.width,
                height: placement.region.height,
            };

            let visible_h = scrolled_region.x + scrolled_region.width > content_region.x
                && scrolled_region.x < content_region.x + content_region.width;
            let visible_v = scrolled_region.y + scrolled_region.height > content_region.y
                && scrolled_region.y < content_region.y + content_region.height;

            if visible_h && visible_v {
                child.render(canvas, scrolled_region);
            }
        }
        canvas.pop_clip();

        let render_vertical = self.render_vertical_scrollbar(layout.show_vertical);
        let render_horizontal = self.render_horizontal_scrollbar(layout.show_horizontal);

        // Render vertical scrollbar ON TOP of chrome
        if render_vertical {
            let v_region =
                self.vertical_scrollbar_region_with_flags(inner_region, layout.show_horizontal);
            let (thumb_color, track_color) = self.vertical_colors();
            let (thumb_color, track_color, draw_thumb) = ScrollBarRender::compose_colors(
                thumb_color,
                track_color,
                self.style.inherited_background.clone(),
            );

            ScrollBarRender::render_vertical(
                canvas,
                v_region,
                layout.virtual_height as f32,
                content_region.height as f32,
                offset_y as f32,
                thumb_color,
                track_color,
                draw_thumb,
            );
        }

        // Render horizontal scrollbar ON TOP of chrome
        if render_horizontal {
            let h_region =
                self.horizontal_scrollbar_region_with_flags(inner_region, layout.show_vertical);
            let (thumb_color, track_color) = self.horizontal_colors();
            let (thumb_color, track_color, draw_thumb) = ScrollBarRender::compose_colors(
                thumb_color,
                track_color,
                self.style.inherited_background.clone(),
            );

            ScrollBarRender::render_horizontal(
                canvas,
                h_region,
                layout.virtual_width as f32,
                content_region.width as f32,
                offset_x as f32,
                thumb_color,
                track_color,
                draw_thumb,
            );
        }

        // Render corner if both scrollbars visible
        if render_vertical && render_horizontal {
            let corner_region = self.corner_region(inner_region);
            let style = self.scrollbar_style();
            let mut corner = ScrollBarCorner::new(style.size.vertical, style.size.horizontal);
            <ScrollBarCorner as Widget<M>>::set_style(&mut corner, self.style.clone());
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
        self.dirty || self.children.iter().any(|c| c.is_dirty())
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
        for child in &mut self.children {
            child.mark_clean();
        }
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "ScrollableContainer",
            id: self.id.clone(),
            classes: self.classes.clone(),
            states: WidgetStates::empty(),
        }
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn add_class(&mut self, class: &str) {
        if !self.classes.iter().any(|c| c == class) {
            self.classes.push(class.to_string());
            self.dirty = true;
        }
    }

    fn remove_class(&mut self, class: &str) {
        if let Some(pos) = self.classes.iter().position(|c| c == class) {
            self.classes.remove(pos);
            self.dirty = true;
        }
    }

    fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c == class)
    }

    fn set_classes(&mut self, classes: &str) {
        self.classes = classes.split_whitespace().map(String::from).collect();
        self.dirty = true;
    }

    fn classes(&self) -> Vec<String> {
        self.classes.clone()
    }

    fn on_resize(&mut self, size: Size) {
        for child in &mut self.children {
            child.on_resize(size);
        }
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
                let x = if self.allow_horizontal_scroll() {
                    Some(0.0)
                } else {
                    None
                };
                let y = if self.allow_vertical_scroll() {
                    Some(0.0)
                } else {
                    None
                };
                self.scroll.borrow_mut().scroll_to(x, y);
                self.dirty = true;
                return None;
            }
            KeyCode::End if self.allow_vertical_scroll() || self.allow_horizontal_scroll() => {
                let mut scroll = self.scroll.borrow_mut();
                let x = if self.allow_horizontal_scroll() {
                    Some(scroll.max_scroll_x() as f32)
                } else {
                    None
                };
                let y = if self.allow_vertical_scroll() {
                    Some(scroll.max_scroll_y() as f32)
                } else {
                    None
                };
                scroll.scroll_to(x, y);
                drop(scroll);
                self.dirty = true;
                return None;
            }
            _ => {}
        }

        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        let inner_region = self.compute_inner_region(region);
        let viewport = layouts::Viewport::from(region);
        let layout = self.compute_scroll_layout(inner_region, viewport);
        let content_region = layout.content_region;

        // Check scrollbar regions first
        let v_region =
            self.vertical_scrollbar_region_with_flags(inner_region, layout.show_horizontal);
        let h_region =
            self.horizontal_scrollbar_region_with_flags(inner_region, layout.show_vertical);

        let render_vertical = self.render_vertical_scrollbar(layout.show_vertical);
        let render_horizontal = self.render_horizontal_scrollbar(layout.show_horizontal);
        let on_vertical = render_vertical && v_region.contains_point(mx, my);
        let on_horizontal = render_horizontal && h_region.contains_point(mx, my);

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
                layout.show_horizontal,
                h_region.contains_point(mx, my),
                on_horizontal
            );
        }

        match event.kind {
            MouseEventKind::Moved => {
                // Handle drag
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    return self.handle_scrollbar_drag(event, inner_region, vertical, grab_offset);
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
                    let (offset_x, offset_y) = {
                        let scroll = self.scroll.borrow();
                        (scroll.offset_x, scroll.offset_y)
                    };
                    for placement in &layout.placements {
                        let scrolled = Region {
                            x: placement.region.x - offset_x,
                            y: placement.region.y - offset_y,
                            width: placement.region.width,
                            height: placement.region.height,
                        };
                        if scrolled.contains_point(mx, my) {
                            return self.children[placement.child_index].on_mouse(event, scrolled);
                        }
                    }
                }
                None
            }

            MouseEventKind::Down(_) => {
                if on_vertical {
                    return self.handle_vertical_scrollbar_click(event, v_region);
                } else if on_horizontal {
                    return self.handle_horizontal_scrollbar_click(event, h_region);
                } else if content_region.contains_point(mx, my) {
                    let (offset_x, offset_y) = {
                        let scroll = self.scroll.borrow();
                        (scroll.offset_x, scroll.offset_y)
                    };
                    for placement in &layout.placements {
                        let scrolled = Region {
                            x: placement.region.x - offset_x,
                            y: placement.region.y - offset_y,
                            width: placement.region.width,
                            height: placement.region.height,
                        };
                        if scrolled.contains_point(mx, my) {
                            return self.children[placement.child_index].on_mouse(event, scrolled);
                        }
                    }
                }
                None
            }

            MouseEventKind::Drag(_) => {
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    return self.handle_scrollbar_drag(event, inner_region, vertical, grab_offset);
                }
                // Pass to content
                if content_region.contains_point(mx, my) {
                    let (offset_x, offset_y) = {
                        let scroll = self.scroll.borrow();
                        (scroll.offset_x, scroll.offset_y)
                    };
                    for placement in &layout.placements {
                        let scrolled = Region {
                            x: placement.region.x - offset_x,
                            y: placement.region.y - offset_y,
                            width: placement.region.width,
                            height: placement.region.height,
                        };
                        if scrolled.contains_point(mx, my) {
                            return self.children[placement.child_index].on_mouse(event, scrolled);
                        }
                    }
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
                    let (offset_x, offset_y) = {
                        let scroll = self.scroll.borrow();
                        (scroll.offset_x, scroll.offset_y)
                    };
                    for placement in &layout.placements {
                        let scrolled = Region {
                            x: placement.region.x - offset_x,
                            y: placement.region.y - offset_y,
                            width: placement.region.width,
                            height: placement.region.height,
                        };
                        if scrolled.contains_point(mx, my) {
                            return self.children[placement.child_index].on_mouse(event, scrolled);
                        }
                    }
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
        self.children
            .iter()
            .filter(|c| c.participates_in_layout())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_focus();
            }
        }
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        let mut remaining = n;
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let count = child.count_focusable();
            if remaining < count {
                return child.focus_nth(remaining);
            }
            remaining -= count;
        }
        false
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
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_hover();
            }
        }
    }
}

impl<M> ScrollableContainer<M> {
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
