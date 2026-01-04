use textual::{App, Compose, Horizontal, KeyCode, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct VisibilityContainersApp {
    quit: bool,
}

impl VisibilityContainersApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

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

    fn on_key(&mut self, key: textual::KeyCode) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> textual::Result<()> {
    let mut app = VisibilityContainersApp::new();
    app.run()
}
