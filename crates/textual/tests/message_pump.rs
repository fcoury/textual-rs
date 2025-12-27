//! Message Pump Integration Tests
//!
//! Tests for the async event loop, message bubbling, and timer lifecycle.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use textual::context::AppContext;
use textual::message::MessageEnvelope;
use textual::tree::WidgetTree;
use textual::widget::Widget;
use textual::{Canvas, Region, Size};
use tokio::sync::mpsc;

// =============================================================================
// Test Widgets
// =============================================================================

/// A test widget that tracks how many times handle_message was called.
struct TrackingWidget {
    name: &'static str,
    children: Vec<Box<dyn Widget<TestMessage>>>,
    focusable: bool,
    call_log: Arc<Mutex<Vec<&'static str>>>,
}

#[derive(Debug, Clone, PartialEq)]
enum TestMessage {
    Ping,
}

impl TrackingWidget {
    fn new(name: &'static str, call_log: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            name,
            children: Vec::new(),
            focusable: false,
            call_log,
        }
    }

    fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    fn with_child(mut self, child: TrackingWidget) -> Self {
        self.children.push(Box::new(child));
        self
    }

    fn boxed(self) -> Box<dyn Widget<TestMessage>> {
        Box::new(self)
    }
}

impl Widget<TestMessage> for TrackingWidget {
    fn render(&self, _canvas: &mut Canvas, _region: Region) {}

    fn desired_size(&self) -> Size {
        Size {
            width: 1,
            height: 1,
        }
    }

    fn is_focusable(&self) -> bool {
        self.focusable
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<TestMessage> + '_)> {
        if index < self.children.len() {
            Some(self.children[index].as_mut())
        } else {
            None
        }
    }

    fn handle_message(
        &mut self,
        _envelope: &mut MessageEnvelope<TestMessage>,
    ) -> Option<TestMessage> {
        // Record that this widget handled the message
        self.call_log.lock().unwrap().push(self.name);
        None // Pass through unchanged
    }

    fn type_name(&self) -> &'static str {
        self.name
    }
}

// =============================================================================
// Unit Tests: WidgetTree Bubbling
// =============================================================================

/// Validate that a message from a deep child hits every parent exactly once.
/// Regression test for [P2]: root handler invoked twice.
#[test]
fn test_message_bubbling_order() {
    let call_log = Arc::new(Mutex::new(Vec::new()));

    // Build a 3-level tree: Root -> Container -> Leaf (focusable)
    let root = TrackingWidget::new("Root", call_log.clone())
        .with_child(
            TrackingWidget::new("Container", call_log.clone())
                .with_child(TrackingWidget::new("Leaf", call_log.clone()).focusable()),
        )
        .boxed();

    let mut tree = WidgetTree::new(root);

    // Focus on the leaf (first focusable widget)
    tree.update_focus(0);
    assert_eq!(tree.focus_path().indices(), &[0, 0], "Path should be [0, 0]");

    // Create a message and bubble it
    let envelope = MessageEnvelope::new(TestMessage::Ping, None, "Test");
    let _result = tree.bubble_message(envelope);

    // Verify the order: should go from Container (parent of leaf) up to Root
    // Note: Leaf itself is NOT called because it produced the message
    let log = call_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["Container", "Root"],
        "Message should bubble from Container to Root, each called exactly once"
    );
}

/// Regression test for [P2]: Ensure root is only handled once.
#[test]
fn test_root_only_handles_once() {
    let call_log = Arc::new(Mutex::new(Vec::new()));

    // Simple 2-level tree: Root -> Child (focusable)
    let root = TrackingWidget::new("Root", call_log.clone())
        .with_child(TrackingWidget::new("Child", call_log.clone()).focusable())
        .boxed();

    let mut tree = WidgetTree::new(root);
    tree.update_focus(0);

    let envelope = MessageEnvelope::new(TestMessage::Ping, None, "Test");
    let _result = tree.bubble_message(envelope);

    let log = call_log.lock().unwrap();

    // Count how many times "Root" appears
    let root_count = log.iter().filter(|&&name| name == "Root").count();
    assert_eq!(root_count, 1, "Root should handle message exactly once");
}

