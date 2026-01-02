use textual::{App, Compose, KeyCode, Static, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain."#;

#[derive(Clone)]
enum Message {}

struct ScrollbarGutterApp {
    quit: bool,
}

impl ScrollbarGutterApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for ScrollbarGutterApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static(TEXT, id: "text-box")
        }
    }
}

impl App for ScrollbarGutterApp {
    const CSS: &'static str = include_str!("scrollbar_gutter.tcss");

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
    let mut app = ScrollbarGutterApp::new();
    app.run()
}
