use tcss::{StyleOverride, types::RgbaColor};
use textual::{App, Label, Widget};

#[derive(Clone)]
enum Message {}

struct WrapApp;

impl App for WrapApp {
    type Message = Message;

    const CSS: &'static str = include_str!("tint.tcss");

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

fn main() -> textual::Result<()> {
    let mut app = WrapApp;
    app.run()
}
