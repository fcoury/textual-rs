use textual::{App, Compose, Container, Grid, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MarginAllApp;

impl Compose for MarginAllApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Container(classes: "bordered") { Placeholder(id: "p1", label: "no margin") }
                Container(classes: "bordered") { Placeholder(id: "p2", label: "margin: 1") }
                Container(classes: "bordered") { Placeholder(id: "p3", label: "margin: 1 5") }
                Container(classes: "bordered") { Placeholder(id: "p4", label: "margin: 1 1 2 6") }
                Container(classes: "bordered") { Placeholder(id: "p5", label: "margin-top: 4") }
                Container(classes: "bordered") { Placeholder(id: "p6", label: "margin-right: 3") }
                Container(classes: "bordered") { Placeholder(id: "p7", label: "margin-bottom: 4") }
                Container(classes: "bordered") { Placeholder(id: "p8", label: "margin-left: 3") }
            }
        }
    }
}

impl App for MarginAllApp {
    const CSS: &'static str = include_str!("margin_all.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = MarginAllApp;
    app.run()
}
