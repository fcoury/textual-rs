use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WrapApp {
    quit: bool,
}

impl WrapApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for WrapApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Widget 1")
            Label("Widget 2", classes: "invisible")
            Label("Widget 3")
        }
    }
}

impl App for WrapApp {
    const CSS: &'static str = include_str!("visibility.tcss");

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
    let mut app = WrapApp::new();
    app.run()
}
