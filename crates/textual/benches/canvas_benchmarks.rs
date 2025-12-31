use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use textual::canvas::{Canvas, TextAttributes};
use textual::segment::{Segment, Style};
use textual::strip::Strip;

fn bench_canvas_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_new");

    let sizes = [(80, 24), (120, 40), (200, 50)];

    for (w, h) in sizes {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));
        group.bench_with_input(
            BenchmarkId::new("new", format!("{}x{}", w, h)),
            &(w, h),
            |b, &(w, h)| b.iter(|| Canvas::new(black_box(w), black_box(h))),
        );
    }

    group.finish();
}

fn bench_canvas_put_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_put_str");

    let mut canvas = Canvas::new(200, 50);

    // Different string lengths
    for len in [10, 50, 80, 120] {
        let text = "x".repeat(len);
        group.bench_with_input(BenchmarkId::new("put_str", len), &text, |b, text| {
            b.iter(|| {
                canvas.put_str(
                    black_box(0),
                    black_box(0),
                    black_box(text),
                    None,
                    None,
                    TextAttributes::default(),
                );
            })
        });
    }

    group.finish();
}

fn bench_canvas_put_str_with_colors(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_put_str_with_colors");

    let mut canvas = Canvas::new(120, 40);
    let text = "Hello, World! This is a test string with colors.";
    let fg = Some(tcss::types::RgbaColor::rgb(255, 128, 0));
    let bg = Some(tcss::types::RgbaColor::rgb(0, 64, 128));

    group.bench_function("with_fg_only", |b| {
        b.iter(|| {
            canvas.put_str(
                black_box(0),
                black_box(0),
                black_box(text),
                fg.clone(),
                None,
                TextAttributes::default(),
            );
        })
    });

    group.bench_function("with_fg_and_bg", |b| {
        b.iter(|| {
            canvas.put_str(
                black_box(0),
                black_box(0),
                black_box(text),
                fg.clone(),
                bg.clone(),
                TextAttributes::default(),
            );
        })
    });

    let attrs = TextAttributes {
        bold: true,
        italic: true,
        underline: true,
        ..Default::default()
    };
    group.bench_function("with_colors_and_attrs", |b| {
        b.iter(|| {
            canvas.put_str(
                black_box(0),
                black_box(0),
                black_box(text),
                fg.clone(),
                bg.clone(),
                attrs,
            );
        })
    });

    group.finish();
}

fn bench_canvas_render_strip(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_render_strip");

    let mut canvas = Canvas::new(120, 40);

    // Single segment strip
    let single = Strip::from_segment(Segment::new("Hello, World! This is a single segment."));

    // Multi-segment strip (typical styled text)
    let multi = Strip::from_segments(vec![
        Segment::styled(
            "Label: ",
            Style {
                bold: true,
                ..Default::default()
            },
        ),
        Segment::new("This is the value with "),
        Segment::styled(
            "emphasis",
            Style {
                italic: true,
                ..Default::default()
            },
        ),
        Segment::new(" and more text."),
    ]);

    // Full-width strip (80 chars)
    let full_width = Strip::from_segment(Segment::new(&"x".repeat(80)));

    group.bench_function("render_single_segment", |b| {
        b.iter(|| {
            canvas.render_strip(black_box(&single), black_box(0), black_box(0));
        })
    });

    group.bench_function("render_multi_segment", |b| {
        b.iter(|| {
            canvas.render_strip(black_box(&multi), black_box(0), black_box(0));
        })
    });

    group.bench_function("render_full_width", |b| {
        b.iter(|| {
            canvas.render_strip(black_box(&full_width), black_box(0), black_box(0));
        })
    });

    group.finish();
}

fn bench_canvas_render_strips(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_render_strips");

    let mut canvas = Canvas::new(80, 24);

    // Create realistic content - 24 lines of text
    let strips: Vec<Strip> = (0..24)
        .map(|i| {
            Strip::from_segments(vec![
                Segment::styled(
                    format!("Line {:02}: ", i),
                    Style {
                        bold: true,
                        ..Default::default()
                    },
                ),
                Segment::new(format!("Content for line {} with some text", i)),
            ])
        })
        .collect();

    group.throughput(Throughput::Elements(24));
    group.bench_function("render_24_strips", |b| {
        b.iter(|| {
            canvas.render_strips(black_box(&strips), black_box(0), black_box(0));
        })
    });

    group.finish();
}

fn bench_canvas_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_full_frame");

    // Standard terminal sizes
    for (w, h) in [(80, 24), (120, 40)] {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));

        // Create a full frame of content
        let strips: Vec<Strip> = (0..h)
            .map(|i| {
                Strip::from_segments(vec![
                    Segment::styled(
                        format!("R{:02} ", i),
                        Style {
                            bold: true,
                            ..Default::default()
                        },
                    ),
                    Segment::new(&"X".repeat((w as usize) - 4)),
                ])
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("full_frame", format!("{}x{}", w, h)),
            &strips,
            |b, strips| {
                let mut canvas = Canvas::new(w, h);
                b.iter(|| {
                    canvas.render_strips(black_box(strips), black_box(0), black_box(0));
                })
            },
        );
    }

    group.finish();
}

fn bench_canvas_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_clear");

    for (w, h) in [(80, 24), (120, 40), (200, 50)] {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));

        let mut canvas = Canvas::new(w, h);
        // Fill with some content first
        for y in 0..h {
            canvas.put_str(0, y as i32, &"X".repeat(w as usize), None, None, TextAttributes::default());
        }

        group.bench_with_input(
            BenchmarkId::new("clear", format!("{}x{}", w, h)),
            &(),
            |b, _| {
                b.iter(|| {
                    canvas.clear();
                })
            },
        );
    }

    group.finish();
}

fn bench_canvas_clipping(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_clipping");

    let mut canvas = Canvas::new(120, 40);
    let text = "This is some text that might be clipped";

    // Without clipping
    group.bench_function("no_clip", |b| {
        b.iter(|| {
            canvas.put_str(
                black_box(10),
                black_box(10),
                black_box(text),
                None,
                None,
                TextAttributes::default(),
            );
        })
    });

    // With clipping (text partially visible)
    canvas.push_clip(textual::canvas::Region::new(15, 5, 20, 20));
    group.bench_function("with_clip", |b| {
        b.iter(|| {
            canvas.put_str(
                black_box(10),
                black_box(10),
                black_box(text),
                None,
                None,
                TextAttributes::default(),
            );
        })
    });
    canvas.pop_clip();

    group.finish();
}

fn bench_canvas_to_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas_to_snapshot");

    for (w, h) in [(80, 24), (120, 40)] {
        let cells = (w * h) as u64;
        group.throughput(Throughput::Elements(cells));

        let mut canvas = Canvas::new(w, h);
        // Fill with realistic content
        for y in 0..h {
            let text = format!("Line {:02}: {}", y, "x".repeat((w as usize) - 10));
            canvas.put_str(0, y as i32, &text, None, None, TextAttributes::default());
        }

        group.bench_with_input(
            BenchmarkId::new("to_snapshot", format!("{}x{}", w, h)),
            &canvas,
            |b, canvas| {
                b.iter(|| black_box(canvas).to_snapshot())
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_canvas_new,
    bench_canvas_put_str,
    bench_canvas_put_str_with_colors,
    bench_canvas_render_strip,
    bench_canvas_render_strips,
    bench_canvas_full_frame,
    bench_canvas_clear,
    bench_canvas_clipping,
    bench_canvas_to_snapshot
);
criterion_main!(benches);
