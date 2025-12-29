//! PTY-based test harness for running examples in a virtual terminal.
//!
//! This module provides a harness for spawning example applications in a
//! pseudo-terminal (PTY) and capturing their output through terminal emulation.

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use vt100::Parser;

/// Configuration for the PTY test harness.
#[derive(Clone, Debug)]
pub struct PtyConfig {
    /// Terminal width in columns.
    pub cols: u16,
    /// Terminal height in rows.
    pub rows: u16,
    /// Timeout for operations.
    pub timeout: Duration,
    /// Environment variables to set.
    pub env: Vec<(String, String)>,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            timeout: Duration::from_secs(5),
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("NO_COLOR".to_string(), "".to_string()),
            ],
        }
    }
}

impl PtyConfig {
    /// Create a new config with custom terminal size.
    pub fn with_size(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            ..Default::default()
        }
    }

    /// Set the timeout duration.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add an environment variable.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }
}

/// Keyboard keys for sending to the terminal.
#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Tab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Ctrl(char),
    Alt(char),
}

impl Key {
    /// Convert key to ANSI escape sequence.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Key::Char(c) => vec![*c as u8],
            Key::Enter => vec![b'\r'],
            Key::Escape => vec![0x1b],
            Key::Tab => vec![b'\t'],
            Key::Backspace => vec![0x7f],
            Key::Up => vec![0x1b, b'[', b'A'],
            Key::Down => vec![0x1b, b'[', b'B'],
            Key::Right => vec![0x1b, b'[', b'C'],
            Key::Left => vec![0x1b, b'[', b'D'],
            Key::Home => vec![0x1b, b'[', b'H'],
            Key::End => vec![0x1b, b'[', b'F'],
            Key::PageUp => vec![0x1b, b'[', b'5', b'~'],
            Key::PageDown => vec![0x1b, b'[', b'6', b'~'],
            Key::Delete => vec![0x1b, b'[', b'3', b'~'],
            Key::Insert => vec![0x1b, b'[', b'2', b'~'],
            Key::F(n) => match n {
                1 => vec![0x1b, b'O', b'P'],
                2 => vec![0x1b, b'O', b'Q'],
                3 => vec![0x1b, b'O', b'R'],
                4 => vec![0x1b, b'O', b'S'],
                5 => vec![0x1b, b'[', b'1', b'5', b'~'],
                6 => vec![0x1b, b'[', b'1', b'7', b'~'],
                7 => vec![0x1b, b'[', b'1', b'8', b'~'],
                8 => vec![0x1b, b'[', b'1', b'9', b'~'],
                9 => vec![0x1b, b'[', b'2', b'0', b'~'],
                10 => vec![0x1b, b'[', b'2', b'1', b'~'],
                11 => vec![0x1b, b'[', b'2', b'3', b'~'],
                12 => vec![0x1b, b'[', b'2', b'4', b'~'],
                _ => vec![],
            },
            Key::Ctrl(c) => {
                // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                let byte = (*c as u8).wrapping_sub(b'a').wrapping_add(1);
                vec![byte]
            }
            Key::Alt(c) => vec![0x1b, *c as u8],
        }
    }
}

/// A cell in the virtual terminal screen.
#[derive(Clone, Debug)]
pub struct ScreenCell {
    pub char: char,
    pub fg: vt100::Color,
    pub bg: vt100::Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl Default for ScreenCell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg: vt100::Color::Default,
            bg: vt100::Color::Default,
            bold: false,
            italic: false,
            underline: false,
            inverse: false,
        }
    }
}

