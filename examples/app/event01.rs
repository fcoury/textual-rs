use tcss::{StyleOverride, types::RgbaColor};
use textual::{App, EventContext, KeyCode, MountContext, Widget};

#[derive(Clone)]
enum Message {}

struct EventApp;

const COLORS: [&'static str; 10] = [
    "white", "maroon", "red", "purple", "fuchsia", "olive", "yellow", "navy", "teal", "aqua",
];

impl App for EventApp {
    type Message = Message;

    fn on_mount(&mut self, ctx: &mut MountContext<Self::Message>) {
        let background = RgbaColor::parse("darkblue").unwrap();
        ctx.screen_mut(|screen| {
            screen.set_inline_style(StyleOverride::new().background(background));
        });
    }

    fn on_key(&mut self, key: KeyCode, ctx: &mut EventContext<Self::Message>) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.request_quit();
            return;
        }
        if let KeyCode::Char(ch) = key {
            if let Some(digit) = ch.to_digit(10) {
                if let Some(color_name) = COLORS.get(digit as usize) {
                    let background = RgbaColor::parse(color_name).unwrap();
                    ctx.screen_mut(|screen| {
                        screen.set_inline_style(StyleOverride::new().background(background));
                    });
                }
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = EventApp;
    app.run()
}
