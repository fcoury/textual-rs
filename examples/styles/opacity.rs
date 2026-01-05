use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct OpacityApp {
    quit: bool,
}

impl OpacityApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for OpacityApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("opacity: 0%", id: "zero-opacity")
            Label("opacity: 25%", id: "quarter-opacity")
            Label("opacity: 50%", id: "half-opacity")
            Label("opacity: 75%", id: "three-quarter-opacity")
            Label("opacity: 100%", id: "full-opacity")
        }
    }
}

impl App for OpacityApp {
    const CSS: &'static str = include_str!("opacity.tcss");

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
    let mut app = OpacityApp::new();
    app.run()
}
