//! Widget tree management with focus path caching.
//!
//! The `WidgetTree` provides O(d) event dispatch by caching the path from
//! root to the focused widget. Instead of searching through all containers,
//! events go directly to the focused widget and bubble up through the cached path.

use crate::message::MessageEnvelope;
use crate::widget::Widget;
use crate::KeyCode;

/// Sender information extracted from a widget.
#[derive(Debug, Clone)]
pub struct SenderInfo {
    /// The widget's ID (if set via `with_id`).
    pub id: Option<String>,
    /// The widget's type name.
    pub type_name: &'static str,
}

/// A path from the root to a specific widget in the tree.
///
/// Each element is the child index at that level of the tree.
/// For example, `[0, 2, 1]` means:
/// - Start at root
/// - Take child 0
/// - Take child 2 of that
/// - Take child 1 of that (the target)
#[derive(Debug, Clone, Default)]
pub struct FocusPath {
    /// Child indices from root to focused widget
    indices: Vec<usize>,
}

impl FocusPath {
    /// Create an empty path (pointing to root).
    pub fn new() -> Self {
        Self { indices: Vec::new() }
    }

    /// Get the path indices.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Get the depth of this path.
    pub fn depth(&self) -> usize {
        self.indices.len()
    }

    /// Push a child index onto the path.
    pub fn push(&mut self, index: usize) {
        self.indices.push(index);
    }

    /// Pop the last index from the path.
    pub fn pop(&mut self) -> Option<usize> {
        self.indices.pop()
    }

    /// Clear the path.
    pub fn clear(&mut self) {
        self.indices.clear();
    }
}

/// Manages a widget tree with cached focus path for O(d) dispatch.
pub struct WidgetTree<M> {
    /// The root widget of the tree.
    root: Box<dyn Widget<M>>,
    /// Cached path from root to the focused widget.
    focus_path: FocusPath,
    /// The focus index that was used to compute the current path.
    current_focus_index: usize,
}

impl<M> WidgetTree<M> {
    /// Create a new widget tree from a root widget.
    pub fn new(root: Box<dyn Widget<M>>) -> Self {
        Self {
            root,
            focus_path: FocusPath::new(),
            current_focus_index: 0,
        }
    }

    /// Get a reference to the root widget.
    pub fn root(&self) -> &dyn Widget<M> {
        self.root.as_ref()
    }

    /// Get a mutable reference to the root widget.
    pub fn root_mut(&mut self) -> &mut dyn Widget<M> {
        self.root.as_mut()
    }

    /// Get the current focus path.
    pub fn focus_path(&self) -> &FocusPath {
        &self.focus_path
    }

    /// Update the focus path for a given focus index.
    ///
    /// This walks the tree to find the nth focusable widget and caches
    /// the path to it. Call this when focus changes.
    pub fn update_focus(&mut self, focus_index: usize) {
        if focus_index == self.current_focus_index && !self.focus_path.indices.is_empty() {
            return; // Path is already cached for this focus index
        }

        self.focus_path.clear();
        self.current_focus_index = focus_index;

        // Walk the tree to find the path to the nth focusable widget
        let mut remaining = focus_index;
        find_focus_path_recursive(self.root.as_mut(), &mut self.focus_path, &mut remaining);
    }

    /// Navigate to the focused widget and call the given function on it.
    ///
    /// Returns the result of the function, or None if the path is invalid.
    pub fn with_focused<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        let path = self.focus_path.indices.clone();
        let mut current: &mut dyn Widget<M> = self.root.as_mut();

        for &index in &path {
            current = current.get_child_mut(index)?;
        }

