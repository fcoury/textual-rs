pub mod canvas;
pub mod containers;
pub mod error;
pub mod widget;

pub use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use crossterm::{cursor, event, execute, terminal};

pub use canvas::{Canvas, Region, Size};
pub use containers::{Center, Middle, horizontal::Horizontal, vertical::Vertical};
pub use error::Result;
pub use widget::{Compose, Widget, switch::Switch};

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
        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

        let (cols, rows) = terminal::size()?;
        let mut canvas = Canvas::new(cols, rows);

        while !self.should_quit() {
            // Render phase
            let root = self.compose();

            canvas.clear();
            let region = Region {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            };
            root.render(&mut canvas, region);
            canvas.flush()?;

            // Event handling
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    // Pass to the widget tree
                    if let Some(msg) = self.compose().on_event(key_event.code) {
                        self.handle_message(msg);
                    }
                    // Pass to global handler
                    self.on_key(key_event.code);
                }
            }
        }

        // Cleanup
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
