use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BackgroundApp;

impl Compose for BackgroundApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Widget 1", id: "static1")
            Label("Widget 2", id: "static2")
            Label("Widget 3", id: "static3")
        }
    }
}

impl App for BackgroundApp {
    const CSS: &'static str = include_str!("background.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = BackgroundApp;
    app.run()
}
