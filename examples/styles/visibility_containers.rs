use textual::{App, Compose, Horizontal, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct VisibilityContainersApp;

impl Compose for VisibilityContainersApp {
    type Message = Message;

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

impl App for VisibilityContainersApp {
    const CSS: &'static str = include_str!("visibility_containers.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = VisibilityContainersApp;
    app.run()
}
