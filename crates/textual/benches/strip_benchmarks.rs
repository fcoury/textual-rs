use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use textual::segment::{Segment, Style};
use textual::strip::Strip;

fn bench_strip_from_segments(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_from_segments");

    // This benchmark tests creating strips from pre-existing Vec
    // (includes Vec clone cost)
    for count in [1, 2, 3, 4, 5, 10, 50] {
        let segments: Vec<_> = (0..count)
            .map(|i| Segment::new(format!("segment_{}", i)))
            .collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("from_segments", count),
            &segments,
            |b, segs| b.iter(|| Strip::from_segments(black_box(segs.clone()))),
        );
    }
    group.finish();
}

fn bench_strip_inline_creation(c: &mut Criterion) {
    // This benchmark shows the real benefit of SmallVec:
    // Creating strips from a small fixed number of segments
    // without pre-allocating a Vec (typical real-world usage)
    let mut group = c.benchmark_group("strip_inline_creation");

    // 1 segment (most common case)
    group.bench_function("inline_1_segment", |b| {
        b.iter(|| Strip::from_iter([Segment::new("Hello")]))
    });

    // 2 segments (e.g., "Label: " + "value")
    group.bench_function("inline_2_segments", |b| {
        b.iter(|| Strip::from_iter([Segment::new("Label: "), Segment::new("value")]))
    });

    // 3 segments
    group.bench_function("inline_3_segments", |b| {
        b.iter(|| {
            Strip::from_iter([
                Segment::new("Name: "),
                Segment::new("John"),
                Segment::new(" Doe"),
            ])
        })
    });

    // 4 segments (max inline storage)
    group.bench_function("inline_4_segments", |b| {
        b.iter(|| {
            Strip::from_iter([
                Segment::new("a"),
                Segment::new("b"),
                Segment::new("c"),
                Segment::new("d"),
            ])
        })
    });

    group.finish();
}

fn bench_strip_from_segment(c: &mut Criterion) {
    let segment = Segment::new("Hello, World!");

    c.bench_function("strip_from_segment", |b| {
        b.iter(|| Strip::from_segment(black_box(segment.clone())))
    });
}

fn bench_strip_blank(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_blank");

    for width in [10, 40, 80, 120] {
        group.bench_with_input(BenchmarkId::new("blank", width), &width, |b, &w| {
            b.iter(|| Strip::blank(black_box(w), None))
        });
    }

    group.finish();
}

fn bench_strip_crop(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_crop");

    // Single segment strip
    let single = Strip::from_segment(Segment::new(&"x".repeat(200)));

    // Multi-segment strip (typical real-world scenario)
    let multi_segments: Vec<_> = (0..20)
        .map(|i| Segment::new(format!("segment_{:02}", i)))
        .collect();
    let multi = Strip::from_segments(multi_segments);

    group.bench_function("crop_single_early", |b| {
        b.iter(|| black_box(&single).crop(0, 20))
    });

    group.bench_function("crop_single_middle", |b| {
        b.iter(|| black_box(&single).crop(50, 150))
    });

    group.bench_function("crop_single_late", |b| {
        b.iter(|| black_box(&single).crop(180, 200))
    });

    group.bench_function("crop_multi_early", |b| {
        b.iter(|| black_box(&multi).crop(0, 30))
    });

    group.bench_function("crop_multi_middle", |b| {
        b.iter(|| black_box(&multi).crop(50, 100))
    });

    group.bench_function("crop_multi_late", |b| {
        b.iter(|| black_box(&multi).crop(150, 200))
    });

    group.finish();
}

fn bench_strip_cell_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_cell_length");

    // Strip cell_length should be O(1) since it's cached
    let segments: Vec<_> = (0..50)
        .map(|i| Segment::new(format!("segment_{}", i)))
        .collect();
    let strip = Strip::from_segments(segments);

    group.bench_function("cell_length", |b| {
        b.iter(|| black_box(&strip).cell_length())
    });

    // Compare with calculating from scratch (what would happen without caching)
    group.bench_function("cell_length_from_segments_sum", |b| {
        b.iter(|| {
            let s = black_box(&strip);
            s.segments()
                .iter()
                .map(|seg| seg.cell_length())
                .sum::<usize>()
        })
    });

    group.finish();
}

fn bench_strip_simplify(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_simplify");

    // All same style - should merge into 1 segment
    let same_style = Style::with_fg(tcss::types::RgbaColor::rgb(255, 0, 0));
    let same_segments: Vec<_> = (0..20)
        .map(|_| Segment::styled("word ", same_style.clone()))
        .collect();
    let same_strip = Strip::from_segments(same_segments);

    // Alternating styles - no merging possible
    let style_a = Style::with_fg(tcss::types::RgbaColor::rgb(255, 0, 0));
    let style_b = Style::with_fg(tcss::types::RgbaColor::rgb(0, 255, 0));
    let alt_segments: Vec<_> = (0..20)
        .map(|i| {
            if i % 2 == 0 {
                Segment::styled("word ", style_a.clone())
            } else {
                Segment::styled("word ", style_b.clone())
            }
        })
        .collect();
    let alt_strip = Strip::from_segments(alt_segments);

    group.bench_function("simplify_same_style", |b| {
        b.iter(|| black_box(&same_strip).simplify())
    });

    group.bench_function("simplify_alternating", |b| {
        b.iter(|| black_box(&alt_strip).simplify())
    });

    group.finish();
}

fn bench_strip_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_join");

    for count in [2, 5, 10, 20] {
        let strips: Vec<_> = (0..count)
            .map(|i| Strip::from_segment(Segment::new(format!("strip_{}", i))))
            .collect();

        group.bench_with_input(BenchmarkId::new("join", count), &strips, |b, strips| {
            b.iter(|| Strip::join(black_box(strips.clone())))
        });
    }

    group.finish();
}

fn bench_strip_apply_style(c: &mut Criterion) {
    let segments: Vec<_> = (0..10)
        .map(|i| Segment::new(format!("segment_{}", i)))
        .collect();
    let strip = Strip::from_segments(segments);

    let style = Style {
        bold: true,
        italic: true,
        ..Default::default()
    };

    c.bench_function("strip_apply_style", |b| {
        b.iter(|| black_box(&strip).apply_style(&style))
    });
}

fn bench_strip_text_align(c: &mut Criterion) {
    let strip = Strip::from_segment(Segment::new("Hello World"));
    let width = 40;

    let mut group = c.benchmark_group("strip_text_align");

    group.bench_function("align_left", |b| {
        b.iter(|| {
            black_box(&strip).text_align(
                tcss::types::text::AlignHorizontal::Left,
                black_box(width),
                None,
            )
        })
    });

    group.bench_function("align_center", |b| {
        b.iter(|| {
            black_box(&strip).text_align(
                tcss::types::text::AlignHorizontal::Center,
                black_box(width),
                None,
            )
        })
    });

    group.bench_function("align_right", |b| {
        b.iter(|| {
            black_box(&strip).text_align(
                tcss::types::text::AlignHorizontal::Right,
                black_box(width),
                None,
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_strip_from_segments,
    bench_strip_inline_creation,
    bench_strip_from_segment,
    bench_strip_blank,
    bench_strip_crop,
    bench_strip_cell_length,
    bench_strip_simplify,
    bench_strip_join,
    bench_strip_apply_style,
    bench_strip_text_align
);
criterion_main!(benches);
