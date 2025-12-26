use crossterm::{cursor, execute};
use std::io::Write;

/// The physical dimensions of a widget or terminal.
#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// A specific area on the screen where a widget is allowed to draw.
#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct Canvas {
    size: Size,
    cells: Vec<char>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            size: Size { width, height },
            cells: vec![' '; (width * height) as usize],
        }
    }

    /// Place a single character at a global coordinate.
    pub fn put_char(&mut self, x: u16, y: u16, c: char) {
        if x < self.size.width && y < self.size.height {
            let index = (y * self.size.width + x) as usize;
            self.cells[index] = c;
        }
    }

    /// Helper to place a string horizontally.
    pub fn put_str(&mut self, x: u16, y: u16, s: &str) {
        for (i, c) in s.chars().enumerate() {
            self.put_char(x + i as u16, y, c);
        }
    }

    /// Clears the buffer for the next frame.
    pub fn clear(&mut self) {
        self.cells.fill(' ');
    }

    /// Renders the entire buffer to the terminal.
    pub fn flush(&mut self) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        execute!(out, cursor::MoveTo(0, 0))?;

        // .chunks(width) gives us a slice for each row
        for row in self.cells.chunks(self.size.width as usize) {
            let line: String = row.iter().collect();
            write!(out, "{}\r\n", line)?;
        }

        out.flush()?;
        Ok(())
    }
}
