use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;
use crate::widget::static_widget::Static;
use tcss::types::geometry::Unit;
use tcss::types::{Position, Scalar, Visibility};
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

/// A simple tooltip overlay widget.
///
/// This is a lightweight Static wrapper that can be positioned via offsets.
pub struct Tooltip<M> {
    inner: Static<M>,
    visible: bool,
    offset: (i32, i32),
}

impl<M: 'static> Tooltip<M> {
    pub fn new() -> Self {
        let inner = Static::new("").with_markup(false);
        Self {
            inner,
            visible: false,
            offset: (0, 0),
        }
    }

    pub fn show(&mut self, content: impl Into<String>) {
        self.visible = true;
        self.inner.update(content);
        self.inner.mark_dirty();
    }

    pub fn hide(&mut self) {
        if self.visible {
            self.visible = false;
            self.inner.mark_dirty();
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        if self.offset != (x, y) {
            self.offset = (x, y);
            self.inner.mark_dirty();
        }
    }
}

impl<M: 'static> Default for Tooltip<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> Widget<M> for Tooltip<M> {
    fn default_css(&self) -> &'static str {
        r#"
Tooltip {
    position: absolute;
    padding: 1 2;
    background: $panel;
    width: auto;
    height: auto;
    max-width: 40;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.visible || self.get_style().visibility == Visibility::Hidden {
            return;
        }
        let style = self.inner.get_style();
        let desired = self.inner.desired_size();

        let mut width = if desired.width == u16::MAX {
            region.width.max(0) as u16
        } else {
            desired.width
        } as i32;

        if let Some(max_w) = &style.max_width {
            let max_width = resolve_scalar_to_cells(max_w, region, false);
            width = width.min(max_width);
        }

        if let Some(min_w) = &style.min_width {
            let min_width = resolve_scalar_to_cells(min_w, region, false);
            width = width.max(min_width);
        }

        let mut height = if let Some(h) = &style.height {
            if h.unit == Unit::Auto {
                self.inner.intrinsic_height_for_width(width.max(0) as u16) as i32
            } else {
                resolve_scalar_to_cells(h, region, true)
            }
        } else {
            self.inner.intrinsic_height_for_width(width.max(0) as u16) as i32
        };

        if let Some(max_h) = &style.max_height {
            let max_height = resolve_scalar_to_cells(max_h, region, true);
            height = height.min(max_height);
        }

        if let Some(min_h) = &style.min_height {
            let min_height = resolve_scalar_to_cells(min_h, region, true);
            height = height.max(min_height);
        }

        let tooltip_region =
            Region::new(self.offset.0, self.offset.1, width, height).intersection(&region);
        if tooltip_region.is_empty() {
            return;
        }
        self.inner.render(canvas, tooltip_region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "Tooltip";
        meta.type_names = vec!["Tooltip", "Static", "Widget", "DOMNode"];
        meta.states = WidgetStates::empty();
        meta
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn set_style(&mut self, mut style: ComputedStyle) {
        style.position = Position::Absolute;
        style.offset_x = Some(Scalar::cells(self.offset.0 as f64));
        style.offset_y = Some(Scalar::cells(self.offset.1 as f64));
        self.inner.set_style(style);
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inner.set_inline_style(style);
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }

    fn get_state(&self) -> WidgetStates {
        WidgetStates::empty()
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

fn resolve_scalar_to_cells(value: &Scalar, available: Region, is_height: bool) -> i32 {
    let main = if is_height {
        available.height
    } else {
        available.width
    };
    match value.unit {
        Unit::Cells => value.value as i32,
        Unit::Percent => ((value.value / 100.0) * main as f64).round() as i32,
        Unit::Width => ((value.value / 100.0) * available.width as f64).round() as i32,
        Unit::Height => ((value.value / 100.0) * available.height as f64).round() as i32,
        Unit::ViewWidth => ((value.value / 100.0) * available.width as f64).round() as i32,
        Unit::ViewHeight => ((value.value / 100.0) * available.height as f64).round() as i32,
        Unit::Fraction => main,
        Unit::Auto => main,
    }
}
