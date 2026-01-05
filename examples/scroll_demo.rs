//! Scroll Demo - Demonstrates the ScrollableContainer with scrollbars.
//!
//! Run with: cargo run --example scroll_demo
//!
//! Controls:
//! - Up/Down arrows: scroll content
//! - Page Up/Page Down: scroll by page
//! - Mouse wheel: scroll content
//! - Click scrollbar track: jump scroll
//! - Drag scrollbar thumb: smooth scroll
//! - 'q': quit

use textual::canvas::TextAttributes;
use textual::containers::scrollable::ScrollableContainer;
use textual::widget::Widget;
use textual::{App, Canvas, KeyCode, MessageEnvelope, Region, Result, Size};

/// A simple widget that renders multiple lines of text for scrolling demo.
struct TextList {
    lines: Vec<String>,
    dirty: bool,
}

impl TextList {
    fn new(lines: Vec<String>) -> Self {
        Self { lines, dirty: true }
    }
}

impl<M> Widget<M> for TextList {
    fn desired_size(&self) -> Size {
        // Width is the longest line, height is number of lines
        let width = self.lines.iter().map(|l| l.len()).max().unwrap_or(0);
        Size {
            width: width as u16,
            height: self.lines.len() as u16,
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        for (i, line) in self.lines.iter().enumerate() {
            let y = region.y + i as i32;
            if y >= region.y && y < region.y + region.height {
                canvas.put_str(region.x, y, line, None, None, TextAttributes::default());
            }
        }
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
}

enum Message {}

struct ScrollApp {
    running: bool,
}

impl ScrollApp {
    fn new() -> Self {
        Self { running: true }
    }
}

impl App for ScrollApp {
    type Message = Message;

    const CSS: &'static str = r#"
        ScrollableContainer {
            overflow-y: auto;
            overflow-x: hidden;
            scrollbar-color: #00CCFF;
            scrollbar-color-hover: #66DDFF;
            scrollbar-color-active: #FFFFFF;
            scrollbar-background: #333333;
            scrollbar-background-hover: #444444;
            scrollbar-size-vertical: 1;
            scrollbar-size-horizontal: 0;
        }
    "#;

    fn on_key(&mut self, key: KeyCode, _ctx: &mut textual::EventContext<Self::Message>) {
        if key == KeyCode::Char('q') {
            self.running = false;
        }
    }

    fn should_quit(&self) -> bool {
        !self.running
    }

    fn handle_message(
        &mut self,
        _envelope: MessageEnvelope<Message>,
        _ctx: &mut textual::EventContext<Self::Message>,
    ) {
        // No messages from TextList
    }

    fn compose(&self) -> Vec<Box<dyn Widget<Message>>> {
        // Create 50 lines of content to scroll through
        let lines: Vec<String> = (1..=50)
            .map(|i| {
                let emoji = match i % 5 {
                    0 => "★",
                    1 => "●",
                    2 => "◆",
                    3 => "▲",
                    _ => "■",
                };
                format!(
                    " {} Line {:02}: This is a scrollable line of text.",
                    emoji, i
                )
            })
            .collect();

        let content = TextList::new(lines);

        // Wrap in ScrollableContainer
        vec![Box::new(ScrollableContainer::from_child(Box::new(content)))]
    }
}

fn main() -> Result<()> {
    let mut app = ScrollApp::new();
    app.run()
}
