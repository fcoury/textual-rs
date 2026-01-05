use textual::{App, Compose, Container, Widget, ui};

#[derive(Clone)]
enum Message {}

struct HeightApp;

impl Compose for HeightApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        // NOTE: this example doesn't match the one from textual python exactly. On python it uses
        // a Widget instead of a Container. However, in our current Rust version, Widget is a trait
        // and therefore can't be used as a concrete type. However, the purpose of the example is to
        // show height behavior, which is the same for both Widget and Container.
        ui! {
            Container {}
        }
    }
}

impl App for HeightApp {
    const CSS: &'static str = include_str!("height.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = HeightApp;
    app.run()
}
