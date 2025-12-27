//! Async Timer Example
//!
//! Demonstrates the message pump features:
//! - `on_mount()` lifecycle hook for starting timers
//! - `set_interval()` for periodic messages
//! - `MessageEnvelope` with sender metadata

use std::time::Duration;

use textual::{
    App, AppContext, Compose, IntervalHandle, KeyCode, MessageEnvelope, Result, Switch, Vertical,
    Widget, log, ui,
};

#[derive(Debug)]
enum Message {
    /// Periodic tick from the interval timer
    Tick,
    /// Switch was toggled
    SwitchToggled(bool),
}

struct TimerApp {
    running: bool,
    tick_count: u32,
    timer_enabled: bool,
    /// Handle to the interval timer - must be stored to keep the timer alive.
    /// Dropping or taking this handle cancels the interval.
    interval_handle: Option<IntervalHandle>,
}

impl TimerApp {
    fn new() -> Self {
        Self {
            running: true,
            tick_count: 0,
            timer_enabled: true,
            interval_handle: None,
        }
    }
}

impl Compose for TimerApp {
    type Message = Message;

    fn compose(&self) -> Box<dyn Widget<Message>> {
        ui! {
            Middle {
                Center {
                    Vertical {
                        Switch::new(self.timer_enabled, Message::SwitchToggled)
                            .with_id("timer-toggle")
                    }
                }
            }
        }
    }
}

impl App for TimerApp {
    const CSS: &'static str = "
        Switch { color: #00FF00; }
        Switch:focus { color: #FFFF00; background: #333333; }
    ";

    /// Called once when the application starts.
    ///
    /// This is where you set up timers, intervals, and other async tasks.
    fn on_mount(&mut self, ctx: &AppContext<Message>) {
        log::info!("App mounted! Starting 1-second interval...");

        // Start a repeating interval that sends Tick messages.
        // IMPORTANT: Store the handle to keep the timer alive!
        // Dropping the handle automatically cancels the interval.
        let handle = ctx.set_interval(Duration::from_secs(1), || Message::Tick);
        self.interval_handle = Some(handle);

        // You could also set a one-shot timer:
        // ctx.set_timer(Duration::from_secs(5), Message::Timeout);
    }

    fn on_key(&mut self, key: KeyCode) {
        if key == KeyCode::Char('q') {
            self.running = false;
        }
    }

    fn should_quit(&self) -> bool {
        !self.running
    }

    fn focus_index(&self) -> usize {
        0
    }

    /// Handle messages from widgets and timers.
    ///
    /// The envelope provides metadata about where the message came from.
    fn handle_message(&mut self, envelope: MessageEnvelope<Message>) {
        match envelope.message {
            Message::Tick => {
                self.tick_count += 1;
                log::info!(
                    "Tick #{} (from: {})",
                    self.tick_count,
                    envelope.sender_type
                );
            }
            Message::SwitchToggled(enabled) => {
                self.timer_enabled = enabled;

                if !enabled {
                    // Cancel the interval by dropping the handle
                    self.interval_handle.take();
                    log::info!(
                        "Timer cancelled (from: {:?})",
                        envelope.sender_id.as_deref().unwrap_or("unknown")
                    );
                } else {
                    // Note: Can't restart here without storing AppContext.
                    // In a real app, you'd store ctx in the struct or use a different pattern.
                    log::info!(
                        "Timer enabled but can't restart without AppContext (from: {:?})",
                        envelope.sender_id.as_deref().unwrap_or("unknown")
                    );
                }
            }
        }
    }
}

fn main() -> Result<()> {
    // Initialize the logger to see timer messages
    textual::init_logger("timer.log");

    log::info!("Starting async timer example...");
    log::info!("Press 'q' to quit, Space/Enter to toggle timer");

    let mut app = TimerApp::new();
    app.run()
}
