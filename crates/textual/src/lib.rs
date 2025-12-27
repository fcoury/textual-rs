pub mod canvas;
pub mod containers;
pub mod error;
pub mod log;
pub mod style_resolver;
pub mod widget;

pub use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use crossterm::{cursor, event, execute, terminal};

pub use canvas::{Canvas, Region, Size};
pub use containers::{Center, Middle, horizontal::Horizontal, vertical::Vertical};
pub use error::Result;
pub use log::init_logger;
pub use tcss::TcssError;
pub use tcss::{parser::parse_stylesheet, types::Theme};
pub use widget::{Compose, Widget, switch::Switch};

use crate::{error::TextualError, style_resolver::resolve_styles};

/// The main application trait. Implement this to create a TUI application.
///
/// The `Message` associated type (from `Compose`) defines the events your UI can produce.
/// This enables type-safe event handling with exhaustive pattern matching.
pub trait App: Compose {
    const CSS: &'static str = "";

    /// Handle a message produced by a widget.
    /// Use pattern matching to handle each variant of your Message enum.
    fn handle_message(&mut self, message: Self::Message);

    /// Handle global key events (e.g., 'q' to quit).
    /// Called after widget event handling.
    fn on_key(&mut self, key: KeyCode);

    /// Return true when the application should exit.
    fn should_quit(&self) -> bool;

    /// Run the application event loop.
    fn run(&mut self) -> Result<()> {
        let mut stdout = std::io::stdout();
        // Enable raw mode and enter alternate screen to take control of the terminal
        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

        // 1. Initial Setup: Parse CSS and define the theme
        let stylesheet = tcss::parser::parse_stylesheet(Self::CSS)
            .map_err(|e| TextualError::InvalidCss(e.to_string()))?;
        let theme = tcss::types::Theme::new("default", true);

        let (mut cols, mut rows) = terminal::size()?;
        let mut canvas = Canvas::new(cols, rows);

        // Flag to prevent redundant re-renders and eliminate jitter
        let mut needs_render = true;

        while !self.should_quit() {
            if needs_render {
                // 2. Render Phase: Compose, Style, and Draw
                let mut root = self.compose();

                // Resolve CSS styles recursively for the entire widget tree
                let mut ancestors = Vec::new();
                crate::style_resolver::resolve_styles(
                    root.as_mut(),
                    &stylesheet,
                    &theme,
                    &mut ancestors,
                );

                canvas.clear();
                let region = Region {
                    x: 0,
                    y: 0,
                    width: cols,
                    height: rows,
                };
                root.render(&mut canvas, region);
                canvas.flush()?;

                needs_render = false;
            }

            // 3. Event Handling: Block for input to save CPU and stop flickering
            if event::poll(std::time::Duration::from_millis(10))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        // Re-compose temporary tree to find the widget handling the event
                        let mut root = self.compose();
                        if let Some(msg) = root.on_event(key_event.code) {
                            self.handle_message(msg);
                            needs_render = true;
                        }
                        self.on_key(key_event.code);
                        needs_render = true;
                    }
                    Event::Resize(nw, nh) => {
                        // Handle terminal window resizing
                        cols = nw;
                        rows = nh;
                        canvas = Canvas::new(cols, rows);
                        needs_render = true;
                    }
                    _ => {}
                }
            }
        }

        // Cleanup: Restore terminal state on exit
        execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
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
