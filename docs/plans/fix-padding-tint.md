# Plan: Fix Padding and Tint CSS Properties

## Problem Summary

Two CSS properties are not working in the grid example:

1. **`padding: 1 2;`** - Parsed and stored but **never applied during rendering**
2. **`tint: magenta 40%;`** - **Not implemented at all**

---

## Part 1: Fix Padding Implementation

### Current State

| Stage     | Status                                 |
| --------- | -------------------------------------- |
| Parsing   | ✅ `crates/tcss/src/parser/mod.rs:151` |
| Storage   | ✅ `ComputedStyle.padding: Spacing`    |
| Cascade   | ✅ Applied in `cascade.rs:248`         |
| Rendering | ❌ **Not used**                        |

### Root Cause

`RenderCache::inner_size()` only subtracts border thickness, ignoring padding:

```rust
// Current code - NO PADDING!
pub fn inner_size(&self, width: usize, height: usize) -> (usize, usize) {
    if self.has_border && width >= 2 && height >= 2 {
        (width - 2, height - 2)  // Only borders
    } else {
        (width, height)  // No padding at all
    }
}
```

### Files to Modify

| File                                         | Changes                                                           |
| -------------------------------------------- | ----------------------------------------------------------------- |
| `crates/textual/src/render_cache.rs`         | Add padding fields, update `inner_size()`, update `render_line()` |
| `crates/textual/src/border_render.rs`        | Update `render_middle_row()` to handle padding offset             |
| `crates/textual/src/widget/static_widget.rs` | Adjust content line indexing for padding                          |

### Implementation Steps

#### Step 1: Add padding fields to RenderCache

In `crates/textual/src/render_cache.rs`:

```rust
pub struct RenderCache {
    border_box: Option<[BoxSegments; 3]>,
    has_border: bool,
    style: ComputedStyle,
    // NEW: Store padding as cells
    padding_top: usize,
    padding_right: usize,
    padding_bottom: usize,
    padding_left: usize,
}
```

#### Step 2: Extract padding in `RenderCache::new()`

```rust
pub fn new(style: &ComputedStyle) -> Self {
    // ... existing border logic ...

    // Extract padding (convert Scalar to cells)
    let padding_top = style.padding.top.value as usize;
    let padding_right = style.padding.right.value as usize;
    let padding_bottom = style.padding.bottom.value as usize;
    let padding_left = style.padding.left.value as usize;

    Self {
        border_box,
        has_border,
        style: style.clone(),
        padding_top,
        padding_right,
        padding_bottom,
        padding_left,
    }
}
```

#### Step 3: Update `inner_size()` to account for padding

```rust
pub fn inner_size(&self, width: usize, height: usize) -> (usize, usize) {
    // First, account for borders
    let (w, h) = if self.has_border && width >= 2 && height >= 2 {
        (width - 2, height - 2)
    } else if self.has_border {
        (0, 0)
    } else {
        (width, height)
    };

    // Then account for padding
    let padded_w = w.saturating_sub(self.padding_left + self.padding_right);
    let padded_h = h.saturating_sub(self.padding_top + self.padding_bottom);
    (padded_w, padded_h)
}
```

#### Step 4: Add padding getters

```rust
pub fn padding_left(&self) -> usize { self.padding_left }
pub fn padding_top(&self) -> usize { self.padding_top }
pub fn padding_right(&self) -> usize { self.padding_right }
pub fn padding_bottom(&self) -> usize { self.padding_bottom }
```

#### Step 5: Update `render_middle_row()` in border_render.rs

Pass padding to `render_middle_row()` and prepend/append blank padding cells:

```rust
pub fn render_middle_row(
    box_segs: &BoxSegments,
    content: Option<&Strip>,
    width: usize,
    pad_style: Option<Style>,
    padding_left: usize,   // NEW
    padding_right: usize,  // NEW
) -> Strip {
    // Build: left_border + padding_left + content + padding_right + right_border
}
```

#### Step 6: Update Static widget render for vertical padding

In `static_widget.rs`, adjust content line indexing:

```rust
for y in 0..height {
    let border_offset = if cache.has_border() { 1 } else { 0 };
    let padding_offset = cache.padding_top();

    // Content starts after border + padding_top
    // And ends before padding_bottom + border
    let content_start = border_offset + padding_offset;
    let content_end = height.saturating_sub(border_offset + cache.padding_bottom());

    if y >= content_start && y < content_end {
        let content_y = y - content_start;
        let content_line = aligned_lines.get(content_y);
        // render with content
    } else if y < border_offset || y >= height - border_offset {
        // render border row
    } else {
        // render padding row (blank with background)
    }
}
```

---

## Part 2: Implement Tint Property

### Current State

| Stage     | Status             |
| --------- | ------------------ |
| Parsing   | ❌ Not implemented |
| Storage   | ❌ Not implemented |
| Cascade   | ❌ Not implemented |
| Rendering | ❌ Not implemented |

### What is Tint?

Tint applies a color overlay to a widget. `tint: magenta 40%;` blends 40% magenta over the widget's colors using alpha composition.

### Files to Modify

| File                                         | Changes                                    |
| -------------------------------------------- | ------------------------------------------ |
| `crates/tcss/src/parser/stylesheet.rs`       | Add `Tint(RgbaColor)` declaration          |
| `crates/tcss/src/parser/mod.rs`              | Add tint parsing                           |
| `crates/tcss/src/types/mod.rs`               | Add `tint` field to `ComputedStyle`        |
| `crates/tcss/src/parser/cascade.rs`          | Apply tint in cascade                      |
| `crates/tcss/src/types/color.rs`             | Add `blend()` method for alpha composition |
| `crates/textual/src/segment.rs`              | Add `tint` to rendering `Style`            |
| `crates/textual/src/widget/static_widget.rs` | Apply tint to colors during render         |

