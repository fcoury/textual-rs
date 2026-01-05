use textual::{App, Label, Vertical, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BackgroundApp;

impl App for BackgroundApp {
    type Message = Message;

    const CSS: &'static str = include_str!("background_tint.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Vertical(id: "tint1") {
                Label("0%")
            }
            Vertical(id: "tint2") {
                Label("25%")
            }
            Vertical(id: "tint3") {
                Label("50%")
            }
            Vertical(id: "tint4") {
                Label("75%")
            }
            Vertical(id: "tint5") {
                Label("100%")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = BackgroundApp;
    app.run()
}
