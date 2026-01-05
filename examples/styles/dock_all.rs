use textual::{App, Compose, Container, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct DockAllApp {
    quit: bool,
}

impl DockAllApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for DockAllApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Container(id: "big_container") {
                Container(id: "left") { Label("left") }
                Container(id: "top") { Label("top") }
                Container(id: "right") { Label("right") }
                Container(id: "bottom") { Label("bottom") }
            }
        }
    }
}

impl App for DockAllApp {
    const CSS: &'static str = include_str!("dock_all.tcss");

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
    let mut app = DockAllApp::new();
    app.run()
}
