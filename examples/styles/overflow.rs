use textual::{App, Compose, Horizontal, Static, VerticalScroll, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain."#;

#[derive(Clone)]
enum Message {}

struct OutlineBorderApp;

impl Compose for OutlineBorderApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                VerticalScroll(id: "left") { Static(TEXT) Static(TEXT) Static(TEXT) }
                VerticalScroll(id: "right") { Static(TEXT) Static(TEXT) Static(TEXT) }
            }
        }
    }
}

impl App for OutlineBorderApp {
    const CSS: &'static str = include_str!("overflow.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = OutlineBorderApp;
    app.run()
}
