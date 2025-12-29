use textual::{App, Compose, KeyCode, Label, Vertical, Widget, ui};

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
            Vertical(id: "tint1") {
                Label("0%")
            }
            Vertical(id: "tint2") {
                Label("25%")
            }
            Vertical(id: "tint3") {
                Label("50%")
            }
            Vertical(id: "tint4") {
                Label("75%")
            }
            Vertical(id: "tint5") {
                Label("100%")
            }
        }
    }
}

impl App for BackgroundApp {
    const CSS: &'static str = include_str!("background_tint.tcss");

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
