use textual::{App, Compose, Grid, KeyCode, Label, Widget};

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

    fn compose(&self) -> Box<dyn Widget<Self::Message>> {
        Box::new(Grid::new(vec![
            Box::new(Label::new("1")),
            Box::new(Label::new("2")),
            Box::new(Label::new("3")),
            Box::new(Label::new("4")),
            Box::new(Label::new("5")),
        ]))
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
