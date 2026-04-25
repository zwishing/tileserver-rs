//! Benchmarks for STAC pure-function hot paths.
//!
//! Network I/O (the actual STAC `/search` round-trip + COG fetch) is excluded
//! by design — those are dominated by RTT, not CPU. What's measured here are
//! the pure functions that run on every tile request after the network reply
//! arrives: bbox merging, MIME type checks, URL classification, and the
//! tile→WGS-84 coordinate transform that builds the bbox query.
//!
//! Run with: `cargo bench --bench stac --features stac`

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tileserver_rs::sources::stac::{
    StacAsset, build_search_url, compute_merged_bounds, is_cog_mime_type, is_stac_api_url,
    tile_to_wgs84_bbox,
};

fn make_asset(id: u32, west: f64, south: f64) -> StacAsset {
    StacAsset {
        id: format!("asset_{id}"),
        href: format!("https://example.com/cog/{id}.tif"),
        bbox: [west, south, west + 1.0, south + 1.0],
        title: format!("Asset {id}"),
        datetime: Some("2026-04-01T00:00:00Z".into()),
        cloud_cover: Some(5.0),
    }
}

fn bench_compute_merged_bounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("stac_compute_merged_bounds");

    for &n in &[1_usize, 5, 25, 100] {
        let assets: Vec<_> = (0..n)
            .map(|i| make_asset(i as u32, (i as f64) * 0.1, (i as f64) * 0.1))
            .collect();
        group.bench_function(format!("{n}_assets"), |b| {
            b.iter(|| black_box(compute_merged_bounds(black_box(&assets))));
        });
    }
    group.finish();
}

fn bench_is_cog_mime_type(c: &mut Criterion) {
    c.bench_function("stac_is_cog_mime_type", |b| {
        b.iter(|| {
            black_box(is_cog_mime_type(black_box(
                "image/tiff; application=geotiff",
            )));
            black_box(is_cog_mime_type(black_box(
                "image/tiff; application=geotiff; profile=cloud-optimized",
            )));
            black_box(is_cog_mime_type(black_box("image/png")));
        });
    });
}

fn bench_is_stac_api_url(c: &mut Criterion) {
    c.bench_function("stac_is_stac_api_url", |b| {
        b.iter(|| {
            black_box(is_stac_api_url(black_box(
                "https://earth-search.aws.element84.com/v1",
            )));
            black_box(is_stac_api_url(black_box("/local/path/catalog.json")));
            black_box(is_stac_api_url(black_box("s3://bucket/catalog.json")));
        });
    });
}

fn bench_tile_to_wgs84_bbox(c: &mut Criterion) {
    let mut group = c.benchmark_group("stac_tile_to_wgs84_bbox");
    for &(z, x, y) in &[
        (0_u8, 0_u32, 0_u32),
        (8, 73, 90),
        (14, 8589, 5712),
        (18, 137440, 91392),
    ] {
        group.bench_function(format!("z{z}"), |b| {
            b.iter(|| black_box(tile_to_wgs84_bbox(black_box(z), black_box(x), black_box(y))));
        });
    }
    group.finish();
}

fn bench_build_search_url(c: &mut Criterion) {
    c.bench_function("stac_build_search_url", |b| {
        b.iter(|| {
            black_box(build_search_url(black_box(
                "https://earth-search.aws.element84.com/v1",
            )));
        });
    });
}

criterion_group!(
    benches,
    bench_compute_merged_bounds,
    bench_is_cog_mime_type,
    bench_is_stac_api_url,
    bench_tile_to_wgs84_bbox,
    bench_build_search_url,
);
criterion_main!(benches);
