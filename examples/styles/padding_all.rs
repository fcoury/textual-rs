use textual::{App, Grid, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct PaddingAllApp;

impl App for PaddingAllApp {
    type Message = Message;

    const CSS: &'static str = include_str!("padding_all.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Placeholder(id: "p1", label: "no padding")
                Placeholder(id: "p2", label: "padding: 1")
                Placeholder(id: "p3", label: "padding: 1 5")
                Placeholder(id: "p4", label: "padding: 1 1 2 6")
                Placeholder(id: "p5", label: "padding-top: 4")
                Placeholder(id: "p6", label: "padding-right: 3")
                Placeholder(id: "p7", label: "padding-bottom: 4")
                Placeholder(id: "p8", label: "padding-left: 3")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = PaddingAllApp;
    app.run()
}
