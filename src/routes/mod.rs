//! Route handlers for all HTTP endpoints.
//!
//! This module assembles the API router and delegates to
//! domain-specific sub-modules for each endpoint group.

mod data;
mod files;
mod fonts;
mod render;
mod spatial;
mod styles;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};

use crate::admin;
use crate::reload::SharedState;
use crate::sources::TileJson;
use crate::upload;

/// TileJSON response for raster style tiles
#[derive(serde::Serialize)]
pub(crate) struct RasterTileJson {
    pub(crate) tilejson: &'static str,
    pub(crate) name: String,
    pub(crate) tiles: Vec<String>,
    pub(crate) minzoom: u8,
    pub(crate) maxzoom: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) attribution: Option<String>,
}

/// Combined index entry for /index.json
#[derive(serde::Serialize)]
#[serde(untagged)]
enum IndexEntry {
    Data(Box<TileJson>),
    Style(RasterTileJson),
}

#[derive(Debug, serde::Deserialize, Default)]
struct IndexQueryParams {
    key: Option<String>,
}

pub fn api_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ping", get(admin::ping_check))
        .route("/index.json", get(get_index_json))
        // Style endpoints
        .route("/styles.json", get(styles::get_all_styles))
        .route("/styles/{style_json}", get(styles::get_style_tilejson))
        .route("/styles/{style}/style.json", get(styles::get_style_json))
        .route(
            "/styles/{style}/wmts.xml",
            get(styles::get_wmts_capabilities),
        )
        .route("/styles/{style}/{sprite_file}", get(styles::get_sprite))
        .route(
            "/styles/{style}/{z}/{x}/{y_fmt}",
            get(render::get_raster_tile),
        )
        .route(
            "/styles/{style}/{tile_size}/{z}/{x}/{y_fmt}",
            get(render::get_raster_tile_with_size),
        )
        .route(
            "/styles/{style}/static/{static_type}/{size_fmt}",
            get(render::get_static_image),
        )
        // Font endpoints
        .route("/fonts.json", get(fonts::get_fonts_list))
        .route("/fonts/{fontstack}/{range}", get(fonts::get_font_glyphs))
        // Data endpoints
        .route("/data.json", get(data::get_all_sources))
        .route("/data/{source}", get(data::get_source_tilejson))
        .route("/data/{source}/{z}/{x}/{y_fmt}", get(data::get_tile))
        // Static files endpoint
        .route("/files/{*filepath}", get(files::get_static_file))
        // Upload endpoints (streaming, large body limit)
        .route(
            "/api/upload",
            post(upload::upload_file)
                .get(upload::list_uploads)
                .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/api/upload/{id}", delete(upload::delete_upload))
        // Spatial API endpoints (for LLM tool integration)
        .route(
            "/api/spatial/schema/{source}",
            get(spatial::get_spatial_schema),
        )
        .route(
            "/api/spatial/stats/{source}",
            get(spatial::get_spatial_stats),
        )
        .route("/api/spatial/query", post(spatial::post_spatial_query))
        .with_state(state)
}

async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

/// Get combined TileJSON array for all data sources and styles
async fn get_index_json(
    State(shared): State<SharedState>,
    Query(query): Query<IndexQueryParams>,
) -> Json<Vec<IndexEntry>> {
    let state = shared.load();
    let mut entries = Vec::with_capacity(state.sources.len() + state.styles.len());

    let key_query = query
        .key
        .as_ref()
        .map(|k| format!("?key={}", urlencoding::encode(k)))
        .unwrap_or_default();

    for metadata in state.sources.all_metadata() {
        entries.push(IndexEntry::Data(Box::new(
            metadata.to_tilejson_with_key(&state.base_url, query.key.as_deref()),
        )));
    }

    for style in state.styles.all() {
        let tile_url = format!(
            "{}/styles/{}/{{z}}/{{x}}/{{y}}.png{}",
            state.base_url, style.id, key_query
        );
        entries.push(IndexEntry::Style(RasterTileJson {
            tilejson: "3.0.0",
            name: style.name.clone(),
            tiles: vec![tile_url],
            minzoom: 0,
            maxzoom: 22,
            attribution: None,
        }));
    }

    Json(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_index_entry_data_serialization() {
        let tilejson = TileJson {
            tilejson: "3.0.0".to_string(),
            id: "test-source".to_string(),
            tiles: vec!["http://localhost:8080/data/test/{z}/{x}/{y}.pbf".to_string()],
            name: "Test Source".to_string(),
            description: None,
            minzoom: 0,
            maxzoom: 14,
            bounds: None,
            center: None,
            vector_layers: None,
            attribution: None,
            encoding: None,
        };
        let entry = IndexEntry::Data(Box::new(tilejson));
        let json = serde_json::to_value(&entry).unwrap();

        assert_eq!(json["id"], "test-source");
        assert_eq!(json["tilejson"], "3.0.0");
        assert_eq!(json["minzoom"], 0);
        assert_eq!(json["maxzoom"], 14);
    }

    #[test]
    fn test_index_entry_style_serialization() {
        let style_entry = IndexEntry::Style(RasterTileJson {
            tilejson: "3.0.0",
            name: "osm-bright".to_string(),
            tiles: vec!["http://localhost:8080/styles/osm-bright/{z}/{x}/{y}.png".to_string()],
            minzoom: 0,
            maxzoom: 22,
            attribution: None,
        });
        let json = serde_json::to_value(&style_entry).unwrap();

        assert_eq!(json["name"], "osm-bright");
        assert_eq!(json["tilejson"], "3.0.0");
        assert_eq!(json["minzoom"], 0);
    }
}
