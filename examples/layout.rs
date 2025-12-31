use textual::{App, Compose, Container, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct LayoutApp {
    quit: bool,
}

impl LayoutApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for LayoutApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Container(id: "vertical-layout") {
                Label("Layout")
                Label("Is")
                Label("Vertical")
            }
            Container(id: "horizontal-layout") {
                Label("Layout")
                Label("Is")
                Label("Horizontal")
            }
        }
    }
}

impl App for LayoutApp {
    const CSS: &'static str = include_str!("layout.tcss");

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
    let mut app = LayoutApp::new();
    app.run()
}
