//! Benchmarks for the PostGIS source's pure-function hot paths.
//!
//! Network round-trips and `ST_AsMVT` execution are not benched here —
//! those are PostgreSQL-bound and dominate end-to-end latency in a way
//! tile-server-side optimisation can't address. What's measured is the
//! per-tile preflight logic that runs in our process before the SQL is
//! ever sent to the connection: geometry-type classification (which
//! decides whether to use a buffered envelope) and PostgreSQL→JSON
//! Schema mapping (used by `/queryables` and `/sortables`).
//!
//! Run with: `cargo bench --bench postgres --features postgres`

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tileserver_rs::sources::postgres::{is_point_geometry, pg_type_to_json_schema};

fn bench_is_point_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("postgres_is_point_geometry");

    for (label, input) in [
        ("point", "POINT"),
        ("multipoint", "MULTIPOINT"),
        ("point_z", "POINT Z"),
        ("point_zm", "POINT ZM"),
        ("polygon", "POLYGON"),
        ("linestring", "LINESTRING"),
        ("lowercase_point", "point"),
        ("mixed_case_multipoint", "MultiPoint Z"),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| black_box(is_point_geometry(black_box(input))));
        });
    }
    group.finish();
}

fn bench_pg_type_to_json_schema(c: &mut Criterion) {
    let mut group = c.benchmark_group("postgres_pg_type_to_json_schema");

    for (label, input) in [
        ("integer", "integer"),
        ("bigint", "bigint"),
        ("numeric", "numeric"),
        ("text", "text"),
        ("character_varying", "character varying"),
        ("uuid", "uuid"),
        ("boolean", "boolean"),
        ("jsonb", "jsonb"),
        ("array", "ARRAY"),
        ("unknown_type", "unknown_custom_type"),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| black_box(pg_type_to_json_schema(black_box(input))));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_is_point_geometry,
    bench_pg_type_to_json_schema
);
criterion_main!(benches);
