//! Generic container widget with CSS-driven layout dispatch.
//!
//! Container is the base for all layout containers. It dispatches to the
//! appropriate layout algorithm based on the `layout` CSS property.

use std::cell::RefCell;

use tcss::types::Layout as LayoutDirection;
use tcss::types::Overflow;
use tcss::types::RgbaColor;
use tcss::types::ScrollbarGutter;
use tcss::types::ScrollbarVisibility;
use tcss::types::Visibility;
use tcss::types::keyline::KeylineStyle;
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region, Size};
use crate::content::Content;
use crate::keyline_canvas::KeylineCanvas;
use crate::layouts::{self, Layout, Viewport, WidgetPlacement};
use crate::render_cache::RenderCache;
use crate::scroll::ScrollState;
use crate::scrollbar::ScrollBarRender;
use crate::segment::Style;
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent, MouseEventKind};

/// Cached layout computation result.
/// Stores the computed placements along with the region/viewport they were computed for.
#[derive(Debug, Clone)]
struct CachedLayout {
    placements: Vec<WidgetPlacement>,
    region: Region,
    viewport: Viewport,
}

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
    inline_style: StyleOverride,
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
    /// Cached layout placements to avoid recomputing on every render.
    /// Uses RefCell for interior mutability since render takes &self.
    cached_layout: RefCell<Option<CachedLayout>>,
    /// Scroll state for overflow scrolling.
    /// Uses RefCell for interior mutability since render takes &self.
    scroll: RefCell<ScrollState>,
    /// Which scrollbar is being hovered (Some(true) = vertical, Some(false) = horizontal).
    scrollbar_hover: Option<bool>,
    /// Scrollbar drag state: (is_vertical, grab_offset).
    scrollbar_drag: Option<(bool, i32)>,
}

