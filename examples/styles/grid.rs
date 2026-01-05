use textual::{App, Grid, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct GridApp;

impl App for GridApp {
    type Message = Message;

    const CSS: &'static str = include_str!("grid.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
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

fn main() -> textual::Result<()> {
    let mut app = GridApp;
    app.run()
}
