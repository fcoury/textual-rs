use textual::{App, Compose, Grid, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MyApp {
    quit: bool,
}

impl MyApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

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
    const CSS: &'static str = include_str!("grid_size_columns.tcss");

    fn on_key(&mut self, key: textual::KeyCode) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> textual::Result<()> {
    let mut app = MyApp::new();
    app.run()
}
