use textual::{App, Compose, KeyCode, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MaxWidthApp {
    quit: bool,
}

impl MaxWidthApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for MaxWidthApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Placeholder(id: "p1", label: "max-width: 50h")
                Placeholder(id: "p2", label: "max-width: 999")
                Placeholder(id: "p3", label: "max-width: 50%")
                Placeholder(id: "p4", label: "max-width: 10")
            }
        }
    }
}

impl App for MaxWidthApp {
    const CSS: &'static str = include_str!("max_width.tcss");

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
    let mut app = MaxWidthApp::new();
    app.run()
}
