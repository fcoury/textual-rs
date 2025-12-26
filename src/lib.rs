pub mod canvas;
pub mod containers;
pub mod error;
pub mod events;
pub mod widget;

pub use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use crossterm::{event, terminal};

pub use canvas::{Canvas, Region, Size};
pub use error::Result;
pub use widget::{Compose, Widget, switch::Switch};

pub trait App: Compose {
    const CSS: &'static str = "";

    // Required: What the user implements (like in Python)
    fn on_key(&mut self, key: KeyCode);

    // Optional: Override if you want a custom exit condition
    fn should_quit(&self) -> bool;

    // The Engine: This handles the terminal logic
    fn run(&mut self) -> Result<()> {
        // 1. Setup Terminal
        terminal::enable_raw_mode()?;
        let (cols, rows) = terminal::size()?;
        let mut canvas = Canvas::new(cols, rows);

        // 2. Main Loop
        while !self.should_quit() {
            // "Compose" the UI tree
            let root_widget = self.compose();

            // Render
            canvas.clear();
            let screen_region = Region {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            };
            root_widget.render(&mut canvas, screen_region);
            canvas.flush()?;

            // Event Handling
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key_event) = event::read()? {
                    self.on_key(key_event.code);
                }
            }
        }

        // 3. Cleanup
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

#[macro_export]
macro_rules! ui {
    // Pattern 1: A container with a child inside braces
    // Example: Middle { Center { ... } }
    ($container:ident { $($inner:tt)* }) => {
        Box::new($container::new(
            $crate::ui! { $($inner)* }
        ))
    };

    // Pattern 2: A terminal expression (the leaf widget)
    // Example: Switch::new()
    ($leaf:ident :: new ()) => {
        Box::new($leaf::new())
    };

    // Pattern 3: Fallback for generic expressions
    ($e:expr) => {
        Box::new($e)
    };
}
