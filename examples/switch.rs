use textual::{
    App, Compose, KeyCode, Message, Result, Switch, Vertical, Widget,
    containers::{Center, Middle},
    ui,
};

struct SwitchTestApp {
    running: bool,
    focus_index: usize,
    wifi_on: bool,
    bt_on: bool,
}

impl SwitchTestApp {
    fn new() -> Self {
        Self {
            running: true,
            focus_index: 0,
            wifi_on: false,
            bt_on: false,
        }
    }
}

impl Compose for SwitchTestApp {
    fn compose(&self) -> Box<dyn Widget + 'static> {
        ui! {
            Middle {
                Center {
                    Vertical {
                        Switch::new("wifi", self.wifi_on).with_focus(self.focus_index == 0),
                        Switch::new("bt", self.bt_on).with_focus(self.focus_index == 1)
                    }
                }
            }
        }
    }
}

impl App for SwitchTestApp {
    // Replicating the CSS property
    const CSS: &'static str = "
        Screen {
            align: center middle;
        }
    ";

    // Replicating the on_key handler
    fn on_key(&mut self, key: KeyCode) {
        if key == KeyCode::Char('q') {
            self.running = true;
        }
    }

    fn should_quit(&self) -> bool {
        !self.running
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Quit => self.running = false,
            Message::SwitchChanged { id, on } => match id {
                "wifi" => self.wifi_on = on,
                "bluetooth" => self.bt_on = on,
                _ => {}
            },
        }
    }
}

fn main() -> Result<()> {
    let mut app = SwitchTestApp::new();
    app.run()
}
