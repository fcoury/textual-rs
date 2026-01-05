use textual::{App, Container, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WidthApp;

impl App for WidthApp {
    type Message = Message;

    const CSS: &'static str = include_str!("width.tcss");

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

fn main() -> textual::Result<()> {
    let mut app = WidthApp;
    app.run()
}
