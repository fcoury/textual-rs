use textual::{App, Compose, Grid, KeyCode, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct GridApp {
    quit: bool,
}

impl GridApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for GridApp {
    type Message = Message;

    fn compose(&self) -> Box<dyn Widget<Self::Message>> {
        ui! {
            Grid {
                Static("Grid cell 1\n\nrow-span: 3;\ncolumn-span: 2;", id: "static1")
                Static("Grid cell 2", id: "static2")
                Static("Grid cell 3", id: "static3")
                Static("Grid cell 4", id: "static4")
                Static("Grid cell 5", id: "static5")
                Static("Grid cell 6", id: "static6")
                Static("Grid cell 7", id: "static7")
            }
        }
    }
}

impl App for GridApp {
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
    let mut app = GridApp::new();
    app.run()
}
