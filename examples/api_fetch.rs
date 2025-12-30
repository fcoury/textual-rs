//! API Fetch Example - Reactive Attributes Demo
//!
//! Demonstrates the reactive attribute system:
//! - Loading state with animated spinner
//! - Simulated async API fetch
//! - Visibility toggling
//! - Disabled state
//!
//! The workflow:
//! 1. App starts with switches in loading state (showing spinner)
//! 2. Spinner animation runs at 100ms intervals
//! 3. After 2s, simulated API response sets actual values
//! 4. Third switch demonstrates disabled state

use std::time::Duration;

use textual::{
    App, AppContext, Center, Compose, IntervalHandle, KeyCode, MessageEnvelope, Middle, Result,
    Switch, Vertical, Widget, log, ui,
};

#[derive(Debug, Clone)]
enum Message {
    /// Animate loading spinners (every 100ms)
    SpinnerTick,
    /// Simulated API response with wifi status
    WifiLoaded(bool),
    /// Simulated API response with bluetooth status
    BluetoothLoaded(bool),
    /// User toggled wifi switch
    WifiToggled(bool),
    /// User toggled bluetooth switch
    BluetoothToggled(bool),
    /// User toggled the disabled switch (won't fire - it's disabled!)
    #[allow(dead_code)]
    DisabledToggled(bool),
}

struct ApiApp {
    running: bool,
    // Loading states
    wifi_loading: bool,
    bluetooth_loading: bool,
    // Actual values (set after "API" returns)
    wifi_enabled: bool,
    bluetooth_enabled: bool,
    // Timer handles
    spinner_handle: Option<IntervalHandle>,
    ctx: Option<AppContext<Message>>,
    // Spinner frame counter (shared across all loading widgets)
    spinner_frame: usize,
    // Focus navigation
    focus_idx: usize,
}

impl ApiApp {
    fn new() -> Self {
        Self {
            running: true,
            wifi_loading: true,
            bluetooth_loading: true,
            wifi_enabled: false,
            bluetooth_enabled: false,
            spinner_handle: None,
            ctx: None,
            spinner_frame: 0,
            focus_idx: 0,
        }
    }
}

impl Compose for ApiApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Message>>> {
        ui! {
            Middle {
                Center {
                    Vertical {
                        // WiFi switch - starts loading, then shows actual state
                        Switch(self.wifi_enabled, Message::WifiToggled,
                            id: "wifi", loading: self.wifi_loading, spinner_frame: self.spinner_frame)
                        // Bluetooth switch - starts loading, then shows actual state
                        Switch(self.bluetooth_enabled, Message::BluetoothToggled,
                            id: "bluetooth", loading: self.bluetooth_loading, spinner_frame: self.spinner_frame)
                        // Disabled switch - always disabled, shows how disabled state works
                        Switch(false, Message::DisabledToggled, id: "disabled-demo", disabled: true)
                    }
                }
            }
        }
    }
}

impl App for ApiApp {
    const CSS: &'static str = "
        Switch { color: #00FF00; }
        Switch:focus { color: #FFFF00; background: #333333; }
        Switch:disabled { color: #666666; }
    ";

    fn on_mount(&mut self, ctx: &mut textual::MountContext<Message>) {
        log::info!("App mounted - starting API fetch simulation...");

        // Store context for timer management
        self.ctx = Some(ctx.app_context().clone());

        // Start spinner animation (100ms = smooth animation)
        let handle = ctx.set_interval(Duration::from_millis(100), || Message::SpinnerTick);
        self.spinner_handle = Some(handle);

        // Simulate WiFi API call (responds after 2 seconds)
        ctx.set_timer(Duration::from_secs(2), Message::WifiLoaded(true));

        // Simulate Bluetooth API call (responds after 3 seconds)
        ctx.set_timer(Duration::from_secs(3), Message::BluetoothLoaded(false));

        log::info!("Waiting for API responses...");
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Tab | KeyCode::Down => {
                // Cycle forward through focusable widgets (2 enabled switches)
                self.focus_idx = (self.focus_idx + 1) % 2;
            }
            KeyCode::BackTab | KeyCode::Up => {
                // Cycle backward
                self.focus_idx = if self.focus_idx == 0 {
                    1
                } else {
                    self.focus_idx - 1
                };
            }
            _ => {}
        }
    }

    fn should_quit(&self) -> bool {
        !self.running
    }

    fn focus_index(&self) -> usize {
        self.focus_idx
    }

    /// Use Elm-style recomposition: rebuild tree after every state change
    fn needs_recompose(&self) -> bool {
        true
    }

    fn handle_message(&mut self, envelope: MessageEnvelope<Message>) {
        match envelope.message {
            Message::SpinnerTick => {
                // Advance the global spinner frame
                self.spinner_frame = (self.spinner_frame + 1) % 10;
                // Note: In a real app, you'd call tick_spinner() on actual widget refs.
                // For now, the compose() recreates widgets each frame anyway.
            }
            Message::WifiLoaded(status) => {
                log::info!("WiFi API returned: {}", if status { "ON" } else { "OFF" });
                self.wifi_loading = false;
                self.wifi_enabled = status;
            }
            Message::BluetoothLoaded(status) => {
                log::info!(
                    "Bluetooth API returned: {}",
                    if status { "ON" } else { "OFF" }
                );
                self.bluetooth_loading = false;
                self.bluetooth_enabled = status;

                // Stop spinner animation when all loading is done
                if !self.wifi_loading && !self.bluetooth_loading {
                    if self.spinner_handle.take().is_some() {
                        log::info!("All data loaded - stopping spinner animation");
                    }
                }
            }
            Message::WifiToggled(enabled) => {
                log::info!(
                    "WiFi toggled to {} by {:?}",
                    enabled,
                    envelope.sender_id.as_deref().unwrap_or("unknown")
                );
                self.wifi_enabled = enabled;
            }
            Message::BluetoothToggled(enabled) => {
                log::info!(
                    "Bluetooth toggled to {} by {:?}",
                    enabled,
                    envelope.sender_id.as_deref().unwrap_or("unknown")
                );
                self.bluetooth_enabled = enabled;
            }
            Message::DisabledToggled(_) => {
                // This should never fire because the switch is disabled
                log::warn!("Disabled switch was somehow toggled! This shouldn't happen.");
            }
        }
    }
}

fn main() -> Result<()> {
    textual::init_logger("api_fetch.log");

    log::info!("=== API Fetch Example ===");
    log::info!("Demonstrates: loading state, spinner animation, disabled state");
    log::info!("Press 'q' to quit");
    log::info!("");

    let mut app = ApiApp::new();
    app.run()
}
