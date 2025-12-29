use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AlignApp {
    quit: bool,
}

impl AlignApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for AlignApp {
    type Message = Message;

    fn compose(&self) -> Box<dyn Widget<Self::Message>> {
        ui! {
            Label("Vertical alignment with [b]Textual[/]", classes: "box")
            Label("Take note, browsers.", classes: "box")
        }
    }
}

impl App for AlignApp {
    const CSS: &'static str = include_str!("align.tcss");

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
    let mut app = AlignApp::new();
    app.run()
}
