//! Integration tests for the Widget Query API (Phase 1).
//!
//! These tests verify that `with_widget_by_id` and `with_widget_by_type` work
//! correctly with real widgets like Label and Container.

use textual::context::{AppContext, MountContext};
use textual::tree::WidgetTree;
use textual::{Container, Label, Widget};
use tokio::sync::mpsc;

/// Create a simple widget tree with nested containers and labels.
fn create_test_tree() -> Box<dyn Widget<()>> {
    // Structure:
    // Container (root)
    //   ├── Label (id: "header")
    //   ├── Container (id: "content")
    //   │   ├── Label (id: "item-1")
    //   │   └── Label (id: "item-2")
    //   └── Label (id: "footer")

    let header = Label::<()>::new("Header").with_id("header");
    let item1 = Label::<()>::new("Item 1").with_id("item-1");
    let item2 = Label::<()>::new("Item 2").with_id("item-2");
    let footer = Label::<()>::new("Footer").with_id("footer");

    let content = Container::<()>::new(vec![
        Box::new(item1),
        Box::new(item2),
    ]).with_id("content");

    let root = Container::<()>::new(vec![
        Box::new(header),
        Box::new(content),
        Box::new(footer),
    ]).with_id("root");

    Box::new(root)
}

#[test]
fn integration_query_by_id_finds_root_container() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    let found = tree.with_widget_by_id("root", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Container".to_string()));
}

#[test]
fn integration_query_by_id_finds_direct_child() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    let found = tree.with_widget_by_id("header", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));
}

#[test]
fn integration_query_by_id_finds_nested_container() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    let found = tree.with_widget_by_id("content", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Container".to_string()));
}

#[test]
fn integration_query_by_id_finds_deeply_nested_label() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // item-1 and item-2 are nested inside "content" container
    let found1 = tree.with_widget_by_id("item-1", |widget| {
        widget.type_name().to_string()
    });

    let found2 = tree.with_widget_by_id("item-2", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found1, Some("Label".to_string()));
    assert_eq!(found2, Some("Label".to_string()));
}

#[test]
fn integration_query_by_type_finds_first_container() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // The root is a Container, so it should be found first
    let found = tree.with_widget_by_type("Container", |widget| {
        widget.id().map(|s| s.to_string())
    });

    assert_eq!(found, Some(Some("root".to_string())));
}

#[test]
fn integration_query_by_type_finds_first_label() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // "header" is the first Label in depth-first order
    let found = tree.with_widget_by_type("Label", |widget| {
        widget.id().map(|s| s.to_string())
    });

    assert_eq!(found, Some(Some("header".to_string())));
}

#[test]
fn integration_query_can_modify_border_title() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Set border title on the header label
    tree.with_widget_by_id("header", |widget| {
        widget.set_border_title("Modified Header");
    });

    // Verify the modification
    let title = tree.with_widget_by_id("header", |widget| {
        widget.border_title().map(|s| s.to_string())
    });

    assert_eq!(title, Some(Some("Modified Header".to_string())));
}

#[test]
fn integration_query_can_modify_border_subtitle() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Set border subtitle on the content container
    tree.with_widget_by_id("content", |widget| {
        widget.set_border_subtitle("Content Subtitle");
    });

    // Verify the modification
    let subtitle = tree.with_widget_by_id("content", |widget| {
        widget.border_subtitle().map(|s| s.to_string())
    });

    assert_eq!(subtitle, Some(Some("Content Subtitle".to_string())));
}

#[test]
fn integration_query_returns_none_for_nonexistent_id() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    let found = tree.with_widget_by_id("does-not-exist", |_| ());

    assert!(found.is_none());
}

#[test]
fn integration_query_returns_none_for_nonexistent_type() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    let found = tree.with_widget_by_type("Button", |_| ());

    assert!(found.is_none());
}

#[test]
fn integration_mount_context_query_works() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let app_ctx: AppContext<()> = AppContext::new(tx);

    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);
    let mut ctx = MountContext::new(app_ctx, &mut tree);

    // Query through MountContext
    let found = ctx.with_widget_by_id("footer", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));
}

