use textual::{App, Compose, Grid, KeyCode, Label, Widget};

#[derive(Clone)]
enum Message {}

struct MyApp {
    quit: bool,
}

impl MyApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for MyApp {
    type Message = Message;

    fn compose(&self) -> Box<dyn Widget<Self::Message>> {
        Box::new(Grid::new(vec![
            Box::new(Label::new("1fr")),
            Box::new(Label::new("width = 16")),
            Box::new(Label::new("2fr")),
            Box::new(Label::new("1fr")),
            Box::new(Label::new("width = 16")),
            Box::new(Label::new("1fr")),
            Box::new(Label::new("width = 16")),
            Box::new(Label::new("2fr")),
            Box::new(Label::new("1fr")),
            Box::new(Label::new("width = 16")),
        ]))
    }
}

impl App for MyApp {
    fn handle_message(&mut self, _envelope: textual::MessageEnvelope<Self::Message>) {}

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
    let mut app = MyApp::new();
    app.run()
}
