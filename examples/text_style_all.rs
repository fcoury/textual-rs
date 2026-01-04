use textual::{App, Compose, Grid, KeyCode, Label, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain."#;

#[derive(Clone)]
enum Message {}

struct OutlineApp {
    quit: bool,
}

impl OutlineApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for OutlineApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Label(format!("none\n{}", TEXT), id: "lbl1")
                Label(format!("bold\n{}", TEXT), id: "lbl2")
                Label(format!("italic\n{}", TEXT), id: "lbl3")
                Label(format!("reverse\n{}", TEXT), id: "lbl4")
                Label(format!("strike\n{}", TEXT), id: "lbl5")
                Label(format!("underline\n{}", TEXT), id: "lbl6")
                Label(format!("bold italic\n{}", TEXT), id: "lbl7")
                Label(format!("reverse strike\n{}", TEXT), id: "lbl8")
            }
        }
    }
}

impl App for OutlineApp {
    const CSS: &'static str = include_str!("text_style_all.tcss");

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
    let mut app = OutlineApp::new();
    app.run()
}
