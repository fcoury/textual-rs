//! Breakpoints Example - Responsive Grid Layout
//!
//! This example demonstrates responsive layouts using:
//! - **Grid container** with CSS-controlled column count
//! - **Placeholder widgets** with auto-cycling color palette
//! - **Custom breakpoints** that change layout based on terminal width
//! - **CSS `&` nesting** for clean, organized styles
//!
//! Resize your terminal to see the grid automatically adjust:
//! - Narrow (< 40): 1 column
//! - Normal (40-79): 2 columns
//! - Wide (80-119): 4 columns
//! - Very wide (>= 120): 6 columns
//!
//! Run with: cargo run --example breakpoints

use textual::{App, Grid, KeyCode, MessageEnvelope, Placeholder, Widget};

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

impl App for BreakpointApp {
    type Message = Message;

    // CSS with nested breakpoint rules
    // The `&` selector refers to the parent (Screen)
    const CSS: &'static str = r#"
        Screen {
            /* Default: 1 column for narrow screens */
            Grid {
                grid-size: 1;
                grid-gutter: 1;
            }

            /* Normal width: 2 columns */
            &.-normal {
                Grid {
                    grid-size: 2;
                }
            }

            /* Wide: 4 columns */
            &.-wide {
                Grid {
                    grid-size: 4;
                }
            }

            /* Very wide: 6 columns */
            &.-very-wide {
                Grid {
                    grid-size: 6;
                }
            }
        }

        Placeholder {
            height: 5;
        }
    "#;

    // Custom horizontal breakpoints for this app
    fn horizontal_breakpoints(&self) -> &'static [(u16, &'static str)] {
        &[
            (0, "-narrow"),      // < 40 columns
            (40, "-normal"),     // 40-79 columns
            (80, "-wide"),       // 80-119 columns
            (120, "-very-wide"), // >= 120 columns
        ]
    }

    fn handle_message(
        &mut self,
        _envelope: MessageEnvelope<Self::Message>,
        _ctx: &mut textual::EventContext<Self::Message>,
    ) {
    }

    fn on_key(&mut self, key: KeyCode, _ctx: &mut textual::EventContext<Self::Message>) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        // Create a grid of placeholders - the grid-size is controlled by CSS
        // based on the current breakpoint class
        vec![Box::new(Grid::new(vec![
            Box::new(Placeholder::new().with_label("Item 1")),
            Box::new(Placeholder::new().with_label("Item 2")),
            Box::new(Placeholder::new().with_label("Item 3")),
            Box::new(Placeholder::new().with_label("Item 4")),
            Box::new(Placeholder::new().with_label("Item 5")),
            Box::new(Placeholder::new().with_label("Item 6")),
            Box::new(Placeholder::new().with_label("Item 7")),
            Box::new(Placeholder::new().with_label("Item 8")),
            Box::new(Placeholder::new().with_label("Item 9")),
            Box::new(Placeholder::new().with_label("Item 10")),
            Box::new(Placeholder::new().with_label("Item 11")),
            Box::new(Placeholder::new().with_label("Item 12")),
        ]))]
    }
}

fn main() -> textual::Result<()> {
    let mut app = BreakpointApp::new();
    app.run()
}
