use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WrapApp;

impl Compose for WrapApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Widget 1")
            Label("Widget 2", classes: "invisible")
            Label("Widget 3")
        }
    }
}

impl App for WrapApp {
    const CSS: &'static str = include_str!("visibility.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = WrapApp;
    app.run()
}
