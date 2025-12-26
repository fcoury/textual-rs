pub mod canvas;
pub mod containers;
pub mod error;
pub mod events;
pub mod widget;

pub use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use crossterm::{cursor, event, execute, terminal};

pub use canvas::{Canvas, Region, Size};
pub use containers::{Center, Middle, vertical::Vertical};
pub use error::Result;
pub use events::Message;
pub use widget::{Compose, Widget, switch::Switch};

pub trait App: Compose {
    const CSS: &'static str = "";

    // The user handles high-level messages (e.g., SwitchChanged)
    // instead of raw key codes here.
    fn handle_message(&mut self, message: Message);

    // We still keep on_key for global app-level shortcuts (like 'q')
    fn on_key(&mut self, key: KeyCode);

    fn should_quit(&self) -> bool;

    fn run(&mut self) -> Result<()> {
        let mut stdout = std::io::stdout();
        terminal::enable_raw_mode()?;
        // 1. Hide cursor and enter alternate screen to prevent flicker/scroll
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

        let (cols, rows) = terminal::size()?;
        let mut canvas = Canvas::new(cols, rows);

        while !self.should_quit() {
            // 2. Render phase
            let root = self.compose();
            canvas.clear();
            let region = Region {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            };
            root.render(&mut canvas, region);
            canvas.flush()?; // Ensure this writes to stdout and calls .flush()

            // 3. Event Handling (The "Blocking" fix)
            // If you poll with 0 or very low duration, it might spin too fast.
            // Increase to 100ms for testing, or block indefinitely:
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    // Pass to the tree
                    if let Some(msg) = self.compose().on_event(key_event.code) {
                        self.handle_message(msg);
                    }
                    // Pass to global handler
                    self.on_key(key_event.code);
                }
            }
        }

        // 4. Cleanup
        execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

#[macro_export]
macro_rules! ui {
    // 1. Multi-child (Vertical): Use tt to allow recursive macro matching
    (Vertical { $($child:tt { $($inner:tt)* }),+ $(,)? }) => {
        Box::new(Vertical::new(vec![
            $( $crate::ui!($child { $($inner)* }) ),+
        ]))
    };

    // 1b. Multi-child for Leaf-style nodes in a list
    (Vertical { $($leaf:ident :: new ( $($args:tt)* ) $( . $meth:ident ( $($m_args:tt)* ) )* ),+ $(,)? }) => {
        Box::new(Vertical::new(vec![
            $( $crate::ui!($leaf :: new ( $($args)* ) $( . $meth ( $($m_args)* ) )* ) ),+
        ]))
    };

    // 2. Single-child Nesting (Middle, Center)
    ($container:ident { $($inner:tt)* }) => {
        Box::new($container::new(
            $crate::ui!($($inner)*)
        ))
    };

    // 3. Leaf Widgets: The actual boxing happens here
    ($leaf:ident :: new ( $($args:expr),* ) $( . $meth:ident ( $($m_args:expr),* ) )*) => {
        Box::new($leaf::new( $($args),* ) $( . $meth ( $($m_args),* ) )*)
    };

    // 4. Fallback
    ($e:expr) => {
        $e
    };
}
