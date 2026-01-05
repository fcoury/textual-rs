use textual::{App, Compose, Grid, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MyApp;

impl Compose for MyApp {
    type Message = Message;

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

impl App for MyApp {
    const CSS: &'static str = include_str!("grid_size_both.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = MyApp;
    app.run()
}
