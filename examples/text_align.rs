use textual::{App, Compose, Grid, KeyCode, Label, Widget, ui};

const TEXT: &str = concat!(
    "I must not fear. Fear is the mind-killer. Fear is the little-death that ",
    "brings total obliteration. I will face my fear. I will permit it to pass over ",
    "me and through me."
);

#[derive(Clone)]
enum Message {}

struct TextAlign {
    quit: bool,
}

impl TextAlign {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for TextAlign {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        let text = TEXT;
        ui! {
            Grid {
                Label(format!("[b]Left aligned[/]\n{text}"), id: "one")
                Label(format!("[b]Center aligned[/]\n{text}"), id: "two")
                Label(format!("[b]Right aligned[/]\n{text}"), id: "three")
                Label(format!("[b]Justified[/]\n{text}"), id: "four")
            }
        }
    }
}

impl App for TextAlign {
    const CSS: &'static str = include_str!("text_align.tcss");

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
    let mut app = TextAlign::new();
    app.run()
}
