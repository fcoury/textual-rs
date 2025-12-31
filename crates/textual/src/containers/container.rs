//! Generic container widget with CSS-driven layout dispatch.
//!
//! Container is the base for all layout containers. It dispatches to the
//! appropriate layout algorithm based on the `layout` CSS property.

use tcss::types::Layout as LayoutDirection;
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region, Size};
use crate::content::Content;
use crate::layouts::{self, Layout};
use crate::render_cache::RenderCache;
use crate::segment::Style;
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};

// Re-export for use by Horizontal/Vertical
pub use tcss::types::Layout as ContainerLayoutDirection;

/// A generic container that arranges children using CSS-driven layout.
///
/// The layout algorithm is determined by the `layout` CSS property:
/// - `layout: vertical` - stacks children top-to-bottom (default)
/// - `layout: horizontal` - stacks children left-to-right
/// - `layout: grid` - CSS Grid-like 2D layout
///
/// Containers are the building blocks for complex layouts. Use the
/// type aliases (`Grid`, `Vertical`, `Horizontal`) for semantic clarity.
pub struct Container<M> {
    children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    dirty: bool,
    id: Option<String>,
    /// CSS classes for styling.
    classes: Vec<String>,
    /// Title displayed in the top border (supports markup).
    border_title: Option<String>,
    /// Subtitle displayed in the bottom border (supports markup).
    border_subtitle: Option<String>,
    /// Optional layout direction override (takes precedence over CSS).
    /// Used by Horizontal/Vertical wrappers to enforce their layout mode.
    layout_override: Option<LayoutDirection>,
    /// Cached viewport size from on_resize (terminal dimensions).
    /// Used for content_height_for_scroll to match layout calculations.
    viewport: Size,
}

impl<M> Container<M> {
    /// Create a new Container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            dirty: true,
            id: None,
            classes: Vec::new(),
            border_title: None,
            border_subtitle: None,
            layout_override: None,
            viewport: Size::new(80, 24), // Default until on_resize is called
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the layout direction, overriding CSS.
    ///
    /// This is used by Horizontal/Vertical wrappers to enforce their layout
    /// mode regardless of CSS settings.
    pub fn with_layout(mut self, direction: LayoutDirection) -> Self {
        self.layout_override = Some(direction);
        self
    }

    /// Set the border title (displayed in the top border).
    ///
    /// The title supports markup for styling (e.g., `[b]Bold Title[/]`).
    pub fn with_border_title(mut self, title: impl Into<String>) -> Self {
        self.border_title = Some(title.into());
        self
    }

