# Textual-RS Benchmarks

Performance benchmarks for the Textual-RS rendering pipeline.

## Quick Start

```bash
# Run all benchmarks
cargo bench -p textual

# Run specific benchmark suite
cargo bench -p textual --bench segment_benchmarks
cargo bench -p textual --bench strip_benchmarks
cargo bench -p textual --bench canvas_benchmarks
cargo bench -p textual --bench render_benchmarks
```

## Benchmark Suites

| Suite | Description | Key Metrics |
|-------|-------------|-------------|
| `segment_benchmarks` | Segment operations | `cell_length()`, creation, splitting |
| `strip_benchmarks` | Strip operations | `from_segments()`, cropping, joining |
| `canvas_benchmarks` | Canvas operations | `put_str()`, `render_strip()`, clipping |
| `render_benchmarks` | End-to-end rendering | Full pipeline: CSS → layout → canvas |

## Comparing Before/After Optimizations

```bash
# Save baseline before making changes
cargo bench -p textual -- --save-baseline before

# Make your optimization changes...

# Compare against baseline
cargo bench -p textual -- --baseline before
```

## Viewing HTML Reports

After running benchmarks, open the HTML report:

```bash
open target/criterion/report/index.html
```

## Manual Benchmarking with Hyperfine

For wall-clock timing of example apps, use [hyperfine](https://github.com/sharkdp/hyperfine):

```bash
# Install hyperfine
brew install hyperfine  # macOS
# or: cargo install hyperfine

# Benchmark example compilation + startup
hyperfine --warmup 3 'cargo run --release --example buttons'

# Compare two versions (in separate worktrees)
hyperfine --warmup 3 \
    'cargo run --release --example buttons' \
    '../texrs-baseline/target/release/examples/buttons'

# Benchmark with specific iteration count
hyperfine --runs 10 'cargo run --release --example hello'

# Export results to JSON for tracking
hyperfine --export-json bench-results.json \
    'cargo run --release --example buttons'
```

### Benchmarking Interactive Apps

For apps that don't exit automatically, create a benchmark wrapper:

```bash
# Run app for N frames then exit (if supported)
timeout 5s cargo run --release --example buttons

# Or use expect/tmux for scripted interaction
```

## Key Optimizations Tracked

| Optimization | Benchmark | Expected Improvement |
|--------------|-----------|---------------------|
| Segment width caching | `segment_cell_length/*` | O(N) → O(1) |
| Differential rendering | `render_repeated/*` | 90%+ fewer bytes written |
| Output batching | `canvas_flush/*` | Fewer syscalls |
| SmallVec for strips | `strip_from_segments/*` | Fewer heap allocations |

## Performance Targets

| Operation | Target | Current |
|-----------|--------|---------|
| `cell_length()` | < 1ns | ✅ ~0.6ps (cached) |
| Full frame 80x24 | < 1ms | TBD |
| Incremental update | < 100µs | TBD |
