use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct ContentAlignApp {
    quit: bool,
}

impl ContentAlignApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for ContentAlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("With [i]content-align[/] you can...", id: "box1")
            Label("...[b]Easily align content[/]...", id: "box2")
            Label("...Horizontally [i]and[/] vertically!", id: "box3")
        }
    }
}

impl App for ContentAlignApp {
    const CSS: &'static str = include_str!("content_align.tcss");

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
    let mut app = ContentAlignApp::new();
    app.run()
}
