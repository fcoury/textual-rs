use textual::{App, Horizontal, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct KeylineApp;

impl App for KeylineApp {
    type Message = Message;

    const CSS: &'static str = include_str!("keyline_horizontal.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                Placeholder()
                Placeholder()
                Placeholder()
            }
        }
    }
}

fn main() -> textual::Result<()> {
    textual::init_logger("height_comparison.log");
    let mut app = KeylineApp;
    app.run()
}