impl<M> Container<M> {
    /// Create a new Container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
            dirty: true,
            id: None,
            classes: Vec::new(),
            border_title: None,
            border_subtitle: None,
            layout_override: None,
            viewport: Size::new(80, 24), // Default until on_resize is called
            cached_layout: RefCell::new(None),
            scroll: RefCell::new(ScrollState::default()),
            scrollbar_hover: None,
            scrollbar_drag: None,
        }
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

    /// Check if vertical scrollbar should be shown based on overflow-y and content.
    fn show_vertical_scrollbar(&self) -> bool {
        match self.style.overflow_y {
            Overflow::Scroll => true,
            Overflow::Auto => {
                let scroll = self.scroll.borrow();
                scroll.can_scroll_y()
            }
            Overflow::Hidden => false,
        }
    }

    /// Check if horizontal scrollbar should be shown based on overflow-x and content.
    fn show_horizontal_scrollbar(&self) -> bool {
        match self.style.overflow_x {
            Overflow::Scroll => true,
            Overflow::Auto => {
                let scroll = self.scroll.borrow();
                scroll.can_scroll_x()
            }
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

    /// Calculate content region with scrollbar space subtracted.
    fn content_region_for_scroll(&self, region: Region) -> Region {
        let style = &self.style.scrollbar;
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

    /// Calculate the horizontal scrollbar region.
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

    /// Get colors for vertical scrollbar based on hover/drag state.
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

    /// Get colors for horizontal scrollbar based on hover/drag state.
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

    /// Compute child placements using the appropriate layout algorithm.
    /// Uses caching to avoid recomputation when region/viewport haven't changed.
    fn compute_child_placements(&self, region: Region, viewport: Viewport) -> Vec<WidgetPlacement> {
        // Check cache first - if region and viewport match, return cached placements
        {
            let cache = self.cached_layout.borrow();
            if let Some(ref cached) = *cache {
                if cached.region == region && cached.viewport == viewport {
                    return cached.placements.clone();
                }
            }
        }

        // Cache miss - compute fresh placements
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

        // Create a modified style with the effective layout direction
        let mut effective_style = self.style.clone();
        effective_style.layout = self.effective_layout();

        // Dispatch to layout (handles layers internally when needed)
        let placements = layouts::arrange_children_with_layers(
            &effective_style,
            &children_with_styles,
            region,
            viewport,
        );

        // Store in cache
        *self.cached_layout.borrow_mut() = Some(CachedLayout {
            placements: placements.clone(),
            region,
            viewport,
        });

        placements
    }

    /// Render keylines for the container.
    ///
    /// For horizontal layout: draws outer box + vertical dividers between children
    /// For vertical layout: draws outer box + horizontal dividers between children
    fn render_keylines(
        &self,
        canvas: &mut Canvas,
        region: Region,
        placements: &[layouts::WidgetPlacement],
    ) {
        if placements.is_empty() {
            return;
        }

        let line_type = self.style.keyline.style.line_type();
        let mut keyline_canvas = KeylineCanvas::new(
            region.width as usize,
            region.height as usize,
            line_type,
            self.style.keyline.color.clone(),
        );

        match self.effective_layout() {
            LayoutDirection::Horizontal => {
                // Horizontal: draw outer box + vertical dividers at child boundaries
                let mut col_positions: Vec<usize> = Vec::new();

                // First column position is 0
                col_positions.push(0);

                // Add divider positions at the END of each child (which is the START of the next)
                for placement in placements {
                    let right_edge =
                        (placement.region.x + placement.region.width - region.x) as usize;
                    if !col_positions.contains(&right_edge) && right_edge < region.width as usize {
                        col_positions.push(right_edge);
                    }
                }

                // Add the right edge of the container
                let last_pos = (region.width as usize).saturating_sub(1);
                if !col_positions.contains(&last_pos) {
                    col_positions.push(last_pos);
                }

                col_positions.sort();
                col_positions.dedup();

                // Single row spanning the full height
                let row_positions: Vec<usize> = vec![0, (region.height as usize).saturating_sub(1)];

                keyline_canvas.add_grid(&col_positions, &row_positions);
            }
            LayoutDirection::Vertical => {
                // Vertical: draw outer box + horizontal dividers at child boundaries
                let mut row_positions: Vec<usize> = Vec::new();

                // First row position is 0
                row_positions.push(0);

                // Add divider positions at the END of each child (which is the START of the next)
                for placement in placements {
                    let bottom_edge =
                        (placement.region.y + placement.region.height - region.y) as usize;
                    if !row_positions.contains(&bottom_edge) && bottom_edge < region.height as usize
                    {
                        row_positions.push(bottom_edge);
                    }
                }

                // Add the bottom edge of the container
                let last_pos = (region.height as usize).saturating_sub(1);
                if !row_positions.contains(&last_pos) {
                    row_positions.push(last_pos);
                }

                row_positions.sort();
                row_positions.dedup();

                // Single column spanning the full width
                let col_positions: Vec<usize> = vec![0, (region.width as usize).saturating_sub(1)];

                keyline_canvas.add_grid(&col_positions, &row_positions);
            }
            LayoutDirection::Grid => {
                // Grid layout - use GridLayout to get track info
                // This shouldn't normally happen since Grid has its own widget,
                // but handle it for completeness
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

                let layout = layouts::GridLayout::default();
                let track_info =
                    layout.compute_track_info(&self.style, &children_with_styles, region);

                if track_info.col_positions.len() >= 2 && track_info.row_positions.len() >= 2 {
                    let col_positions: Vec<usize> = track_info
                        .col_positions
                        .iter()
                        .map(|&x| x.max(0) as usize)
                        .collect();
                    let row_positions: Vec<usize> = track_info
                        .row_positions
                        .iter()
                        .map(|&y| y.max(0) as usize)
                        .collect();

                    keyline_canvas.add_grid(&col_positions, &row_positions);
                }
            }
        }

        keyline_canvas.render(canvas, region);
    }

    /// Calculate intrinsic size based on children and layout mode.
    fn calculate_intrinsic_size(&self) -> Size {
        if self.children.is_empty() {
            // Empty container: minimal size (account for border)
            let border_size = if self.style.border.is_none() { 0 } else { 2 };
            let padding_h =
                self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
            let padding_v =
                self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
            return Size::new(border_size + padding_h, border_size + padding_v);
        }

        // Collect children sizes with margin collapsing support
        // u16::MAX signals "fill available space" - propagate this signal
        let mut total_width: u16 = 0;
        let mut total_height: u16 = 0;
        let mut max_width: u16 = 0;
        let mut max_height: u16 = 0;
        let mut any_child_wants_fill_width = false;
        let mut any_child_wants_fill_height = false;

        // For margin collapsing in vertical layout
        let mut prev_margin_bottom: u16 = 0;
        let mut first_child_margin_top: u16 = 0;
        let mut last_child_margin_bottom: u16 = 0;
        let mut is_first_child = true;

        // For margin collapsing in horizontal layout
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

            // Check for "fill available space" signals
            if child_size.width == u16::MAX {
                any_child_wants_fill_width = true;
            }
            if child_size.height == u16::MAX {
                any_child_wants_fill_height = true;
            }

            // Track max dimensions (cap at reasonable max to avoid overflow, but preserve MAX signal)
            let capped_width = if child_size.width == u16::MAX {
                0
            } else {
                child_size.width.min(1000)
            };
            let capped_height = if child_size.height == u16::MAX {
                0
            } else {
                child_size.height.min(1000)
            };

            let margin_left = child_style.margin.left.value as u16;
            let margin_right = child_style.margin.right.value as u16;
            let margin_top = child_style.margin.top.value as u16;
            let margin_bottom = child_style.margin.bottom.value as u16;

            // For horizontal layout: calculate width with margin collapsing
            if is_first_h_child {
                first_child_margin_left = margin_left;
                total_width = total_width.saturating_add(capped_width);
                is_first_h_child = false;
            } else {
                // CSS margin collapsing: use max of adjacent margins, not sum
                let collapsed_margin = margin_left.max(prev_margin_right);
                total_width = total_width
                    .saturating_add(collapsed_margin)
                    .saturating_add(capped_width);
            }
            prev_margin_right = margin_right;
            last_child_margin_right = margin_right;

            // For vertical layout: calculate height with margin collapsing
            if is_first_child {
                first_child_margin_top = margin_top;
                total_height = total_height.saturating_add(capped_height);
                is_first_child = false;
            } else {
                // CSS margin collapsing: use max of adjacent margins, not sum
                let collapsed_margin = margin_top.max(prev_margin_bottom);
                total_height = total_height
                    .saturating_add(collapsed_margin)
                    .saturating_add(capped_height);
            }
            prev_margin_bottom = margin_bottom;
            last_child_margin_bottom = margin_bottom;

            // Max dimensions include full margins (for cross-axis)
            let width_with_margin = capped_width
                .saturating_add(margin_left)
                .saturating_add(margin_right);
            let height_with_margin = capped_height
                .saturating_add(margin_top)
                .saturating_add(margin_bottom);
            max_width = max_width.max(width_with_margin);
            max_height = max_height.max(height_with_margin);
        }

        // Add outer margins (first and last child's outer margins)
        total_width = first_child_margin_left
            .saturating_add(total_width)
            .saturating_add(last_child_margin_right);
        total_height = first_child_margin_top
            .saturating_add(total_height)
            .saturating_add(last_child_margin_bottom);

        // Calculate based on layout mode (use effective layout, not just CSS)
        let (content_w, content_h) = match self.effective_layout() {
            LayoutDirection::Vertical => {
                // Stacked vertically: width = max child width, height = sum of heights (with collapsing)
                (max_width, total_height)
            }
            LayoutDirection::Horizontal => {
                // Stacked horizontally: width = sum of widths (with collapsing), height = max child height
                (total_width, max_height)
            }
            LayoutDirection::Grid => {
                // Grid: use max dimensions as approximation
                (max_width, max_height)
            }
        };

        // Add border and padding
        let border_size = if self.style.border.is_none() { 0 } else { 2 };
        let padding_h =
            self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
        let padding_v =
            self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;

        // If any child wants to fill available space, propagate that signal
        let final_width = if any_child_wants_fill_width {
            u16::MAX
        } else {
            content_w
                .saturating_add(border_size)
                .saturating_add(padding_h)
        };

        let final_height = if any_child_wants_fill_height {
            u16::MAX
        } else {
            content_h
                .saturating_add(border_size)
                .saturating_add(padding_v)
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
        // Apply opacity by blending toward inherited background
        let base_title_fg = self
            .style
            .border_title_color
            .clone()
            .or_else(|| self.style.border.top.color.clone())
            .or_else(|| self.style.color.clone());
        let title_fg = match (&base_title_fg, &self.style.inherited_background) {
            (Some(color), Some(bg)) => Some(color.blend_toward(bg, self.style.opacity)),
            (Some(color), None) => Some(color.with_opacity(self.style.opacity)),
            _ => None,
        };
        let base_subtitle_fg = self
            .style
            .border_subtitle_color
            .clone()
            .or_else(|| self.style.border.bottom.color.clone())
            .or_else(|| self.style.color.clone());
        let subtitle_fg = match (&base_subtitle_fg, &self.style.inherited_background) {
            (Some(color), Some(bg)) => Some(color.blend_toward(bg, self.style.opacity)),
            (Some(color), None) => Some(color.with_opacity(self.style.opacity)),
            _ => None,
        };

        // Use border-title-background if set, otherwise fall back to widget background
        // Note: Background opacity is already handled through effective_background()
        let title_bg = self
            .style
            .border_title_background
            .clone()
            .or_else(|| self.style.background.clone());
        let subtitle_bg = self
            .style
            .border_subtitle_background
            .clone()
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

        // 5. Update scroll state and compute content region
        let viewport = canvas.viewport();

        // First compute placements to determine virtual content height
        let initial_placements = self.compute_child_placements(inner_region, viewport);

        // Calculate virtual content dimensions from placements (max extent relative to inner_region)
        let virtual_height = initial_placements
            .iter()
            .map(|p| {
                let margin_bottom = self
                    .children
                    .get(p.child_index)
                    .map(|child| child.get_style().margin.bottom.value as i32)
                    .unwrap_or(0);
                (p.region.y - inner_region.y) + p.region.height + margin_bottom
            })
            .max()
            .unwrap_or(0);

        let virtual_width = initial_placements
            .iter()
            .map(|p| {
                let margin_right = self
                    .children
                    .get(p.child_index)
                    .map(|child| child.get_style().margin.right.value as i32)
                    .unwrap_or(0);
                (p.region.x - inner_region.x) + p.region.width + margin_right
            })
            .max()
            .unwrap_or(0);

        // Update scroll state with virtual size and viewport
        {
            let mut scroll = self.scroll.borrow_mut();
            scroll.set_virtual_size(virtual_width, virtual_height);
            scroll.set_viewport(inner_region.width, inner_region.height);
        }

        // Determine if we need scrollbars and adjust content region
        let show_v_scrollbar = self.show_vertical_scrollbar();
        let content_region = self.content_region_for_scroll(inner_region);

        // If scrollbar visibility changed the content width, we need to recompute placements
        // and update virtual dimensions (important for width-based units like `w` in min-height: 40w)
        let placements = if show_v_scrollbar && content_region.width < inner_region.width {
            // Invalidate cache since we need to recalculate with smaller width
            *self.cached_layout.borrow_mut() = None;
            let new_placements = self.compute_child_placements(content_region, viewport);

            // Recalculate virtual dimensions with the new placements (content_region based)
            let new_virtual_height = new_placements
                .iter()
                .map(|p| {
                    let margin_bottom = self
                        .children
                        .get(p.child_index)
                        .map(|child| child.get_style().margin.bottom.value as i32)
                        .unwrap_or(0);
                    (p.region.y - content_region.y) + p.region.height + margin_bottom
                })
                .max()
                .unwrap_or(0);

            let new_virtual_width = new_placements
                .iter()
                .map(|p| {
                    let margin_right = self
                        .children
                        .get(p.child_index)
                        .map(|child| child.get_style().margin.right.value as i32)
                        .unwrap_or(0);
                    (p.region.x - content_region.x) + p.region.width + margin_right
                })
                .max()
                .unwrap_or(0);

            // Update scroll state with corrected virtual dimensions
            {
                let mut scroll = self.scroll.borrow_mut();
                scroll.set_virtual_size(new_virtual_width, new_virtual_height);
                scroll.set_viewport(content_region.width, content_region.height);
            }

            new_placements
        } else {
            initial_placements
        };

        // Get scroll offsets for rendering
        let (offset_x, offset_y) = {
            let scroll = self.scroll.borrow();
            (scroll.offset_x, scroll.offset_y)
        };

        // 6. Render children with scroll offset applied
        canvas.push_clip(content_region);
        for placement in &placements {
            let child = &self.children[placement.child_index];
            // Skip rendering if visibility is hidden (but widget still occupies space)
            if child.get_style().visibility == Visibility::Hidden {
                continue;
            }

            // Apply scroll offset to child region
            let scrolled_region = Region {
                x: placement.region.x - offset_x,
                y: placement.region.y - offset_y,
                width: placement.region.width,
                height: placement.region.height,
            };

            // Only render if at least partially visible (both horizontally and vertically)
            let visible_h = scrolled_region.x + scrolled_region.width > content_region.x
                && scrolled_region.x < content_region.x + content_region.width;
            let visible_v = scrolled_region.y + scrolled_region.height > content_region.y
                && scrolled_region.y < content_region.y + content_region.height;

            if visible_h && visible_v {
                child.render(canvas, scrolled_region);
            }
        }

        // 7. Render keylines on top of children (if enabled)
        if self.style.keyline.style != KeylineStyle::None {
            self.render_keylines(canvas, content_region, &placements);
        }

        canvas.pop_clip();

        // 8. Render vertical scrollbar if needed
        if self.render_vertical_scrollbar() {
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

        // 9. Render horizontal scrollbar if needed
        if self.render_horizontal_scrollbar() {
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
                Unit::Percent
                | Unit::ViewWidth
                | Unit::ViewHeight
                | Unit::Fraction
                | Unit::Width
                | Unit::Height => {
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
                Unit::Percent
                | Unit::ViewWidth
                | Unit::ViewHeight
                | Unit::Fraction
                | Unit::Width
                | Unit::Height => {
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

    fn content_height_for_scroll(&self, available_width: u16, _available_height: u16) -> u16 {
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
        let available_width = available_width.max(1);

        // Accumulate heights as f64 using the same floor arithmetic as vertical layout
        let mut current_y: f64 = 0.0;
        // Track previous margin bottom for CSS margin collapsing
        let mut prev_margin_bottom: f64 = 0.0;

        for (i, child) in self.children.iter().enumerate() {
            if !child.participates_in_layout() {
                continue;
            }
            let child_style = child.get_style();
            let child_desired = child.desired_size();

            // Resolve child width to compute intrinsic height for wrapped content.
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;
            let base_width = super::super::layouts::size_resolver::resolve_width_with_intrinsic(
                &child_style,
                child_desired.width,
                available_width as i32,
            );
            let should_reduce_by_margins = match &child_style.width {
                Some(w) => matches!(
                    w.unit,
                    Unit::Percent
                        | Unit::Width
                        | Unit::Height
                        | Unit::ViewWidth
                        | Unit::ViewHeight
                        | Unit::Fraction
                ),
                None => true,
            };
            let mut child_width = if should_reduce_by_margins {
                (base_width - margin_left - margin_right).max(0)
            } else {
                base_width
            };
            if let Some(max_w) = &child_style.max_width {
                let max_width_value = match max_w.unit {
                    Unit::Cells => max_w.value as i32,
                    Unit::Percent => ((max_w.value / 100.0) * available_width as f64) as i32,
                    Unit::Width => ((max_w.value / 100.0) * available_width as f64) as i32,
                    Unit::Height => ((max_w.value / 100.0) * viewport_height as f64) as i32,
                    Unit::ViewWidth => ((max_w.value / 100.0) * viewport_width as f64) as i32,
                    Unit::ViewHeight => ((max_w.value / 100.0) * viewport_height as f64) as i32,
                    _ => max_w.value as i32,
                };
                child_width = child_width.min(max_width_value);
            }
            if let Some(min_w) = &child_style.min_width {
                let min_width_value = match min_w.unit {
                    Unit::Cells => min_w.value as i32,
                    Unit::Percent => ((min_w.value / 100.0) * available_width as f64) as i32,
                    Unit::Width => ((min_w.value / 100.0) * available_width as f64) as i32,
                    Unit::Height => ((min_w.value / 100.0) * viewport_height as f64) as i32,
                    Unit::ViewWidth => ((min_w.value / 100.0) * viewport_width as f64) as i32,
                    Unit::ViewHeight => ((min_w.value / 100.0) * viewport_height as f64) as i32,
                    _ => min_w.value as i32,
                };
                child_width = child_width.max(min_width_value);
            }
            let child_width_u16 = child_width.clamp(0, u16::MAX as i32) as u16;

            // Get vertical margins from child style
            let margin_top = child_style.margin.top.value as f64;
            let margin_bottom = child_style.margin.bottom.value as f64;

            // CSS margin collapsing: adjacent margins collapse to the larger value
            // For the first child, use full top margin
            // For subsequent children, use max(current_top, prev_bottom) - prev_bottom
            let effective_top_margin = if i == 0 {
                margin_top
            } else {
                (margin_top - prev_margin_bottom).max(0.0)
            };

            // Apply top margin
            current_y += effective_top_margin;

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
                    Unit::ViewWidth => (h.value / 100.0) * viewport_width as f64,
                    Unit::ViewHeight => (h.value / 100.0) * viewport_height as f64,
                    Unit::Fraction => {
                        // fr units use their value as minimum height for scroll estimation
                        // 1fr = 1 row minimum, 2fr = 2 rows minimum, etc.
                        h.value.max(1.0)
                    }
                    Unit::Auto => {
                        // Auto uses intrinsic height based on resolved width
                        child.intrinsic_height_for_width(child_width_u16) as f64
                    }
                }
            } else if child_desired.height == u16::MAX {
                // No CSS height, child wants to fill - recurse
                child.content_height_for_scroll(available_width, viewport_height) as f64
            } else {
                // No CSS height - use intrinsic height based on available width
                child.intrinsic_height_for_width(child_width_u16) as f64
            };

            // Use floor arithmetic matching the vertical layout:
            // region.height = floor(next_y) - floor(y)
            let next_y = current_y + box_height;
            let _region_height = (next_y.floor() as i32 - current_y.floor() as i32).max(0);

            // Apply bottom margin and track for next iteration
            current_y = next_y + margin_bottom;
            prev_margin_bottom = margin_bottom;
        }

        // Total height is floor of final Y position
        let total_height = current_y.floor() as u16;

        // Add container chrome (border + padding)
        let border_size = if self.style.border.is_none() { 0 } else { 2 };
        let padding_v =
            self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;
        let result = total_height
            .saturating_add(border_size)
            .saturating_add(padding_v);

        result
    }

    fn content_width_for_scroll(&self, _available_width: u16) -> u16 {
        use tcss::types::Unit;

        // Calculate actual content width for scrolling, resolving percentage/fr units
        // This is used when desired_size returns u16::MAX (fill available space)
        // Use stored viewport dimensions to match layout calculations
        let viewport_width = self.viewport.width;
        let viewport_height = self.viewport.height;

        // Determine layout direction
        let is_horizontal = self.effective_layout() == LayoutDirection::Horizontal;

        if is_horizontal {
            // For horizontal layout, sum up all child widths (with floor arithmetic)
            let mut current_x: f64 = 0.0;

            for child in &self.children {
                if !child.participates_in_layout() {
                    continue;
                }
                let child_style = child.get_style();
                let child_desired = child.desired_size();

                // Calculate child width based on its CSS width property (as f64)
                let mut box_width: f64 = if let Some(w) = &child_style.width {
                    match w.unit {
                        Unit::Cells => w.value,
                        Unit::Percent | Unit::Width => (w.value / 100.0) * viewport_width as f64,
                        Unit::Height => (w.value / 100.0) * viewport_height as f64,
                        Unit::ViewWidth => (w.value / 100.0) * viewport_width as f64,
                        Unit::ViewHeight => (w.value / 100.0) * viewport_height as f64,
                        Unit::Fraction => {
                            // fr units use their value as minimum width for scroll estimation
                            w.value.max(1.0)
                        }
                        Unit::Auto => {
                            // Auto uses intrinsic width
                            if child_desired.width == u16::MAX {
                                child.content_width_for_scroll(viewport_width) as f64
                            } else {
                                child_desired.width as f64
                            }
                        }
                    }
                } else if child_desired.width == u16::MAX {
                    // No CSS width, child wants to fill - recurse
                    child.content_width_for_scroll(viewport_width) as f64
                } else {
                    child_desired.width as f64
                };

                // Apply min-width constraint
                if let Some(min_w) = &child_style.min_width {
                    let min_width_px: f64 = match min_w.unit {
                        Unit::Cells => min_w.value,
                        Unit::Percent | Unit::Width => {
                            (min_w.value / 100.0) * viewport_width as f64
                        }
                        Unit::Height => (min_w.value / 100.0) * viewport_height as f64,
                        Unit::ViewWidth => (min_w.value / 100.0) * viewport_width as f64,
                        Unit::ViewHeight => (min_w.value / 100.0) * viewport_height as f64,
                        _ => min_w.value,
                    };
                    box_width = box_width.max(min_width_px);
                }

                current_x += box_width;
            }

            // Total width is floor of final X position
            let total_width = current_x.floor() as u16;

            // Add container chrome (border + padding)
            let border_size = if self.style.border.is_none() { 0 } else { 2 };
            let padding_h =
                self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
            total_width
                .saturating_add(border_size)
                .saturating_add(padding_h)
        } else {
            // For vertical layout, find the maximum child width (with min-width applied)
            let mut max_width: f64 = 0.0;

            for child in &self.children {
                if !child.participates_in_layout() {
                    continue;
                }
                let child_style = child.get_style();
                let child_desired = child.desired_size();

                // Calculate child width based on its CSS width property
                let mut box_width: f64 = if let Some(w) = &child_style.width {
                    match w.unit {
                        Unit::Cells => w.value,
                        Unit::Percent | Unit::Width => (w.value / 100.0) * viewport_width as f64,
                        Unit::Height => (w.value / 100.0) * viewport_height as f64,
                        Unit::ViewWidth => (w.value / 100.0) * viewport_width as f64,
                        Unit::ViewHeight => (w.value / 100.0) * viewport_height as f64,
                        Unit::Fraction => {
                            // fr units - for max calculation, use the viewport width
                            viewport_width as f64
                        }
                        Unit::Auto => {
                            if child_desired.width == u16::MAX {
                                child.content_width_for_scroll(viewport_width) as f64
                            } else {
                                child_desired.width as f64
                            }
                        }
                    }
                } else if child_desired.width == u16::MAX {
                    child.content_width_for_scroll(viewport_width) as f64
                } else {
                    child_desired.width as f64
                };

                // Apply min-width constraint
                if let Some(min_w) = &child_style.min_width {
                    let min_width_px: f64 = match min_w.unit {
                        Unit::Cells => min_w.value,
                        Unit::Percent | Unit::Width => {
                            (min_w.value / 100.0) * viewport_width as f64
                        }
                        Unit::Height => (min_w.value / 100.0) * viewport_height as f64,
                        Unit::ViewWidth => (min_w.value / 100.0) * viewport_width as f64,
                        Unit::ViewHeight => (min_w.value / 100.0) * viewport_height as f64,
                        _ => min_w.value,
                    };
                    box_width = box_width.max(min_width_px);
                }

                max_width = max_width.max(box_width);
            }

            let total_width = max_width.floor() as u16;

            // Add container chrome (border + padding)
            let border_size = if self.style.border.is_none() { 0 } else { 2 };
            let padding_h =
                self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
            total_width
                .saturating_add(border_size)
                .saturating_add(padding_h)
        }
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Container",
            type_names: vec!["Container", "Widget", "DOMNode"],
            id: self.id.clone(),
            classes: self.classes.clone(),
            states: WidgetStates::empty(),
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
        // Invalidate layout cache since style may affect layout
        *self.cached_layout.borrow_mut() = None;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inline_style = style;
        self.dirty = true;
        *self.cached_layout.borrow_mut() = None;
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
        *self.cached_layout.borrow_mut() = None;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        // Invalidate layout cache when container is marked dirty
        *self.cached_layout.borrow_mut() = None;
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
        // Invalidate layout cache since viewport changed
        *self.cached_layout.borrow_mut() = None;
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

        // Handle mouse wheel for scrolling
        const SCROLL_AMOUNT: i32 = 3; // Scroll 3 lines per wheel event

        match event.kind {
            MouseEventKind::ScrollDown => {
                if self.show_vertical_scrollbar() || self.style.overflow_y == Overflow::Auto {
                    self.scroll.borrow_mut().scroll_down(SCROLL_AMOUNT);
                    self.dirty = true;
                    return None;
                }
            }
            MouseEventKind::ScrollUp => {
                if self.show_vertical_scrollbar() || self.style.overflow_y == Overflow::Auto {
                    self.scroll.borrow_mut().scroll_up(SCROLL_AMOUNT);
                    self.dirty = true;
                    return None;
                }
            }
            _ => {}
        }

        // Compute placements and dispatch mouse events
        // For mouse handling, approximate viewport as region (only available during render)
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);

        // Get scroll offset to adjust mouse coordinates for children
        let offset_y = self.scroll.borrow().offset_y;

        for placement in placements {
            // Adjust placement for scroll offset when checking hit
            let scrolled_placement = Region {
                x: placement.region.x,
                y: placement.region.y - offset_y,
                width: placement.region.width,
                height: placement.region.height,
            };

            if scrolled_placement.contains_point(mx, my) {
                if let Some(msg) =
                    self.children[placement.child_index].on_mouse(event, scrolled_placement)
                {
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
