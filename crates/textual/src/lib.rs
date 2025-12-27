pub mod canvas;
pub mod containers;
pub mod context;
pub mod error;
mod log_init;
pub mod message;
pub mod style_resolver;
pub mod tree;
pub mod widget;

pub use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::{cursor, execute, terminal};
use futures::StreamExt;
use tokio::sync::mpsc;

pub use canvas::{Canvas, Region, Size};
pub use containers::{Center, Middle, horizontal::Horizontal, vertical::Vertical};
pub use context::{AppContext, IntervalHandle};
pub use error::Result;
pub use log_init::init_logger;
pub use message::MessageEnvelope;
pub use tcss::TcssError;

// Re-export the log crate so users can use textual::log::info!, etc.
pub use log;
pub use tcss::{parser::parse_stylesheet, types::Theme};
pub use widget::{Compose, Widget, switch::Switch};

use crate::{error::TextualError, style_resolver::{resolve_dirty_styles, resolve_styles}, tree::WidgetTree};

/// The main application trait. Implement this to create a TUI application.
///
/// The `Message` associated type (from `Compose`) defines the events your UI can produce.
/// This enables type-safe event handling with exhaustive pattern matching.
pub trait App: Compose
where
    Self::Message: Send + 'static,
{
    const CSS: &'static str = "";

    /// Handle a message produced by a widget.
    ///
    /// Messages are wrapped in an envelope that carries metadata:
    /// - `envelope.message` - the actual message payload
    /// - `envelope.sender_id` - optional widget ID that produced the message
    /// - `envelope.sender_type` - type name of the widget that produced the message
    ///
    /// Use pattern matching on `envelope.message` to handle each variant of your Message enum.
    fn handle_message(&mut self, envelope: MessageEnvelope<Self::Message>);

    /// Handle global key events (e.g., 'q' to quit).
    /// Called after widget event handling.
    fn on_key(&mut self, key: KeyCode);

    /// Return true when the application should exit.
    fn should_quit(&self) -> bool;

    /// Returns the current focus index for the widget tree.
    /// The run loop uses this to set focus on the nth focusable widget.
    fn focus_index(&self) -> usize {
        0
    }

    /// Called once when the application starts, after the widget tree is built.
    ///
    /// Use this to start timers, spawn background tasks, or perform other
    /// initialization that requires the `AppContext`.
    ///
    /// # Example
    /// ```ignore
    /// fn on_mount(&mut self, ctx: &AppContext<Self::Message>) {
    ///     ctx.set_interval(Duration::from_secs(1), || Message::Tick);
    /// }
    /// ```
    fn on_mount(&mut self, _ctx: &AppContext<Self::Message>) {
        // Default: do nothing
    }

    /// Run the application event loop.
    ///
    /// This uses a **persistent widget tree** - the tree is built once at startup
    /// and mutated in place. Events go to the existing widgets, which update their
    /// own state and mark themselves dirty for restyling.
    ///
    /// The event loop uses `tokio::select!` to poll:
    /// - Terminal events via `crossterm::event::EventStream`
    /// - Async messages via `tokio::sync::mpsc` channel
    ///
    /// Uses a single-threaded tokio runtime since TUI rendering is inherently
    /// single-threaded. Background tasks can still be spawned for async work.
    fn run(&mut self) -> Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TextualError::RuntimeInit(e.to_string()))?;

        rt.block_on(async {
            let mut stdout = std::io::stdout();
            // Enable raw mode, mouse capture, and enter alternate screen
            terminal::enable_raw_mode()?;
            execute!(
                stdout,
                terminal::EnterAlternateScreen,
                cursor::Hide,
                EnableMouseCapture
            )?;

            let result = self.event_loop_async().await;

            // Cleanup: Restore terminal state on exit
            execute!(
                stdout,
                DisableMouseCapture,
                cursor::Show,
                terminal::LeaveAlternateScreen
            )?;
            terminal::disable_raw_mode()?;

            result
        })
    }

    /// The main async event loop.
    fn event_loop_async(&mut self) -> impl std::future::Future<Output = Result<()>> + '_ {
        async move {
            // 1. Initial Setup: Parse CSS and define the theme
            let stylesheet = tcss::parser::parse_stylesheet(Self::CSS)
                .map_err(|e| TextualError::InvalidCss(e.to_string()))?;
            let theme = tcss::types::Theme::new("default", true);

            let (mut cols, mut rows) = terminal::size()?;
            let mut canvas = Canvas::new(cols, rows);

            // 2. Build the widget tree ONCE (persistent tree)
            // Use WidgetTree for O(d) focus-targeted dispatch and message bubbling
            let root = self.compose();
            let mut tree = WidgetTree::new(root);

            // Set initial focus and cache the focus path
            tree.root_mut().clear_focus();
            tree.root_mut().focus_nth(self.focus_index());
            tree.update_focus(self.focus_index());

            // Initial style resolution for all widgets
            let mut ancestors = Vec::new();
            resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

            // 3. Create message channel for async communication
            let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<Self::Message>>();
            let ctx = AppContext::new(tx);

            // Call lifecycle hook
            self.on_mount(&ctx);

            // 4. Create async event stream
            let mut event_stream = EventStream::new();

            // Track the previous focus index to detect changes
            let mut last_focus_index = self.focus_index();

            // Flag to prevent redundant re-renders
            let mut needs_render = true;

            while !self.should_quit() {
                // Check if focus changed
                let current_focus = self.focus_index();
                if current_focus != last_focus_index {
                    tree.root_mut().clear_focus();
                    tree.root_mut().focus_nth(current_focus);
                    tree.update_focus(current_focus);
                    last_focus_index = current_focus;
                    needs_render = true;
                }

                if needs_render {
                    // Resolve styles only for dirty widgets
                    let mut ancestors = Vec::new();
                    resolve_dirty_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors, false);

                    canvas.clear();
                    let region = Region {
                        x: 0,
                        y: 0,
                        width: cols,
                        height: rows,
                    };
                    tree.root().render(&mut canvas, region);
                    canvas.flush()?;

                    needs_render = false;
                }

                // 5. Event Handling: Use tokio::select! for async polling
                tokio::select! {
                    biased;

                    // Terminal events from crossterm
                    maybe_event = event_stream.next() => {
                        match maybe_event {
                            Some(Ok(Event::Key(key_event))) => {
                                // Dispatch key event to the focused widget using cached focus path
                                if let Some(msg) = tree.dispatch_key(key_event.code) {
                                    // Get sender info using cached focus path (O(d) access)
                                    let sender = tree.focused_sender_info();

                                    // Create envelope and bubble through ancestor widgets
                                    let envelope = MessageEnvelope::new(msg, sender.id.as_deref(), sender.type_name);
                                    let bubbled = tree.bubble_message(envelope);

                                    // App is always the final handler (even if bubbling was stopped)
                                    self.handle_message(bubbled);
                                }
                                self.on_key(key_event.code);
                                needs_render = true;
                            }
                            Some(Ok(Event::Resize(nw, nh))) => {
                                // Handle terminal window resizing
                                cols = nw;
                                rows = nh;
                                canvas = Canvas::new(cols, rows);
                                needs_render = true;
                            }
                            Some(Ok(Event::Mouse(mouse_event))) => {
                                // Compute the full-screen region for mouse event routing
                                let region = Region {
                                    x: 0,
                                    y: 0,
                                    width: cols,
                                    height: rows,
                                };

                                // Route mouse event through widget tree hit-testing
                                // NOTE: Mouse events use hit-testing, not focus path.
                                // Full mouse bubbling would require on_mouse to track the hit path.
                                // For now, messages go directly to App without parent interception.
                                if let Some(msg) = tree.root_mut().on_mouse(mouse_event, region) {
                                    let envelope = MessageEnvelope::new(msg, None, "Widget");
                                    self.handle_message(envelope);
                                }
                                needs_render = true;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => return Err(TextualError::IO(e)),
                            None => break, // Stream ended
                        }
                    }

                    // Messages from async tasks (timers, background work)
                    Some(envelope) = rx.recv() => {
                        self.handle_message(envelope);
                        needs_render = true;
                    }
                }
            }

            Ok(())
        }
    }
}

