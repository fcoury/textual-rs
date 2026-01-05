use textual::{App, Label, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain.
"#;

#[derive(Clone)]
enum Message {}

struct Scrollbar2App;

impl App for Scrollbar2App {
    type Message = Message;

    const CSS: &'static str = include_str!("scrollbars2.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label(TEXT.repeat(10))
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = Scrollbar2App;
    app.run()
}
