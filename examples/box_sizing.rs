use textual::{App, Compose, KeyCode, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BoxSizingApp {
    quit: bool,
}

impl BoxSizingApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for BoxSizingApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("I'm using border-box!", id: "static1")
            Static("I'm using content-box!", id: "static2")
        }
    }
}

impl App for BoxSizingApp {
    const CSS: &'static str = include_str!("box_sizing.tcss");

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
    let mut app = BoxSizingApp::new();
    app.run()
}
