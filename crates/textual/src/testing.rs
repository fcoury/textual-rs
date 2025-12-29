//! Test utilities for snapshot testing textual applications.
//!
//! This module provides two testing approaches:
//!
//! 1. **Full app testing** via `render_to_canvas()` - Renders a complete `Compose`
//!    implementation with CSS styling and Screen wrapper
//!
//! 2. **Widget-level testing** via `TestCanvas` - Renders individual widgets
//!    in isolation for focused unit testing
//!
//! # Examples
//!
//! ## Full App Testing
//! ```ignore
//! let app = MyApp::new();
//! let canvas = render_to_canvas(&app, MyApp::CSS, 80, 24);
//! assert_snapshot!(canvas.to_snapshot());
//! ```
//!
//! ## Widget-Level Testing
//! ```ignore
//! use textual::testing::{TestCanvas, RenderSnapshot};
//! use textual::Static;
//!
//! #[test]
//! fn test_static_widget_renders_text() {
//!     let widget: Static<()> = Static::new("Hello, World!");
//!     let mut canvas = TestCanvas::new(20, 3);
//!     let snapshot = canvas.render_widget(&widget);
//!     insta::assert_snapshot!(snapshot.to_text());
//! }
//! ```

use crossterm::style::Color;

use crate::canvas::{Canvas, Cell, Region, TextAttributes};
use crate::widget::Widget;
use crate::{
    style_resolver::resolve_styles,
    tree::WidgetTree,
    widget::{screen::Screen, Compose},
    Size,
};

// =============================================================================
// Full App Testing (render_to_canvas)
// =============================================================================

/// Render a Compose implementation to a Canvas without running the event loop.
///
/// This function:
/// 1. Parses the CSS from the provided string
/// 2. Builds a widget tree wrapped in Screen
/// 3. Resolves styles using the textual-dark theme
/// 4. Renders to a Canvas
///
/// # Example
/// ```ignore
/// let app = MyApp::new();
/// let canvas = render_to_canvas(&app, "MyApp::CSS", 80, 24);
/// assert_snapshot!(canvas.to_snapshot());
/// ```
pub fn render_to_canvas<T, M>(app: &T, css: &str, width: u16, height: u16) -> Canvas
where
    T: Compose<Message = M>,
    M: Send + 'static,
{
    // Parse CSS
    let stylesheet = tcss::parser::parse_stylesheet(css).expect("CSS parsing failed");
    let themes = tcss::types::Theme::standard_themes();
    let theme = themes
        .get("textual-dark")
        .cloned()
        .unwrap_or_else(|| tcss::types::Theme::new("default", true));

    // Build widget tree (wrapped in implicit Screen)
    let root = Box::new(Screen::new(app.compose()));
    let mut tree = WidgetTree::new(root);

    // Initialize Screen with size for breakpoints
    tree.root_mut().on_resize(Size::new(width, height));

    // Resolve styles
    let mut ancestors = Vec::new();
    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

    // Render to canvas
    let mut canvas = Canvas::new(width, height);
    let region = Region::from_u16(0, 0, width, height);
    tree.root().render(&mut canvas, region);

    canvas
}

// =============================================================================
// Widget-Level Testing (TestCanvas + RenderSnapshot)
// =============================================================================

/// A test-focused canvas wrapper that provides snapshot-friendly output.
pub struct TestCanvas {
    canvas: Canvas,
    width: u16,
    height: u16,
}

/// Captured rendering output from a widget.
pub struct RenderSnapshot {
    cells: Vec<Vec<CellSnapshot>>,
    width: usize,
    height: usize,
}

/// A single cell's captured state.
#[derive(Clone, Debug)]
pub struct CellSnapshot {
    pub symbol: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attrs: TextAttributes,
}

impl From<&Cell> for CellSnapshot {
    fn from(cell: &Cell) -> Self {
        Self {
            symbol: cell.symbol,
            fg: cell.fg,
            bg: cell.bg,
            attrs: cell.attrs,
        }
    }
}

