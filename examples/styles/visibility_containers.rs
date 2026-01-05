use textual::{App, Horizontal, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct VisibilityContainersApp;

impl App for VisibilityContainersApp {
    type Message = Message;

    const CSS: &'static str = include_str!("visibility_containers.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Horizontal(id: "top") {
                    Placeholder()
                    Placeholder()
                    Placeholder()
                }
                Horizontal(id: "middle") {
                    Placeholder()
                    Placeholder()
                    Placeholder()
                }
                Horizontal(id: "bot") {
                    Placeholder()
                    Placeholder()
                    Placeholder()
                }
            }

        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = VisibilityContainersApp;
    app.run()
}
