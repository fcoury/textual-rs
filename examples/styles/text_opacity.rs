use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct TextOpacityApp;

impl App for TextOpacityApp {
    type Message = Message;

    const CSS: &'static str = include_str!("text_opacity.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("text-opacity: 0%", id: "zero-opacity")
            Label("text-opacity: 25%", id: "quarter-opacity")
            Label("text-opacity: 50%", id: "half-opacity")
            Label("text-opacity: 75%", id: "three-quarter-opacity")
            Label("text-opacity: 100%", id: "full-opacity")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = TextOpacityApp;
    app.run()
}
