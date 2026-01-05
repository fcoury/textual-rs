use textual::{App, Compose, Static, Widget, ui};

static TEXT: &str = "I must not fear. Fear is the mind-killer. Fear is the little-death that brings total obliteration. I will face my fear.";

#[derive(Clone)]
enum Message {}

struct WrapApp;

impl Compose for WrapApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static(TEXT, id: "static1")
            Static(TEXT, id: "static2")
        }
    }
}

impl App for WrapApp {
    const CSS: &'static str = include_str!("text_wrap.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = WrapApp;
    app.run()
}
