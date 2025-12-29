use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BackgroundApp {
    quit: bool,
}

impl BackgroundApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for BackgroundApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Widget 1", id: "static1")
            Label("Widget 2", id: "static2")
            Label("Widget 3", id: "static3")
        }
    }
}

impl App for BackgroundApp {
    const CSS: &'static str = include_str!("background.tcss");

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
    let mut app = BackgroundApp::new();
    app.run()
}
