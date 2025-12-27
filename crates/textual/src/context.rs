//! Application context for widgets to interact with the event system.
//!
//! The `AppContext` is passed to widgets during lifecycle events like `on_mount`,
//! allowing them to:
//! - Post messages to be processed in the next event loop tick
//! - Set timers and intervals for delayed/periodic messages
//! - Spawn async tasks that can send messages back

use std::time::Duration;

use tokio::sync::mpsc;

use crate::message::MessageEnvelope;

/// Unique identifier for a widget in the tree.
pub type WidgetId = usize;

/// Context provided to widgets for posting messages and spawning async tasks.
///
/// Clone this context to share it with async tasks. The underlying channel
/// is designed to be shared across threads.
#[derive(Clone)]
pub struct AppContext<M> {
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    /// The widget ID this context belongs to (set during mount).
    widget_id: Option<WidgetId>,
}

impl<M> AppContext<M> {
    /// Create a new context with the given message sender.
    pub fn new(sender: mpsc::UnboundedSender<MessageEnvelope<M>>) -> Self {
        Self {
            sender,
            widget_id: None,
        }
    }

    /// Create a context bound to a specific widget.
    pub fn with_widget_id(mut self, id: WidgetId) -> Self {
        self.widget_id = Some(id);
        self
    }

    /// Get the widget ID this context belongs to.
    pub fn widget_id(&self) -> Option<WidgetId> {
        self.widget_id
    }

    /// Get a clone of the sender for use in async tasks.
    ///
    /// Use this when you need to send messages from a spawned task:
    /// ```ignore
    /// let sender = ctx.sender();
    /// tokio::spawn(async move {
    ///     // do some async work
    ///     let _ = sender.send(envelope);
    /// });
    /// ```
    pub fn sender(&self) -> mpsc::UnboundedSender<MessageEnvelope<M>> {
        self.sender.clone()
    }
}

impl<M: Send + 'static> AppContext<M> {
    /// Post a message to be processed in the next event loop tick.
    ///
    /// The message will be wrapped in an envelope and delivered to
    /// `App::handle_message` without going through the bubbling mechanism.
    pub fn post(&self, message: M) {
        let envelope = MessageEnvelope::new(message, None, "AppContext");
        let _ = self.sender.send(envelope);
    }

    /// Set a one-shot timer that fires a message after a delay.
    ///
    /// # Example
    /// ```ignore
    /// ctx.set_timer(Duration::from_secs(5), Message::Timeout);
    /// ```
    pub fn set_timer(&self, delay: Duration, message: M) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let envelope = MessageEnvelope::new(message, None, "Timer");
            let _ = sender.send(envelope);
        });
    }

    /// Set a repeating interval that fires a message periodically.
    ///
    /// Returns a handle that can be used to cancel the interval.
    ///
    /// # Example
    /// ```ignore
    /// let handle = ctx.set_interval(Duration::from_millis(100), || Message::Tick);
    /// // Later...
    /// handle.cancel();
    /// ```
    pub fn set_interval<F>(&self, interval: Duration, message_fn: F) -> IntervalHandle
    where
        F: Fn() -> M + Send + 'static,
    {
        let sender = self.sender.clone();
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let envelope = MessageEnvelope::new(message_fn(), None, "Interval");
                        if sender.send(envelope).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    _ = &mut cancel_rx => {
                        break; // Cancelled
                    }
                }
            }
        });

        IntervalHandle {
            cancel_tx: Some(cancel_tx),
        }
    }
}

/// Handle to cancel a running interval.
///
/// The interval is automatically cancelled when this handle is dropped.
pub struct IntervalHandle {
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl IntervalHandle {
    /// Cancel the interval, stopping further messages from being sent.
    pub fn cancel(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for IntervalHandle {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_post_message() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ctx: AppContext<i32> = AppContext::new(tx);

        ctx.post(42);

        let envelope = rx.recv().await.unwrap();
        assert_eq!(envelope.message, 42);
        assert_eq!(envelope.sender_type, "AppContext");
    }

    #[tokio::test]
    async fn test_timer_fires() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ctx: AppContext<&str> = AppContext::new(tx);

        ctx.set_timer(Duration::from_millis(10), "timeout");

        let envelope = rx.recv().await.unwrap();
        assert_eq!(envelope.message, "timeout");
        assert_eq!(envelope.sender_type, "Timer");
    }

    #[tokio::test]
    async fn test_interval_fires_multiple_times() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ctx: AppContext<i32> = AppContext::new(tx);

        let _handle = ctx.set_interval(Duration::from_millis(10), || 42);

        // Should receive at least 2 messages
        let msg1 = rx.recv().await.unwrap();
        let msg2 = rx.recv().await.unwrap();
        assert_eq!(msg1.message, 42);
        assert_eq!(msg2.message, 42);
    }

    #[tokio::test]
    async fn test_interval_cancel() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ctx: AppContext<i32> = AppContext::new(tx);

        let mut handle = ctx.set_interval(Duration::from_millis(5), || 1);

        // Wait for one tick
        let _ = rx.recv().await.unwrap();

        // Cancel and wait a bit
        handle.cancel();
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Channel should be empty (or have at most one more message from race)
        // Using try_recv to avoid blocking
        let remaining: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert!(remaining.len() <= 1, "Should have stopped after cancel");
    }
}
