use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct ContentAlignApp;

impl Compose for ContentAlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("With [i]content-align[/] you can...", id: "box1")
            Label("...[b]Easily align content[/]...", id: "box2")
            Label("...Horizontally [i]and[/] vertically!", id: "box3")
        }
    }
}

impl App for ContentAlignApp {
    const CSS: &'static str = include_str!("content_align.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = ContentAlignApp;
    app.run()
}
