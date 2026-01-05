use textual::{App, Compose, KeyCode, Placeholder, Ruler, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct HeightApp {
    quit: bool,
}

impl HeightApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for HeightApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Placeholder(id: "cells")
                Placeholder(id: "percent")
                Placeholder(id: "w")
                Placeholder(id: "h")
                Placeholder(id: "vw")
                Placeholder(id: "vh")
                Placeholder(id: "auto")
                Placeholder(id: "fr1")
                Placeholder(id: "fr2")
            }
            Ruler {}
        }
    }
}

impl App for HeightApp {
    const CSS: &'static str = include_str!("height_comparison.tcss");

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
    textual::init_logger("height_comparison.log");
    let mut app = HeightApp::new();
    app.run()
}
