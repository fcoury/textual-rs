use textual::{App, Grid, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MyApp;

impl App for MyApp {
    type Message = Message;

    const CSS: &'static str = include_str!("grid_columns.tcss");

    fn handle_message(
        &mut self,
        _envelope: textual::MessageEnvelope<Self::Message>,
        _ctx: &mut textual::EventContext<Self::Message>,
    ) {
    }

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Label("1fr")
                Label("width = 16")
                Label("2fr")
                Label("1fr")
                Label("width = 16")
                Label("1fr")
                Label("width = 16")
                Label("2fr")
                Label("1fr")
                Label("width = 16")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = MyApp;
    app.run()
}