#[test]
fn integration_mount_context_can_modify_multiple_widgets() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let app_ctx: AppContext<()> = AppContext::new(tx);

    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);
    let mut ctx = MountContext::new(app_ctx, &mut tree);

    // Modify multiple widgets
    ctx.with_widget_by_id("header", |widget| {
        widget.set_border_title("Welcome");
    });

    ctx.with_widget_by_id("footer", |widget| {
        widget.set_border_subtitle("Goodbye");
    });

    // Verify both modifications
    let header_title = ctx.with_widget_by_id("header", |widget| {
        widget.border_title().map(|s| s.to_string())
    });

    let footer_subtitle = ctx.with_widget_by_id("footer", |widget| {
        widget.border_subtitle().map(|s| s.to_string())
    });

    assert_eq!(header_title, Some(Some("Welcome".to_string())));
    assert_eq!(footer_subtitle, Some(Some("Goodbye".to_string())));
}

#[test]
fn integration_query_depth_first_order() {
    // Verify depth-first traversal order
    // Tree:
    // Container (root)
    //   ├── Container (branch-a)
    //   │   └── Label (leaf-a)
    //   └── Label (sibling)

    let leaf_a = Label::<()>::new("Leaf A").with_id("leaf-a");
    let branch_a = Container::<()>::new(vec![
        Box::new(leaf_a),
    ]).with_id("branch-a");

    let sibling = Label::<()>::new("Sibling").with_id("sibling");

    let root = Container::<()>::new(vec![
        Box::new(branch_a),
        Box::new(sibling),
    ]).with_id("root");

    let mut tree = WidgetTree::new(Box::new(root) as Box<dyn Widget<()>>);

    // Depth-first: should find "leaf-a" before "sibling"
    let first_label = tree.with_widget_by_type("Label", |widget| {
        widget.id().map(|s| s.to_string())
    });

    assert_eq!(first_label, Some(Some("leaf-a".to_string())));
}

// =============================================================================
// Phase 2: query_one with selector syntax
// =============================================================================

#[test]
fn integration_query_one_id_selector() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query using #id selector
    let found = tree.query_one("#header", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));
}

#[test]
fn integration_query_one_type_selector() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query using type selector - should find root Container
    let found = tree.query_one("Container", |widget| {
        widget.id().map(|s| s.to_string())
    });

    assert_eq!(found, Some(Some("root".to_string())));
}

#[test]
fn integration_query_one_combined_selector() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query using Type#id selector
    let found = tree.query_one("Label#footer", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));
}

#[test]
fn integration_query_one_combined_selector_nested() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query for nested content container
    let found = tree.query_one("Container#content", |widget| {
        widget.id().map(|s| s.to_string())
    });

    assert_eq!(found, Some(Some("content".to_string())));
}

#[test]
fn integration_query_one_combined_type_mismatch() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query for Button#header - but header is a Label, not Button
    let found = tree.query_one("Button#header", |_| ());

    assert!(found.is_none());
}

#[test]
fn integration_query_one_deeply_nested() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query for deeply nested item
    let found = tree.query_one("#item-1", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));
}

#[test]
fn integration_query_one_modify_via_selector() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Modify using selector
    tree.query_one("#header", |widget| {
        widget.set_border_title("Updated Header");
    });

    // Verify
    let title = tree.query_one("#header", |widget| {
        widget.border_title().map(|s| s.to_string())
    });

    assert_eq!(title, Some(Some("Updated Header".to_string())));
}

#[test]
fn integration_mount_context_query_one() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let app_ctx: AppContext<()> = AppContext::new(tx);

    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);
    let mut ctx = MountContext::new(app_ctx, &mut tree);

    // Query using MountContext.query_one
    let found = ctx.query_one("#footer", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));
}

#[test]
fn integration_mount_context_query_one_modify() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let app_ctx: AppContext<()> = AppContext::new(tx);

    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);
    let mut ctx = MountContext::new(app_ctx, &mut tree);

    // Modify via MountContext.query_one
    ctx.query_one("Container#content", |widget| {
        widget.set_border_title("Content Area");
        widget.set_border_subtitle("Items below");
    });

    // Verify modifications
    let title = ctx.query_one("#content", |widget| {
        widget.border_title().map(|s| s.to_string())
    });
    let subtitle = ctx.query_one("#content", |widget| {
        widget.border_subtitle().map(|s| s.to_string())
    });

    assert_eq!(title, Some(Some("Content Area".to_string())));
    assert_eq!(subtitle, Some(Some("Items below".to_string())));
}

