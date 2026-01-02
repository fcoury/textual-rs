use textual::{App, Compose, KeyCode, Label, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain.
"#;

#[derive(Clone)]
enum Message {}

struct Scrollbar2App {
    quit: bool,
}

impl Scrollbar2App {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for Scrollbar2App {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label(TEXT.repeat(10))
        }
    }
}

impl App for Scrollbar2App {
    const CSS: &'static str = include_str!("scrollbars2.tcss");

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
    let mut app = Scrollbar2App::new();
    app.run()
}
