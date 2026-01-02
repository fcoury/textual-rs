//! Screen - The root container for a widget tree.
//!
//! Screen is an implicit wrapper that provides:
//! - A CSS-targetable root container (type name "Screen")
//! - CSS-driven layout dispatch (grid, vertical, horizontal)
//! - Responsive breakpoint classes based on terminal size
//! - Resize event propagation to children
//!
//! ## Custom Breakpoints
//!
//! Apps can define custom breakpoints by implementing `horizontal_breakpoints`
//! and `vertical_breakpoints` on the App trait. Breakpoints are (threshold, class_name)
//! pairs where the class is applied when the dimension >= threshold.
//!
//! The last matching breakpoint wins (iterate in order).

use std::cell::RefCell;

use crate::canvas::{Canvas, Region, Size};
use crate::layouts;
use crate::scroll::ScrollState;
use crate::scrollbar::ScrollBarRender;
use crate::widget::Widget;
use crate::widget::scrollbar_corner::ScrollBarCorner;
use crate::{KeyCode, MouseEvent};
use crossterm::event::KeyModifiers;
use tcss::types::{Overflow, RgbaColor, ScrollbarGutter, ScrollbarVisibility, Unit};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

/// Breakpoint configuration: threshold and class name to apply.
pub type Breakpoint = (u16, &'static str);

/// Default horizontal breakpoints (matches Textual).
pub const DEFAULT_HORIZONTAL_BREAKPOINTS: &[Breakpoint] = &[(0, "-narrow"), (80, "-wide")];

/// Default vertical breakpoints (matches Textual).
pub const DEFAULT_VERTICAL_BREAKPOINTS: &[Breakpoint] = &[(0, "-short"), (24, "-tall")];

const PAGE_SCROLL_RATIO: f32 = 0.9;

/// The root container for a widget tree.
///
/// `Screen` is responsible for:
/// 1. Providing the root context for CSS matching (type name "Screen").
/// 2. Managing responsive breakpoint classes based on terminal size.
/// 3. Dispatching to layout algorithms based on the `layout` CSS property.
///
/// It mimics Textual's Screen behavior where the app's content is implicitly
/// wrapped in a Screen widget.
pub struct Screen<M> {
    children: Vec<Box<dyn Widget<M>>>,
    /// Responsive classes are static strings from breakpoints, avoiding allocations.
    responsive_classes: Vec<&'static str>,
    style: ComputedStyle,
    is_dirty: bool,
    scroll: RefCell<ScrollState>,
    /// Which scrollbar is being hovered (Some(true) = vertical, Some(false) = horizontal).
    scrollbar_hover: Option<bool>,
    /// Active drag state: (vertical?, grab_offset)
    scrollbar_drag: Option<(bool, i32)>,
    horizontal_breakpoints: &'static [Breakpoint],
    vertical_breakpoints: &'static [Breakpoint],
}

impl<M> Screen<M> {
    /// Create a new Screen with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            responsive_classes: Vec::new(),
            style: ComputedStyle::default(),
            is_dirty: true,
            scroll: RefCell::new(ScrollState::default()),
            scrollbar_hover: None,
            scrollbar_drag: None,
            horizontal_breakpoints: DEFAULT_HORIZONTAL_BREAKPOINTS,
            vertical_breakpoints: DEFAULT_VERTICAL_BREAKPOINTS,
        }
    }

    /// Set custom horizontal breakpoints.
    pub fn with_horizontal_breakpoints(mut self, breakpoints: &'static [Breakpoint]) -> Self {
        self.horizontal_breakpoints = breakpoints;
        self
    }

    /// Set custom vertical breakpoints.
    pub fn with_vertical_breakpoints(mut self, breakpoints: &'static [Breakpoint]) -> Self {
        self.vertical_breakpoints = breakpoints;
        self
    }

    /// Updates the responsive classes based on dimensions.
    ///
    /// For each axis, finds the last matching breakpoint (threshold <= dimension)
    /// and applies that class.
    fn update_breakpoints(&mut self, width: u16, height: u16) {
        let old_classes = self.responsive_classes.clone();
        self.responsive_classes.clear();

        // Find matching horizontal breakpoint (last one where width >= threshold)
        if let Some((_, class)) = self
            .horizontal_breakpoints
            .iter()
            .filter(|(threshold, _)| width >= *threshold)
            .last()
        {
            self.responsive_classes.push(*class);
        }

        // Find matching vertical breakpoint (last one where height >= threshold)
        if let Some((_, class)) = self
            .vertical_breakpoints
            .iter()
            .filter(|(threshold, _)| height >= *threshold)
            .last()
        {
            self.responsive_classes.push(*class);
        }

        if old_classes != self.responsive_classes {
            self.is_dirty = true;
        }
    }

    /// Compute child placements using the appropriate layout algorithm.
    fn compute_child_placements(
        &self,
        region: Region,
        viewport: layouts::Viewport,
    ) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
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

        // Dispatch to layout (handles layers internally when needed)
        layouts::arrange_children_with_layers(&self.style, &children_with_styles, region, viewport)
    }

    fn compute_virtual_size(
        &self,
        placements: &[layouts::WidgetPlacement],
        base_region: Region,
    ) -> (i32, i32, bool) {
        let mut virtual_width = 0;
        let mut virtual_height = 0;
        let mut max_from_auto = false;

        for placement in placements {
            let width = (placement.region.x - base_region.x) + placement.region.width;
            let height = (placement.region.y - base_region.y) + placement.region.height;
            if width > virtual_width {
                virtual_width = width;
                max_from_auto = self
                    .children
                    .get(placement.child_index)
                    .and_then(|child| child.get_style().width)
                    .map(|scalar| scalar.unit == Unit::Auto)
                    .unwrap_or(false);
            } else if width == virtual_width {
                let is_auto = self
                    .children
                    .get(placement.child_index)
                    .and_then(|child| child.get_style().width)
                    .map(|scalar| scalar.unit == Unit::Auto)
                    .unwrap_or(false);
                max_from_auto = max_from_auto || is_auto;
            }
            if height > virtual_height {
                virtual_height = height;
            }
        }

        (virtual_width, virtual_height, max_from_auto)
    }

    fn dispatch_mouse_to_children(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);
        let (offset_x, offset_y) = {
            let scroll = self.scroll.borrow();
            (scroll.offset_x, scroll.offset_y)
        };

        for placement in placements {
            let scrolled_region = Region {
                x: placement.region.x - offset_x,
                y: placement.region.y - offset_y,
                width: placement.region.width,
                height: placement.region.height,
            };
            if scrolled_region.contains_point(event.column as i32, event.row as i32) {
                if let Some(child) = self.children.get_mut(placement.child_index) {
                    return child.on_mouse(event, scrolled_region);
                }
            }
        }

        None
    }

    fn show_vertical_scrollbar(&self) -> bool {
        match self.style.overflow_y {
            Overflow::Scroll => true,
            Overflow::Auto => self.scroll.borrow().can_scroll_y(),
            Overflow::Hidden => false,
        }
    }

    fn show_horizontal_scrollbar(&self) -> bool {
        match self.style.overflow_x {
            Overflow::Scroll => true,
            Overflow::Auto => self.scroll.borrow().can_scroll_x(),
            Overflow::Hidden => false,
        }
    }

    fn render_vertical_scrollbar(&self) -> bool {
        let style = &self.style.scrollbar;
        self.show_vertical_scrollbar()
            && style.visibility == ScrollbarVisibility::Visible
            && style.size.vertical > 0
    }

    fn render_horizontal_scrollbar(&self) -> bool {
        let style = &self.style.scrollbar;
        self.show_horizontal_scrollbar()
            && style.visibility == ScrollbarVisibility::Visible
            && style.size.horizontal > 0
    }

    fn content_region_for_scroll(&self, region: Region) -> Region {
        let style = &self.style.scrollbar;
        // Vertical gutter: apply when showing scrollbar OR when stable gutter with overflow-y: auto
        // (matches Python Textual behavior where scrollbar-gutter only affects vertical scrollbar)
        let v_size = if self.show_vertical_scrollbar()
            || (style.gutter == ScrollbarGutter::Stable && self.style.overflow_y == Overflow::Auto)
        {
            style.size.vertical as i32
        } else {
            0
        };
        // Horizontal gutter: only when actually showing scrollbar (no stable gutter support)
        let h_size = if self.show_horizontal_scrollbar() {
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

    fn vertical_scrollbar_region(&self, region: Region) -> Region {
        let style = &self.style.scrollbar;
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

    fn horizontal_scrollbar_region(&self, region: Region) -> Region {
        let style = &self.style.scrollbar;
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

    fn vertical_colors(&self) -> (RgbaColor, RgbaColor) {
        let style = &self.style.scrollbar;
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

    fn horizontal_colors(&self) -> (RgbaColor, RgbaColor) {
        let style = &self.style.scrollbar;
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

    fn handle_vertical_scrollbar_click(&mut self, event: MouseEvent, region: Region) {
        let my = event.row as i32;
        let pos_in_bar = my - region.y;

        let scroll = self.scroll.borrow();
        let (thumb_start, thumb_end) = ScrollBarRender::thumb_bounds(
            region.height,
            scroll.virtual_height as f32,
            scroll.viewport_height as f32,
            scroll.offset_y as f32,
        );
        drop(scroll);

        if pos_in_bar >= thumb_start && pos_in_bar < thumb_end {
            self.scrollbar_drag = Some((true, pos_in_bar - thumb_start));
            self.is_dirty = true;
        } else if pos_in_bar < thumb_start {
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_up(amount);
            drop(scroll);
            self.is_dirty = true;
        } else {
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_height as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_down(amount);
            drop(scroll);
            self.is_dirty = true;
        }
    }

    fn handle_horizontal_scrollbar_click(&mut self, event: MouseEvent, region: Region) {
        let mx = event.column as i32;
        let pos_in_bar = mx - region.x;

        let scroll = self.scroll.borrow();
        let (thumb_start, thumb_end) = ScrollBarRender::thumb_bounds(
            region.width,
            scroll.virtual_width as f32,
            scroll.viewport_width as f32,
            scroll.offset_x as f32,
        );
        drop(scroll);

        if pos_in_bar >= thumb_start && pos_in_bar < thumb_end {
            self.scrollbar_drag = Some((false, pos_in_bar - thumb_start));
            self.is_dirty = true;
        } else if pos_in_bar < thumb_start {
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_width as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_left(amount);
            drop(scroll);
            self.is_dirty = true;
        } else {
            let mut scroll = self.scroll.borrow_mut();
            let amount = (scroll.viewport_width as f32 * PAGE_SCROLL_RATIO) as i32;
            scroll.scroll_right(amount);
            drop(scroll);
            self.is_dirty = true;
        }
    }

    fn handle_scrollbar_drag(
        &mut self,
        event: MouseEvent,
        region: Region,
        vertical: bool,
        grab_offset: i32,
    ) {
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
                self.is_dirty = true;
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
                self.is_dirty = true;
            }
        }
    }
}

impl<M> Widget<M> for Screen<M> {
    fn default_css(&self) -> &'static str {
        // Match Python Textual's Screen DEFAULT_CSS
        r#"
Screen {
    layout: vertical;
    overflow-y: auto;
    background: $background;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // Render background/border and get inner region
        let inner_region = crate::containers::render_container_chrome(canvas, region, &self.style);

        // Use the canvas viewport (screen dimensions) for CSS vw/vh units
        let viewport = canvas.viewport();

        let initial_placements = self.compute_child_placements(inner_region, viewport);
        let (virtual_width, virtual_height, _) =
            self.compute_virtual_size(&initial_placements, inner_region);

        // Do not pad virtual size here; padding can fabricate overflow and force
        // Screen-level scrollbars even when content fits (differs from Python Textual).

        {
            let mut scroll = self.scroll.borrow_mut();
            scroll.set_virtual_size(virtual_width, virtual_height);
            scroll.set_viewport(inner_region.width, inner_region.height);
        }

        let show_v_scrollbar = self.show_vertical_scrollbar();

        let content_region = self.content_region_for_scroll(inner_region);

        let placements = if show_v_scrollbar && content_region.width < inner_region.width {
            let new_placements = self.compute_child_placements(content_region, viewport);
            let (new_virtual_width, new_virtual_height, _new_max_from_auto) =
                self.compute_virtual_size(&new_placements, content_region);

            {
                let mut scroll = self.scroll.borrow_mut();
                scroll.set_virtual_size(new_virtual_width, new_virtual_height);
                scroll.set_viewport(content_region.width, content_region.height);
            }
            new_placements
        } else {
            initial_placements
        };

        let (offset_x, offset_y) = {
            let scroll = self.scroll.borrow();
            (scroll.offset_x, scroll.offset_y)
        };

        canvas.push_clip(content_region);
        for placement in &placements {
            if let Some(child) = self.children.get(placement.child_index) {
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
        }
        canvas.pop_clip();

        let render_vertical = self.render_vertical_scrollbar();
        let render_horizontal = self.render_horizontal_scrollbar();

        if render_vertical {
            let v_region = self.vertical_scrollbar_region(inner_region);
            let (thumb_color, track_color) = self.vertical_colors();
            let (thumb_color, track_color, draw_thumb) = ScrollBarRender::compose_colors(
                thumb_color,
                track_color,
                self.style.inherited_background.clone(),
            );
            let scroll = self.scroll.borrow();
            ScrollBarRender::render_vertical(
                canvas,
                v_region,
                scroll.virtual_height as f32,
                content_region.height as f32,
                scroll.offset_y as f32,
                thumb_color,
                track_color,
                draw_thumb,
            );
        }

        if render_horizontal {
            let h_region = self.horizontal_scrollbar_region(inner_region);
            let (thumb_color, track_color) = self.horizontal_colors();
            let (thumb_color, track_color, draw_thumb) = ScrollBarRender::compose_colors(
                thumb_color,
                track_color,
                self.style.inherited_background.clone(),
            );
            let scroll = self.scroll.borrow();
            ScrollBarRender::render_horizontal(
                canvas,
                h_region,
                scroll.virtual_width as f32,
                content_region.width as f32,
                scroll.offset_x as f32,
                thumb_color,
                track_color,
                draw_thumb,
            );
        }

        if render_vertical && render_horizontal {
            let corner_region = Region {
                x: inner_region.x + inner_region.width - self.style.scrollbar.size.vertical as i32,
                y: inner_region.y + inner_region.height
                    - self.style.scrollbar.size.horizontal as i32,
                width: self.style.scrollbar.size.vertical as i32,
                height: self.style.scrollbar.size.horizontal as i32,
            };
            let mut corner = ScrollBarCorner::new(
                self.style.scrollbar.size.vertical,
                self.style.scrollbar.size.horizontal,
            );
            <ScrollBarCorner as Widget<M>>::set_style(&mut corner, self.style.clone());
            <ScrollBarCorner as Widget<M>>::render(&corner, canvas, corner_region);
        }
    }

    fn desired_size(&self) -> Size {
        // Screen fills available space
        Size::new(u16::MAX, u16::MAX)
    }

    fn on_resize(&mut self, size: Size) {
        self.update_breakpoints(size.width, size.height);
        // Propagate resize to children
        for child in &mut self.children {
            child.on_resize(size);
        }
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Screen",
            // Convert &'static str to String only when metadata is requested
            classes: self
                .responsive_classes
                .iter()
                .map(|s| s.to_string())
                .collect(),
            states: WidgetStates::empty(), // Screen typically doesn't have focus/hover itself
            id: None,
        }
    }

    // Delegate hierarchy traversal
    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index < self.children.len() {
            Some(self.children[index].as_mut())
        } else {
            None
        }
    }

    // Delegate state management
    fn is_dirty(&self) -> bool {
        self.is_dirty || self.children.iter().any(|c| c.is_dirty())
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
        for child in &mut self.children {
            child.mark_dirty();
        }
    }

    fn mark_clean(&mut self) {
        self.is_dirty = false;
        for child in &mut self.children {
            child.mark_clean();
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    // Delegate event handling
    fn on_event(&mut self, key: KeyCode) -> Option<M> {
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

        if !region.contains_point(mx, my) {
            return None;
        }

        // Scrollbar interaction regions
        let content_region = self.content_region_for_scroll(region);
        let v_region = self.vertical_scrollbar_region(region);
        let h_region = self.horizontal_scrollbar_region(region);
        let render_vertical = self.render_vertical_scrollbar();
        let render_horizontal = self.render_horizontal_scrollbar();
        let on_vertical = render_vertical && v_region.contains_point(mx, my);
        let on_horizontal = render_horizontal && h_region.contains_point(mx, my);

        const SCROLL_AMOUNT: i32 = 3;
        match event.kind {
            crossterm::event::MouseEventKind::Moved => {
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    self.handle_scrollbar_drag(event, region, vertical, grab_offset);
                    return None;
                }

                let new_hover = if on_vertical {
                    Some(true)
                } else if on_horizontal {
                    Some(false)
                } else {
                    None
                };

                if self.scrollbar_hover != new_hover {
                    self.scrollbar_hover = new_hover;
                    self.is_dirty = true;
                }

                if content_region.contains_point(mx, my) {
                    return self.dispatch_mouse_to_children(event, content_region);
                }
                return None;
            }
            crossterm::event::MouseEventKind::Down(_) => {
                if on_vertical {
                    self.handle_vertical_scrollbar_click(event, v_region);
                    return None;
                }
                if on_horizontal {
                    self.handle_horizontal_scrollbar_click(event, h_region);
                    return None;
                }
                if content_region.contains_point(mx, my) {
                    return self.dispatch_mouse_to_children(event, content_region);
                }
                return None;
            }
            crossterm::event::MouseEventKind::Drag(_) => {
                if let Some((vertical, grab_offset)) = self.scrollbar_drag {
                    self.handle_scrollbar_drag(event, region, vertical, grab_offset);
                    return None;
                }
                if content_region.contains_point(mx, my) {
                    return self.dispatch_mouse_to_children(event, content_region);
                }
                return None;
            }
            crossterm::event::MouseEventKind::Up(_) => {
                if self.scrollbar_drag.is_some() {
                    self.scrollbar_drag = None;
                    self.is_dirty = true;
                }
                if content_region.contains_point(mx, my) {
                    return self.dispatch_mouse_to_children(event, content_region);
                }
                return None;
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                let allow_horizontal = self.show_horizontal_scrollbar();
                let allow_vertical = self.show_vertical_scrollbar();
                if event.modifiers.contains(KeyModifiers::SHIFT)
                    || event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if allow_horizontal {
                        self.scroll.borrow_mut().scroll_right(SCROLL_AMOUNT);
                        self.is_dirty = true;
                        return None;
                    }
                } else if allow_vertical {
                    self.scroll.borrow_mut().scroll_down(SCROLL_AMOUNT);
                    self.is_dirty = true;
                    return None;
                }
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                let allow_horizontal = self.show_horizontal_scrollbar();
                let allow_vertical = self.show_vertical_scrollbar();
                if event.modifiers.contains(KeyModifiers::SHIFT)
                    || event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if allow_horizontal {
                        self.scroll.borrow_mut().scroll_left(SCROLL_AMOUNT);
                        self.is_dirty = true;
                        return None;
                    }
                } else if allow_vertical {
                    self.scroll.borrow_mut().scroll_up(SCROLL_AMOUNT);
                    self.is_dirty = true;
                    return None;
                }
            }
            crossterm::event::MouseEventKind::ScrollLeft => {
                if self.show_horizontal_scrollbar() {
                    self.scroll.borrow_mut().scroll_left(SCROLL_AMOUNT);
                    self.is_dirty = true;
                    return None;
                }
            }
            crossterm::event::MouseEventKind::ScrollRight => {
                if self.show_horizontal_scrollbar() {
                    self.scroll.borrow_mut().scroll_right(SCROLL_AMOUNT);
                    self.is_dirty = true;
                    return None;
                }
            }
        }

        // Compute placements and dispatch mouse events
        // For mouse handling, we approximate viewport as region
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);
        let offset_y = self.scroll.borrow().offset_y;

        for placement in placements {
            let scrolled_region = Region {
                x: placement.region.x,
                y: placement.region.y - offset_y,
                width: placement.region.width,
                height: placement.region.height,
            };
            if scrolled_region.contains_point(mx, my) {
                if let Some(child) = self.children.get_mut(placement.child_index) {
                    if let Some(msg) = child.on_mouse(event, scrolled_region) {
                        return Some(msg);
                    }
                }
            }
        }

        None
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        let mut changed = false;
        for child in &mut self.children {
            if child.set_hover(is_hovered) {
                changed = true;
            }
        }
        changed
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_hover();
            }
        }
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        let mut changed = false;
        for child in &mut self.children {
            if child.set_active(is_active) {
                changed = true;
            }
        }
        changed
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        for child in &mut self.children {
            if let Some(msg) = child.handle_message(envelope) {
                return Some(msg);
            }
        }
        None
    }

    // Focus delegation
    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.participates_in_layout())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn focus_nth(&mut self, mut n: usize) -> bool {
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let count = child.count_focusable();
            if n < count {
                return child.focus_nth(n);
            }
            n -= count;
        }
        false
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_focus();
            }
        }
    }
}
