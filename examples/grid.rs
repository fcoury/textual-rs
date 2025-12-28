use textual::{App, Compose, Grid, KeyCode, Static, Widget};

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
            Box::new(
                Static::new("Grid cell 1\n\nrow-span: 3;\ncolumn-span: 2;").with_id("static1"),
            ),
            Box::new(Static::new("Grid cell 2").with_id("static2")),
            Box::new(Static::new("Grid cell 3").with_id("static3")),
            Box::new(Static::new("Grid cell 4").with_id("static4")),
            Box::new(Static::new("Grid cell 5").with_id("static5")),
            Box::new(Static::new("Grid cell 6").with_id("static6")),
            Box::new(Static::new("Grid cell 7").with_id("static7")),
        ]))
    }
}

impl App for MyApp {
    const CSS: &'static str = include_str!("grid.tcss");

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
