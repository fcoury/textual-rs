use textual::{App, Compose, Horizontal, KeyCode, Result, Switch, Vertical, Widget, log, ui};

enum Message {
    WifiToggled(bool),
    BluetoothToggled(bool),
}

struct SwitchApp {
    running: bool,
    focus_index: usize,
}

impl SwitchApp {
    fn new() -> Self {
        Self {
            running: true,
            focus_index: 0,
        }
    }
}

impl Compose for SwitchApp {
    type Message = Message;

    /// Build the widget tree ONCE (persistent tree architecture).
    ///
    /// Note: We don't pass `.with_focus()` here anymore - focus is managed
    /// by the run loop via `clear_focus()` and `focus_nth()`.
    fn compose(&self) -> Box<dyn Widget<Message>> {
        let wifi_msg = Message::WifiToggled as fn(bool) -> Message;
        let bt_msg = Message::BluetoothToggled as fn(bool) -> Message;

        ui! {
            Middle {
                Center {
                    Vertical{
                        Horizontal {
                            // Widgets start with their initial value
                            // They own their state and toggle it themselves
                            Switch::new(false, wifi_msg),
                            Switch::new(false, bt_msg)
                        }
                    }
                }
            }
        }
    }
}

impl App for SwitchApp {
    const CSS: &'static str = "
        Switch { color: #00FF00; }
        Switch:hover { color: #66FF66; background: #222222; }
        Switch:focus { color: #FFFF00; background: #333333; }
        Switch:active { color: #FF6600; background: #444444; }
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

    /// Returns the current focus index.
    /// The run loop uses this to update focus in the persistent widget tree.
    fn focus_index(&self) -> usize {
        self.focus_index
    }

    /// Handle messages from widgets.
    ///
    /// In the persistent tree model, widgets update their own state.
    /// Messages are for the app to react (e.g., make API calls, show notifications).
    fn handle_message(&mut self, message: Message) {
        match message {
            Message::WifiToggled(on) => {
                log::info!("WiFi toggled to: {}", on);
            }
            Message::BluetoothToggled(on) => {
                log::info!("Bluetooth toggled to: {}", on);
            }
        }
    }
}

fn main() -> Result<()> {
    // Initialize the logger to write to "app.log"
    textual::init_logger("app.log");

    let mut app = SwitchApp::new();
    app.run()
}