impl TestCanvas {
    /// Create a new test canvas with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            canvas: Canvas::new(width, height),
            width,
            height,
        }
    }

    /// Render a widget to the canvas and capture the result.
    ///
    /// The widget is rendered at position (0, 0) filling the entire canvas.
    pub fn render_widget<M>(&mut self, widget: &dyn Widget<M>) -> RenderSnapshot {
        self.render_widget_at(widget, Region::from_u16(0, 0, self.width, self.height))
    }

    /// Render a widget to a specific region and capture the result.
    pub fn render_widget_at<M>(&mut self, widget: &dyn Widget<M>, region: Region) -> RenderSnapshot {
        self.canvas.clear();
        widget.render(&mut self.canvas, region);
        self.capture()
    }

    /// Access the underlying canvas for advanced use cases.
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Access the underlying canvas mutably.
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    /// Capture the current canvas state as a snapshot.
    fn capture(&self) -> RenderSnapshot {
        let mut cells = Vec::with_capacity(self.height as usize);

        for y in 0..self.height as i32 {
            let mut row = Vec::with_capacity(self.width as usize);
            for x in 0..self.width as i32 {
                if let Some(cell) = self.canvas.get_cell_at(x, y) {
                    row.push(CellSnapshot::from(cell));
                } else {
                    row.push(CellSnapshot {
                        symbol: ' ',
                        fg: None,
                        bg: None,
                        attrs: TextAttributes::default(),
                    });
                }
            }
            cells.push(row);
        }

        RenderSnapshot {
            cells,
            width: self.width as usize,
            height: self.height as usize,
        }
    }
}

