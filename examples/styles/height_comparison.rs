use textual::{App, Compose, Placeholder, Ruler, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct HeightApp;

impl Compose for HeightApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Placeholder(id: "cells")
                Placeholder(id: "percent")
                Placeholder(id: "w")
                Placeholder(id: "h")
                Placeholder(id: "vw")
                Placeholder(id: "vh")
                Placeholder(id: "auto")
                Placeholder(id: "fr1")
                Placeholder(id: "fr2")
            }
            Ruler {}
        }
    }
}

impl App for HeightApp {
    const CSS: &'static str = include_str!("height_comparison.tcss");
}

fn main() -> textual::Result<()> {
    textual::init_logger("height_comparison.log");
    let mut app = HeightApp;
    app.run()
}
