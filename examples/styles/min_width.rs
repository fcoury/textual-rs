use textual::{App, Compose, KeyCode, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MinWidthApp {
    quit: bool,
}

impl MinWidthApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for MinWidthApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Placeholder(id: "p1", label: "min-width: 25%")
                Placeholder(id: "p2", label: "min-width: 75%")
                Placeholder(id: "p3", label: "min-width: 100")
                Placeholder(id: "p4", label: "min-width: 400h")
            }
        }
    }
}

impl App for MinWidthApp {
    const CSS: &'static str = include_str!("min_width.tcss");

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
    let mut app = MinWidthApp::new();
    app.run()
}
