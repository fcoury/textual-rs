use crossterm::{
    cursor, execute,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use std::io::Write;
use tcss::types::RgbaColor;

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

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub symbol: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

pub struct Canvas {
    size: Size,
    cells: Vec<Cell>,
    // Track current active colors to minimize ANSI escape codes
    // TODO: Use these for optimization in flush()
    #[allow(dead_code)]
    current_fg: Option<Color>,
    #[allow(dead_code)]
    current_bg: Option<Color>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            size: Size { width, height },
            cells: vec![
                Cell {
                    symbol: ' ',
                    fg: None,
                    bg: None
                };
                (width * height) as usize
            ],
            current_fg: None,
            current_bg: None,
        }
    }

    pub fn put_char(
        &mut self,
        x: u16,
        y: u16,
        c: char,
        fg: Option<RgbaColor>,
        bg: Option<RgbaColor>,
    ) {
        if x < self.size.width && y < self.size.height {
            let index = (y * self.size.width + x) as usize;
            self.cells[index] = Cell {
                symbol: c,
                fg: fg.map(to_crossterm_color),
                bg: bg.map(to_crossterm_color),
            };
        }
    }

    pub fn put_str(
        &mut self,
        x: u16,
        y: u16,
        s: &str,
        fg: Option<RgbaColor>,
        bg: Option<RgbaColor>,
    ) {
        for (i, c) in s.chars().enumerate() {
            self.put_char(x + i as u16, y, c, fg.clone(), bg.clone());
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        execute!(out, cursor::MoveTo(0, 0))?;

        let mut last_fg = None;
        let mut last_bg = None;

        for row in self.cells.chunks(self.size.width as usize) {
            for cell in row {
                // Only send escape code if the color actually changed
                if cell.fg != last_fg {
                    let color = cell.fg.unwrap_or(Color::Reset);
                    execute!(out, SetForegroundColor(color))?;
                    last_fg = cell.fg;
                }
                if cell.bg != last_bg {
                    let color = cell.bg.unwrap_or(Color::Reset);
                    execute!(out, SetBackgroundColor(color))?;
                    last_bg = cell.bg;
                }
                write!(out, "{}", cell.symbol)?;
            }
            write!(out, "\r\n")?;
        }
        out.flush()?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell {
            symbol: ' ',
            fg: None,
            bg: None,
        });
    }
}

fn to_crossterm_color(c: RgbaColor) -> Color {
    Color::Rgb {
        r: c.r,
        g: c.g,
        b: c.b,
    }
}
