use textual::{App, Static, Widget, ui};

static TEXT: &str = "I must not fear. Fear is the mind-killer. Fear is the little-death that brings total obliteration. I will face my fear.";

#[derive(Clone)]
enum Message {}

struct WrapApp;

impl App for WrapApp {
    type Message = Message;

    const CSS: &'static str = include_str!("text_wrap.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static(TEXT, id: "static1")
            Static(TEXT, id: "static2")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = WrapApp;
    app.run()
}
