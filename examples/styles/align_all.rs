use textual::{App, Compose, Container, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AlignApp;

impl Compose for AlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Container(id: "left-top")      { Label("left top") }
            Container(id: "center-top")    { Label("center top") }
            Container(id: "right-top")     { Label("right top") }
            Container(id: "left-middle")   { Label("left middle") }
            Container(id: "center-middle") { Label("center middle") }
            Container(id: "right-middle")  { Label("right middle") }
            Container(id: "left-bottom")   { Label("left bottom") }
            Container(id: "center-bottom") { Label("center bottom") }
            Container(id: "right-bottom")  { Label("right bottom") }
        }
    }
}

impl App for AlignApp {
    const CSS: &'static str = include_str!("align_all.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = AlignApp;
    app.run()
}
