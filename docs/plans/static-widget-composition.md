# Plan: Implement Static Widget + Refactor Label with Composition

## Overview

Port Python Textual's `Static` widget and refactor `Label` to use composition instead of duplicating code. Create a reusable delegation macro for the Widget trait.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Widget (trait)                               │
│  • render, desired_size, get_meta, set_style, get_style          │
│  • focus, hover, disabled, visible, dirty                        │
│  • on_event, on_mouse, handle_message                            │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ implements
┌─────────────────────────────────────────────────────────────────┐
│                     Static<M>                                    │
│  • content: VisualType                                           │
│  • visual_cache: Option<Visual>                                  │
│  • expand, shrink, markup flags                                  │
│  • update() method for dynamic content                           │
│  • Full Widget implementation                                    │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ contains (composition)
┌─────────────────────────────────────────────────────────────────┐
│                     Label<M>                                     │
│  • inner: Static<M>                                              │
│  • variant: Option<LabelVariant>                                 │
│  • Delegates to inner via macro                                  │
│  • Overrides: get_meta() for type_name                           │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Steps

### Step 1: Create Delegation Macro

**File**: `crates/textual/src/macros.rs` (new file)

Create a macro that generates Widget trait delegation. This macro will be reusable by client apps.

```rust
/// Generates Widget trait implementation that delegates to an inner widget.
///
/// # Usage
/// ```ignore
/// impl_widget_delegation!(Label<M> => inner);
/// impl_widget_delegation!(MyWidget<M> => base, type_name = "MyWidget");
/// ```
#[macro_export]
macro_rules! impl_widget_delegation {
    ($ty:ident<$m:ident> => $field:ident) => {
        impl_widget_delegation!($ty<$m> => $field, type_name = stringify!($ty));
    };

    ($ty:ident<$m:ident> => $field:ident, type_name = $name:expr) => {
        impl<$m> Widget<$m> for $ty<$m> {
            fn render(&self, canvas: &mut Canvas, region: Region) {
                self.$field.render(canvas, region)
            }

            fn desired_size(&self) -> Size {
                self.$field.desired_size()
            }

            fn get_meta(&self) -> WidgetMeta {
                let mut meta = self.$field.get_meta();
                meta.type_name = $name.to_string();
                meta
            }

            // ... all other Widget methods delegated
        }
    };
}
```

### Step 2: Create Static Widget

**File**: `crates/textual/src/widget/static_widget.rs` (new file)

```rust
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};
use crate::{Canvas, Content, KeyCode, MouseEvent, Region, Size, VisualType, Widget};
use crate::render_cache::RenderCache;
use crate::segment::Style;
use crate::strip::Strip;

/// A widget that displays static or updateable content.
///
/// Static is the base for text-displaying widgets. It handles:
/// - Content storage and caching
/// - Visual rendering with alignment
/// - The `update()` method for dynamic content
///
/// # Example
/// ```ignore
/// let status = Static::new("Ready");
/// // Later:
/// status.update("Processing...");
/// ```
#[derive(Debug, Clone)]
pub struct Static<M> {
    content: VisualType,
    visual_cache: Option<()>,  // TODO: Visual type when implemented
    expand: bool,
    shrink: bool,
    markup: bool,
    name: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    disabled: bool,
    style: ComputedStyle,
    dirty: bool,
    _phantom: std::marker::PhantomData<M>,
}

impl<M> Static<M> {
    pub fn new(content: impl Into<String>) -> Self { ... }

    /// Update the content and mark dirty.
    pub fn update(&mut self, content: impl Into<String>) {
        self.content = VisualType::from(content.into());
        self.visual_cache = None;
        self.dirty = true;
    }

    // Builder methods
    pub fn with_id(mut self, id: impl Into<String>) -> Self { ... }
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self { ... }
    pub fn with_expand(mut self, expand: bool) -> Self { ... }
    pub fn with_shrink(mut self, shrink: bool) -> Self { ... }
    pub fn with_markup(mut self, markup: bool) -> Self { ... }

    /// Access classes for Label variant support.
    pub fn add_class(&mut self, class: impl Into<String>) {
        self.classes.push(class.into());
    }
}

impl<M> Widget<M> for Static<M> {
    // Full implementation (moved from current Label)
    fn render(&self, canvas: &mut Canvas, region: Region) { ... }
    fn desired_size(&self) -> Size { ... }
    fn get_meta(&self) -> WidgetMeta { ... }
    // ... etc
}
```

### Step 3: Refactor Label to Use Composition

**File**: `crates/textual/src/widget/label.rs` (modify)

```rust
use crate::{impl_widget_delegation, Widget};
use crate::widget::static_widget::Static;

#[derive(Debug, Clone)]
pub enum LabelVariant {
    Default,
    Success,
    Error,
    Warning,
    Primary,
    Secondary,
    Accent,
}

