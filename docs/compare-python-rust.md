Python vs Rust Comparison Findings

1. Canvas & Geometry (canvas.rs vs canvas.py + geometry.py)

| Aspect           | Python (Textual)                                                                                             | Rust (texrs)                          | Gap                                                                                          |
| ---------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------- | -------------------------------------------------------------------------------------------- |
| Purpose          | Keyline rendering (primitives like lines, rectangles)                                                        | Direct cell-based rendering           | Different goals - Python Canvas is for drawing primitives; texrs Canvas is the render buffer |
| Coordinate Types | int (signed by default)                                                                                      | u16 (unsigned)                        | Critical gap - need i32 for scrolling                                                        |
| Region           | Full geometry.py with 50+ methods: intersection, contains_point, union, split, grow, shrink, translate, etc. | Bare struct with 4 fields, no methods | Major gap - need at least intersection, contains_point                                       |
| Size             | NamedTuple with area, contains, arithmetic ops                                                               | Simple struct with width/height       | Minor gap                                                                                    |
| Offset           | NamedTuple with blend, clamp, arithmetic                                                                     | None in texrs                         | Could add later                                                                              |
| Clipping         | Via x_range/y_range helpers                                                                                  | None                                  | Critical gap - Stage 9 adds this                                                             |
| Spacing          | Full Spacing type for margins/padding                                                                        | None                                  | Future need                                                                                  |

Key Python Region methods we need:

# Python Region has these critical methods:

Region.intersection(self, region) # Get overlapping area
Region.contains_point(self, point) # Hit testing
Region.contains(self, x, y) # Alt hit testing
Region.from_corners(x1, y1, x2, y2)# Constructor
Region.grow(margin) # Expand by spacing
Region.shrink(margin) # Contract by spacing

---

2. Containers (vertical.rs vs containers.py + layouts/vertical.py)

| Aspect          | Python (Textual)                                                                                                                                       | Rust (texrs)                                | Gap                      |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------- | ------------------------ |
| Architecture    | Separated: Container (Widget) + Layout (positioning logic)                                                                                             | Combined: Container does both               | Architectural difference |
| Container types | Container, ScrollableContainer, Vertical, VerticalGroup, VerticalScroll, Horizontal, HorizontalGroup, HorizontalScroll, Center, Middle, Grid, ItemGrid | Vertical, Horizontal, Center, Middle        | Many containers missing  |
| Scrolling       | Built into ScrollableContainer with overflow: auto auto CSS                                                                                            | None                                        | Critical gap - Stage 10  |
| Layout logic    | Separate VerticalLayout class with Fraction-based sizing, margin collapsing                                                                            | Inline in render() with simple accumulation | Simpler in texrs         |
| Box model       | Full margin resolution with resolve_box_models()                                                                                                       | None - widgets just stack                   | Future need              |
| CSS             | Each container has DEFAULT_CSS with width: 1fr; height: 1fr; layout: vertical; overflow: hidden hidden                                                 | None                                        | Need CSS defaults        |

Python scrolling containers:
class ScrollableContainer(Widget, can_focus=True):
DEFAULT_CSS = """
ScrollableContainer {
layout: vertical;
overflow: auto auto; # <-- This triggers scrollbars
}
"""
BINDINGS = [
Binding("up", "scroll_up", ...),
Binding("down", "scroll_down", ...),

# ... pageup, pagedown, home, end

]

---

3. Switch Widget (switch.rs vs widgets/\_switch.py)

| Aspect    | Python (Textual)                                       | Rust (texrs)                         | Gap                          |
| --------- | ------------------------------------------------------ | ------------------------------------ | ---------------------------- |
| Rendering | Uses ScrollBarRender for smooth slider animation!      | Text-based [ ON ]/[OFF ]             | Python is more sophisticated |
| Animation | \_slider_position reactive with animate() method       | None                                 | Could add                    |
| Size      | get_content_width() returns 4, height 1                | desired_size() returns 10×3          | Different sizes              |
| Styling   | Component classes (switch--slider), CSS pseudo-classes | ComputedStyle struct                 | Similar approach             |
| State     | reactive[bool] with watchers                           | Plain bool with manual dirty marking | texrs is simpler             |
| Events    | \_on_click async handler                               | on_event/on_mouse sync               | Both work                    |
| Loading   | Not built-in                                           | Built-in with spinner                | texrs has more!              |
| Disabled  | CSS class based                                        | Built-in state                       | Both have                    |

Interesting Python pattern:
def render(self) -> RenderResult:
style = self.get_component_rich_style("switch--slider")
return ScrollBarRender( # Reuses scrollbar renderer!
virtual_size=100,
window_size=50,
position=self.\_slider_position \* 50,
style=style,
vertical=False,
)

---

4. Scrollbar (none vs scrollbar.py)

| Aspect            | Python (Textual)                                                      | Rust (texrs) | Gap                 |
| ----------------- | --------------------------------------------------------------------- | ------------ | ------------------- |
| Existence         | Full ScrollBar widget + ScrollBarRender                               | None         | Major gap           |
| Glyphs            | VERTICAL_BARS = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', ' ']              | None         | Stage 10 adds these |
| Colors            | CSS: scrollbar-color, scrollbar-background, hover/active variants     | None         | Stage 10 adds       |
| Thumb calculation | bar_ratio = virtual_size / size; thumb_size = window_size / bar_ratio | None         | Stage 10 adds       |
| Corner widget     | ScrollBarCorner for when both scrollbars present                      | None         | Nice to have        |

Python scrollbar math (critical for Stage 10):
bar_ratio = virtual_size / size
thumb_size = max(1, window_size / bar_ratio)
position_ratio = position / (virtual_size - window_size)
position = (size - thumb_size) \* position_ratio

---

5. Summary: Critical Gaps for Stage 9 & 10

| Priority | Gap                          | Stage | Notes                                   |
| -------- | ---------------------------- | ----- | --------------------------------------- |
| P0       | Region i32 coordinates       | 9     | Python uses signed int natively         |
| P0       | Region::intersection()       | 9     | Python has this, needed for clipping    |
| P0       | Region::contains_point()     | 9     | Python has this, needed for hit testing |
| P0       | Canvas clipping stack        | 9     | Python uses x_range/y_range             |
| P1       | Scrollbar widget             | 10    | Full ScrollBarRender class              |
| P1       | Scrollbar glyphs             | 10    | VERTICAL_BARS/HORIZONTAL_BARS           |
| P1       | Scrollbar CSS properties     | 10    | 7 CSS properties                        |
| P1       | Container scroll state       | 10    | scroll_y/scroll_x                       |
| P2       | ScrollableContainer          | 10    | Separate container type                 |
| P2       | Keyboard bindings for scroll | 10    | up/down/pageup/pagedown                 |

---

Key Architectural Insight

Python Textual separates concerns:

1. geometry.py - All coordinate/region math (1300+ lines)
2. containers.py - Simple widget wrappers with CSS defaults
3. layouts/vertical.py - Layout algorithm with margin resolution
4. scrollbar.py - Scrollbar widget + render class
5. widget.py - Base with scroll support built-in (44k tokens!)

texrs combines them:

- canvas.rs - Region + Size + Canvas (134 lines)
- containers/vertical.rs - Widget + Layout combined (158 lines)

This is fine for now, but Stage 9's plan correctly identifies that we need to add Region methods before scrolling can work.
