//! Benchmarks for the GeoParquet source's pure-function hot paths.
//!
//! Parquet file reads + Arrow array decoding are not benched here —
//! they're dominated by I/O and the Apache Arrow C++ kernels. What's
//! measured is the per-tile preflight: tile→bbox conversion (Web
//! Mercator math) and the buffered variant that adds an MVT-extent
//! pixel margin to the bbox before scanning the parquet row groups.
//!
//! Run with: `cargo bench --bench geoparquet --features geoparquet`

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tileserver_rs::sources::geoparquet::{tile_to_bbox, tile_to_bbox_with_buffer};

fn bench_tile_to_bbox(c: &mut Criterion) {
    let mut group = c.benchmark_group("geoparquet_tile_to_bbox");
    for &(z, x, y) in &[
        (0_u8, 0_u32, 0_u32),
        (8, 73, 90),
        (14, 8589, 5712),
        (18, 137440, 91392),
    ] {
        group.bench_function(format!("z{z}"), |b| {
            b.iter(|| black_box(tile_to_bbox(black_box(z), black_box(x), black_box(y))));
        });
    }
    group.finish();
}

fn bench_tile_to_bbox_with_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("geoparquet_tile_to_bbox_with_buffer");
    for &buf in &[0_u32, 64, 256, 512] {
        group.bench_function(format!("buf_{buf}px"), |b| {
            b.iter(|| {
                black_box(tile_to_bbox_with_buffer(
                    black_box(14),
                    black_box(8589),
                    black_box(5712),
                    black_box(buf),
                ))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tile_to_bbox, bench_tile_to_bbox_with_buffer);
criterion_main!(benches);