impl RenderSnapshot {
    /// Convert to text-only format (just characters).
    ///
    /// This produces a simple string representation where each line
    /// represents one row of the canvas.
    pub fn to_text(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().map(|c| c.symbol).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert to text with trailing spaces trimmed.
    ///
    /// Useful for cleaner snapshots when widgets don't fill the canvas.
    pub fn to_text_trimmed(&self) -> String {
        self.cells
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| c.symbol)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    /// Convert to styled format with color and attribute information.
    ///
    /// Format: Each character is followed by style info in brackets if styled.
    /// Example: `H[fg:red,bold]e[fg:red,bold]l[fg:red,bold]`
    pub fn to_styled(&self) -> String {
        self.cells
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| format_styled_cell(c))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get a specific row as text.
    pub fn row(&self, y: usize) -> Option<String> {
        self.cells.get(y).map(|row| row.iter().map(|c| c.symbol).collect())
    }

    /// Get a specific row as text, trimmed.
    pub fn row_trimmed(&self, y: usize) -> Option<String> {
        self.row(y).map(|s| s.trim_end().to_string())
    }

    /// Get a specific cell.
    pub fn cell(&self, x: usize, y: usize) -> Option<&CellSnapshot> {
        self.cells.get(y).and_then(|row| row.get(x))
    }

    /// Check if any cell has styling applied.
    pub fn has_styling(&self) -> bool {
        self.cells.iter().flatten().any(|c| {
            c.fg.is_some()
                || c.bg.is_some()
                || c.attrs.bold
                || c.attrs.italic
                || c.attrs.underline
                || c.attrs.dim
                || c.attrs.strike
                || c.attrs.reverse
        })
    }

    /// Get the width of the snapshot.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the snapshot.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Check if a specific cell has a foreground color.
    pub fn has_fg_at(&self, x: usize, y: usize) -> bool {
        self.cell(x, y).map(|c| c.fg.is_some()).unwrap_or(false)
    }

    /// Check if a specific cell has a background color.
    pub fn has_bg_at(&self, x: usize, y: usize) -> bool {
        self.cell(x, y).map(|c| c.bg.is_some()).unwrap_or(false)
    }

    /// Check if a specific cell is bold.
    pub fn is_bold_at(&self, x: usize, y: usize) -> bool {
        self.cell(x, y).map(|c| c.attrs.bold).unwrap_or(false)
    }

    /// Extract a rectangular region as text.
    pub fn region_text(&self, x: usize, y: usize, width: usize, height: usize) -> String {
        let mut result = Vec::new();
        for row_idx in y..(y + height).min(self.height) {
            if let Some(row) = self.cells.get(row_idx) {
                let line: String = row
                    .iter()
                    .skip(x)
                    .take(width)
                    .map(|c| c.symbol)
                    .collect();
                result.push(line);
            }
        }
        result.join("\n")
    }
}

/// Format a single cell with its style information.
fn format_styled_cell(cell: &CellSnapshot) -> String {
    let mut style_parts = Vec::new();

    if let Some(fg) = &cell.fg {
        style_parts.push(format!("fg:{}", color_to_string(fg)));
    }
    if let Some(bg) = &cell.bg {
        style_parts.push(format!("bg:{}", color_to_string(bg)));
    }
    if cell.attrs.bold {
        style_parts.push("bold".to_string());
    }
    if cell.attrs.italic {
        style_parts.push("italic".to_string());
    }
    if cell.attrs.underline {
        style_parts.push("underline".to_string());
    }
    if cell.attrs.dim {
        style_parts.push("dim".to_string());
    }
    if cell.attrs.strike {
        style_parts.push("strike".to_string());
    }
    if cell.attrs.reverse {
        style_parts.push("reverse".to_string());
    }

    if style_parts.is_empty() {
        cell.symbol.to_string()
    } else {
        format!("{}[{}]", cell.symbol, style_parts.join(","))
    }
}

/// Convert a crossterm Color to a readable string.
fn color_to_string(color: &Color) -> String {
    match color {
        Color::Reset => "reset".to_string(),
        Color::Black => "black".to_string(),
        Color::DarkGrey => "darkgrey".to_string(),
        Color::Red => "red".to_string(),
        Color::DarkRed => "darkred".to_string(),
        Color::Green => "green".to_string(),
        Color::DarkGreen => "darkgreen".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::DarkYellow => "darkyellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::DarkBlue => "darkblue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::DarkMagenta => "darkmagenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::DarkCyan => "darkcyan".to_string(),
        Color::White => "white".to_string(),
        Color::Grey => "grey".to_string(),
        Color::Rgb { r, g, b } => format!("#{:02x}{:02x}{:02x}", r, g, b),
        Color::AnsiValue(v) => format!("ansi({})", v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Static;

    #[test]
    fn test_canvas_renders_static_widget() {
        let widget: Static<()> = Static::new("Hello");
        let mut canvas = TestCanvas::new(10, 1);
        let snapshot = canvas.render_widget(&widget);

        assert_eq!(snapshot.to_text_trimmed(), "Hello");
    }

    #[test]
    fn test_canvas_renders_multiline() {
        let widget: Static<()> = Static::new("Line 1\nLine 2");
        let mut canvas = TestCanvas::new(10, 3);
        let snapshot = canvas.render_widget(&widget);

        assert!(snapshot.to_text_trimmed().contains("Line 1"));
        assert!(snapshot.to_text_trimmed().contains("Line 2"));
    }

    #[test]
    fn test_cell_access() {
        let widget: Static<()> = Static::new("ABC");
        let mut canvas = TestCanvas::new(5, 1);
        let snapshot = canvas.render_widget(&widget);

        assert_eq!(snapshot.cell(0, 0).unwrap().symbol, 'A');
        assert_eq!(snapshot.cell(1, 0).unwrap().symbol, 'B');
        assert_eq!(snapshot.cell(2, 0).unwrap().symbol, 'C');
    }

    #[test]
    fn test_row_access() {
        let widget: Static<()> = Static::new("Test");
        let mut canvas = TestCanvas::new(10, 1);
        let snapshot = canvas.render_widget(&widget);

        assert_eq!(snapshot.row_trimmed(0), Some("Test".to_string()));
    }

    #[test]
    fn test_region_extraction() {
        let widget: Static<()> = Static::new("Hello World");
        let mut canvas = TestCanvas::new(15, 1);
        let snapshot = canvas.render_widget(&widget);

        // Extract "World" (positions 6-10)
        let region = snapshot.region_text(6, 0, 5, 1);
        assert_eq!(region, "World");
    }
}