    /// Set the border subtitle (displayed in the bottom border).
    ///
    /// The subtitle supports markup for styling (e.g., `[i]Italic Subtitle[/]`).
    pub fn with_border_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.border_subtitle = Some(subtitle.into());
        self
    }

    /// Set the border title at runtime.
    pub fn set_border_title(&mut self, title: impl Into<String>) {
        self.border_title = Some(title.into());
        self.dirty = true;
    }

    /// Set the border subtitle at runtime.
    pub fn set_border_subtitle(&mut self, subtitle: impl Into<String>) {
        self.border_subtitle = Some(subtitle.into());
        self.dirty = true;
    }

    /// Get the border title.
    pub fn border_title(&self) -> Option<&str> {
        self.border_title.as_deref()
    }

    /// Get the border subtitle.
    pub fn border_subtitle(&self) -> Option<&str> {
        self.border_subtitle.as_deref()
    }

    /// Get the effective layout direction (override takes precedence over CSS).
    fn effective_layout(&self) -> LayoutDirection {
        self.layout_override.unwrap_or(self.style.layout)
    }

    /// Compute child placements using the appropriate layout algorithm.
    fn compute_child_placements(
        &self,
        region: Region,
        viewport: layouts::Viewport,
    ) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
        let children_with_styles: Vec<_> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.participates_in_layout())
            .map(|(i, c)| (i, c.get_style(), c.desired_size()))
            .collect();

        // Create a modified style with the effective layout direction
        let mut effective_style = self.style.clone();
        effective_style.layout = self.effective_layout();

        // Dispatch to layout based on effective layout, using the propagated viewport
        layouts::arrange_children_with_viewport(&effective_style, &children_with_styles, region, viewport)
    }

    /// Calculate intrinsic size based on children and layout mode.
    fn calculate_intrinsic_size(&self) -> Size {
        if self.children.is_empty() {
            // Empty container: minimal size (account for border)
            let border_size = if self.style.border.is_none() { 0 } else { 2 };
            let padding_h = self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
            let padding_v = self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
            return Size::new(border_size + padding_h, border_size + padding_v);
        }

        // Collect children sizes
        // u16::MAX signals "fill available space" - propagate this signal
        let mut total_width: u16 = 0;
        let mut total_height: u16 = 0;
        let mut max_width: u16 = 0;
        let mut max_height: u16 = 0;
        let mut any_child_wants_fill_width = false;
        let mut any_child_wants_fill_height = false;

        for child in &self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let child_size = child.desired_size();
            let child_style = child.get_style();

            // Check for "fill available space" signals
            if child_size.width == u16::MAX {
                any_child_wants_fill_width = true;
            }
            if child_size.height == u16::MAX {
                any_child_wants_fill_height = true;
            }

            // Track max dimensions (cap at reasonable max to avoid overflow, but preserve MAX signal)
            let capped_width = if child_size.width == u16::MAX { 0 } else { child_size.width.min(1000) };
            let capped_height = if child_size.height == u16::MAX { 0 } else { child_size.height.min(1000) };

            // Include child margins in the size calculation
            let margin_h = child_style.margin.left.value as u16 + child_style.margin.right.value as u16;
            let margin_v = child_style.margin.top.value as u16 + child_style.margin.bottom.value as u16;

            let width_with_margin = capped_width.saturating_add(margin_h);
            let height_with_margin = capped_height.saturating_add(margin_v);

            max_width = max_width.max(width_with_margin);
            max_height = max_height.max(height_with_margin);

            // Track totals for stacking
            total_width = total_width.saturating_add(width_with_margin);
            total_height = total_height.saturating_add(height_with_margin);
        }

        // Calculate based on layout mode (use effective layout, not just CSS)
        let (content_w, content_h) = match self.effective_layout() {
            LayoutDirection::Vertical => {
                // Stacked vertically: width = max child width, height = sum of heights
                (max_width, total_height)
            }
            LayoutDirection::Horizontal => {
                // Stacked horizontally: width = sum of widths, height = max child height
                (total_width, max_height)
            }
            LayoutDirection::Grid => {
                // Grid: use max dimensions as approximation
                (max_width, max_height)
            }
        };

        // Add border and padding
        let border_size = if self.style.border.is_none() { 0 } else { 2 };
        let padding_h = self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
        let padding_v = self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;

        // If any child wants to fill available space, propagate that signal
        let final_width = if any_child_wants_fill_width {
            u16::MAX
        } else {
            content_w.saturating_add(border_size).saturating_add(padding_h)
        };

        let final_height = if any_child_wants_fill_height {
            u16::MAX
        } else {
            content_h.saturating_add(border_size).saturating_add(padding_v)
        };

        Size::new(final_width, final_height)
    }
}

impl<M: 'static> Widget<M> for Container<M> {
    fn default_css(&self) -> &'static str {
        // Match Python Textual's Container DEFAULT_CSS
        r#"
