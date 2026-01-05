use textual::{App, Compose, Horizontal, KeyCode, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct KeylineApp {
    quit: bool,
}

impl KeylineApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for KeylineApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                Placeholder()
                Placeholder()
                Placeholder()
            }
        }
    }
}

impl App for KeylineApp {
    const CSS: &'static str = include_str!("keyline_horizontal.tcss");

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
    textual::init_logger("height_comparison.log");
    let mut app = KeylineApp::new();
    app.run()
}
