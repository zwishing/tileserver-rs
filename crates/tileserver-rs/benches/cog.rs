//! Benchmarks for COG (Cloud-Optimized GeoTIFF) hot paths.
//!
//! GDAL/dataset opens are not benched here — they're I/O-bound and
//! dominated by the underlying file or network range read. What's
//! measured is the pure path classification logic that decides whether
//! GDAL will use the local file driver, `/vsicurl/` HTTP range reads,
//! or `/vsis3/` for S3.
//!
//! Run with: `cargo bench --bench cog --features stac`
//! (cog path classification lives behind the `stac` feature gate.)

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tileserver_rs::sources::stac::to_gdal_cog_path;

fn bench_to_gdal_cog_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("cog_to_gdal_path");

    group.bench_function("https", |b| {
        b.iter(|| {
            black_box(to_gdal_cog_path(black_box(
                "https://earth-search.aws.element84.com/v1/sentinel-2/2026/04/asset.tif",
            )))
        });
    });

    group.bench_function("http", |b| {
        b.iter(|| {
            black_box(to_gdal_cog_path(black_box(
                "http://example.com/local/raster.tif",
            )))
        });
    });

    group.bench_function("s3", |b| {
        b.iter(|| {
            black_box(to_gdal_cog_path(black_box(
                "s3://sentinel-cogs/sentinel-s2-l2a-cogs/9/U/UA/2024/12/data.tif",
            )))
        });
    });

    group.bench_function("local", |b| {
        b.iter(|| {
            black_box(to_gdal_cog_path(black_box(
                "/data/raster/benchmark-rgb.cog.tif",
            )))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_to_gdal_cog_path);
criterion_main!(benches);
