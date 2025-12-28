# Plan: Full Textual Rendering Parity

## Goal
Implement Python Textual's complete rendering pipeline in texrs:
```
Content → Strip[] → Segment[] → Canvas
```

Plus full border rendering with BORDER_CHARS, BORDER_LOCATIONS, caching, and label support.

## Target CSS (grid_columns example)
```css
Grid {
    grid-size: 5 2;
    grid-columns: 1fr 16 2fr;
}

Label {
    border: round white;
    content-align-horizontal: center;
    width: 100%;
    height: 100%;
}
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Rendering Pipeline                          │
│                                                                 │
│  Text → Content → Strip[] → Segment[] → Canvas                  │
│           │          │          │                               │
│           │          │          └─ (text, Style)                │
│           │          └─ line of Segments + operations           │
│           └─ parses text, produces Strips                       │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     Border Rendering                            │
│                                                                 │
│  BORDER_CHARS["round"] → get_box() → render_row() → Strip       │
│        ↓                    ↓            ↓                      │
│  3×3 char grid      BoxSegments    horizontal line              │
│  (╭─╮/│ │/╰─╯)       (cached)      with labels                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Core Rendering Types

### 1.1 Segment Type
**File:** `crates/textual/src/segment.rs` (NEW)

### 1.2 Strip Type
**File:** `crates/textual/src/strip.rs` (NEW)

### 1.3 Content Type
**File:** `crates/textual/src/content.rs` (NEW)

---

## Phase 2: Border Infrastructure

### 2.1 Border Character Definitions
**File:** `crates/textual/src/border_chars.rs` (NEW)

### 2.2 get_box with Caching
**File:** `crates/textual/src/border_box.rs` (NEW)

### 2.3 render_row Function
**File:** `crates/textual/src/border_render.rs` (NEW)

---

## Phase 3: Render Cache

### 3.1 Line Rendering Orchestration
**File:** `crates/textual/src/render_cache.rs` (NEW)

---

## Phase 4: Canvas Integration

### 4.1 Canvas Updates
**File:** `crates/textual/src/canvas.rs` (MODIFY)

---

## Phase 5: CSS Parsing

### 5.1 content-align-horizontal
**Files:**
- `crates/tcss/src/parser/stylesheet.rs` - Add Declaration::ContentAlignHorizontal
- `crates/tcss/src/parser/values.rs` - Add parse_align_horizontal()
- `crates/tcss/src/parser/mod.rs` - Add parsing case
- `crates/tcss/src/parser/cascade.rs` - Add cascade application

---

## Phase 6: Widget Updates

### 6.1 Label Widget Refactor
**File:** `crates/textual/src/widget/label.rs` (MODIFY)

---

## Implementation Order

| Order | Phase | Files | Dependencies |
|-------|-------|-------|--------------|
| 1 | 1.1 Segment | `segment.rs` | None |
| 2 | 1.2 Strip | `strip.rs` | Segment |
| 3 | 4.1 Canvas | `canvas.rs` | Strip |
| 4 | 1.3 Content | `content.rs` | Strip |
| 5 | 2.1 BorderChars | `border_chars.rs` | None |
| 6 | 2.2 get_box | `border_box.rs` | Segment, BorderChars |
| 7 | 2.3 render_row | `border_render.rs` | Strip, get_box |
| 8 | 3.1 RenderCache | `render_cache.rs` | All above |
| 9 | 5.1 CSS parsing | tcss parser files | None (parallel) |
| 10 | 6.1 Label | `label.rs` | Content, RenderCache |

---

## New Dependencies

```toml
# Cargo.toml
[dependencies]
phf = { version = "0.11", features = ["macros"] }  # Static hash maps
once_cell = "1.19"                                  # Lazy static cache
unicode-width = "0.2"                               # Cell width calculation
```
