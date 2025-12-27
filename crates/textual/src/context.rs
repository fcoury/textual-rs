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

/// Context provided to widgets for posting messages and spawning async tasks.
///
/// Clone this context to share it with async tasks. The underlying channel
/// is designed to be shared across threads.
///
/// When bound to a widget via `with_sender_info`, messages sent through this
/// context will include the widget's ID and type for attribution.
#[derive(Clone)]
pub struct AppContext<M> {
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    /// The widget's string ID (from `Widget::id()`).
    sender_id: Option<String>,
    /// The widget's type name (from `Widget::type_name()`).
    sender_type: String,
}

impl<M> AppContext<M> {
    /// Create a new context with the given message sender.
    pub fn new(sender: mpsc::UnboundedSender<MessageEnvelope<M>>) -> Self {
        Self {
            sender,
            sender_id: None,
            sender_type: "AppContext".to_string(),
        }
    }

    /// Bind this context to a widget's sender info.
    ///
    /// Messages sent via `post`, `set_timer`, or `set_interval` will include
    /// this sender metadata, allowing the App to identify the source widget.
    pub fn with_sender_info(mut self, id: Option<&str>, type_name: &str) -> Self {
        self.sender_id = id.map(String::from);
        self.sender_type = type_name.to_string();
        self
    }

    /// Get the sender ID this context is bound to.
    pub fn sender_id(&self) -> Option<&str> {
        self.sender_id.as_deref()
    }

    /// Get the sender type this context is bound to.
    pub fn sender_type(&self) -> &str {
        &self.sender_type
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
    ///
    /// If this context is bound to a widget via `with_sender_info`, the
    /// envelope will include the widget's ID and type.
    pub fn post(&self, message: M) {
        let envelope = MessageEnvelope::new(message, self.sender_id.as_deref(), &self.sender_type);
        let _ = self.sender.send(envelope);
    }

    /// Set a one-shot timer that fires a message after a delay.
    ///
    /// If this context is bound to a widget, the timer message will include
    /// the widget's sender info for attribution.
    ///
    /// # Example
    /// ```ignore
    /// ctx.set_timer(Duration::from_secs(5), Message::Timeout);
    /// ```
    pub fn set_timer(&self, delay: Duration, message: M) {
        let sender = self.sender.clone();
        let sender_id = self.sender_id.clone();
        let sender_type = self.sender_type.clone();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let envelope = MessageEnvelope::new(message, sender_id.as_deref(), &sender_type);
            let _ = sender.send(envelope);
        });
    }

    /// Set a repeating interval that fires a message periodically.
    ///
    /// Returns a handle that can be used to cancel the interval.
    ///
    /// If this context is bound to a widget, interval messages will include
    /// the widget's sender info for attribution.
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
        let sender_id = self.sender_id.clone();
        let sender_type = self.sender_type.clone();
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            // Use interval_at with delayed start to ensure cancellation can suppress all ticks.
            // tokio::time::interval() fires immediately on first tick, creating a race with cancel.
            let start = tokio::time::Instant::now() + interval;
            let mut ticker = tokio::time::interval_at(start, interval);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let envelope = MessageEnvelope::new(message_fn(), sender_id.as_deref(), &sender_type);
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
        // Unbound context uses default sender_type
        assert_eq!(envelope.sender_type, "AppContext");
    }

    #[tokio::test]
    async fn test_timer_preserves_sender_info() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ctx: AppContext<&str> = AppContext::new(tx)
            .with_sender_info(Some("my-timer-widget"), "TimerWidget");

        ctx.set_timer(Duration::from_millis(10), "timeout");

        let envelope = rx.recv().await.unwrap();
        assert_eq!(envelope.message, "timeout");
        assert_eq!(envelope.sender_id, Some("my-timer-widget".to_string()));
        assert_eq!(envelope.sender_type, "TimerWidget");
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
