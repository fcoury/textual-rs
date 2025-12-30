use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AllContentAlignApp {
    quit: bool,
}

impl AllContentAlignApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for AllContentAlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("left top", id: "left-top")
            Label("center top", id: "center-top")
            Label("right top", id: "right-top")
            Label("left middle", id: "left-middle")
            Label("center middle", id: "center-middle")
            Label("right middle", id: "right-middle")
            Label("left bottom", id: "left-bottom")
            Label("center bottom", id: "center-bottom")
            Label("right bottom", id: "right-bottom")
        }
    }
}

impl App for AllContentAlignApp {
    const CSS: &'static str = include_str!("content_align_all.tcss");

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
    let mut app = AllContentAlignApp::new();
    app.run()
}
