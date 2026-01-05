use textual::{App, Container, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct DockAllApp;

impl App for DockAllApp {
    type Message = Message;

    const CSS: &'static str = include_str!("dock_all.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Container(id: "big_container") {
                Container(id: "left") { Label("left") }
                Container(id: "top") { Label("top") }
                Container(id: "right") { Label("right") }
                Container(id: "bottom") { Label("bottom") }
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = DockAllApp;
    app.run()
}
