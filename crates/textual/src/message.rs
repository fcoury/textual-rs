//! Message envelope for bubbling events through the widget tree.
//!
//! Messages are wrapped in envelopes that carry metadata about their origin
//! and control whether they should continue bubbling up to parent widgets.

/// Envelope wrapping a message with source metadata and bubbling control.
///
/// When a widget produces a message (e.g., a Switch toggled), the message
/// is wrapped in an envelope that tracks:
/// - The original message payload
/// - Which widget sent it (by ID and type name)
/// - Whether it should continue bubbling up the tree
#[derive(Debug, Clone)]
pub struct MessageEnvelope<M> {
    /// The actual message payload.
    pub message: M,

    /// Optional ID of the widget that produced this message.
    /// Set via `Widget::with_id("my-widget")`.
    pub sender_id: Option<String>,

    /// Type name of the widget that produced this message.
    /// Automatically extracted from the widget's type.
    pub sender_type: String,

    /// Whether this message should continue bubbling up to parent widgets.
    /// Call `stop()` to prevent further propagation.
    bubbling: bool,
}

impl<M> MessageEnvelope<M> {
    /// Create a new envelope for a message.
    ///
    /// # Arguments
    /// * `message` - The message payload
    /// * `sender_id` - Optional widget ID (from `Widget::id()`)
    /// * `sender_type` - Widget type name (from `Widget::type_name()`)
    pub fn new(message: M, sender_id: Option<&str>, sender_type: &str) -> Self {
        Self {
            message,
            sender_id: sender_id.map(String::from),
            sender_type: sender_type.to_string(),
            bubbling: true,
        }
    }

    /// Stop this message from bubbling further up the widget tree.
    ///
    /// After calling this, parent widgets will not receive the message
    /// via `handle_message`. The message will still be delivered to
    /// `App::handle_message` as the final handler.
    pub fn stop(&mut self) {
        self.bubbling = false;
    }

    /// Check if this message should continue bubbling up the tree.
    pub fn is_bubbling(&self) -> bool {
        self.bubbling
    }

    /// Transform the message payload while preserving envelope metadata.
    ///
    /// Useful when a parent widget wants to transform a child's message
    /// into a different message type.
    pub fn map<N, F>(self, f: F) -> MessageEnvelope<N>
    where
        F: FnOnce(M) -> N,
    {
        MessageEnvelope {
            message: f(self.message),
            sender_id: self.sender_id,
            sender_type: self.sender_type,
            bubbling: self.bubbling,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_creation() {
        let envelope = MessageEnvelope::new("test", Some("my-id"), "Switch");

        assert_eq!(envelope.message, "test");
        assert_eq!(envelope.sender_id, Some("my-id".to_string()));
        assert_eq!(envelope.sender_type, "Switch");
        assert!(envelope.is_bubbling());
    }

    #[test]
    fn test_stop_bubbling() {
        let mut envelope = MessageEnvelope::new(42, None, "Button");

        assert!(envelope.is_bubbling());
        envelope.stop();
        assert!(!envelope.is_bubbling());
    }

    #[test]
    fn test_map_transforms_message() {
        let envelope = MessageEnvelope::new(10, Some("counter"), "Counter");
        let mapped = envelope.map(|n| n * 2);

        assert_eq!(mapped.message, 20);
        assert_eq!(mapped.sender_id, Some("counter".to_string()));
        assert_eq!(mapped.sender_type, "Counter");
    }
}
