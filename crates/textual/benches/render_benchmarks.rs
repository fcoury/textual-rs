//! End-to-end rendering benchmarks.
//!
//! These benchmarks measure the full rendering pipeline:
//! CSS parsing → style resolution → layout → strip generation → canvas rendering
//!
//! Run with: `cargo bench -p textual --bench render_benchmarks`

use std::collections::VecDeque;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use textual::canvas::{Canvas, Region, Size};
use textual::style_resolver::resolve_styles;
use textual::testing::{build_combined_css, render_to_canvas};
use textual::tree::WidgetTree;
use textual::{Compose, Label, Screen, Widget, ui};

// ============================================================================
// Simple App - Single Label
// ============================================================================

mod simple_app {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct SimpleApp;

    impl Compose for SimpleApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Hello, World!")
            }
        }
    }

    pub const CSS: &str = r#"
Label {
    width: 100%;
    height: 100%;
    content-align: center middle;
}
"#;
}

// ============================================================================
// Border App - Multiple styled labels with borders
// ============================================================================

mod border_app {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderApp;

    impl Compose for BorderApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("My border is solid red", id: "label1")
                Label("My border is dashed green", id: "label2")
                Label("My border is tall blue", id: "label3")
            }
        }
    }

    pub const CSS: &str = r#"
#label1 {
    background: red 20%;
    color: red;
    border: solid red;
}

#label2 {
    background: green 20%;
    color: green;
    border: dashed green;
}

#label3 {
    background: blue 20%;
    color: blue;
    border: tall blue;
}

Screen {
    background: white;
}

Screen > Label {
    width: 100%;
    height: 5;
    content-align: center middle;
    color: white;
    margin: 1;
    box-sizing: border-box;
}
"#;
}

// ============================================================================
// Grid App - Complex grid layout
// ============================================================================

mod grid_app {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct GridApp;

    impl Compose for GridApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Cell 1", id: "c1")
                Label("Cell 2", id: "c2")
                Label("Cell 3", id: "c3")
                Label("Cell 4", id: "c4")
                Label("Cell 5", id: "c5")
                Label("Cell 6", id: "c6")
                Label("Cell 7", id: "c7")
                Label("Cell 8", id: "c8")
                Label("Cell 9", id: "c9")
            }
        }
    }

    pub const CSS: &str = r#"
Screen {
    layout: grid;
    grid-size: 3 3;
    grid-gutter: 1;
    background: #1a1a2e;
}

Label {
    width: 100%;
    height: 100%;
    content-align: center middle;
    background: #16213e;
    color: #e94560;
    border: solid #0f3460;
}

#c1, #c5, #c9 { background: #0f3460; color: #e94560; }
#c2, #c4, #c6, #c8 { background: #16213e; color: #00fff5; }
#c3, #c7 { background: #1a1a2e; color: #ffd700; }
"#;
}

// ============================================================================
// Text Heavy App - Many lines of text
// ============================================================================

mod text_heavy_app {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct TextHeavyApp {
        pub line_count: usize,
    }

    impl Compose for TextHeavyApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            let labels: Vec<Box<dyn Widget<Self::Message>>> = (0..self.line_count)
                .map(|i| {
                    Box::new(Label::new(format!(
                        "Line {:03}: This is some sample text content for benchmarking purposes.",
                        i
                    ))) as Box<dyn Widget<Self::Message>>
                })
                .collect();
            labels
        }
    }

    pub const CSS: &str = r#"
Screen {
    background: #1e1e1e;
}

Label {
    width: 100%;
    height: 1;
    color: #d4d4d4;
    background: #252526;
}
"#;
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_simple_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_simple");

    let app = simple_app::SimpleApp;

    for (w, h) in [(80, 24), (120, 40), (200, 50)] {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));
        group.bench_with_input(
            BenchmarkId::new("simple", format!("{}x{}", w, h)),
            &(w, h),
            |b, &(w, h)| {
                b.iter(|| render_to_canvas(black_box(&app), simple_app::CSS, w, h))
            },
        );
    }

    group.finish();
}

fn bench_border_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_border");

    let app = border_app::BorderApp;

    for (w, h) in [(80, 24), (120, 40)] {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));
        group.bench_with_input(
            BenchmarkId::new("border", format!("{}x{}", w, h)),
            &(w, h),
            |b, &(w, h)| {
                b.iter(|| render_to_canvas(black_box(&app), border_app::CSS, w, h))
            },
        );
    }

    group.finish();
}

