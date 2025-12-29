//! Generic container widget with CSS-driven layout dispatch.
//!
//! Container is the base for all layout containers. It dispatches to the
//! appropriate layout algorithm based on the `layout` CSS property.

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region, Size};
use crate::content::Content;
use crate::layouts::{self, Layout};
use crate::render_cache::RenderCache;
use crate::segment::Style;
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};

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
    /// Title displayed in the top border (supports markup).
    border_title: Option<String>,
    /// Subtitle displayed in the bottom border (supports markup).
    border_subtitle: Option<String>,
}

impl<M> Container<M> {
    /// Create a new Container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            dirty: true,
            id: None,
            border_title: None,
            border_subtitle: None,
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
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

    /// Compute child placements using the appropriate layout algorithm.
    fn compute_child_placements(&self, region: Region) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
        let children_with_styles: Vec<_> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_visible())
            .map(|(i, c)| (i, c.get_style(), c.desired_size()))
            .collect();

        // Dispatch to layout based on CSS
        layouts::arrange_children(&self.style, &children_with_styles, region)
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
        let mut total_width: u16 = 0;
        let mut total_height: u16 = 0;
        let mut max_width: u16 = 0;
        let mut max_height: u16 = 0;

        for child in &self.children {
            if !child.is_visible() {
                continue;
            }
            let child_size = child.desired_size();

            // Track max dimensions (cap at reasonable max to avoid overflow)
            max_width = max_width.max(child_size.width.min(1000));
            max_height = max_height.max(child_size.height.min(1000));

            // Track totals for stacking
            total_width = total_width.saturating_add(child_size.width.min(1000));
            total_height = total_height.saturating_add(child_size.height.min(1000));
        }

        // Calculate based on layout mode
        let (content_w, content_h) = match self.style.layout {
            tcss::types::Layout::Vertical => {
                // Stacked vertically: width = max child width, height = sum of heights
                (max_width, total_height)
            }
            tcss::types::Layout::Horizontal => {
                // Stacked horizontally: width = sum of widths, height = max child height
                (total_width, max_height)
            }
            tcss::types::Layout::Grid => {
                // Grid: use max dimensions as approximation
                (max_width, max_height)
            }
        };

        // Add border and padding
        let border_size = if self.style.border.is_none() { 0 } else { 2 };
        let padding_h = self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
        let padding_v = self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;

        Size::new(
            content_w.saturating_add(border_size).saturating_add(padding_h),
            content_h.saturating_add(border_size).saturating_add(padding_v),
        )
    }
}

impl<M> Widget<M> for Container<M> {
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

        let title_style = Style {
            fg: title_fg,
            bg: self.style.background.clone(),
            ..Default::default()
        };
        let subtitle_style = Style {
            fg: subtitle_fg,
            bg: self.style.background.clone(),
            ..Default::default()
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

        // 5. Render children in inner region
        canvas.push_clip(inner_region);
        for placement in self.compute_child_placements(inner_region) {
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
        for child in &mut self.children {
            child.on_resize(size);
        }
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        for child in &mut self.children {
            if !child.is_visible() {
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
        let placements = self.compute_child_placements(region);

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
            .filter(|c| c.is_visible())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.is_visible() {
                child.clear_focus();
            }
        }
    }

    fn focus_nth(&mut self, mut n: usize) -> bool {
        for child in &mut self.children {
            if !child.is_visible() {
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
            if child.is_visible() {
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

    fn pre_layout(&mut self, _layout: &mut dyn Layout) {
        // Default container doesn't configure layout
        // Override in ItemGrid to set min_column_width, etc.
    }
}
