//! Application context for widgets to interact with the event system.
//!
//! The `AppContext` is passed to widgets during lifecycle events like `on_mount`,
//! allowing them to:
//! - Post messages to be processed in the next event loop tick
//! - Set timers and intervals for delayed/periodic messages
//! - Spawn async tasks that can send messages back
//!
//! The `MountContext` extends `AppContext` with widget tree access for querying
//! and modifying widgets during the `on_mount` lifecycle hook.

use std::time::Duration;

use tokio::sync::mpsc;

use crate::message::MessageEnvelope;
use crate::tree::WidgetTree;
use crate::Widget;

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
        log::debug!("TIMER: Spawning timer for {:?}", delay);
        tokio::spawn(async move {
            log::debug!("TIMER: Task started, sleeping for {:?}", delay);
            tokio::time::sleep(delay).await;
            log::debug!("TIMER: Woke up, sending message");
            let envelope = MessageEnvelope::new(message, sender_id.as_deref(), &sender_type);
            match sender.send(envelope) {
                Ok(_) => log::debug!("TIMER: Message sent successfully"),
                Err(e) => log::error!("TIMER: Failed to send message: {:?}", e),
            }
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

        log::debug!("INTERVAL: Spawning interval for {:?}", interval);
        tokio::spawn(async move {
            // Use interval_at with delayed start to ensure cancellation can suppress all ticks.
            // tokio::time::interval() fires immediately on first tick, creating a race with cancel.
            let start = tokio::time::Instant::now() + interval;
            let mut ticker = tokio::time::interval_at(start, interval);
            log::debug!("INTERVAL: Task started, first tick at {:?}", start);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        log::debug!("INTERVAL: Tick! Sending message");
                        let envelope = MessageEnvelope::new(message_fn(), sender_id.as_deref(), &sender_type);
                        if sender.send(envelope).is_err() {
                            log::error!("INTERVAL: Receiver dropped");
                            break; // Receiver dropped
                        }
                    }
                    _ = &mut cancel_rx => {
                        log::debug!("INTERVAL: Cancelled");
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

// =============================================================================
// MountContext - Context with widget tree access for on_mount
// =============================================================================

/// Context provided during the `on_mount` lifecycle hook.
///
/// `MountContext` wraps `AppContext` and adds access to the widget tree,
/// enabling widget queries and modifications during initialization.
///
/// # Example
/// ```ignore
/// fn on_mount(&mut self, ctx: &mut MountContext<Self::Message>) {
///     // Find a widget by ID and modify it
///     ctx.with_widget_by_id("my-label", |widget| {
///         widget.set_border_title("Textual Rocks!");
///     });
///
///     // Use AppContext methods for timers/intervals
///     ctx.set_interval(Duration::from_secs(1), || Message::Tick);
/// }
/// ```
pub struct MountContext<'a, M> {
    app_ctx: AppContext<M>,
    tree: &'a mut WidgetTree<M>,
}

impl<'a, M> MountContext<'a, M> {
    /// Create a new MountContext wrapping an AppContext and WidgetTree.
    pub fn new(app_ctx: AppContext<M>, tree: &'a mut WidgetTree<M>) -> Self {
        Self { app_ctx, tree }
    }

    /// Find a widget by ID and call a closure with mutable access.
    ///
    /// Returns `Some(R)` if the widget was found, `None` otherwise.
    ///
    /// # Example
    /// ```ignore
    /// ctx.with_widget_by_id("status-label", |widget| {
    ///     widget.set_border_title("Ready");
    /// });
    /// ```
    pub fn with_widget_by_id<F, R>(&mut self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        self.tree.with_widget_by_id(id, f)
    }

    /// Find a widget by type name and call a closure with mutable access.
    ///
    /// Finds the first widget with the given type name (e.g., "Label", "Switch").
    ///
    /// # Example
    /// ```ignore
    /// ctx.with_widget_by_type("Label", |widget| {
    ///     widget.set_border_subtitle("Found!");
    /// });
    /// ```
    pub fn with_widget_by_type<F, R>(&mut self, type_name: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        self.tree.with_widget_by_type(type_name, f)
    }

    /// Query for a single widget using a CSS-like selector.
    ///
    /// Supports the following selector formats:
    /// - `"#my-id"` - ID selector (finds widget with id="my-id")
    /// - `"Label"` - Type selector (finds first Label widget)
    /// - `"Button#submit"` - Combined Type#ID (finds Button with id="submit")
    ///
    /// # Example
    /// ```ignore
    /// // Find by ID
    /// ctx.query_one("#my-label", |widget| {
    ///     widget.set_border_title("Found by ID!");
    /// });
    ///
    /// // Find by type
    /// ctx.query_one("Label", |widget| {
    ///     widget.set_border_title("Found first Label!");
    /// });
    ///
    /// // Find by type AND ID
    /// ctx.query_one("Button#submit", |widget| {
    ///     widget.set_border_title("Found Submit Button!");
    /// });
    /// ```
    pub fn query_one<F, R>(&mut self, selector: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        self.tree.query_one(selector, f)
    }

    /// Query for a single widget using a selector, with typed downcast access.
    ///
    /// This is like `query_one`, but downcasts the widget to a concrete type,
    /// giving you access to type-specific methods instead of just `&mut dyn Widget`.
    ///
    /// Returns `Some(R)` if the widget was found AND could be downcast to type `W`.
    /// Returns `None` if the widget wasn't found or couldn't be downcast.
    ///
    /// # Example
    /// ```ignore
    /// use textual::Label;
    ///
    /// // Get typed access to a Label widget
    /// ctx.query_one_as::<Label<_>, _, _>("#my-label", |label| {
    ///     // label is &mut Label, not &mut dyn Widget
    ///     label.update("New text!");
    /// });
    ///
    /// // Combined selector with typed access
    /// ctx.query_one_as::<Container<_>, _, _>("Container#sidebar", |container| {
    ///     container.set_border_title("Sidebar");
    /// });
    /// ```
    pub fn query_one_as<W, F, R>(&mut self, selector: &str, f: F) -> Option<R>
    where
        W: 'static,
        F: FnOnce(&mut W) -> R,
    {
        self.tree.query_one_as::<W, F, R>(selector, f)
    }

    /// Get the underlying AppContext for timer/interval operations.
    pub fn app_context(&self) -> &AppContext<M> {
        &self.app_ctx
    }
}

// Delegate AppContext methods to MountContext
impl<'a, M: Send + 'static> MountContext<'a, M> {
    /// Post a message to be processed in the next event loop tick.
    ///
    /// See [`AppContext::post`] for details.
    pub fn post(&self, message: M) {
        self.app_ctx.post(message);
    }

    /// Set a one-shot timer that fires a message after a delay.
    ///
    /// See [`AppContext::set_timer`] for details.
    pub fn set_timer(&self, delay: Duration, message: M) {
        self.app_ctx.set_timer(delay, message);
    }

    /// Set a repeating interval that fires a message periodically.
    ///
    /// See [`AppContext::set_interval`] for details.
    pub fn set_interval<F>(&self, interval: Duration, message_fn: F) -> IntervalHandle
    where
        F: Fn() -> M + Send + 'static,
    {
        self.app_ctx.set_interval(interval, message_fn)
    }

    /// Get a clone of the sender for use in async tasks.
    ///
    /// See [`AppContext::sender`] for details.
    pub fn sender(&self) -> mpsc::UnboundedSender<MessageEnvelope<M>> {
        self.app_ctx.sender()
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

    // ========================================================================
    // MountContext tests
    // ========================================================================

    use crate::canvas::Size;
    use crate::Canvas;
    use crate::Region;

    /// Minimal test widget for MountContext tests
    struct MountTestWidget {
        id: Option<String>,
        type_name: &'static str,
        children: Vec<Box<dyn Widget<()>>>,
        border_title: Option<String>,
    }

    impl MountTestWidget {
        fn new(type_name: &'static str) -> Self {
            Self {
                id: None,
                type_name,
                children: Vec::new(),
                border_title: None,
            }
        }

        fn with_id(mut self, id: &str) -> Self {
            self.id = Some(id.to_string());
            self
        }

        fn with_children(mut self, children: Vec<Box<dyn Widget<()>>>) -> Self {
            self.children = children;
            self
        }

        fn boxed(self) -> Box<dyn Widget<()>> {
            Box::new(self)
        }
    }

    impl Widget<()> for MountTestWidget {
        fn render(&self, _canvas: &mut Canvas, _region: Region) {}

        fn desired_size(&self) -> Size {
            Size { width: 1, height: 1 }
        }

        fn id(&self) -> Option<&str> {
            self.id.as_deref()
        }

        fn type_name(&self) -> &'static str {
            self.type_name
        }

        fn child_count(&self) -> usize {
            self.children.len()
        }

        fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<()> + '_)> {
            if index < self.children.len() {
                Some(self.children[index].as_mut())
            } else {
                None
            }
        }

        fn set_border_title(&mut self, title: &str) {
            self.border_title = Some(title.to_string());
        }

        fn border_title(&self) -> Option<&str> {
            self.border_title.as_deref()
        }
    }

    #[test]
    fn test_mount_context_with_widget_by_id() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<()> = AppContext::new(tx);

        let root = MountTestWidget::new("Container")
            .with_children(vec![
                MountTestWidget::new("Label").with_id("my-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);
        let mut ctx = MountContext::new(app_ctx, &mut tree);

        // Query by ID and modify
        let result = ctx.with_widget_by_id("my-label", |widget| {
            widget.set_border_title("Modified Title");
            widget.type_name().to_string()
        });

        assert_eq!(result, Some("Label".to_string()));

        // Verify the modification persisted
        let title = ctx.with_widget_by_id("my-label", |widget| {
            widget.border_title().map(|s| s.to_string())
        });

        assert_eq!(title, Some(Some("Modified Title".to_string())));
    }

    #[test]
    fn test_mount_context_with_widget_by_id_not_found() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<()> = AppContext::new(tx);

        let root = MountTestWidget::new("Container").boxed();

        let mut tree = WidgetTree::new(root);
        let mut ctx = MountContext::new(app_ctx, &mut tree);

        let result = ctx.with_widget_by_id("nonexistent", |_| ());

        assert!(result.is_none());
    }

    #[test]
    fn test_mount_context_with_widget_by_type() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<()> = AppContext::new(tx);

        let root = MountTestWidget::new("Container")
            .with_children(vec![
                MountTestWidget::new("Button").with_id("btn").boxed(),
                MountTestWidget::new("Label").with_id("lbl").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);
        let mut ctx = MountContext::new(app_ctx, &mut tree);

        // Query by type - should find Button first
        let result = ctx.with_widget_by_type("Button", |widget| {
            widget.id().map(|s| s.to_string())
        });

        assert_eq!(result, Some(Some("btn".to_string())));
    }

    #[test]
    fn test_mount_context_with_widget_by_type_not_found() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<()> = AppContext::new(tx);

        let root = MountTestWidget::new("Container").boxed();

        let mut tree = WidgetTree::new(root);
        let mut ctx = MountContext::new(app_ctx, &mut tree);

        let result = ctx.with_widget_by_type("NonexistentType", |_| ());

        assert!(result.is_none());
    }

    #[test]
    fn test_mount_context_nested_query() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<()> = AppContext::new(tx);

        // Deep nesting: Container > Container > Container > Label
        let root = MountTestWidget::new("Container")
            .with_children(vec![
                MountTestWidget::new("Container")
                    .with_children(vec![
                        MountTestWidget::new("Container")
                            .with_children(vec![
                                MountTestWidget::new("Label").with_id("deep").boxed(),
                            ])
                            .boxed(),
                    ])
                    .boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);
        let mut ctx = MountContext::new(app_ctx, &mut tree);

        // Should find deeply nested widget
        let result = ctx.with_widget_by_id("deep", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(result, Some("Label".to_string()));
    }

    #[tokio::test]
    async fn test_mount_context_post_delegates_to_app_context() {
        // MountTestWidget uses () as message type, so we need a different test widget for i32
        struct IntWidget;
        impl Widget<i32> for IntWidget {
            fn render(&self, _canvas: &mut Canvas, _region: Region) {}
            fn desired_size(&self) -> Size { Size { width: 1, height: 1 } }
        }

        let (tx, mut rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<i32> = AppContext::new(tx);

        let root: Box<dyn Widget<i32>> = Box::new(IntWidget);
        let mut tree = WidgetTree::new(root);
        let ctx = MountContext::new(app_ctx, &mut tree);

        // post() should delegate to AppContext
        ctx.post(123);

        let envelope = rx.recv().await.unwrap();
        assert_eq!(envelope.message, 123);
    }

    #[test]
    fn test_mount_context_app_context_accessor() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app_ctx: AppContext<()> = AppContext::new(tx)
            .with_sender_info(Some("widget-id"), "WidgetType");

        let root = MountTestWidget::new("Container").boxed();

        let mut tree = WidgetTree::new(root);
        let ctx = MountContext::new(app_ctx, &mut tree);

        // Can access the underlying AppContext
        assert_eq!(ctx.app_context().sender_id(), Some("widget-id"));
        assert_eq!(ctx.app_context().sender_type(), "WidgetType");
    }
}
