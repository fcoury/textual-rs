use textual::{App, Compose, Grid, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MyApp;

impl Compose for MyApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Placeholder(id: "p1")
                Placeholder(id: "p2")
                Placeholder(id: "p3")
                Placeholder(id: "p4")
                Placeholder(id: "p5")
                Placeholder(id: "p6")
                Placeholder(id: "p7")
            }
        }
    }
}

impl App for MyApp {
    const CSS: &'static str = include_str!("column_span.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = MyApp;
    app.run()
}