### Implementation Steps

#### Step 1: Add Declaration variant

In `crates/tcss/src/parser/stylesheet.rs`:

```rust
pub enum Declaration {
    // ... existing variants ...
    Tint(RgbaColor),  // NEW
}
```

#### Step 2: Add parser entry

In `crates/tcss/src/parser/mod.rs`, add to `parse_single_declaration`:

```rust
"tint" => map(values::parse_color, Declaration::Tint)(input)?,
```

#### Step 3: Add field to ComputedStyle

In `crates/tcss/src/types/mod.rs`:

```rust
pub struct ComputedStyle {
    // ... existing fields ...
    pub tint: Option<RgbaColor>,  // NEW
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            tint: None,
        }
    }
}
```

#### Step 4: Apply in cascade

In `crates/tcss/src/parser/cascade.rs`, add to `apply_declaration`:

```rust
Declaration::Tint(c) => {
    style.tint = Some(resolve_theme_color(c, theme));
}
```

#### Step 5: Add color blending method

In `crates/tcss/src/types/color.rs`:

```rust
impl RgbaColor {
    /// Blends another color on top of this one using alpha composition.
    pub fn blend(&self, overlay: &RgbaColor) -> RgbaColor {
        let base_r = self.r as f32 / 255.0;
        let base_g = self.g as f32 / 255.0;
        let base_b = self.b as f32 / 255.0;

        let over_r = overlay.r as f32 / 255.0;
        let over_g = overlay.g as f32 / 255.0;
        let over_b = overlay.b as f32 / 255.0;
        let over_a = overlay.a;

        // Alpha composition: result = overlay * alpha + base * (1 - alpha)
        let out_r = (over_r * over_a + base_r * (1.0 - over_a)).min(1.0);
        let out_g = (over_g * over_a + base_g * (1.0 - over_a)).min(1.0);
        let out_b = (over_b * over_a + base_b * (1.0 - over_a)).min(1.0);

        Self::rgba(
            (out_r * 255.0).round() as u8,
            (out_g * 255.0).round() as u8,
            (out_b * 255.0).round() as u8,
            self.a,
        )
    }
}
```

#### Step 6: Apply tint in Static widget rendering

In `static_widget.rs`, modify `rendering_style()` to apply tint:

```rust
fn rendering_style(&self) -> Style {
    let mut style = Style {
        fg: self.style.color.clone(),
        bg: self.style.background.clone(),
        // ... other fields ...
    };

    // Apply tint overlay
    if let Some(tint) = &self.style.tint {
        if let Some(bg) = &style.bg {
            style.bg = Some(bg.blend(tint));
        }
        if let Some(fg) = &style.fg {
            style.fg = Some(fg.blend(tint));
        }
    }

    style
}
```

---

## Testing Plan

### Padding Tests

```rust
#[test]
fn test_inner_size_with_padding() {
    let mut style = ComputedStyle::default();
    style.padding = Spacing::all(Scalar::cells(2.0));
    let cache = RenderCache::new(&style);
    // 20x10 widget with 2-cell padding = 16x6 content
    assert_eq!(cache.inner_size(20, 10), (16, 6));
}

#[test]
fn test_inner_size_with_border_and_padding() {
    let mut style = ComputedStyle::default();
    style.border = Border::all(BorderEdge { kind: BorderKind::Solid, color: None });
    style.padding = Spacing::all(Scalar::cells(1.0));
    let cache = RenderCache::new(&style);
    // 20x10 - 2 border - 2 padding = 16x6
    assert_eq!(cache.inner_size(20, 10), (16, 6));
}
```

### Tint Tests

```rust
#[test]
fn test_color_blend() {
    let base = RgbaColor::rgb(100, 100, 100);  // Gray
    let overlay = RgbaColor::rgba(255, 0, 0, 0.5);  // 50% red
    let result = base.blend(&overlay);
    assert_eq!(result.r, 178);  // (255*0.5 + 100*0.5)
    assert_eq!(result.g, 50);   // (0*0.5 + 100*0.5)
    assert_eq!(result.b, 50);
}
```

### Manual Testing

```bash
cargo run --example grid
# Verify:
# - Static widgets have padding (text offset from edges)
# - #static1 has magenta tint overlay
```

---

## Implementation Order

1. **Padding first** (simpler, already half-implemented)
   - Add padding fields to RenderCache
   - Update inner_size()
   - Update render_middle_row()
   - Update Static widget render loop
   - Add tests

2. **Tint second** (more components)
   - Add Declaration::Tint
   - Add parser entry
   - Add ComputedStyle.tint
   - Add cascade handling
   - Add RgbaColor::blend()
   - Apply in Static rendering
   - Add tests

3. **Final verification**
   - Run grid example
   - Compare with Python Textual screenshot

---

## Verification Checklist

- [ ] Padding: `cargo test` passes
- [ ] Padding: Static widgets show correct content offset
- [ ] Padding: Border + padding combination works
- [ ] Tint: `cargo test` passes
- [ ] Tint: `tint: magenta 40%` parses correctly
- [ ] Tint: Colors visually blend in grid example
- [ ] Grid example matches Python Textual output