#[test]
fn integration_query_one_with_real_widget_types() {
    // Create a tree with multiple widget types having same ID pattern
    let label1 = Label::<()>::new("Label Submit").with_id("submit");
    let container = Container::<()>::new(vec![
        Box::new(label1),
    ]).with_id("form");

    let mut tree = WidgetTree::new(Box::new(container) as Box<dyn Widget<()>>);

    // Query for Label#submit specifically
    let found = tree.query_one("Label#submit", |widget| {
        widget.type_name().to_string()
    });

    assert_eq!(found, Some("Label".to_string()));

    // Query for Container#submit should fail (no container with that ID)
    let not_found = tree.query_one("Container#submit", |_| ());
    assert!(not_found.is_none());
}

// =============================================================================
// Phase 2.5: query_one_as with typed downcast
// =============================================================================

#[test]
fn integration_query_one_as_finds_label_typed() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query using typed access - should get &mut Label<()>, not &mut dyn Widget
    let found = tree.query_one_as::<Label<()>, _, _>("#header", |label| {
        // Can call Label-specific methods
        label.update("Updated Header");
        label.variant()
    });

    // variant() returns Option<LabelVariant>
    assert!(found.is_some());
}

#[test]
fn integration_query_one_as_finds_container_typed() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query for Container by ID with typed access
    let found = tree.query_one_as::<Container<()>, _, _>("#content", |container| {
        // Can call Container-specific methods
        container.set_border_title("Content Section");
        container.border_title().map(|s| s.to_string())
    });

    assert_eq!(found, Some(Some("Content Section".to_string())));
}

#[test]
fn integration_query_one_as_type_mismatch_returns_none() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query for #header as Container - but header is a Label!
    // This should return None because the downcast fails
    let found = tree.query_one_as::<Container<()>, _, _>("#header", |_| ());

    assert!(found.is_none());
}

#[test]
fn integration_query_one_as_combined_selector_typed() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Use Type#ID selector with typed access
    let found = tree.query_one_as::<Label<()>, _, _>("Label#footer", |label| {
        label.update("New Footer");
        true
    });

    assert_eq!(found, Some(true));
}

#[test]
fn integration_query_one_as_deeply_nested() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query deeply nested item-1 with typed access
    let found = tree.query_one_as::<Label<()>, _, _>("#item-1", |label| {
        label.update("Item 1 Updated");
        "success"
    });

    assert_eq!(found, Some("success"));
}

#[test]
fn integration_mount_context_query_one_as() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let app_ctx: AppContext<()> = AppContext::new(tx);

    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);
    let mut ctx = MountContext::new(app_ctx, &mut tree);

    // Use typed query through MountContext
    let found = ctx.query_one_as::<Label<()>, _, _>("#footer", |label| {
        label.update("Modified via MountContext");
        label.variant()
    });

    assert!(found.is_some());
}

#[test]
fn integration_mount_context_query_one_as_modify_and_verify() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let app_ctx: AppContext<()> = AppContext::new(tx);

    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);
    let mut ctx = MountContext::new(app_ctx, &mut tree);

    // Modify Label via typed query
    ctx.query_one_as::<Label<()>, _, _>("#header", |label| {
        label.set_border_title("Typed Title");
    });

    // Verify modification with untyped query
    let title = ctx.query_one("#header", |widget| {
        widget.border_title().map(|s| s.to_string())
    });

    assert_eq!(title, Some(Some("Typed Title".to_string())));
}

#[test]
fn integration_query_one_as_type_selector_only() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query first Container with typed access
    let found = tree.query_one_as::<Container<()>, _, _>("Container", |container| {
        container.id().map(|s| s.to_string())
    });

    // Should find root container
    assert_eq!(found, Some(Some("root".to_string())));
}

#[test]
fn integration_query_one_as_first_label() {
    let root = create_test_tree();
    let mut tree = WidgetTree::new(root);

    // Query first Label - should be "header" in depth-first order
    let found = tree.query_one_as::<Label<()>, _, _>("Label", |label| {
        // Access the inner Static widget
        label.as_static().id().map(|s| s.to_string())
    });

    assert_eq!(found, Some(Some("header".to_string())));
}
