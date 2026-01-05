use textual::{App, Horizontal, Placeholder, Ruler, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WidthComparisonApp;

impl App for WidthComparisonApp {
    type Message = Message;

    const CSS: &'static str = include_str!("width_comparison.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        let mut widgets = ui! {
            Horizontal {
                Placeholder(id: "cells")
                Placeholder(id: "percent")
                Placeholder(id: "w")
                Placeholder(id: "h")
                Placeholder(id: "vw")
                Placeholder(id: "vh")
                Placeholder(id: "auto")
                Placeholder(id: "fr1")
                Placeholder(id: "fr3")
            }
        };

        widgets.push(Box::new(Ruler::horizontal()));
        widgets
    }
}

fn main() -> textual::Result<()> {
    let mut app = WidthComparisonApp;
    app.run()
}
