//! Integration tests using PTY-based terminal emulation.
//!
//! These tests run actual examples in a pseudo-terminal and verify behavior.
//!
//! Run with: cargo test --test pty_tests -- --test-threads=1 --ignored

#[path = "support/pty_harness.rs"]
#[allow(dead_code)]
mod pty_harness;
#[path = "support/helpers.rs"]
#[allow(dead_code)]
mod helpers;

use helpers::TestScenario;
use pty_harness::Key;

// ============================================================================
// Border Example Tests
// ============================================================================

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_border_initial_render() {
    let mut scenario = TestScenario::new("border").expect("Failed to spawn border example");
    scenario.wait_ready().expect("App failed to start");

    // Verify the labels are rendered
    scenario.assert_contains("My border is solid red");
    scenario.assert_contains("My border is dashed green");
    scenario.assert_contains("My border is tall blue");

    insta::assert_snapshot!("border_initial", scenario.snapshot());

    scenario.finish().expect("Failed to quit");
}

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_border_quit_with_q() {
    let mut scenario = TestScenario::new("border").expect("Failed to spawn border example");
    scenario.wait_ready().expect("App failed to start");

    // Verify we can see content before quitting
    scenario.assert_contains("My border");

    // Press q to quit
    scenario.finish().expect("Failed to quit with q");
}

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_border_quit_with_escape() {
    let mut scenario = TestScenario::new("border").expect("Failed to spawn border example");
    scenario.wait_ready().expect("App failed to start");

    // Press Escape to quit
    scenario.finish_with(Key::Escape).expect("Failed to quit with Escape");
}

// ============================================================================
// Switch Example Tests
// ============================================================================

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_switch_initial_render() {
    let mut scenario = TestScenario::with_size("switch", 60, 20).expect("Failed to spawn switch example");
    scenario.wait_ready().expect("App failed to start");

    insta::assert_snapshot!("switch_initial", scenario.snapshot());

    scenario.finish().expect("Failed to quit");
}

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_switch_toggle_with_enter() {
    let mut scenario = TestScenario::with_size("switch", 60, 20).expect("Failed to spawn switch example");
    scenario.wait_ready().expect("App failed to start");

    // Take initial snapshot
    let initial = scenario.snapshot();

    // Press Enter to toggle the focused switch
    scenario.press(Key::Enter).expect("Failed to press Enter");

    // The screen should have changed (switch toggled)
    let after_toggle = scenario.snapshot();
    assert_ne!(initial, after_toggle, "Screen should change after toggle");

    insta::assert_snapshot!("switch_after_toggle", after_toggle);

    scenario.finish().expect("Failed to quit");
}

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_switch_navigate_with_tab() {
    let mut scenario = TestScenario::with_size("switch", 60, 20).expect("Failed to spawn switch example");
    scenario.wait_ready().expect("App failed to start");

    // Press Tab to move focus
    scenario.press(Key::Tab).expect("Failed to press Tab");

    // The screen should have changed (focus moved)
    let after_tab = scenario.snapshot();

    // Note: Focus change might be subtle - just verify no crash
    insta::assert_snapshot!("switch_after_tab", after_tab);

    scenario.finish().expect("Failed to quit");
}

#[test]
#[ignore = "PTY tests need cargo build first"]
fn test_switch_navigate_with_arrows() {
    let mut scenario = TestScenario::with_size("switch", 60, 20).expect("Failed to spawn switch example");
    scenario.wait_ready().expect("App failed to start");

    // Press Down to move focus
    scenario.press(Key::Down).expect("Failed to press Down");
    let after_down = scenario.snapshot();

    // Press Up to move focus back
    scenario.press(Key::Up).expect("Failed to press Up");
    let after_up = scenario.snapshot();

    insta::assert_snapshot!("switch_after_down", after_down);
    insta::assert_snapshot!("switch_after_up", after_up);

    scenario.finish().expect("Failed to quit");
}

// ============================================================================
// Helper function to run PTY tests
// ============================================================================

/// Run all PTY tests. Call this from the command line with:
/// `cargo test --test integration -- --test-threads=1 --ignored`
#[test]
#[ignore = "Run with --ignored flag to execute PTY tests"]
fn run_all_pty_tests() {
    // This is a marker test - run with --ignored to execute PTY tests
}
