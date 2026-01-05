use textual::{App, Compose, Static, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain."#;

#[derive(Clone)]
enum Message {}

struct ScrollbarGutterApp;

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
}

fn main() -> textual::Result<()> {
    let mut app = ScrollbarGutterApp;
    app.run()
}