/// Test bubbling stops when envelope.stop() is called.
#[test]
fn test_bubbling_stops_on_stop() {
    let call_log = Arc::new(Mutex::new(Vec::new()));

    // Widget that stops bubbling
    struct StoppingWidget {
        name: &'static str,
        children: Vec<Box<dyn Widget<TestMessage>>>,
        call_log: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Widget<TestMessage> for StoppingWidget {
        fn render(&self, _canvas: &mut Canvas, _region: Region) {}
        fn desired_size(&self) -> Size {
            Size {
                width: 1,
                height: 1,
            }
        }
        fn child_count(&self) -> usize {
            self.children.len()
        }
        fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<TestMessage> + '_)> {
            if index < self.children.len() {
                Some(self.children[index].as_mut())
            } else {
                None
            }
        }
        fn handle_message(
            &mut self,
            envelope: &mut MessageEnvelope<TestMessage>,
        ) -> Option<TestMessage> {
            self.call_log.lock().unwrap().push(self.name);
            envelope.stop(); // Stop bubbling here
            None
        }
    }

    let stopping_container = StoppingWidget {
        name: "Stopper",
        children: vec![TrackingWidget::new("Leaf", call_log.clone())
            .focusable()
            .boxed()],
        call_log: call_log.clone(),
    };

    let root: Box<dyn Widget<TestMessage>> = Box::new(StoppingWidget {
        name: "Root",
        children: vec![Box::new(stopping_container)],
        call_log: call_log.clone(),
    });

    let mut tree = WidgetTree::new(root);
    tree.update_focus(0);

    let envelope = MessageEnvelope::new(TestMessage::Ping, None, "Test");
    let _result = tree.bubble_message(envelope);

    let log = call_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["Stopper"],
        "Bubbling should stop at Stopper, Root should not be called"
    );
}

// =============================================================================
// Unit Tests: FocusPath Stability
// =============================================================================

#[test]
fn test_focus_path_indices_valid() {
    let call_log = Arc::new(Mutex::new(Vec::new()));

    // Tree with multiple focusable widgets
    let root = TrackingWidget::new("Root", call_log.clone())
        .with_child(TrackingWidget::new("Child0", call_log.clone()).focusable())
        .with_child(
            TrackingWidget::new("Child1", call_log.clone())
                .with_child(TrackingWidget::new("Nested", call_log.clone()).focusable()),
        )
        .boxed();

    let mut tree = WidgetTree::new(root);

    // Focus first focusable (Child0)
    tree.update_focus(0);
    assert_eq!(tree.focus_path().indices(), &[0]);

    // Focus second focusable (Nested inside Child1)
    tree.update_focus(1);
    assert_eq!(tree.focus_path().indices(), &[1, 0]);
}

#[test]
fn test_with_focused_returns_correct_widget() {
    let call_log = Arc::new(Mutex::new(Vec::new()));

    let root = TrackingWidget::new("Root", call_log.clone())
        .with_child(TrackingWidget::new("First", call_log.clone()).focusable())
        .with_child(TrackingWidget::new("Second", call_log.clone()).focusable())
        .boxed();

    let mut tree = WidgetTree::new(root);

    tree.update_focus(0);
    let name0 = tree.with_focused(|w| w.type_name());
    assert_eq!(name0, Some("First"));

    tree.update_focus(1);
    let name1 = tree.with_focused(|w| w.type_name());
    assert_eq!(name1, Some("Second"));
}

// =============================================================================
// Integration Tests: AppContext Channel Integrity
// =============================================================================

#[tokio::test]
async fn test_post_message_arrives() {
    let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<i32>>();
    let ctx = AppContext::new(tx);

    ctx.post(42);

    let envelope = rx.recv().await.expect("Should receive message");
    assert_eq!(envelope.message, 42);
    assert_eq!(envelope.sender_type, "AppContext");
}

