use textual::{App, Grid, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MyApp;

impl App for MyApp {
    type Message = Message;

    const CSS: &'static str = include_str!("grid_size_both.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Label("1")
                Label("2")
                Label("3")
                Label("4")
                Label("5")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = MyApp;
    app.run()
}