#[macro_export]
macro_rules! ui {
    // === Entry Points for Layouts ===
    (Vertical { $($children:tt)* }) => {
        $crate::ui!(@collect Vertical, [], $($children)*)
    };

    (Horizontal { $($children:tt)* }) => {
        $crate::ui!(@collect Horizontal, [], $($children)*)
    };

    // === Entry Points for Single-child Wrappers ===
    (Middle { $($inner:tt)* }) => {
        Box::new($crate::Middle::new($crate::ui!($($inner)*)))
    };

    (Center { $($inner:tt)* }) => {
        Box::new($crate::Center::new($crate::ui!($($inner)*)))
    };

    // === The Collector (Muncher) ===
    // This part moves items from the "todo" list into the "accumulator" list

    // 1. Process a nested container child
    (@collect $kind:ident, [$($acc:expr),*], $child:ident { $($inner:tt)* }, $($rest:tt)*) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($child { $($inner)* })], $($rest)*)
    };
    (@collect $kind:ident, [$($acc:expr),*], $child:ident { $($inner:tt)* }) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($child { $($inner)* })])
    };

    // 2. Process a leaf widget child (e.g. Switch::new)
    (@collect $kind:ident, [$($acc:expr),*], $leaf:ident :: new ( $($args:tt)* ) $( . $meth:ident ( $($m_args:tt)* ) )* , $($rest:tt)*) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($leaf :: new ( $($args)* ) $( . $meth ( $($m_args)* ) )*)], $($rest)*)
    };
    (@collect $kind:ident, [$($acc:expr),*], $leaf:ident :: new ( $($args:tt)* ) $( . $meth:ident ( $($m_args:tt)* ) )*) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($leaf :: new ( $($args)* ) $( . $meth ( $($m_args)* ) )*)])
    };

    // 3. Finalization: All children are in the accumulator, create the Boxed container
    (@collect $kind:ident, [$($acc:expr),*]) => {
        Box::new($kind::new(vec![$($acc),*]))
    };

    // === Leaf Widget & Fallback ===
    ($leaf:ident :: new ( $($args:expr),* ) $( . $meth:ident ( $($m_args:expr),* ) )*) => {
        Box::new($leaf::new( $($args),* ) $( . $meth ( $($m_args),* ) )*)
    };

    ($e:expr) => {
        $e
    };
}