/// PTY test harness for spawning and interacting with terminal applications.
pub struct PtyTestHarness {
    parser: Parser,
    writer: Box<dyn Write + Send>,
    reader_rx: mpsc::Receiver<Vec<u8>>,
    config: PtyConfig,
    #[allow(dead_code)]
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyTestHarness {
    /// Spawn an example application in a PTY.
    ///
    /// The example is built with `cargo build --example <name>` and then executed.
    pub fn spawn_example(name: &str, config: PtyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Find the project root (where Cargo.toml is)
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

        // Build the example first
        let status = std::process::Command::new("cargo")
            .args(["build", "--example", name])
            .current_dir(workspace_root)
            .status()?;

        if !status.success() {
            return Err(format!("Failed to build example '{}'", name).into());
        }

        // Find the example binary
        let example_path = workspace_root
            .join("target")
            .join("debug")
            .join("examples")
            .join(name);

        if !example_path.exists() {
            return Err(format!("Example binary not found at {:?}", example_path).into());
        }

        Self::spawn_command(example_path.to_string_lossy().as_ref(), &[], config)
    }

    /// Spawn a command in a PTY.
    pub fn spawn_command(
        command: &str,
        args: &[&str],
        config: PtyConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = NativePtySystem::default();

        let pair = pty_system.openpty(PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        let child = pair.slave.spawn_command(cmd)?;

        // Get writer for sending input
        let writer = pair.master.take_writer()?;

        // Set up reader in a separate thread
        let mut reader = pair.master.try_clone_reader()?;
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let parser = Parser::new(config.rows, config.cols, 0);

        Ok(Self {
            parser,
            writer,
            reader_rx: rx,
            config,
            child,
        })
    }

    /// Read available output and update the virtual terminal.
    pub fn read_output(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Read all available data with a short timeout
        let deadline = Instant::now() + Duration::from_millis(100);
        while Instant::now() < deadline {
            match self.reader_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(data) => {
                    self.parser.process(&data);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
    }

    /// Wait for output to stabilize (no new data for a duration).
    pub fn wait_for_stable(&mut self, stable_duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let deadline = Instant::now() + self.config.timeout;
        let mut last_change = Instant::now();
        let mut last_content = String::new();

        while Instant::now() < deadline {
            self.read_output()?;
            let current_content = self.screen_text();

            if current_content != last_content {
                last_content = current_content;
                last_change = Instant::now();
            } else if Instant::now().duration_since(last_change) >= stable_duration {
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        Err("Timeout waiting for stable output".into())
    }

    /// Get the current screen content as text.
    pub fn screen_text(&self) -> String {
        let screen = self.parser.screen();
        let mut lines = Vec::new();

        for row in 0..screen.size().0 {
            let mut line = String::new();
            for col in 0..screen.size().1 {
                let cell = screen.cell(row, col);
                if let Some(cell) = cell {
                    line.push_str(&cell.contents());
                } else {
                    line.push(' ');
                }
            }
            lines.push(line.trim_end().to_string());
        }

        // Remove trailing empty lines
        while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.pop();
        }

        lines.join("\n")
    }

    /// Get the screen content as trimmed text (no trailing whitespace).
    pub fn screen_text_trimmed(&self) -> String {
        self.screen_text()
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    /// Get a specific row of the screen.
    pub fn screen_row(&self, row: u16) -> String {
        let screen = self.parser.screen();
        let mut line = String::new();

        for col in 0..screen.size().1 {
            let cell = screen.cell(row, col);
            if let Some(cell) = cell {
                line.push_str(&cell.contents());
            } else {
                line.push(' ');
            }
        }

        line.trim_end().to_string()
    }

    /// Send a key to the terminal.
    pub fn send_key(&mut self, key: Key) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(&key.to_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    /// Send a string to the terminal.
    pub fn send_str(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    /// Wait for specific text to appear on screen.
    pub fn wait_for_text(&mut self, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
        let deadline = Instant::now() + self.config.timeout;

        while Instant::now() < deadline {
            self.read_output()?;
            if self.screen_text().contains(expected) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        Err(format!(
            "Timeout waiting for text '{}'. Current screen:\n{}",
            expected,
            self.screen_text()
        )
        .into())
    }

    /// Wait for text to disappear from the screen.
    pub fn wait_for_text_gone(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let deadline = Instant::now() + self.config.timeout;

        while Instant::now() < deadline {
            self.read_output()?;
            if !self.screen_text().contains(text) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        Err(format!(
            "Timeout waiting for text '{}' to disappear. Current screen:\n{}",
            text,
            self.screen_text()
        )
        .into())
    }

    /// Check if the screen contains specific text.
    pub fn contains(&self, text: &str) -> bool {
        self.screen_text().contains(text)
    }

    /// Take a snapshot of the current screen for insta testing.
    pub fn snapshot(&self) -> String {
        self.screen_text_trimmed()
    }

    /// Get detailed cell information at a position.
    pub fn get_cell(&self, row: u16, col: u16) -> ScreenCell {
        let screen = self.parser.screen();
        if let Some(cell) = screen.cell(row, col) {
            ScreenCell {
                char: cell.contents().chars().next().unwrap_or(' '),
                fg: cell.fgcolor(),
                bg: cell.bgcolor(),
                bold: cell.bold(),
                italic: cell.italic(),
                underline: cell.underline(),
                inverse: cell.inverse(),
            }
        } else {
            ScreenCell::default()
        }
    }

    /// Resize the terminal.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.parser.set_size(rows, cols);
        self.config.cols = cols;
        self.config.rows = rows;
    }
}
