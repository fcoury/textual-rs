# textual-rs

A Rust port of Python's [Textual](https://github.com/Textualize/textual) — a modern framework for building rich terminal user interfaces.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Status](https://img.shields.io/badge/status-work%20in%20progress-yellow)

> **Note:** This project is a work in progress. APIs may change, and some features are not yet implemented. Contributions and feedback are welcome!

## Overview

textual-rs brings the power of Textual's design philosophy to Rust, combining a reactive widget system, CSS-like styling (TCSS), and async-first architecture. Build beautiful, responsive TUIs with familiar patterns.

```rust
use textual::{App, Compose, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AlignApp {
    quit: bool,
}

impl AlignApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for AlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Vertical alignment with [b]Textual[/]", classes: "box")
            Label("Take note, browsers.", classes: "box")
        }
    }
}

impl App for AlignApp {
    const CSS: &'static str = include_str!("align.tcss");

    fn on_key(&mut self, key: textual::KeyCode) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> textual::Result<()> {
    let mut app = AlignApp::new();
    app.run()
}
```

## Features

- **Declarative UI** — Build interfaces with a JSX-like `ui!` macro
- **TCSS Styling** — CSS-inspired styling with terminal-specific extensions
- **Rich Markup** — Inline text styling with `[bold red]text[/]` syntax
- **Flexible Layouts** — Vertical, horizontal, and CSS Grid-like layouts
- **Async-First** — Built on Tokio with timers, intervals, and background tasks
- **Responsive Design** — Breakpoint-based styling that adapts to terminal size
- **Message System** — Type-safe event handling with message bubbling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
textual = { git = "https://github.com/fcoury/textual-rs" }
tokio = { version = "1.0", features = ["full"] }
```

## Widgets

### Label

Text display widget with semantic variants and rich markup support.

```rust
// Plain text
Label("Hello, world!")

// With ID and classes
Label("My border is solid red", id: "label1")
Label("Vertical alignment with [b]Textual[/]", classes: "box")

// With rich markup
Label("[bold]Bold[/], [italic red]italic red[/], [underline]underlined[/]")

// With semantic variant (using builder pattern)
Label::new("Operation complete").with_variant(LabelVariant::Success)
```

**Variants:** `Default`, `Primary`, `Secondary`, `Success`, `Error`, `Warning`, `Accent`

### Static

Base widget for displaying updateable content. Supports rich markup and dynamic updates.

```rust
Static("Initial content", id: "status")

// Update later via message handling
widget.update("[green]Updated![/]");
```

### Switch

Interactive toggle with keyboard and mouse support.

```rust
Switch(true, Message::DarkModeToggled, id: "dark-mode")
Switch(false, Message::NotificationsToggled, id: "notifications")
```

### Placeholder

Debug widget that displays its size and ID — useful during layout development.

```rust
Placeholder(id: "debug")
Placeholder(id: "p1", label: "min-height: 25%")
```

### Container

Base wrapper with border, background, and padding support.

```rust
Container(id: "my-container") {
    Label("Wrapped content")
}
```

## Layouts

### Vertical

Stack widgets top-to-bottom.

```rust
Vertical {
    Label("First")
    Label("Second")
    Label("Third")
}
```

### Horizontal

Stack widgets left-to-right.

```rust
Horizontal {
    Label("Left")
    Label("Center")
    Label("Right")
}
```

### Grid

CSS Grid-inspired layout with rows and columns.

```rust
Grid {
    Static("Cell 1", id: "cell1")
    Static("Cell 2", id: "cell2")
    Static("Cell 3", id: "cell3")
}
```

Configure grid layout via TCSS:

```css
Grid {
    grid-columns: 1fr 2fr 1fr;
    grid-rows: auto 1fr auto;
}
```

### Flexible Sizing

Widgets support multiple sizing units:

| Unit     | Example         | Description                 |
| -------- | --------------- | --------------------------- |
| Cells    | `10`            | Fixed terminal cells        |
| Percent  | `50%`           | Percentage of parent        |
| Viewport | `100vw`, `50vh` | Viewport width/height       |
| Fraction | `1fr`, `2fr`    | Flexible space distribution |
| Auto     | `auto`          | Size to content             |

```css
/* TCSS example */
#sidebar {
  width: 30%;
  height: 100vh;
}

#content {
  width: 1fr; /* Take remaining space */
}
```

## Styling (TCSS)

textual-rs uses TCSS (Textual CSS), a CSS dialect designed for terminal interfaces.

### Selectors

```css
/* Type selector */
Label {
  color: white;
}

/* Class selector */
.primary {
  background: blue;
}

/* ID selector */
#header {
  height: 3;
}

/* Descendant */
Container Label {
  margin: 1;
}

/* Child */
Vertical > Label {
  padding: 0 2;
}

/* Pseudo-classes */
Switch:focus {
  border: heavy green;
}

Button:hover {
  background: $primary;
}
```

### Properties

**Colors & Background**

```css
Label {
  color: red;
  background: #1a1a2e;
  background-tint: rgba(255, 0, 0, 0.3);
}
```

**Spacing**

```css
Container {
  margin: 1 2; /* vertical horizontal */
  padding: 1 2 1 2; /* top right bottom left */
}
```

**Borders**

```css
Container {
  border: solid green;
  border-top: double red;
  border-title-align: center;
}
```

Border styles: `solid`, `dashed`, `double`, `heavy`, `wide`, `tall`, `rounded`, `blank`

**Dimensions**

```css
#panel {
  width: 50%;
  height: 1fr;
  min-width: 20;
  max-height: 100vh;
}
```

**Text**

```css
Label {
  text-align: center;
  text-opacity: 0.8;
}
```

**Layout**

```css
Container {
  layout: vertical;
  align-horizontal: center;
  align-vertical: middle;
}

