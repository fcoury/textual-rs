use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use textual::segment::{Segment, Style};

fn bench_cell_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_cell_length");

    // Test different text types - this is the key benchmark for optimization 1.3
    let cases = [
        ("ascii_short", "Hello"),
        ("ascii_medium", "Hello, World! This is a test."),
        ("ascii_long", &"x".repeat(100)),
        ("unicode_cjk", "你好世界日本語한국어"),
        ("unicode_emoji", "Hello 🎉🚀✨🌟💫"),
        ("mixed", "Hello 世界 🎉 test"),
    ];

    for (name, text) in cases {
        let segment = Segment::new(text);
        group.bench_with_input(BenchmarkId::new("cell_length", name), &segment, |b, seg| {
            b.iter(|| black_box(seg).cell_length())
        });
    }
    group.finish();
}

fn bench_cell_length_repeated(c: &mut Criterion) {
    // Benchmark repeated calls to cell_length on the same segment
    // This simulates real-world usage in layout loops
    let segment = Segment::new("Hello, World! This is a typical line of text.");

    c.bench_function("cell_length_10x", |b| {
        b.iter(|| {
            let seg = black_box(&segment);
            for _ in 0..10 {
                black_box(seg.cell_length());
            }
        })
    });

    c.bench_function("cell_length_100x", |b| {
        b.iter(|| {
            let seg = black_box(&segment);
            for _ in 0..100 {
                black_box(seg.cell_length());
            }
        })
    });
}

fn bench_segment_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_creation");

    group.bench_function("new_short", |b| {
        b.iter(|| Segment::new(black_box("Hello World")))
    });

    group.bench_function("new_medium", |b| {
        b.iter(|| Segment::new(black_box("This is a medium-length string for testing")))
    });

    let long_text = "x".repeat(1000);
    group.bench_function("new_long", |b| {
        b.iter(|| Segment::new(black_box(&long_text)))
    });

    group.bench_function("styled_short", |b| {
        let style = Style::default();
        b.iter(|| Segment::styled(black_box("Hello"), style.clone()))
    });

    group.bench_function("blank_10", |b| {
        b.iter(|| Segment::blank(black_box(10), None))
    });

    group.bench_function("blank_80", |b| {
        b.iter(|| Segment::blank(black_box(80), None))
    });

    group.finish();
}

fn bench_segment_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_split");

    let short = Segment::new("Hello");
    let medium = Segment::new("Hello, World! This is a test string.");
    let long = Segment::new(&"x".repeat(200));
    let unicode = Segment::new("Hello 世界 🎉 test string here");

    group.bench_function("split_short", |b| b.iter(|| black_box(&short).split_at(2)));

    group.bench_function("split_medium_early", |b| {
        b.iter(|| black_box(&medium).split_at(5))
    });

    group.bench_function("split_medium_middle", |b| {
        b.iter(|| black_box(&medium).split_at(15))
    });

    group.bench_function("split_long", |b| b.iter(|| black_box(&long).split_at(100)));

    group.bench_function("split_unicode", |b| {
        b.iter(|| black_box(&unicode).split_at(10))
    });

    group.finish();
}

fn bench_segment_style_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_style_ops");

    let segment = Segment::new("Test text");
    let styled = Segment::styled(
        "Styled text",
        Style {
            bold: true,
            ..Default::default()
        },
    );

    let overlay = Style {
        italic: true,
        ..Default::default()
    };

    group.bench_function("apply_style_to_unstyled", |b| {
        b.iter(|| black_box(&segment).apply_style(&overlay))
    });

    group.bench_function("apply_style_to_styled", |b| {
        b.iter(|| black_box(&styled).apply_style(&overlay))
    });

    group.bench_function("set_style", |b| {
        let style = Some(overlay.clone());
        b.iter(|| black_box(&segment).set_style(style.clone()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cell_length,
    bench_cell_length_repeated,
    bench_segment_creation,
    bench_segment_split,
    bench_segment_style_ops
);
criterion_main!(benches);
