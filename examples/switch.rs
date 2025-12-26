use textual::{App, Compose, Horizontal, KeyCode, Result, Switch, Vertical, Widget, ui};

/// Application-specific messages - fully typed, no string IDs!
enum Message {
    WifiToggled(bool),
    BluetoothToggled(bool),
}

struct SwitchApp {
    running: bool,
    focus_index: usize,
    wifi_on: bool,
    bt_on: bool,
}

impl SwitchApp {
    fn new() -> Self {
        Self {
            running: true,
            focus_index: 0,
            wifi_on: false,
            bt_on: false,
        }
    }
}

impl Compose for SwitchApp {
    type Message = Message;

    fn compose(&self) -> Box<dyn Widget<Message>> {
        let wifi_msg = Message::WifiToggled as fn(bool) -> Message;
        let bt_msg = Message::BluetoothToggled as fn(bool) -> Message;

        ui! {
            Middle {
                Center {
                    Vertical{
                        Horizontal {
                            Switch::new(self.wifi_on, wifi_msg)
                                .with_focus(self.focus_index == 0),
                            Switch::new(self.bt_on, bt_msg)
                                .with_focus(self.focus_index == 1)
                        }
                    }
                }
            }
        }
    }
}

impl App for SwitchApp {
    const CSS: &'static str = "
        Screen {
            align: center middle;
        }
    ";

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Tab | KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % 2;
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focus_index = if self.focus_index == 0 { 1 } else { 0 };
            }
            _ => {}
        }
    }

    fn should_quit(&self) -> bool {
        !self.running
    }

    // Exhaustive pattern matching - compiler catches missing variants!
    fn handle_message(&mut self, message: Message) {
        match message {
            Message::WifiToggled(on) => self.wifi_on = on,
            Message::BluetoothToggled(on) => self.bt_on = on,
        }
    }
}

fn main() -> Result<()> {
    let mut app = SwitchApp::new();
    app.run()
}
