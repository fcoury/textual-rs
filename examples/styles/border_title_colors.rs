//! Border title colors example - demonstrates using the query API in on_mount.
//!
//! This is a port of Python Textual's border_title_colors.py example.
//! It shows how to use `MountContext::with_widget_by_id` to modify widgets
//! after the tree is built.

use textual::{App, Compose, Label, MountContext, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BorderTitleApp;

impl Compose for BorderTitleApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Hello, World!", id: "label")
        }
    }
}

impl App for BorderTitleApp {
    const CSS: &'static str = include_str!("border_title_colors.tcss");

    fn on_mount(&mut self, ctx: &mut MountContext<Self::Message>) {
        ctx.query_one("Label", |widget| {
            widget.set_border_title("Textual Rocks");
            widget.set_border_subtitle("Textual Rocks");
        });
    }
}

fn main() -> textual::Result<()> {
    let mut app = BorderTitleApp;
    app.run()
}
