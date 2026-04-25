//! Benchmarks for the DuckDB source's pure-function hot paths.
//!
//! Connection setup and the DuckDB query itself are not benched here —
//! those live behind a `Connection` handle and are dominated by the
//! engine's spatial extension. What's measured is the per-tile preflight
//! that runs before the query is sent: tile→bbox conversion (Web
//! Mercator math) and `{z}/{x}/{y}/{bbox}` template substitution.
//!
//! Run with: `cargo bench --bench duckdb --features duckdb`

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tileserver_rs::sources::duckdb::{substitute_template, tile_to_bbox};

fn bench_tile_to_bbox(c: &mut Criterion) {
    let mut group = c.benchmark_group("duckdb_tile_to_bbox");
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

fn bench_substitute_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("duckdb_substitute_template");
    let bbox = [11.0_f64, 43.0, 12.0, 44.0];

    let simple = "SELECT * FROM tiles WHERE z = {z} AND x = {x} AND y = {y}";
    group.bench_function("simple_zxy", |b| {
        b.iter(|| {
            black_box(substitute_template(
                black_box(simple),
                black_box(14),
                black_box(8589),
                black_box(5712),
                black_box(&bbox),
            ))
        });
    });

    let with_bbox = "SELECT ST_AsMVT(features) FROM points WHERE \
                      ST_Intersects(geom, ST_MakeEnvelope({bbox})) AND z = {z}";
    group.bench_function("with_bbox", |b| {
        b.iter(|| {
            black_box(substitute_template(
                black_box(with_bbox),
                black_box(14),
                black_box(8589),
                black_box(5712),
                black_box(&bbox),
            ))
        });
    });

    let split_bbox = "SELECT * FROM t WHERE x BETWEEN {bbox_xmin} AND {bbox_xmax} \
                      AND y BETWEEN {bbox_ymin} AND {bbox_ymax}";
    group.bench_function("split_bbox", |b| {
        b.iter(|| {
            black_box(substitute_template(
                black_box(split_bbox),
                black_box(14),
                black_box(8589),
                black_box(5712),
                black_box(&bbox),
            ))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_tile_to_bbox, bench_substitute_template);
criterion_main!(benches);
