use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct LinkStyleApp {
    quit: bool,
}

impl LinkStyleApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for LinkStyleApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label(
                "Visit the [link='https://textualize.io']Textualize[/link] website."
                id: "lbl1"
            )
            Label(
                "Click [@click=app.bell]here[/] for the bell sound."
                id: "lbl2"
            )
            Label(
                "You can also click [@click=app.bell]here[/] for the bell sound."
                id: "lbl3"
            )
            Label(
                "[@click=app.quit]Exit this application.[/]"
                id: "lbl4"
            )
        }
    }
}

impl App for LinkStyleApp {
    const CSS: &'static str = include_str!("link_style.tcss");

    fn on_key(&mut self, key: textual::KeyCode) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn request_quit(&mut self) {
        self.quit = true;
    }
}

fn main() -> textual::Result<()> {
    let mut app = LinkStyleApp::new();
    app.run()
}
