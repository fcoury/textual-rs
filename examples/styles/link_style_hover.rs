use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct LinkHoverStyleApp;

impl App for LinkHoverStyleApp {
    type Message = Message;

    const CSS: &'static str = include_str!("link_style_hover.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label(
                "Visit the [link='https://textualize.io']Textualize[/link] website."
                id: "lbl1"
            )
            Label(
                "Click [@click=app.bell]here[/] for the bell sound."
                id: "lbl2"
            )
            Label(
                "You can also click [@click=app.bell]here[/] for the bell sound."
                id: "lbl3"
            )
            Label(
                "[@click=app.quit]Exit this application.[/]"
                id: "lbl4"
            )
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = LinkHoverStyleApp;
    app.run()
}
