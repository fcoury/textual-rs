//! Example demonstrating dynamic widget generation with splat operator.
//!
//! This example shows how to use the `widget!` macro and `..splat` syntax
//! to dynamically generate widgets from a list, similar to Python Textual's
//! ability to interleave loops with UI declarations.

use textual::{App, Horizontal, Static, Vertical, Widget, ui, widget};

const HATCHES: &[&str] = &["cross", "horizontal", "custom", "left", "right"];

#[derive(Clone)]
enum Message {}

struct HatchApp;

impl App for HatchApp {
    type Message = Message;

    const CSS: &'static str = include_str!("hatch.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        // Build widgets dynamically from the HATCHES list
        let hatch_widgets: Vec<_> = HATCHES
            .iter()
            .map(|&hatch| {
                widget! {
                    Vertical {
                        Static("", classes: format!("hatch {}", hatch), border_title: hatch)
                    }
                }
            })
            .collect();

        /*
        You could also do:

        let mut hatch_widgets: Vec<Box<dyn Widget<Message>>> = MATCHES
            .iter()
            .map(|&hatch| {
                Box::new(Vertical::new(vec![
                    Static::new("")
                        .with_classes(format!("hatch {}", hatch))
                        .with_border_title(hatch),
                ])) as Box<dyn Widget<Message>>
            })
            .collect();
        */

        // Use splat operator to inject the dynamic widgets
        ui! {
            Horizontal {
                ..hatch_widgets
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = HatchApp;
    app.run()
}
