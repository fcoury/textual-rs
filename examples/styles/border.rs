use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BorderApp;

impl Compose for BorderApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("My border is solid red", id: "label1")
            Label("My border is dashed green", id: "label2")
            Label("My border is tall blue", id: "label3")
        }
    }
}

impl App for BorderApp {
    const CSS: &'static str = include_str!("border.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = BorderApp;
    app.run()
}
