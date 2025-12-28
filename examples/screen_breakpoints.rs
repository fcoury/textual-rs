//! Screen Breakpoints Demo
//!
//! This example demonstrates how the implicit Screen widget provides
//! responsive CSS classes based on terminal size:
//!
//! - `-narrow` (width < 80) / `-wide` (width >= 80)
//! - `-short` (height < 24) / `-tall` (height >= 24)
//!
//! Try resizing your terminal to see the styles change!
//!
//! Run with: cargo run --example screen_breakpoints

use textual::{
    App, Canvas, Compose, KeyCode, MessageEnvelope, Region, Size, Widget,
};

// A simple label widget that displays text
struct Label {
    text: String,
    id: Option<String>,
}

impl Label {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            id: None,
        }
    }

    fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<M> Widget<M> for Label {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        canvas.put_str(region.x, region.y, &self.text, None, None);
    }

    fn desired_size(&self) -> Size {
        Size::new(self.text.len() as u16, 1)
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

// A box that shows its size - demonstrates resize events
struct SizeDisplay {
    width: u16,
    height: u16,
}

impl SizeDisplay {
    fn new() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

impl<M> Widget<M> for SizeDisplay {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        let text = format!(
            "Terminal: {}x{} | Breakpoints: {} {}",
            self.width,
            self.height,
            if self.width < 80 { "-narrow" } else { "-wide" },
            if self.height < 24 { "-short" } else { "-tall" },
        );
        canvas.put_str(region.x, region.y, &text, None, None);
    }

    fn desired_size(&self) -> Size {
        Size::new(60, 1)
    }

    fn on_resize(&mut self, size: Size) {
        self.width = size.width;
        self.height = size.height;
    }
}

#[derive(Clone)]
enum Message {}

struct BreakpointApp {
    quit: bool,
}

impl BreakpointApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for BreakpointApp {
    type Message = Message;

    fn compose(&self) -> Box<dyn Widget<Self::Message>> {
        use textual::Vertical;

        Box::new(Vertical::new(vec![
            Box::new(Label::new("=== Screen Breakpoints Demo ===").with_id("title")),
            Box::new(Label::new("")),
            Box::new(SizeDisplay::new()),
            Box::new(Label::new("")),
            Box::new(Label::new("The Screen widget automatically adds CSS classes:")),
            Box::new(Label::new("  -narrow / -wide   (threshold: 80 columns)")),
            Box::new(Label::new("  -short  / -tall   (threshold: 24 rows)")),
            Box::new(Label::new("")),
            Box::new(Label::new("Try resizing your terminal window!")),
            Box::new(Label::new("")),
            Box::new(Label::new("CSS example:")),
            Box::new(Label::new("  Screen.-narrow .sidebar { display: none; }")),
            Box::new(Label::new("  Screen.-wide .sidebar { width: 25%; }")),
            Box::new(Label::new("")),
            Box::new(Label::new("Press 'q' to quit")),
        ]))
    }
}

impl App for BreakpointApp {
    const CSS: &'static str = r#"
        Screen {
            background: #1a1a2e;
        }

        /* These styles would apply based on breakpoints */
        Screen.-narrow Label {
            color: #e94560;
        }

        Screen.-wide Label {
            color: #0f3460;
        }
    "#;

    fn handle_message(&mut self, _envelope: MessageEnvelope<Self::Message>) {}

    fn on_key(&mut self, key: KeyCode) {
        if key == KeyCode::Char('q') {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> textual::Result<()> {
    let mut app = BreakpointApp::new();
    app.run()
}