Grid {
  layout: grid;
  grid-columns: 1fr 2fr;
  grid-rows: auto 1fr auto;
  gutter: 1;
}
```

### Theme Colors

Built-in theme variables for consistent styling:

```css
Label {
  color: $text;
  background: $surface;
}

.success {
  color: $success;
}
.error {
  color: $error;
}
.warning {
  color: $warning;
}
.primary {
  color: $primary;
}
.secondary {
  color: $secondary;
}
.accent {
  color: $accent;
}
```

### Responsive Breakpoints

Style differently based on terminal width:

```css
/* Default (narrow terminals) */
#sidebar {
  display: none;
}

/* Wide terminals (≥80 columns) */
Screen.-wide #sidebar {
  display: block;
  width: 30;
}
```

## Rich Markup

Style text inline using bracket notation:

```rust
Label("[bold]Bold text[/]")
Label("[italic red on blue]Styled text[/]")
Label("[underline]Underlined[/] and [strike]strikethrough[/]")
Label("[dim]Dimmed[/] and [reverse]reversed[/]")
```

**Supported attributes:**

- `bold`, `italic`, `underline`, `strike`, `dim`, `reverse`
- Color names: `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`, `white`, `black`
- Background: `on red`, `on blue`, etc.
- Hex colors: `#ff5733`, `#1a1a2e`
- Nesting: `[bold][red]Bold red[/][/]`

## Async Features

### Timers

```rust
impl App for MyApp {
    fn on_mount(&mut self) -> Option<Message> {
        // One-shot timer
        self.set_timer(Duration::from_secs(5), TimerFired);

        // Repeating interval
        self.set_interval(Duration::from_secs(1), Tick);

        None
    }
}
```

### Background Tasks

```rust
impl App for MyApp {
    fn on_mount(&mut self) -> Option<Message> {
        tokio::spawn(async {
            let data = fetch_data().await;
            // Send message back to app
        });
        None
    }
}
```

## Examples

Run examples from the repository:

```bash
# Layout examples
cargo run --example align
cargo run --example grid
cargo run --example scroll_demo

# Styling examples
cargo run --example border
cargo run --example background_tint

# Interactive examples
cargo run --example switch
cargo run --example async_timer
cargo run --example api_fetch

# Responsive design
cargo run --example breakpoints
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Application                         │
├─────────────────────────────────────────────────────────┤
│  Widget Tree          │  Message System                  │
│  ┌─────────────────┐  │  ┌──────────────────────────┐   │
│  │ Screen          │  │  │ Events → Messages        │   │
│  │ ├─ Container    │  │  │ Messages bubble up tree  │   │
│  │ │  ├─ Label     │  │  │ App handles & updates    │   │
│  │ │  └─ Switch    │  │  └──────────────────────────┘   │
│  │ └─ Vertical     │  │                                  │
│  │    └─ ...       │  │  Style System                    │
│  └─────────────────┘  │  ┌──────────────────────────┐   │
│                       │  │ TCSS parsing             │   │
│  Layout Engine        │  │ Cascade resolution       │   │
│  ┌─────────────────┐  │  │ Computed styles          │   │
│  │ Measure phase   │  │  └──────────────────────────┘   │
│  │ Arrange phase   │  │                                  │
│  │ Align phase     │  │                                  │
│  └─────────────────┘  │                                  │
├─────────────────────────────────────────────────────────┤
│                    Render Pipeline                       │
│  Widget → Canvas (double buffer) → Terminal              │
└─────────────────────────────────────────────────────────┘
```

### Crate Structure

| Crate     | Description                                              |
| --------- | -------------------------------------------------------- |
| `textual` | Core framework — widgets, layouts, rendering, event loop |
| `tcss`    | TCSS parser — CSS-like styling with terminal extensions  |
| `rich`    | Rich markup parser — `[bold red]text[/]` syntax          |

## Comparison with Python Textual

| Feature        | Python Textual     | textual-rs   |
| -------------- | ------------------ | ------------ |
| Language       | Python             | Rust         |
| Async Runtime  | asyncio            | Tokio        |
| UI Declaration | `compose()` method | `ui!` macro  |
| Styling        | CSS/TCSS           | TCSS         |
| Type Safety    | Runtime            | Compile-time |
| Performance    | Good               | Excellent    |
| Memory Safety  | GC                 | Ownership    |

## Roadmap

- [x] Core widget system
- [x] Vertical/Horizontal layouts
- [x] Grid layout (basic)
- [x] TCSS styling engine
- [x] Rich markup support
- [x] Border rendering with titles
- [x] Scrollable containers
- [x] Async timers/intervals
- [x] Focus management
- [x] Responsive breakpoints
- [ ] Grid row/column spanning
- [ ] Input widget
- [ ] Button widget
- [ ] DataTable widget
- [ ] Tabs/TabbedContent
- [ ] Tree widget
- [ ] Form validation

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

```bash
# Clone the repository
git clone https://github.com/fcoury/textual-rs
cd textual-rs

# Run tests
cargo test

# Run an example
cargo run --example switch
```

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Textual](https://github.com/Textualize/textual) — The original Python library that inspired this project
- [Rich](https://github.com/Textualize/rich) — Python library for rich text formatting
- [crossterm](https://github.com/crossterm-rs/crossterm) — Cross-platform terminal manipulation
