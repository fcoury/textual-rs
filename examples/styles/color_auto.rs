use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct ColorApp;

impl App for ColorApp {
    type Message = Message;

    const CSS: &'static str = include_str!("color_auto.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl1")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl2")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl3")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl4")
            Label("The quick brown fox jumps over the lazy dog!", id: "lbl5")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = ColorApp;
    app.run()
}
