use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct ColorApp {
    quit: bool,
}

impl ColorApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for ColorApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl1")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl2")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl3")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl4")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl5")
        }
    }
}

impl App for ColorApp {
    const CSS: &'static str = include_str!("color_auto.tcss");

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
    let mut app = ColorApp::new();
    app.run()
}
