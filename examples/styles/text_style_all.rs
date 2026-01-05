use textual::{App, Grid, Label, Widget, ui};

const TEXT: &str = r#"I must not fear.
Fear is the mind-killer.
Fear is the little-death that brings total obliteration.
I will face my fear.
I will permit it to pass over me and through me.
And when it has gone past, I will turn the inner eye to see its path.
Where the fear has gone there will be nothing. Only I will remain."#;

#[derive(Clone)]
enum Message {}

struct OutlineApp;

impl App for OutlineApp {
    type Message = Message;

    const CSS: &'static str = include_str!("text_style_all.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Label(format!("none\n{}", TEXT), id: "lbl1")
                Label(format!("bold\n{}", TEXT), id: "lbl2")
                Label(format!("italic\n{}", TEXT), id: "lbl3")
                Label(format!("reverse\n{}", TEXT), id: "lbl4")
                Label(format!("strike\n{}", TEXT), id: "lbl5")
                Label(format!("underline\n{}", TEXT), id: "lbl6")
                Label(format!("bold italic\n{}", TEXT), id: "lbl7")
                Label(format!("reverse strike\n{}", TEXT), id: "lbl8")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = OutlineApp;
    app.run()
}
