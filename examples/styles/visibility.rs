use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WrapApp;

impl App for WrapApp {
    type Message = Message;

    const CSS: &'static str = include_str!("visibility.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Widget 1")
            Label("Widget 2", classes: "invisible")
            Label("Widget 3")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = WrapApp;
    app.run()
}
