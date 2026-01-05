use textual::{App, Grid, Label, Widget, ui};

const TEXT: &str = concat!(
    "I must not fear. Fear is the mind-killer. Fear is the little-death that ",
    "brings total obliteration. I will face my fear. I will permit it to pass over ",
    "me and through me."
);

#[derive(Clone)]
enum Message {}

struct TextAlign;

impl App for TextAlign {
    type Message = Message;

    const CSS: &'static str = include_str!("text_align.tcss");

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

fn main() -> textual::Result<()> {
    let mut app = TextAlign;
    app.run()
}