        Some(f(current))
    }

    /// Dispatch a key event to the focused widget.
    ///
    /// Returns the message produced, if any.
    pub fn dispatch_key(&mut self, key: KeyCode) -> Option<M> {
        self.with_focused(|widget| widget.on_event(key)).flatten()
    }

    /// Get sender info for the focused widget.
    ///
    /// Uses the cached focus path for O(d) access instead of tree search.
    pub fn focused_sender_info(&mut self) -> SenderInfo {
        self.with_focused(|widget| SenderInfo {
            id: widget.id().map(|s| s.to_string()),
            type_name: widget.type_name(),
        })
        .unwrap_or(SenderInfo {
            id: None,
            type_name: "Widget",
        })
    }

    /// Bubble a message up from the focused widget to ancestors.
    ///
    /// Each ancestor gets a chance to intercept the message via `handle_message`.
    /// If an ancestor calls `envelope.stop()`, bubbling stops.
    ///
    /// Returns the final envelope (possibly transformed by ancestors).
    pub fn bubble_message(&mut self, mut envelope: MessageEnvelope<M>) -> MessageEnvelope<M> {
        let path = self.focus_path.indices.clone();

        // Walk the path in reverse (from focused widget's parent up to root)
        // Skip the focused widget itself (it produced the message)
        // When depth=0, ancestor_path is empty [], so navigate_and_handle calls root directly
        for depth in (0..path.len()).rev() {
            if !envelope.is_bubbling() {
                break;
            }

            // Navigate to the ancestor at this depth and call handle_message
            let ancestor_path = &path[..depth];
            if let Some(new_msg) = navigate_and_handle(self.root.as_mut(), ancestor_path, &mut envelope) {
                envelope.message = new_msg;
            }
        }

        envelope
    }
}

/// Navigate to a widget at the given path and call handle_message on it.
///
/// This is a free function to avoid borrow conflicts in bubble_message.
fn navigate_and_handle<M>(
    mut widget: &mut dyn Widget<M>,
    path: &[usize],
    envelope: &mut MessageEnvelope<M>,
) -> Option<M> {
    for &index in path {
        match widget.get_child_mut(index) {
            Some(child) => widget = child,
            None => return None,
        }
    }
    widget.handle_message(envelope)
}

/// Recursively find the path to the nth focusable widget.
///
/// This is a free function to avoid borrow conflicts when updating the focus path.
/// Returns true if the focused widget was found in this subtree.
fn find_focus_path_recursive<M>(
    widget: &mut dyn Widget<M>,
    path: &mut FocusPath,
    remaining: &mut usize,
) -> bool {
    // Check if this widget is focusable
    if widget.is_focusable() {
        if *remaining == 0 {
            return true; // Found it!
        }
        *remaining -= 1;
    }

    // Check children
    let child_count = widget.child_count();
    for i in 0..child_count {
        path.push(i);
        if let Some(child) = widget.get_child_mut(i) {
            if find_focus_path_recursive(child, path, remaining) {
                return true; // Found in this subtree
            }
        }
        path.pop();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test widget
    struct TestWidget {
        focusable: bool,
        children: Vec<Box<dyn Widget<()>>>,
    }

    impl TestWidget {
        fn focusable() -> Box<dyn Widget<()>> {
            Box::new(Self {
                focusable: true,
                children: Vec::new(),
            })
        }

        fn container(children: Vec<Box<dyn Widget<()>>>) -> Box<dyn Widget<()>> {
            Box::new(Self {
                focusable: false,
                children,
            })
        }
    }

    impl Widget<()> for TestWidget {
        fn render(&self, _canvas: &mut crate::Canvas, _region: crate::Region) {}

        fn desired_size(&self) -> crate::Size {
            crate::Size { width: 1, height: 1 }
        }

        fn is_focusable(&self) -> bool {
            self.focusable
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
    }

    #[test]
    fn test_focus_path_simple() {
        // Tree: Container [ Focusable, Focusable ]
        let tree_root = TestWidget::container(vec![
            TestWidget::focusable(),
            TestWidget::focusable(),
        ]);

        let mut tree = WidgetTree::new(tree_root);

        // Focus on first focusable (index 0)
        tree.update_focus(0);
        assert_eq!(tree.focus_path().indices(), &[0]);

        // Focus on second focusable (index 1)
        tree.update_focus(1);
        assert_eq!(tree.focus_path().indices(), &[1]);
    }

    #[test]
    fn test_focus_path_nested() {
        // Tree: Container [ Container [ Focusable ], Focusable ]
        let tree_root = TestWidget::container(vec![
            TestWidget::container(vec![TestWidget::focusable()]),
            TestWidget::focusable(),
        ]);

        let mut tree = WidgetTree::new(tree_root);

        // Focus on nested focusable (index 0)
        tree.update_focus(0);
        assert_eq!(tree.focus_path().indices(), &[0, 0]);

        // Focus on second focusable (index 1)
        tree.update_focus(1);
        assert_eq!(tree.focus_path().indices(), &[1]);
    }
}
