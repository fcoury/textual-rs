use textual::{App, Label, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain."#;

#[derive(Clone)]
enum Message {}

struct OutlineApp;

impl App for OutlineApp {
    type Message = Message;

    const CSS: &'static str = include_str!("text_style.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label(TEXT, id: "lbl1")
            Label(TEXT, id: "lbl2")
            Label(TEXT, id: "lbl3")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = OutlineApp;
    app.run()
}
