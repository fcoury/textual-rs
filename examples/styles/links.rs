use textual::{App, Static, Widget, ui};

const TEXT: &str = "Here is a [@click='app.bell']link[/] which you can click!\n";

#[derive(Clone)]
enum Message {}

struct LinksApp;

impl App for LinksApp {
    type Message = Message;

    const CSS: &'static str = include_str!("links.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static(format!("{}", TEXT))
            Static(TEXT, id: "custom")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = LinksApp;
    app.run()
}
