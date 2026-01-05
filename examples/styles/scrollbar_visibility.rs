use textual::{App, Compose, Horizontal, Label, VerticalScroll, Widget, ui};

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

struct ScrollbarApp;

impl Compose for ScrollbarApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal  {
                VerticalScroll(classes: "left")  { Label(TEXT.repeat(10)) }
                VerticalScroll(classes: "right") { Label(TEXT.repeat(10)) }
            }
        }
    }
}

impl App for ScrollbarApp {
    const CSS: &'static str = include_str!("scrollbar_visibility.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = ScrollbarApp;
    app.run()
}