fn bench_grid_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_grid");

    let app = grid_app::GridApp;

    for (w, h) in [(80, 24), (120, 40)] {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));
        group.bench_with_input(
            BenchmarkId::new("grid", format!("{}x{}", w, h)),
            &(w, h),
            |b, &(w, h)| {
                b.iter(|| render_to_canvas(black_box(&app), grid_app::CSS, w, h))
            },
        );
    }

    group.finish();
}

fn bench_text_heavy_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_text_heavy");

    // Benchmark with different amounts of text
    for lines in [10, 24, 40, 100] {
        let app = text_heavy_app::TextHeavyApp { line_count: lines };
        let h = lines.max(24) as u16;

        group.throughput(Throughput::Elements((80 * h) as u64));
        group.bench_with_input(
            BenchmarkId::new("text_heavy", format!("{}_lines", lines)),
            &app,
            |b, app| {
                b.iter(|| render_to_canvas(black_box(app), text_heavy_app::CSS, 80, h))
            },
        );
    }

    group.finish();
}

fn bench_repeated_render(c: &mut Criterion) {
    // Simulate multiple frame renders (important for differential rendering)
    let mut group = c.benchmark_group("render_repeated");

    let app = border_app::BorderApp;

    group.bench_function("10_frames_80x24", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = render_to_canvas(black_box(&app), border_app::CSS, 80, 24);
            }
        })
    });

    group.bench_function("100_frames_80x24", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = render_to_canvas(black_box(&app), border_app::CSS, 80, 24);
            }
        })
    });

    group.finish();
}

fn bench_canvas_to_snapshot(c: &mut Criterion) {
    // Benchmark snapshot generation (used in tests)
    let mut group = c.benchmark_group("snapshot_generation");

    let app = border_app::BorderApp;
    let canvas = render_to_canvas(&app, border_app::CSS, 80, 24);

    group.bench_function("to_snapshot_80x24", |b| {
        b.iter(|| black_box(&canvas).to_snapshot())
    });

    group.bench_function("to_ansi_snapshot_80x24", |b| {
        b.iter(|| black_box(&canvas).to_ansi_snapshot())
    });

    group.finish();
}

/// Benchmark layout caching - renders same widget tree multiple times.
/// This tests the optimization where layout is cached and reused when
/// region/viewport haven't changed.
fn bench_layout_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_caching");

    let themes = tcss::types::Theme::standard_themes();
    let theme = themes
        .get("textual-dark")
        .cloned()
        .unwrap_or_else(|| tcss::types::Theme::new("default", true));

    // Test with grid layout (9 cells) - more layout computation
    let app = grid_app::GridApp;
    let root = Box::new(Screen::new(app.compose()));
    let mut tree = WidgetTree::new(root);
    tree.root_mut().on_resize(Size::new(80, 24));

    let combined_css = build_combined_css(tree.root_mut(), grid_app::CSS);
    let stylesheet = tcss::parser::parse_stylesheet(&combined_css).expect("CSS parsing failed");
    let mut ancestors = VecDeque::new();
    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

    let region = Region::from_u16(0, 0, 80, 24);

    // Single render (first render, cache miss)
    group.bench_function("grid_first_render", |b| {
        b.iter(|| {
            // Create fresh tree for first-render scenario
            let root = Box::new(Screen::new(grid_app::GridApp.compose()));
            let mut tree = WidgetTree::new(root);
            tree.root_mut().on_resize(Size::new(80, 24));
            let combined_css = build_combined_css(tree.root_mut(), grid_app::CSS);
            let stylesheet = tcss::parser::parse_stylesheet(&combined_css).unwrap();
            let mut ancestors = VecDeque::new();
            resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);
            let mut canvas = Canvas::new(80, 24);
            tree.root().render(black_box(&mut canvas), region);
        })
    });

    // Multiple renders of same tree (cache hits after first)
    group.bench_function("grid_10_cached_renders", |b| {
        b.iter(|| {
            let mut canvas = Canvas::new(80, 24);
            for _ in 0..10 {
                tree.root().render(black_box(&mut canvas), region);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_render,
    bench_border_render,
    bench_grid_render,
    bench_text_heavy_render,
    bench_repeated_render,
    bench_canvas_to_snapshot,
    bench_layout_caching
);
criterion_main!(benches);
