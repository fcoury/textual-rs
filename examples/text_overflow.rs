use textual::{App, Compose, KeyCode, Static, Widget, ui};

static TEXT: &str = "I must not fear. Fear is the mind-killer. Fear is the little-death that brings total obliteration. I will face my fear.";

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
            Static(TEXT, id: "static1")
            Static(TEXT, id: "static2")
            Static(TEXT, id: "static3")
        }
    }
}

impl App for WrapApp {
    const CSS: &'static str = include_str!("text_overflow.tcss");

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
