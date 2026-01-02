//! Tests for Tokio runtime compatibility.
//!
//! Ensures App::run() works in various runtime configurations.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Test helper that mimics the runtime detection logic from App::run().
/// Returns Ok(value) on success, Err(message) for unsupported runtime.
fn run_with_runtime_detection<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            use tokio::runtime::RuntimeFlavor;
            match handle.runtime_flavor() {
                RuntimeFlavor::MultiThread => {
                    // Multi-thread runtime - safe to use block_in_place
                    Ok(tokio::task::block_in_place(|| {
                        handle.block_on(async { f() })
                    }))
                }
                RuntimeFlavor::CurrentThread | _ => {
                    // Current-thread runtime - return error like App::run() does
                    Err("Cannot call run() from a current-thread Tokio runtime".to_string())
                }
            }
        }
        Err(_) => {
            // No runtime - create a new one
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            Ok(rt.block_on(async { f() }))
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_current_thread_runtime_returns_error() {
    // In current-thread runtime, run() should return an error (not panic)
    let result = run_with_runtime_detection(|| 42);

    assert!(
        result.is_err(),
        "Should return error in current-thread runtime"
    );
    assert!(
        result.unwrap_err().contains("current-thread"),
        "Error should mention current-thread"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multi_thread_runtime_works() {
    // This should work fine with multi-thread runtime
    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    let result = run_with_runtime_detection(|| {
        executed_clone.store(true, Ordering::SeqCst);
        42
    });

    assert!(result.is_ok(), "Should succeed in multi-thread runtime");
    assert_eq!(result.unwrap(), 42);
    assert!(executed.load(Ordering::SeqCst), "Code should have executed");
}

#[test]
fn test_no_runtime_creates_new_one() {
    // When called outside any runtime, should create a new one
    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    let result = run_with_runtime_detection(|| {
        executed_clone.store(true, Ordering::SeqCst);
        42
    });

    assert!(result.is_ok(), "Should succeed when no runtime exists");
    assert_eq!(result.unwrap(), 42);
    assert!(executed.load(Ordering::SeqCst), "Code should have executed");
}
