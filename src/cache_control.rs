//! HTTP cache-control header helpers for tile and static asset responses.

use axum::http::HeaderValue;

/// Set cache headers for tile responses
#[must_use]
pub fn tile_cache_headers() -> HeaderValue {
    HeaderValue::from_static("public, max-age=86400, stale-while-revalidate=604800")
}
