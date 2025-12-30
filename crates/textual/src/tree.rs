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

    // =========================================================================
    // Widget Query API
    // =========================================================================

    /// Find a widget by ID and call a closure with mutable access.
    ///
    /// Performs a depth-first search through the widget tree to find a widget
    /// with the matching ID. If found, calls the closure with the widget and
    /// returns the closure's result.
    ///
    /// # Example
    /// ```ignore
    /// tree.with_widget_by_id("my-label", |widget| {
    ///     widget.set_border_title("Updated!");
    /// });
    /// ```
    pub fn with_widget_by_id<F, R>(&mut self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        let mut result = None;
        let mut f = Some(f);
        find_and_apply_by_id(self.root.as_mut(), id, &mut |widget| {
            if let Some(f) = f.take() {
                result = Some(f(widget));
            }
        });
        result
    }

    /// Find a widget by type name and call a closure with mutable access.
    ///
    /// Performs a depth-first search through the widget tree to find the first
    /// widget with a matching type name (e.g., "Label", "Switch").
    ///
    /// # Example
    /// ```ignore
    /// tree.with_widget_by_type("Label", |widget| {
    ///     widget.set_border_title("Found!");
    /// });
    /// ```
    pub fn with_widget_by_type<F, R>(&mut self, type_name: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        let mut result = None;
        let mut f = Some(f);
        find_and_apply_by_type(self.root.as_mut(), type_name, &mut |widget| {
            if let Some(f) = f.take() {
                result = Some(f(widget));
            }
        });
        result
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
    /// tree.query_one("#my-label", |widget| {
    ///     widget.set_border_title("Found by ID!");
    /// });
    ///
    /// // Find by type
    /// tree.query_one("Label", |widget| {
    ///     widget.set_border_title("Found first Label!");
    /// });
    ///
    /// // Find by type AND ID
    /// tree.query_one("Button#submit", |widget| {
    ///     widget.set_border_title("Found Submit Button!");
    /// });
    /// ```
    pub fn query_one<F, R>(&mut self, selector: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget<M>) -> R,
    {
        let parsed = parse_simple_selector(selector);
        let mut result = None;
        let mut f = Some(f);
        find_and_apply_by_selector(self.root.as_mut(), &parsed, &mut |widget| {
            if let Some(f) = f.take() {
                result = Some(f(widget));
            }
        });
        result
    }

    /// Query for a single widget with typed access via downcasting.
    ///
    /// This is the typed version of `query_one` that provides direct access
    /// to the concrete widget type instead of `&mut dyn Widget<M>`.
    ///
    /// The type parameter `W` must match the actual widget type. If the selector
    /// finds a widget but it's not of type `W`, returns `None`.
    ///
    /// # Example
    /// ```ignore
    /// // Find Label by ID and get typed access
    /// tree.query_one_as::<Label<_>, _, _>("#my-label", |label| {
    ///     // label is &mut Label, not &mut dyn Widget
    ///     label.set_text("Updated!");
    /// });
    ///
    /// // Combined selector with type verification
    /// tree.query_one_as::<Container<_>, _, _>("Container#sidebar", |container| {
    ///     container.set_border_title("Sidebar");
    /// });
    /// ```
    pub fn query_one_as<W, F, R>(&mut self, selector: &str, f: F) -> Option<R>
    where
        W: 'static,
        F: FnOnce(&mut W) -> R,
    {
        let parsed = parse_simple_selector(selector);
        let mut result = None;
        let mut f = Some(f);
        find_and_apply_by_selector_typed::<M, W, _>(
            self.root.as_mut(),
            &parsed,
            &mut |widget: &mut W| {
                if let Some(f) = f.take() {
                    result = Some(f(widget));
                }
            },
        );
        result
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

/// Recursively find a widget by ID and apply a closure.
///
/// Performs depth-first search to find the first widget with the matching ID.
/// Returns true if the widget was found and the closure was applied.
fn find_and_apply_by_id<M, F>(widget: &mut dyn Widget<M>, id: &str, f: &mut F) -> bool
where
    F: FnMut(&mut dyn Widget<M>),
{
    if widget.id() == Some(id) {
        f(widget);
        return true;
    }

    let child_count = widget.child_count();
    for i in 0..child_count {
        if let Some(child) = widget.get_child_mut(i) {
            if find_and_apply_by_id(child, id, f) {
                return true;
            }
        }
    }
    false
}

/// Recursively find a widget by type name and apply a closure.
///
/// Performs depth-first search to find the first widget with the matching type name.
/// Returns true if the widget was found and the closure was applied.
fn find_and_apply_by_type<M, F>(widget: &mut dyn Widget<M>, type_name: &str, f: &mut F) -> bool
where
    F: FnMut(&mut dyn Widget<M>),
{
    if widget.type_name() == type_name {
        f(widget);
        return true;
    }

    let child_count = widget.child_count();
    for i in 0..child_count {
        if let Some(child) = widget.get_child_mut(i) {
            if find_and_apply_by_type(child, type_name, f) {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// Simple Selector Parsing
// =============================================================================

/// A parsed simple selector.
///
/// Supports:
/// - `#id` - ID only
/// - `Type` - Type name only
/// - `Type#id` - Both type and ID
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleSelector {
    /// Type name constraint (e.g., "Label", "Button")
    pub type_name: Option<String>,
    /// ID constraint (e.g., "my-label")
    pub id: Option<String>,
}

impl SimpleSelector {
    /// Check if a widget matches this selector.
    pub fn matches<M>(&self, widget: &dyn Widget<M>) -> bool {
        // Check type constraint if present
        if let Some(ref type_name) = self.type_name {
            if widget.type_name() != type_name {
                return false;
            }
        }

        // Check ID constraint if present
        if let Some(ref id) = self.id {
            if widget.id() != Some(id.as_str()) {
                return false;
            }
        }

        // If no constraints, match nothing (empty selector)
        self.type_name.is_some() || self.id.is_some()
    }
}

/// Parse a simple selector string.
///
/// Formats:
/// - `"#my-id"` → ID selector
/// - `"Label"` → Type selector
/// - `"Button#submit"` → Combined Type#ID
pub fn parse_simple_selector(selector: &str) -> SimpleSelector {
    let selector = selector.trim();

    if selector.is_empty() {
        return SimpleSelector {
            type_name: None,
            id: None,
        };
    }

    // Check for ID-only selector: "#my-id"
    if selector.starts_with('#') {
        return SimpleSelector {
            type_name: None,
            id: Some(selector[1..].to_string()),
        };
    }

    // Check for combined Type#ID selector: "Button#submit"
    if let Some(hash_pos) = selector.find('#') {
        let type_name = &selector[..hash_pos];
        let id = &selector[hash_pos + 1..];
        return SimpleSelector {
            type_name: if type_name.is_empty() {
                None
            } else {
                Some(type_name.to_string())
            },
            id: if id.is_empty() {
                None
            } else {
                Some(id.to_string())
            },
        };
    }

    // Type-only selector: "Label"
    SimpleSelector {
        type_name: Some(selector.to_string()),
        id: None,
    }
}

/// Recursively find a widget matching a selector and apply a closure.
///
/// Performs depth-first search to find the first widget matching the selector.
/// Returns true if the widget was found and the closure was applied.
fn find_and_apply_by_selector<M, F>(
    widget: &mut dyn Widget<M>,
    selector: &SimpleSelector,
    f: &mut F,
) -> bool
where
    F: FnMut(&mut dyn Widget<M>),
{
    if selector.matches(widget) {
        f(widget);
        return true;
    }

    let child_count = widget.child_count();
    for i in 0..child_count {
        if let Some(child) = widget.get_child_mut(i) {
            if find_and_apply_by_selector(child, selector, f) {
                return true;
            }
        }
    }
    false
}

/// Recursively find a widget matching a selector, downcast it, and apply a typed closure.
///
/// Performs depth-first search to find the first widget matching the selector,
/// then attempts to downcast it to the concrete type W. If both the selector matches
/// and downcasting succeeds, the closure is called with the typed reference.
/// Returns true if the widget was found, matched, and the closure was applied.
fn find_and_apply_by_selector_typed<M, W, F>(
    widget: &mut dyn Widget<M>,
    selector: &SimpleSelector,
    f: &mut F,
) -> bool
where
    W: 'static,
    F: FnMut(&mut W),
{
    if selector.matches(widget) {
        // Try to downcast to the concrete type
        if let Some(any) = widget.as_any_mut() {
            if let Some(typed) = any.downcast_mut::<W>() {
                f(typed);
                return true;
            }
        }
        // Selector matched but downcast failed - continue searching
        // (user might have wrong type, or this is a different widget with same selector)
    }

    let child_count = widget.child_count();
    for i in 0..child_count {
        if let Some(child) = widget.get_child_mut(i) {
            if find_and_apply_by_selector_typed::<M, W, F>(child, selector, f) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test widget for focus tests
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

    // ========================================================================
    // Query API Tests
    // ========================================================================

    /// A test widget with ID and type name support for query tests
    struct QueryTestWidget {
        id: Option<String>,
        type_name: &'static str,
        children: Vec<Box<dyn Widget<()>>>,
        border_title: Option<String>,
    }

    impl QueryTestWidget {
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

    impl Widget<()> for QueryTestWidget {
        fn render(&self, _canvas: &mut crate::Canvas, _region: crate::Region) {}

        fn desired_size(&self) -> crate::Size {
            crate::Size { width: 1, height: 1 }
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

    // ------------------------------------------------------------------------
    // with_widget_by_id tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_with_widget_by_id_finds_root() {
        let root = QueryTestWidget::new("Container")
            .with_id("root")
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_id("root", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(found, Some("Container".to_string()));
    }

    #[test]
    fn test_with_widget_by_id_finds_child() {
        let root = QueryTestWidget::new("Container")
            .with_id("root")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("my-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_id("my-label", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(found, Some("Label".to_string()));
    }

    #[test]
    fn test_with_widget_by_id_finds_deeply_nested() {
        // Container > Container > Container > Label(id: "deep")
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Container")
                    .with_children(vec![
                        QueryTestWidget::new("Container")
                            .with_children(vec![
                                QueryTestWidget::new("Label").with_id("deep").boxed(),
                            ])
                            .boxed(),
                    ])
                    .boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_id("deep", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(found, Some("Label".to_string()));
    }

    #[test]
    fn test_with_widget_by_id_returns_none_for_missing() {
        let root = QueryTestWidget::new("Container")
            .with_id("root")
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_id("nonexistent", |_| ());

        assert!(found.is_none());
    }

    #[test]
    fn test_with_widget_by_id_can_modify_widget() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("my-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        // Modify the widget
        tree.with_widget_by_id("my-label", |widget| {
            widget.set_border_title("New Title");
        });

        // Verify the modification persisted
        let title = tree.with_widget_by_id("my-label", |widget| {
            widget.border_title().map(|s| s.to_string())
        });

        assert_eq!(title, Some(Some("New Title".to_string())));
    }

    #[test]
    fn test_with_widget_by_id_finds_first_match() {
        // Two widgets with different IDs at same level
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("first").boxed(),
                QueryTestWidget::new("Button").with_id("second").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let first = tree.with_widget_by_id("first", |w| w.type_name().to_string());
        let second = tree.with_widget_by_id("second", |w| w.type_name().to_string());

        assert_eq!(first, Some("Label".to_string()));
        assert_eq!(second, Some("Button".to_string()));
    }

    // ------------------------------------------------------------------------
    // with_widget_by_type tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_with_widget_by_type_finds_root() {
        let root = QueryTestWidget::new("Container")
            .with_id("root")
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_type("Container", |widget| {
            widget.id().map(|s| s.to_string())
        });

        assert_eq!(found, Some(Some("root".to_string())));
    }

    #[test]
    fn test_with_widget_by_type_finds_child() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("my-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_type("Label", |widget| {
            widget.id().map(|s| s.to_string())
        });

        assert_eq!(found, Some(Some("my-label".to_string())));
    }

    #[test]
    fn test_with_widget_by_type_finds_first_of_type() {
        // Multiple Labels - should find the first one
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("first-label").boxed(),
                QueryTestWidget::new("Label").with_id("second-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_type("Label", |widget| {
            widget.id().map(|s| s.to_string())
        });

        // Should find the first Label
        assert_eq!(found, Some(Some("first-label".to_string())));
    }

    #[test]
    fn test_with_widget_by_type_returns_none_for_missing() {
        let root = QueryTestWidget::new("Container").boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_type("NonexistentWidget", |_| ());

        assert!(found.is_none());
    }

    #[test]
    fn test_with_widget_by_type_can_modify_widget() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("my-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        // Modify the first Label
        tree.with_widget_by_type("Label", |widget| {
            widget.set_border_title("Type Query Title");
        });

        // Verify the modification persisted
        let title = tree.with_widget_by_type("Label", |widget| {
            widget.border_title().map(|s| s.to_string())
        });

        assert_eq!(title, Some(Some("Type Query Title".to_string())));
    }

    #[test]
    fn test_with_widget_by_type_finds_deeply_nested() {
        // Container > Container > Button
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Container")
                    .with_children(vec![
                        QueryTestWidget::new("Button").with_id("deep-button").boxed(),
                    ])
                    .boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_type("Button", |widget| {
            widget.id().map(|s| s.to_string())
        });

        assert_eq!(found, Some(Some("deep-button".to_string())));
    }

    #[test]
    fn test_with_widget_by_type_depth_first_order() {
        // Tree structure:
        // Container
        //   ├── Container
        //   │   └── Label(id: "nested-label")
        //   └── Label(id: "sibling-label")
        //
        // Depth-first should find "nested-label" first
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Container")
                    .with_children(vec![
                        QueryTestWidget::new("Label").with_id("nested-label").boxed(),
                    ])
                    .boxed(),
                QueryTestWidget::new("Label").with_id("sibling-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.with_widget_by_type("Label", |widget| {
            widget.id().map(|s| s.to_string())
        });

        // Depth-first: nested-label should be found first
        assert_eq!(found, Some(Some("nested-label".to_string())));
    }

    // ------------------------------------------------------------------------
    // Selector parsing tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_id_selector() {
        let selector = parse_simple_selector("#my-id");
        assert_eq!(selector.type_name, None);
        assert_eq!(selector.id, Some("my-id".to_string()));
    }

    #[test]
    fn test_parse_type_selector() {
        let selector = parse_simple_selector("Label");
        assert_eq!(selector.type_name, Some("Label".to_string()));
        assert_eq!(selector.id, None);
    }

    #[test]
    fn test_parse_combined_selector() {
        let selector = parse_simple_selector("Button#submit");
        assert_eq!(selector.type_name, Some("Button".to_string()));
        assert_eq!(selector.id, Some("submit".to_string()));
    }

    #[test]
    fn test_parse_empty_selector() {
        let selector = parse_simple_selector("");
        assert_eq!(selector.type_name, None);
        assert_eq!(selector.id, None);
    }

    #[test]
    fn test_parse_whitespace_selector() {
        let selector = parse_simple_selector("  Label  ");
        assert_eq!(selector.type_name, Some("Label".to_string()));
        assert_eq!(selector.id, None);
    }

    #[test]
    fn test_parse_id_with_hyphen() {
        let selector = parse_simple_selector("#my-complex-id");
        assert_eq!(selector.type_name, None);
        assert_eq!(selector.id, Some("my-complex-id".to_string()));
    }

    // ------------------------------------------------------------------------
    // query_one tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_query_one_by_id() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("my-label").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.query_one("#my-label", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(found, Some("Label".to_string()));
    }

    #[test]
    fn test_query_one_by_type() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Button").with_id("btn").boxed(),
                QueryTestWidget::new("Label").with_id("lbl").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.query_one("Button", |widget| {
            widget.id().map(|s| s.to_string())
        });

        assert_eq!(found, Some(Some("btn".to_string())));
    }

    #[test]
    fn test_query_one_combined_type_and_id() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Button").with_id("cancel").boxed(),
                QueryTestWidget::new("Button").with_id("submit").boxed(),
                QueryTestWidget::new("Label").with_id("submit").boxed(), // Same ID, different type
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        // Should find Button with id="submit", not Label with id="submit"
        let found = tree.query_one("Button#submit", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(found, Some("Button".to_string()));
    }

    #[test]
    fn test_query_one_combined_no_match() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Button").with_id("cancel").boxed(),
                QueryTestWidget::new("Label").with_id("submit").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        // No Button with id="submit" exists
        let found = tree.query_one("Button#submit", |_| ());

        assert!(found.is_none());
    }

    #[test]
    fn test_query_one_deeply_nested() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Container")
                    .with_children(vec![
                        QueryTestWidget::new("Container")
                            .with_children(vec![
                                QueryTestWidget::new("Input").with_id("deep-input").boxed(),
                            ])
                            .boxed(),
                    ])
                    .boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.query_one("#deep-input", |widget| {
            widget.type_name().to_string()
        });

        assert_eq!(found, Some("Input".to_string()));
    }

    #[test]
    fn test_query_one_can_modify() {
        let root = QueryTestWidget::new("Container")
            .with_children(vec![
                QueryTestWidget::new("Label").with_id("status").boxed(),
            ])
            .boxed();

        let mut tree = WidgetTree::new(root);

        // Modify using query_one
        tree.query_one("#status", |widget| {
            widget.set_border_title("Updated via query_one");
        });

        // Verify modification
        let title = tree.query_one("#status", |widget| {
            widget.border_title().map(|s| s.to_string())
        });

        assert_eq!(title, Some(Some("Updated via query_one".to_string())));
    }

    #[test]
    fn test_query_one_empty_selector_matches_nothing() {
        let root = QueryTestWidget::new("Container")
            .with_id("root")
            .boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.query_one("", |_| ());

        assert!(found.is_none());
    }

    #[test]
    fn test_query_one_nonexistent_id() {
        let root = QueryTestWidget::new("Container").boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.query_one("#nonexistent", |_| ());

        assert!(found.is_none());
    }

    #[test]
    fn test_query_one_nonexistent_type() {
        let root = QueryTestWidget::new("Container").boxed();

        let mut tree = WidgetTree::new(root);

        let found = tree.query_one("NonexistentWidget", |_| ());

        assert!(found.is_none());
    }
}
