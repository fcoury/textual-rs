use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BorderApp {
    quit: bool,
}

impl BorderApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for BorderApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("My border is solid red", id: "label1")
            Label("My border is dashed green", id: "label2")
            Label("My border is tall blue", id: "label3")
        }
    }
}

impl App for BorderApp {
    const CSS: &'static str = include_str!("border.tcss");

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
    let mut app = BorderApp::new();
    app.run()
}
