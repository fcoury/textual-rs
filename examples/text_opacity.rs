use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct TextOpacityApp {
    quit: bool,
}

impl TextOpacityApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for TextOpacityApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("text-opacity: 0%", id: "zero-opacity")
            Label("text-opacity: 25%", id: "quarter-opacity")
            Label("text-opacity: 50%", id: "half-opacity")
            Label("text-opacity: 75%", id: "three-quarter-opacity")
            Label("text-opacity: 100%", id: "full-opacity")
        }
    }
}

impl App for TextOpacityApp {
    const CSS: &'static str = include_str!("text_opacity.tcss");

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
    let mut app = TextOpacityApp::new();
    app.run()
}
