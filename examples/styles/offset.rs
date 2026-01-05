use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct OffsetApp {
    quit: bool,
}

impl OffsetApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for OffsetApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Paul (offset 8 2)", classes: "paul")
            Label("Duncan (offset 4 10)", classes: "duncan")
            Label("Chani (offset 0 -3)", classes: "chani")
        }
    }
}

impl App for OffsetApp {
    const CSS: &'static str = include_str!("offset.tcss");

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
    let mut app = OffsetApp::new();
    app.run()
}
