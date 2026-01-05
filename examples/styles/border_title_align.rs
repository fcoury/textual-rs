use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BorderTitleAlignApp {
    quit: bool,
}

impl BorderTitleAlignApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for BorderTitleAlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("My title is on the left.", id:"label1", border_title: "< Left")
            Label("My title is centered", id:"label2", border_title: "Centered!")
            Label("My title is on the right", id:"label3", border_title: "Right >")
        }
    }
}

impl App for BorderTitleAlignApp {
    const CSS: &'static str = include_str!("border_title_align.tcss");

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
    let mut app = BorderTitleAlignApp::new();
    app.run()
}
