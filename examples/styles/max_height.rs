use textual::{App, Compose, Horizontal, KeyCode, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MaxHeightApp {
    quit: bool,
}

impl MaxHeightApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for MaxHeightApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                Placeholder(id: "p1", label: "max-height: 10w")
                Placeholder(id: "p2", label: "max-height: 999")
                Placeholder(id: "p3", label: "max-height: 50%")
                Placeholder(id: "p4", label: "max-height: 10")
            }
        }
    }
}

impl App for MaxHeightApp {
    const CSS: &'static str = include_str!("max_height.tcss");

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
    let mut app = MaxHeightApp::new();
    app.run()
}
