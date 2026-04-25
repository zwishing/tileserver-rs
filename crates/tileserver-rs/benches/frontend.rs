//! Benchmarks for the frontend asset-serving hot path.
//!
//! Embedded asset reads via `rust_embed` are constant-time hashtable
//! lookups (~10 ns) and live in the binary, not the library — so they
//! aren't reachable from a bench harness. What's measured here is the
//! pure pre-flight logic that runs on every request to a static asset:
//! MIME-type detection from path extension and the cache-control rule
//! that gives `_nuxt/` chunks the year-long immutable cache header.
//!
//! Run with: `cargo bench --bench frontend`
//! (No `--features` flag — `mime_guess` is unconditionally available.)

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_mime_guess(c: &mut Criterion) {
    let mut group = c.benchmark_group("frontend_mime_guess");

    for (label, path) in [
        ("html", "index.html"),
        ("js_chunk", "_nuxt/entry.dPtTaXJG.js"),
        ("css_chunk", "_nuxt/entry.B8gQ_qNJ.css"),
        ("woff2_font", "_nuxt/Inter-Regular.B5ZdjQ8u.woff2"),
        ("png_icon", "favicon.png"),
        ("svg", "logo.svg"),
        ("json_manifest", "manifest.json"),
        ("unknown", "no-extension-file"),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| {
                black_box(
                    mime_guess::from_path(black_box(path))
                        .first_or_octet_stream()
                        .to_string(),
                )
            });
        });
    }
    group.finish();
}

fn is_immutable_chunk(path: &str) -> bool {
    path.starts_with("_nuxt/")
}

fn bench_cache_rule(c: &mut Criterion) {
    c.bench_function("frontend_cache_rule", |b| {
        b.iter(|| {
            black_box(is_immutable_chunk(black_box("_nuxt/entry.dPtTaXJG.js")));
            black_box(is_immutable_chunk(black_box("index.html")));
            black_box(is_immutable_chunk(black_box("favicon.png")));
            black_box(is_immutable_chunk(black_box("_nuxt/builds/v3.0.json")));
        });
    });
}

criterion_group!(benches, bench_mime_guess, bench_cache_rule);
criterion_main!(benches);