Container {
    width: 1fr;
    height: 1fr;
    layout: vertical;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let width = region.width as usize;
        let height = region.height as usize;

        // 1. Create render cache for border handling
        let cache = RenderCache::new(&self.style);
        let (inner_width, inner_height) = cache.inner_size(width, height);

        // 2. Parse border titles as markup (only if we have a border)
        // Title/subtitle color inheritance:
        // 1. border_title_color / border_subtitle_color if explicitly set
        // 2. border.top.color / border.bottom.color (border color)
        // 3. style.color (text color) as fallback
        let title_fg = self.style.border_title_color.clone()
            .or_else(|| self.style.border.top.color.clone())
            .or_else(|| self.style.color.clone());
        let subtitle_fg = self.style.border_subtitle_color.clone()
            .or_else(|| self.style.border.bottom.color.clone())
            .or_else(|| self.style.color.clone());

        // Use border-title-background if set, otherwise fall back to widget background
        let title_bg = self.style.border_title_background.clone()
            .or_else(|| self.style.background.clone());
        let subtitle_bg = self.style.border_subtitle_background.clone()
            .or_else(|| self.style.background.clone());

        let title_style = Style {
            fg: title_fg,
            bg: title_bg,
            bold: self.style.border_title_style.bold,
            dim: self.style.border_title_style.dim,
            italic: self.style.border_title_style.italic,
            underline: self.style.border_title_style.underline,
            strike: self.style.border_title_style.strike,
            reverse: self.style.border_title_style.reverse,
        };
        let subtitle_style = Style {
            fg: subtitle_fg,
            bg: subtitle_bg,
            bold: self.style.border_subtitle_style.bold,
            dim: self.style.border_subtitle_style.dim,
            italic: self.style.border_subtitle_style.italic,
            underline: self.style.border_subtitle_style.underline,
            strike: self.style.border_subtitle_style.strike,
            reverse: self.style.border_subtitle_style.reverse,
        };

        // For border titles, don't wrap - let render_label_in_row handle truncation with ellipsis
        let title_strip = if cache.has_border() {
            self.border_title.as_ref().and_then(|t| {
                if t.is_empty() {
                    None
                } else {
                    let content = Content::from_markup(t)
                        .unwrap_or_else(|_| Content::new(t))
                        .with_style(title_style.clone());
                    // Use very large width to prevent word-wrapping; truncation happens in render_label_in_row
                    content.wrap(usize::MAX).into_iter().next()
                }
            })
        } else {
            None
        };

        let subtitle_strip = if cache.has_border() {
            self.border_subtitle.as_ref().and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    let content = Content::from_markup(s)
                        .unwrap_or_else(|_| Content::new(s))
                        .with_style(subtitle_style.clone());
                    // Use very large width to prevent word-wrapping; truncation happens in render_label_in_row
                    content.wrap(usize::MAX).into_iter().next()
                }
            })
        } else {
            None
        };

        // 3. Render each line (background fill + borders with titles)
        for y in 0..height {
            let strip = cache.render_line(
                y,
                height,
                width,
                None,
                title_strip.as_ref(),
                subtitle_strip.as_ref(),
            );
            canvas.render_strip(&strip, region.x, region.y + y as i32);
        }

        // 4. Calculate inner region for children
        let border_offset = if cache.has_border() { 1 } else { 0 };
        let padding_left = cache.padding_left() as i32;
        let padding_top = cache.padding_top() as i32;

        let inner_region = Region::new(
            region.x + border_offset + padding_left,
            region.y + border_offset + padding_top,
            inner_width as i32,
            inner_height as i32,
        );

        // 5. Render children in inner region using the canvas viewport
        let viewport = canvas.viewport();
        canvas.push_clip(inner_region);
        for placement in self.compute_child_placements(inner_region, viewport) {
            self.children[placement.child_index].render(canvas, placement.region);
        }
        canvas.pop_clip();
    }

    fn desired_size(&self) -> Size {
        use tcss::types::Unit;

        // Calculate intrinsic size from children
        let intrinsic_size = self.calculate_intrinsic_size();

        // Check CSS dimensions
        // For percentage/fr units, return u16::MAX to signal "fill available space"
        // This matches Python Textual behavior where containers expand to fill
        let width = if let Some(w) = &self.style.width {
            match w.unit {
                Unit::Cells => w.value as u16,
                Unit::Percent | Unit::ViewWidth | Unit::ViewHeight | Unit::Fraction | Unit::Width | Unit::Height => {
                    // Percentage-based or flexible units: signal "fill available space"
                    u16::MAX
                }
                Unit::Auto => intrinsic_size.width,
            }
        } else {
            // No explicit width: use intrinsic size
            intrinsic_size.width
        };

        let height = if let Some(h) = &self.style.height {
            match h.unit {
                Unit::Cells => h.value as u16,
                Unit::Percent | Unit::ViewWidth | Unit::ViewHeight | Unit::Fraction | Unit::Width | Unit::Height => {
                    // Percentage-based or flexible units: signal "fill available space"
                    u16::MAX
                }
                Unit::Auto => intrinsic_size.height,
            }
        } else {
            // No explicit height: use intrinsic size
            intrinsic_size.height
        };

        Size::new(width, height)
    }

    fn content_height_for_scroll(&self, _available_height: u16) -> u16 {
        use tcss::types::Unit;

        // Calculate actual content height for scrolling, resolving percentage/fr units
        // This is used when desired_size returns u16::MAX (fill available space)
        // Use stored viewport dimensions to match layout calculations (Python uses parent.app.size)
        //
        // IMPORTANT: Use f64 accumulation with floor arithmetic to match vertical layout.
        // Simple truncation (as u16) gives wrong results because fractional heights
        // accumulate differently. E.g., 5.875 + 2.125 = 8.0 (8 rows), but
        // floor(5.875) + floor(2.125) = 5 + 2 = 7 rows.
        let viewport_width = self.viewport.width;
        let viewport_height = self.viewport.height;

        // Accumulate heights as f64 using the same floor arithmetic as vertical layout
        let mut current_y: f64 = 0.0;

        for child in &self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let child_style = child.get_style();
            let child_desired = child.desired_size();

            // Calculate child height based on its CSS height property (as f64)
            // Use viewport dimensions to match layout (Python uses parent.app.size for all)
            let box_height: f64 = if let Some(h) = &child_style.height {
                match h.unit {
                    Unit::Cells => h.value,
                    Unit::Percent | Unit::Height => {
                        // Python uses parent.app.size.height for these
                        (h.value / 100.0) * viewport_height as f64
                    }
                    Unit::Width => {
                        // Python uses parent.app.size.width for w units
                        (h.value / 100.0) * viewport_width as f64
                    }
                    Unit::ViewWidth => {
                        (h.value / 100.0) * viewport_width as f64
                    }
                    Unit::ViewHeight => {
                        (h.value / 100.0) * viewport_height as f64
                    }
                    Unit::Fraction => {
                        // fr units use their value as minimum height for scroll estimation
                        // 1fr = 1 row minimum, 2fr = 2 rows minimum, etc.
                        h.value.max(1.0)
                    }
                    Unit::Auto => {
                        // Auto uses intrinsic height
                        if child_desired.height == u16::MAX {
                            child.content_height_for_scroll(viewport_height) as f64
                        } else {
                            child_desired.height as f64
                        }
                    }
                }
            } else if child_desired.height == u16::MAX {
                // No CSS height, child wants to fill - recurse
                child.content_height_for_scroll(viewport_height) as f64
            } else {
                child_desired.height as f64
            };

            // Use floor arithmetic matching the vertical layout:
            // region.height = floor(next_y) - floor(y)
            let next_y = current_y + box_height;
            let _region_height = (next_y.floor() as i32 - current_y.floor() as i32).max(0);
            current_y = next_y;
        }

        // Total height is floor of final Y position
        let total_height = current_y.floor() as u16;

        // Add container chrome (border + padding)
        let border_size = if self.style.border.is_none() { 0 } else { 2 };
        let padding_v = self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
        let result = total_height.saturating_add(border_size).saturating_add(padding_v);

        result
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Container".to_string(),
            id: self.id.clone(),
            classes: Vec::new(),
            states: WidgetStates::empty(),
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
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

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn on_resize(&mut self, size: Size) {
        // Store viewport for content_height_for_scroll calculations
        log::debug!("Container::on_resize: size={}x{}", size.width, size.height);
        self.viewport = size;
        for child in &mut self.children {
            child.on_resize(size);
        }
    }

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

        // Compute placements and dispatch mouse events
        // For mouse handling, approximate viewport as region (only available during render)
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);

        for placement in placements {
            if placement.region.contains_point(mx, my) {
                if let Some(msg) = self.children[placement.child_index].on_mouse(event, placement.region) {
                    return Some(msg);
                }
            }
        }

        None
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

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_hover();
            }
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

    fn border_title(&self) -> Option<&str> {
        self.border_title.as_deref()
    }

    fn border_subtitle(&self) -> Option<&str> {
        self.border_subtitle.as_deref()
    }

    fn set_border_title(&mut self, title: &str) {
        self.border_title = Some(title.to_string());
        self.dirty = true;
    }

    fn set_border_subtitle(&mut self, subtitle: &str) {
        self.border_subtitle = Some(subtitle.to_string());
        self.dirty = true;
    }

    fn pre_layout(&mut self, _layout: &mut dyn Layout) {
        // Default container doesn't configure layout
        // Override in ItemGrid to set min_column_width, etc.
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
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
}
