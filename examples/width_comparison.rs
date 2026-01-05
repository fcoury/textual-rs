use textual::{App, Compose, Horizontal, KeyCode, Placeholder, Ruler, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WidthComparisonApp {
    quit: bool,
}

impl WidthComparisonApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for WidthComparisonApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        let mut widgets = ui! {
            Horizontal {
                Placeholder(id: "cells")
                Placeholder(id: "percent")
                Placeholder(id: "w")
                Placeholder(id: "h")
                Placeholder(id: "vw")
                Placeholder(id: "vh")
                Placeholder(id: "auto")
                Placeholder(id: "fr1")
                Placeholder(id: "fr3")
            }
        };

        widgets.push(Box::new(Ruler::horizontal()));
        widgets
    }
}

impl App for WidthComparisonApp {
    const CSS: &'static str = include_str!("width_comparison.tcss");

    fn on_key(&mut self, key: textual::KeyCode) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> textual::Result<()> {
    let mut app = WidthComparisonApp::new();
    app.run()
}
