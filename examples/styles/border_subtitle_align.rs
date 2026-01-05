use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BorderSubtitleAlignApp {
    quit: bool,
}

impl BorderSubtitleAlignApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for BorderSubtitleAlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("My subtitle is on the left.", id:"label1", border_subtitle: "< Left")
            Label("My subtitle is centered", id:"label2", border_subtitle: "Centered!")
            Label("My subtitle is on the right", id:"label3", border_subtitle: "Right >")
        }
    }
}

impl App for BorderSubtitleAlignApp {
    const CSS: &'static str = include_str!("border_subtitle_align.tcss");

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
    let mut app = BorderSubtitleAlignApp::new();
    app.run()
}
