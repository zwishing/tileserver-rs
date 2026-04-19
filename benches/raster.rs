//! Raster fast-path benchmarks.
//!
//! Measures the four performance surfaces exposed by the
//! /raster pipeline commits #1-#7:
//!
//! 1. `RasterImage` round-trip encode (PNG / JPEG / WebP) so we can
//!    spot regressions in the codec boundary which every raster tile
//!    hits once per response.
//! 2. Mosaic reductions — `first`, `highest`, `mean`, `median` —
//!    each fed a 5-layer, 256×256 stack matching a typical Element84
//!    Sentinel-2 cold tile.
//! 3. Band-math NDVI evaluation over a 256×256 two-band raster, the
//!    query-param path added in commit #6.
//! 4. PNG decode -> RasterImage conversion, the inverse boundary
//!    exercised when the HTTP handler ingests a cached tile for
//!    band-math post-processing.
//!
//! The benches are criterion-based so they plot nicely in
//! benches/target/criterion/report/index.html.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ndarray::{Array2, Array3};
use std::hint::black_box;

use tileserver_rs::config::PixelSelectionMethod;
use tileserver_rs::raster::{
    RasterImage, decode, encode,
    expression::{self, ParsedExpression},
    mosaic,
};

const TILE_HEIGHT: usize = 256;
const TILE_WIDTH: usize = 256;

fn solid_rgba(band_count: usize, seed: u8) -> RasterImage {
    let mut data = Array3::<f32>::zeros((band_count, TILE_HEIGHT, TILE_WIDTH));
    for b in 0..band_count {
        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                data[[b, y, x]] = f32::from(seed.wrapping_add(b as u8).wrapping_add(x as u8));
            }
        }
    }
    RasterImage::from_opaque(data, None)
}

fn half_masked(band_count: usize) -> RasterImage {
    let data = Array3::<f32>::from_elem((band_count, TILE_HEIGHT, TILE_WIDTH), 128.0);
    let mut mask = Array2::from_elem((TILE_HEIGHT, TILE_WIDTH), false);
    for y in 0..TILE_HEIGHT {
        for x in (TILE_WIDTH / 2)..TILE_WIDTH {
            mask[[y, x]] = true;
        }
    }
    RasterImage::new(data, mask, None)
}

fn bench_encode(c: &mut Criterion) {
    let img = solid_rgba(4, 42);

    let mut group = c.benchmark_group("raster_encode_256");
    group.bench_function("png", |b| {
        b.iter(|| black_box(encode::to_png(black_box(&img)).unwrap()));
    });
    group.bench_function("jpeg_q85", |b| {
        b.iter(|| black_box(encode::to_jpeg(black_box(&img), 85).unwrap()));
    });
    group.bench_function("webp_lossless", |b| {
        b.iter(|| black_box(encode::to_webp(black_box(&img)).unwrap()));
    });
    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let img = solid_rgba(4, 42);
    let png = encode::to_png(&img).unwrap();

    c.bench_function("raster_decode_png_256", |b| {
        b.iter(|| black_box(decode::from_bytes(black_box(&png)).unwrap()));
    });
}

fn bench_mosaic(c: &mut Criterion) {
    let mut group = c.benchmark_group("raster_mosaic_5_assets_256");

    let methods = [
        ("first", PixelSelectionMethod::First),
        ("highest", PixelSelectionMethod::Highest),
        ("lowest", PixelSelectionMethod::Lowest),
        ("mean", PixelSelectionMethod::Mean),
        ("median", PixelSelectionMethod::Median),
        ("stdev", PixelSelectionMethod::Stdev),
        ("count", PixelSelectionMethod::Count),
    ];

    for (name, variant) in methods {
        group.bench_with_input(BenchmarkId::new("method", name), &variant, |b, v| {
            b.iter_batched(
                || {
                    (0..5_u8)
                        .map(|i| {
                            if i == 0 {
                                half_masked(3)
                            } else {
                                solid_rgba(3, i.wrapping_mul(37))
                            }
                        })
                        .collect::<Vec<_>>()
                },
                |layers| {
                    let mut m = mosaic::build(*v);
                    for layer in layers {
                        m.feed(black_box(layer));
                        if m.is_done() {
                            break;
                        }
                    }
                    black_box(m.finalize())
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_band_math(c: &mut Criterion) {
    let img = solid_rgba(2, 42);
    let expr = ParsedExpression::parse("(b2 - b1) / (b2 + b1)", 2).unwrap();

    c.bench_function("raster_band_math_ndvi_256", |b| {
        b.iter(|| black_box(expression::apply(black_box(&expr), black_box(&img)).unwrap()));
    });
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_mosaic,
    bench_band_math
);
criterion_main!(benches);
