use textual::{App, Compose, Container, Grid, KeyCode, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MarginAllApp {
    quit: bool,
}

impl MarginAllApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for MarginAllApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Container(classes: "bordered") { Placeholder(id: "p1", label: "no margin") }
                Container(classes: "bordered") { Placeholder(id: "p2", label: "margin: 1") }
                Container(classes: "bordered") { Placeholder(id: "p3", label: "margin: 1 5") }
                Container(classes: "bordered") { Placeholder(id: "p4", label: "margin: 1 1 2 6") }
                Container(classes: "bordered") { Placeholder(id: "p5", label: "margin-top: 4") }
                Container(classes: "bordered") { Placeholder(id: "p6", label: "margin-right: 3") }
                Container(classes: "bordered") { Placeholder(id: "p7", label: "margin-bottom: 4") }
                Container(classes: "bordered") { Placeholder(id: "p8", label: "margin-left: 3") }
            }
        }
    }
}

impl App for MarginAllApp {
    const CSS: &'static str = include_str!("margin_all.tcss");

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
    let mut app = MarginAllApp::new();
    app.run()
}