#[tokio::test]
async fn test_concurrent_post_from_multiple_tasks() {
    let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<i32>>();
    let ctx = AppContext::new(tx);

    const NUM_TASKS: i32 = 10;
    const MESSAGES_PER_TASK: i32 = 10;
    const TOTAL_MESSAGES: i32 = NUM_TASKS * MESSAGES_PER_TASK;

    // Spawn multiple tasks that all post messages
    let mut handles = Vec::new();
    for task_id in 0..NUM_TASKS {
        let ctx_clone = ctx.clone();
        handles.push(tokio::spawn(async move {
            for msg_id in 0..MESSAGES_PER_TASK {
                ctx_clone.post(task_id * 100 + msg_id);
            }
        }));
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Collect all messages
    let mut received = Vec::new();
    while let Ok(envelope) = rx.try_recv() {
        received.push(envelope.message);
    }

    assert_eq!(
        received.len() as i32,
        TOTAL_MESSAGES,
        "Should receive all {} messages",
        TOTAL_MESSAGES
    );
}

// =============================================================================
// Integration Tests: Timer Lifecycle (RAII)
// =============================================================================

/// Critical regression test: Ensure interval stops when handle is dropped.
#[tokio::test]
async fn test_interval_handle_drops_immediately() {
    let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<()>>();
    let ctx = AppContext::new(tx);

    {
        // Start an interval and drop it immediately
        let _handle = ctx.set_interval(Duration::from_millis(5), || ());
    } // _handle dropped here, should cancel the interval

    // Wait long enough for many ticks to have occurred if not cancelled
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Count messages received (should be 0 or at most 1 from race condition)
    let mut count = 0;
    while rx.try_recv().is_ok() {
        count += 1;
    }

    assert!(
        count <= 1,
        "Timer should have stopped; got {} messages",
        count
    );
}

/// Test that keeping the handle alive allows messages to flow.
#[tokio::test]
async fn test_interval_fires_while_handle_alive() {
    let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<i32>>();
    let ctx = AppContext::new(tx);

    let _handle = ctx.set_interval(Duration::from_millis(10), || 1);

    // Wait for at least 3 ticks
    tokio::time::sleep(Duration::from_millis(45)).await;

    let mut count = 0;
    while rx.try_recv().is_ok() {
        count += 1;
    }

    assert!(
        count >= 3,
        "Should receive at least 3 ticks, got {}",
        count
    );
}

/// Test explicit cancel() stops the interval.
#[tokio::test]
async fn test_interval_explicit_cancel() {
    let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<()>>();
    let ctx = AppContext::new(tx);

    let mut handle = ctx.set_interval(Duration::from_millis(5), || ());

    // Wait for a tick
    let _ = rx.recv().await;

    // Explicitly cancel
    handle.cancel();

    // Drain any in-flight messages
    tokio::time::sleep(Duration::from_millis(10)).await;
    while rx.try_recv().is_ok() {}

    // Now wait and ensure no more messages arrive
    tokio::time::sleep(Duration::from_millis(30)).await;

    assert!(
        rx.try_recv().is_err(),
        "No messages should arrive after cancel"
    );
}

/// Test one-shot timer fires exactly once.
#[tokio::test]
async fn test_timer_fires_once() {
    let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<&str>>();
    let ctx = AppContext::new(tx);

    ctx.set_timer(Duration::from_millis(10), "timeout");

    // Wait for the timer
    let envelope = rx.recv().await.expect("Should receive timer message");
    assert_eq!(envelope.message, "timeout");
    assert_eq!(envelope.sender_type, "Timer");

    // Wait more and ensure no duplicate
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(rx.try_recv().is_err(), "Timer should only fire once");
}

// =============================================================================
// Integration Tests: MessageEnvelope
// =============================================================================

#[test]
fn test_envelope_sender_metadata() {
    let envelope = MessageEnvelope::new(42, Some("my-widget"), "Switch");

    assert_eq!(envelope.message, 42);
    assert_eq!(envelope.sender_id, Some("my-widget".to_string()));
    assert_eq!(envelope.sender_type, "Switch");
    assert!(envelope.is_bubbling());
}

#[test]
fn test_envelope_stop_bubbling() {
    let mut envelope = MessageEnvelope::new(42, None, "Test");
    assert!(envelope.is_bubbling());

    envelope.stop();
    assert!(!envelope.is_bubbling());
}
