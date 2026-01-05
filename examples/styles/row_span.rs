use textual::{App, Compose, Grid, KeyCode, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MyApp {
    quit: bool,
}

impl MyApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for MyApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Placeholder(id: "p1")
                Placeholder(id: "p2")
                Placeholder(id: "p3")
                Placeholder(id: "p4")
                Placeholder(id: "p5")
                Placeholder(id: "p6")
                Placeholder(id: "p7")
            }
        }
    }
}

impl App for MyApp {
    const CSS: &'static str = include_str!("row_span.tcss");

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
    let mut app = MyApp::new();
    app.run()
}
