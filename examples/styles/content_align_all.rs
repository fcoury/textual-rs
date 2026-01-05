use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AllContentAlignApp;

impl Compose for AllContentAlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("left top", id: "left-top")
            Label("center top", id: "center-top")
            Label("right top", id: "right-top")
            Label("left middle", id: "left-middle")
            Label("center middle", id: "center-middle")
            Label("right middle", id: "right-middle")
            Label("left bottom", id: "left-bottom")
            Label("center bottom", id: "center-bottom")
            Label("right bottom", id: "right-bottom")
        }
    }
}

impl App for AllContentAlignApp {
    const CSS: &'static str = include_str!("content_align_all.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = AllContentAlignApp;
    app.run()
}
