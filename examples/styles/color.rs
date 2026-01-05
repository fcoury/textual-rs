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
            Label("I'm red!", id: "label1")
            Label("I'm rgb(0, 255, 0)!", id: "label2")
            Label("I'm hsl(240, 100%, 50%)!", id: "label3")
        }
    }
}

impl App for ColorApp {
    const CSS: &'static str = include_str!("color.tcss");

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
