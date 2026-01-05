use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct PositionApp {
    quit: bool,
}

impl PositionApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for PositionApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Absolute", id: "label1")
            Label("Relative", id: "label2")
        }
    }
}

impl App for PositionApp {
    const CSS: &'static str = include_str!("position.tcss");

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
    let mut app = PositionApp::new();
    app.run()
}
