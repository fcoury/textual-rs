//! PTY integration tests that run actual example binaries.
//!
//! These tests spawn example binaries in a pseudo-terminal and capture
//! the rendered output using vt100 terminal emulation.
//!
//! **Important**: Run `cargo build -p textual-rs --examples` before running these tests
//! to pre-compile the examples. Otherwise, cargo's compilation output will be captured
//! instead of the TUI output.

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::time::Duration;

/// Run a pre-built example binary in a PTY and capture the rendered output.
///
/// This function:
/// 1. Spawns the pre-built example binary in a pseudo-terminal
/// 2. Reads output in a background thread
/// 3. Waits briefly for rendering
/// 4. Sends 'q' to quit the app
/// 5. Parses the terminal output with vt100
///
/// **Requires**: The example must be pre-built with `cargo build -p textual-rs --examples`
fn run_example_in_pty(example_name: &str, width: u16, height: u16) -> String {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: height,
            cols: width,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("Failed to open PTY");

    // Run the pre-built binary directly (avoids cargo compilation output in PTY)
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let binary_path = workspace_root.join("target/debug/examples").join(example_name);

    if !binary_path.exists() {
        panic!(
            "Example binary not found: {:?}\n\
             Run `cargo build -p textual-rs --examples` first.",
            binary_path
        );
    }

    let mut cmd = CommandBuilder::new(&binary_path);
    cmd.cwd(workspace_root);

    let mut child = pair.slave.spawn_command(cmd).expect("Failed to spawn command");

    // Drop the slave to avoid blocking reads
    drop(pair.slave);

    // Get writer and reader handles
    let mut writer = pair.master.take_writer().expect("Failed to get writer");
    let mut reader = pair.master.try_clone_reader().expect("Failed to clone reader");

    // Spawn a thread to read output (PTY reads are blocking)
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut output = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => output.extend_from_slice(&buf[..n]),
                Err(_) => break,
            }
        }
        let _ = tx.send(output);
    });

    // Give the app time to render (should be fast since binary is pre-built)
    std::thread::sleep(Duration::from_millis(500));

    // Send 'q' to quit
    let _ = writer.write_all(b"q");
    let _ = writer.flush();

    // Give app time to process quit and clean up
    std::thread::sleep(Duration::from_millis(200));

    drop(writer); // Close writer to signal EOF

    // Wait for process to exit with timeout
    let _ = child.wait();

    // Get output from reader thread with timeout
    let output = rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_default();

    // Parse with vt100
    let mut parser = vt100::Parser::new(height, width, 0);

    // Process output and capture the screen state BEFORE the app exits
    // (before it switches back from alternate screen buffer).
    // We look for the "leave alternate screen" sequence: ESC[?1049l
    let leave_alt_screen = b"\x1b[?1049l";
    let capture_point = output
        .windows(leave_alt_screen.len())
        .position(|w| w == leave_alt_screen)
        .unwrap_or(output.len());

    // Process only up to the point where app leaves alternate screen
    parser.process(&output[..capture_point]);

    // Extract screen contents
    parser.screen().contents()
}

#[test]
#[ignore = "PTY tests are slow and require built examples"]
fn pty_border_example() {
    let output = run_example_in_pty("border", 80, 24);
    insta::assert_snapshot!(output);
}

#[test]
#[ignore = "PTY tests are slow and require built examples"]
fn pty_align_example() {
    let output = run_example_in_pty("align", 80, 24);
    insta::assert_snapshot!(output);
}

#[test]
#[ignore = "PTY tests are slow and require built examples"]
fn pty_background_example() {
    let output = run_example_in_pty("background", 80, 24);
    insta::assert_snapshot!(output);
}

#[test]
#[ignore = "PTY tests are slow and require built examples"]
fn pty_grid_example() {
    let output = run_example_in_pty("grid", 80, 24);
    insta::assert_snapshot!(output);
}
