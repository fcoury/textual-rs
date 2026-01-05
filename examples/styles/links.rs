use textual::{App, Compose, KeyCode, Static, Widget, ui};

const TEXT: &str = "Here is a [@click='app.bell']link[/] which you can click!\n";

#[derive(Clone)]
enum Message {}

struct LinksApp {
    quit: bool,
}

impl LinksApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for LinksApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static(format!("{}", TEXT))
            Static(TEXT, id: "custom")
        }
    }
}

impl App for LinksApp {
    const CSS: &'static str = include_str!("links.tcss");

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
    let mut app = LinksApp::new();
    app.run()
}
