use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BackgroundApp;

impl App for BackgroundApp {
    type Message = Message;

    const CSS: &'static str = include_str!("background.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Widget 1", id: "static1")
            Label("Widget 2", id: "static2")
            Label("Widget 3", id: "static3")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = BackgroundApp;
    app.run()
}
