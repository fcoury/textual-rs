use tcss::{StyleOverride, types::RgbaColor};
use textual::{App, Compose, KeyCode, Label, Widget};

#[derive(Clone)]
enum Message {}

struct WrapApp {
    quit: bool,
}

impl WrapApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for WrapApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        let color = RgbaColor::parse("green").unwrap();
        let mut res = vec![];

        for tint_alpha in (0..101).step_by(10) {
            let mut widget = Label::new(format!("tint: green {tint_alpha}%;"));
            let tint = color.with_alpha(tint_alpha as f32 / 100.0);
            widget.set_inline_style(StyleOverride::new().tint(tint));

            res.push(Box::new(widget) as Box<dyn Widget<Self::Message>>);
        }

        res
    }
}

impl App for WrapApp {
    const CSS: &'static str = include_str!("tint.tcss");

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
    let mut app = WrapApp::new();
    app.run()
}
