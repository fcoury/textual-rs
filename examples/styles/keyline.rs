use textual::{App, Compose, Grid, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct KeylineApp;

impl Compose for KeylineApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Placeholder(id: "foo")
                Placeholder(id: "bar")
                Placeholder()
                Placeholder(classes: "hidden")
                Placeholder(id: "baz")
            }
        }
    }
}

impl App for KeylineApp {
    const CSS: &'static str = include_str!("keyline.tcss");
}

fn main() -> textual::Result<()> {
    textual::init_logger("height_comparison.log");
    let mut app = KeylineApp;
    app.run()
}
