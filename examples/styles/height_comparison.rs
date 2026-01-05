use textual::{App, Placeholder, Ruler, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct HeightApp;

impl App for HeightApp {
    type Message = Message;

    const CSS: &'static str = include_str!("height_comparison.tcss");

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

fn main() -> textual::Result<()> {
    textual::init_logger("height_comparison.log");
    let mut app = HeightApp;
    app.run()
}
