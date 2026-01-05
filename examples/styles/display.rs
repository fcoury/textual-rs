use textual::{App, Compose, KeyCode, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct DisplayApp {
    quit: bool,
}

impl DisplayApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for DisplayApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("Widget 1")
            Static("Widget 2", classes: "remove")
            Static("Widget 3")
        }
    }
}

impl App for DisplayApp {
    const CSS: &'static str = include_str!("display.tcss");

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
    let mut app = DisplayApp::new();
    app.run()
}