impl LabelVariant {
    fn as_class(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Success => "success",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Accent => "accent",
        }
    }
}

/// A styled text label with optional semantic variants.
///
/// Label wraps Static and adds variant-based styling.
/// The variant adds a CSS class (e.g., "success", "error") that
/// can be styled in CSS.
#[derive(Debug, Clone)]
pub struct Label<M> {
    inner: Static<M>,
    variant: Option<LabelVariant>,
}

impl<M> Label<M> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            inner: Static::new(content),
            variant: None,
        }
    }

    /// Set the semantic variant (success, error, warning, etc.).
    pub fn with_variant(mut self, variant: LabelVariant) -> Self {
        let class = variant.as_class();
        if !class.is_empty() {
            self.inner.add_class(class);
        }
        self.variant = Some(variant);
        self
    }

    // Delegate builder methods to inner
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.with_id(id);
        self
    }

    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.inner = self.inner.with_classes(classes);
        self
    }

    /// Update the label content.
    pub fn update(&mut self, content: impl Into<String>) {
        self.inner.update(content);
    }

    /// Access the inner Static widget.
    pub fn as_static(&self) -> &Static<M> {
        &self.inner
    }

    pub fn as_static_mut(&mut self) -> &mut Static<M> {
        &mut self.inner
    }
}

// Use macro for Widget trait delegation
impl_widget_delegation!(Label<M> => inner, type_name = "Label");
```

### Step 4: Update Module Exports

**File**: `crates/textual/src/widget/mod.rs`

```rust
pub mod label;
pub mod static_widget;
// ... other modules

pub use static_widget::Static;
```

**File**: `crates/textual/src/lib.rs`

```rust
mod macros;  // Add macro module

pub use widget::static_widget::Static;  // Export Static
```

### Step 5: Update Existing Examples/Tests

Review and update any code that constructs `Label` to use the new generic signature if needed.

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/textual/src/macros.rs` | CREATE | Delegation macro (`impl_widget_delegation!`) |
| `crates/textual/src/widget/static_widget.rs` | CREATE | Static widget (content + rendering) |
| `crates/textual/src/widget/label.rs` | MODIFY | Refactor to wrap Static |
| `crates/textual/src/widget/mod.rs` | MODIFY | Add `static_widget` module |
| `crates/textual/src/lib.rs` | MODIFY | Add `macros` module, export `Static` |

---

## Delegation Macro - Full Method List

The macro needs to delegate these Widget trait methods:

**Required (no default impl)**:
- `render(&self, canvas, region)`
- `desired_size(&self) -> Size`

**With defaults but should delegate**:
- `get_meta(&self) -> WidgetMeta` ← Override type_name
- `get_state(&self) -> WidgetStates`
- `set_style(&mut self, style)`
- `get_style(&self) -> ComputedStyle`
- `default_css(&self) -> &'static str`
- `is_dirty(&self) -> bool`
- `mark_dirty(&mut self)`
- `mark_clean(&mut self)`
- `on_event(&mut self, key) -> Option<M>`
- `on_mouse(&mut self, event, region) -> Option<M>`
- `set_hover(&mut self, is_hovered) -> bool`
- `set_active(&mut self, is_active) -> bool`
- `clear_hover(&mut self)`
- `is_focusable(&self) -> bool`
- `is_visible(&self) -> bool`
- `set_visible(&mut self, visible)`
- `is_loading(&self) -> bool`
- `set_loading(&mut self, loading)`
- `is_disabled(&self) -> bool`
- `set_disabled(&mut self, disabled)`
- `count_focusable(&self) -> usize`
- `clear_focus(&mut self)`
- `focus_nth(&mut self, n) -> bool`
- `set_focus(&mut self, is_focused)`
- `is_focused(&self) -> bool`
- `child_count(&self) -> usize`
- `get_child_mut(&mut self, index) -> Option<...>`
- `handle_message(&mut self, envelope) -> Option<M>`
- `id(&self) -> Option<&str>`
- `type_name(&self) -> &'static str`
- `on_resize(&mut self, size)`
- `for_each_child(&mut self, f)`

---

## Implementation Order

1. **Create macros.rs** - The delegation macro (can test in isolation)
2. **Create static_widget.rs** - Move Label's logic here, make it generic
3. **Refactor label.rs** - Thin wrapper using composition + macro
4. **Update mod.rs and lib.rs** - Wire up exports
5. **Run tests** - Ensure existing Label functionality works
6. **Add Static-specific tests** - Test update(), expand/shrink

---

## Verification

```bash
# All existing tests should pass
cargo test

# Run grid_size_columns example (uses Label)
cargo run --example grid_size_columns
```

---

## Future Extensions

With this structure in place:
- **Button** can wrap Static and add click handling
- **Input** can wrap Static and add editing
- **ProgressBar** can wrap Static with specialized rendering
- **Any user widget** can use `impl_widget_delegation!` for composition
