//! Test helper utilities for integration testing.
//!
//! Provides a fluent builder API for common test scenarios.

use std::time::Duration;

use super::pty_harness::{Key, PtyConfig, PtyTestHarness};

/// Builder for test scenarios with fluent API.
pub struct TestScenario {
    harness: PtyTestHarness,
    stable_duration: Duration,
}

impl TestScenario {
    /// Create a new test scenario for an example.
    pub fn new(example: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(example, PtyConfig::default())
    }

    /// Create a new test scenario with custom config.
    pub fn with_config(example: &str, config: PtyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let harness = PtyTestHarness::spawn_example(example, config)?;
        Ok(Self {
            harness,
            stable_duration: Duration::from_millis(200),
        })
    }

    /// Create a test scenario with custom terminal size.
    pub fn with_size(example: &str, cols: u16, rows: u16) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(example, PtyConfig::with_size(cols, rows))
    }

    /// Set the stable duration for wait operations.
    pub fn stable_duration(mut self, duration: Duration) -> Self {
        self.stable_duration = duration;
        self
    }

    /// Wait for the application to be ready (initial render complete).
    pub fn wait_ready(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.harness.wait_for_stable(self.stable_duration)?;
        Ok(self)
    }

    /// Wait for specific text to appear.
    pub fn wait_for(&mut self, text: &str) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.harness.wait_for_text(text)?;
        Ok(self)
    }

    /// Wait for text to disappear.
    pub fn wait_for_gone(&mut self, text: &str) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.harness.wait_for_text_gone(text)?;
        Ok(self)
    }

    /// Send a key and wait for output to stabilize.
    pub fn press(&mut self, key: Key) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.harness.send_key(key)?;
        self.harness.wait_for_stable(self.stable_duration)?;
        Ok(self)
    }

    /// Send a character key.
    pub fn press_char(&mut self, c: char) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.press(Key::Char(c))
    }

    /// Send a string (types each character).
    pub fn type_str(&mut self, s: &str) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.harness.send_str(s)?;
        self.harness.wait_for_stable(self.stable_duration)?;
        Ok(self)
    }

    /// Assert that the screen contains specific text.
    pub fn assert_contains(&self, text: &str) -> &Self {
        assert!(
            self.harness.contains(text),
            "Expected screen to contain '{}', but got:\n{}",
            text,
            self.harness.screen_text()
        );
        self
    }

    /// Assert that the screen does not contain specific text.
    pub fn assert_not_contains(&self, text: &str) -> &Self {
        assert!(
            !self.harness.contains(text),
            "Expected screen NOT to contain '{}', but got:\n{}",
            text,
            self.harness.screen_text()
        );
        self
    }

    /// Assert that a specific row contains text.
    pub fn assert_row_contains(&self, row: u16, text: &str) -> &Self {
        let row_text = self.harness.screen_row(row);
        assert!(
            row_text.contains(text),
            "Expected row {} to contain '{}', but got: '{}'",
            row,
            text,
            row_text
        );
        self
    }

    /// Get the current screen content for snapshot.
    pub fn snapshot(&self) -> String {
        self.harness.snapshot()
    }

    /// Get the current screen text.
    pub fn screen(&self) -> String {
        self.harness.screen_text()
    }

    /// Get a specific row.
    pub fn row(&self, row: u16) -> String {
        self.harness.screen_row(row)
    }

    /// Access the underlying harness for advanced operations.
    pub fn harness(&self) -> &PtyTestHarness {
        &self.harness
    }

    /// Access the underlying harness mutably.
    pub fn harness_mut(&mut self) -> &mut PtyTestHarness {
        &mut self.harness
    }

    /// Send quit key (commonly 'q') and finish the test.
    pub fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.harness.send_key(Key::Char('q'))?;
        // Give the app time to quit
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    /// Finish without sending quit key (for apps that don't use 'q' to quit).
    pub fn finish_with(&mut self, key: Key) -> Result<(), Box<dyn std::error::Error>> {
        self.harness.send_key(key)?;
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    }
}

/// Macro for creating snapshot tests more easily.
#[macro_export]
macro_rules! pty_snapshot {
    ($name:ident, $example:expr, $setup:expr) => {
        #[test]
        fn $name() {
            let mut scenario = TestScenario::new($example).expect("Failed to create scenario");
            scenario.wait_ready().expect("App failed to start");
            $setup(&mut scenario);
            insta::assert_snapshot!(scenario.snapshot());
            let _ = scenario.finish();
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_bytes() {
        assert_eq!(Key::Char('a').to_bytes(), vec![b'a']);
        assert_eq!(Key::Enter.to_bytes(), vec![b'\r']);
        assert_eq!(Key::Escape.to_bytes(), vec![0x1b]);
        assert_eq!(Key::Up.to_bytes(), vec![0x1b, b'[', b'A']);
        assert_eq!(Key::Down.to_bytes(), vec![0x1b, b'[', b'B']);
        assert_eq!(Key::Ctrl('c').to_bytes(), vec![3]); // Ctrl+C = 0x03
    }
}
