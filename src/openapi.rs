//! OpenAPI 3.1 specification for tileserver-rs API
//!
//! This module provides the OpenAPI specification using utoipa derive macros
//! for seamless integration with utoipa-swagger-ui.
//!
//! The structs and functions in this module are used solely for documentation
//! generation and are not called directly at runtime.

#![allow(dead_code)]

use utoipa::OpenApi;

/// OpenAPI documentation for tileserver-rs
#[derive(OpenApi)]
#[openapi(
    info(
        title = "tileserver-rs API",
        description = "High-performance vector and raster tile server with native MapLibre rendering",
        version = "2.8.0",
        license(name = "MIT", url = "https://github.com/vinayakkulkarni/tileserver-rs/blob/main/LICENSE"),
        contact(name = "Vinayak Kulkarni", url = "https://github.com/vinayakkulkarni/tileserver-rs")
    ),
    tags(
        (name = "Health", description = "Health check and runtime metadata endpoints"),
        (name = "Data", description = "Tile data sources: vector (PMTiles, MBTiles, PostgreSQL), raster (COG), and OutDB raster (PostGIS)"),
        (name = "Styles", description = "Map styles and raster tile rendering"),
        (name = "Fonts", description = "Font glyphs for map labels"),
        (name = "Files", description = "Static file serving"),
        (name = "Upload", description = "File upload for drag-and-drop visualization"),
        (name = "Spatial", description = "Spatial queries and source schema inspection for LLM tool integration")
    ),
    paths(
        health_check,
        ping_check,
        get_index,
        list_data_sources,
        get_data_source,
        get_tile,
        list_styles,
        get_style_tilejson,
        get_style_json,
        get_raster_tile,
        get_raster_tile_with_size,
        get_static_image,
        get_sprite,
        get_wmts_capabilities,
        list_fonts,
        get_font_glyphs,
        get_static_file,
        upload_file,
        list_uploads,
        delete_upload,
        get_spatial_schema,
        get_spatial_stats,
        post_spatial_query,
    ),
    components(schemas(
        TileJSON,
        VectorLayer,
        StyleInfo,
        GeoJSON,
        ApiError,
        PingResponse,
        UploadResponse,
        UploadInfo,
        SpatialSchemaResponse,
        SpatialStatsResponse,
        SpatialQueryRequest,
        SpatialQueryResponse,
    ))
)]
pub struct ApiDoc;

// ============================================================
// Schema definitions
// ============================================================

/// TileJSON 3.0 metadata
#[derive(utoipa::ToSchema)]
#[schema(example = json!({
    "tilejson": "3.0.0",
    "tiles": ["http://localhost:8080/data/source/{z}/{x}/{y}.pbf"],
    "name": "OpenMapTiles",
    "minzoom": 0,
    "maxzoom": 14
}))]
pub struct TileJSON {
    /// TileJSON version (always "3.0.0")
    pub tilejson: String,
    /// Source identifier
    #[schema(nullable)]
    pub id: Option<String>,
    /// Human-readable name
    #[schema(nullable)]
    pub name: Option<String>,
    /// Description of the tileset
    #[schema(nullable)]
    pub description: Option<String>,
    /// Tile URL templates
    pub tiles: Vec<String>,
    /// Minimum zoom level
    #[schema(minimum = 0, maximum = 22)]
    pub minzoom: u8,
    /// Maximum zoom level
    #[schema(minimum = 0, maximum = 22)]
    pub maxzoom: u8,
    /// Bounding box [west, south, east, north]
    #[schema(nullable)]
    pub bounds: Option<Vec<f64>>,
    /// Center point [longitude, latitude, zoom]
    #[schema(nullable)]
    pub center: Option<Vec<f64>>,
    /// Attribution HTML
    #[schema(nullable)]
    pub attribution: Option<String>,
    /// Vector layer definitions
    #[schema(nullable)]
    pub vector_layers: Option<Vec<VectorLayer>>,
}

/// Vector layer metadata
#[derive(utoipa::ToSchema)]
pub struct VectorLayer {
    /// Layer identifier
    pub id: String,
    /// Layer description
    #[schema(nullable)]
    pub description: Option<String>,
    /// Minimum zoom level
    #[schema(nullable)]
    pub minzoom: Option<u8>,
    /// Maximum zoom level
    #[schema(nullable)]
    pub maxzoom: Option<u8>,
    /// Field names and types
    #[schema(nullable)]
    pub fields: Option<std::collections::HashMap<String, String>>,
}

/// Style metadata
#[derive(utoipa::ToSchema)]
#[schema(example = json!({
    "id": "osm-bright",
    "name": "OSM Bright",
    "url": "http://localhost:8080/styles/osm-bright/style.json"
}))]
pub struct StyleInfo {
    /// Style identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// URL to style.json
    pub url: String,
}

/// GeoJSON FeatureCollection
#[derive(utoipa::ToSchema)]
pub struct GeoJSON {
    /// Always "FeatureCollection"
    #[schema(example = "FeatureCollection")]
    pub r#type: String,
    /// Array of GeoJSON features
    pub features: Vec<serde_json::Value>,
}

/// API error response
#[derive(utoipa::ToSchema)]
#[schema(example = json!({"error": "Source not found: invalid-source"}))]
pub struct ApiError {
    /// Error message
    pub error: String,
}

/// Runtime metadata from /ping
#[derive(utoipa::ToSchema)]
#[schema(example = json!({
    "status": "ok",
    "config_hash": "a1b2c3d4e5f6...",
    "loaded_at_unix": 1700000000,
    "loaded_sources": 3,
    "loaded_styles": 2,
    "renderer_enabled": true,
    "version": "2.8.0"
}))]
pub struct PingResponse {
    pub status: String,
    pub config_hash: String,
    pub loaded_at_unix: u64,
    pub loaded_sources: usize,
    pub loaded_styles: usize,
    pub renderer_enabled: bool,
    pub version: String,
}

/// Upload response
#[derive(utoipa::ToSchema)]
#[schema(example = json!({
    "id": "a1b2c3d4-e5f6-...",
    "source_id": "upload-a1b2c3d4-e5f6-...",
    "file_name": "zurich.mbtiles",
    "format": "mbtiles",
    "tilejson_url": "http://localhost:8080/data/upload-a1b2c3d4-e5f6-....json"
}))]
pub struct UploadResponse {
    /// Unique upload ID (UUID)
    pub id: String,
    /// Source ID for accessing tiles
    pub source_id: String,
    /// Original file name
    pub file_name: String,
    /// Detected format (mbtiles, cog)
    pub format: String,
    /// URL to the TileJSON endpoint for this source
    pub tilejson_url: String,
}

/// Upload tracking info
#[derive(utoipa::ToSchema)]
pub struct UploadInfo {
    /// Upload ID (UUID)
    pub id: String,
    /// Original file name
    pub file_name: String,
    /// Detected format
    pub format: String,
}

/// Spatial schema response
#[derive(utoipa::ToSchema)]
pub struct SpatialSchemaResponse {
    /// Source ID
    pub source: String,
    /// Tile format (pbf, mlt, png, etc.)
    pub format: String,
    /// Minimum zoom level
    pub minzoom: u8,
    /// Maximum zoom level
    pub maxzoom: u8,
    /// Bounding box [west, south, east, north]
    #[schema(nullable)]
    pub bounds: Option<Vec<f64>>,
    /// Vector layer information
    pub layers: Vec<serde_json::Value>,
}

/// Spatial stats response
#[derive(utoipa::ToSchema)]
pub struct SpatialStatsResponse {
    /// Source ID
    pub source: String,
    /// Tile format
    pub format: String,
    /// Minimum zoom level
    pub minzoom: u8,
    /// Maximum zoom level
    pub maxzoom: u8,
    /// Bounding box
    #[schema(nullable)]
    pub bounds: Option<Vec<f64>>,
    /// Center point [lon, lat, zoom]
    #[schema(nullable)]
    pub center: Option<Vec<f64>>,
    /// Number of vector layers
    pub layer_count: usize,
    /// Source name
    #[schema(nullable)]
    pub name: Option<String>,
    /// Source description
    #[schema(nullable)]
    pub description: Option<String>,
    /// Attribution
    #[schema(nullable)]
    pub attribution: Option<String>,
}

/// Spatial query request
#[derive(utoipa::ToSchema)]
#[schema(example = json!({
    "source": "openmaptiles",
    "bbox": [-122.5, 37.7, -122.3, 37.9],
    "zoom": 14,
    "limit": 100
}))]
pub struct SpatialQueryRequest {
    /// Source ID to query
    pub source: String,
    /// Bounding box [west, south, east, north]
    #[schema(nullable)]
    pub bbox: Option<Vec<f64>>,
    /// Zoom level for tile resolution
    pub zoom: u8,
    /// Optional layer filter
    #[schema(nullable)]
    pub layers: Option<Vec<String>>,
    /// Maximum features to return
    pub limit: usize,
}

/// Spatial query response
#[derive(utoipa::ToSchema)]
pub struct SpatialQueryResponse {
    /// Source that was queried
    pub source: String,
    /// Matching features
    pub features: Vec<serde_json::Value>,
    /// Total features returned
    pub total: usize,
    /// Whether results were truncated by limit
    pub truncated: bool,
}

// ============================================================
// Path operations (documentation only - actual handlers in main.rs)
// ============================================================

/// Health check
///
/// Returns OK if the server is running
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Server is healthy", body = String, example = "OK")
    )
)]
pub async fn health_check() {}

/// Runtime metadata
///
/// Returns runtime metadata including config hash, loaded sources/styles count,
/// renderer status, and server version. Useful for monitoring and automation.
#[utoipa::path(
    get,
    path = "/ping",
    tag = "Health",
    responses(
        (status = 200, description = "Runtime metadata", body = PingResponse)
    )
)]
pub async fn ping_check() {}

/// Get all sources and styles
///
/// Returns a combined list of all data sources and styles as TileJSON.
/// The optional `key` parameter is appended to all tile URLs in the response.
#[utoipa::path(
    get,
    path = "/index.json",
    tag = "Data",
    params(
        ("key" = Option<String>, Query, description = "API key to include in all tile URLs")
    ),
    responses(
        (status = 200, description = "Combined list of sources and styles", body = Vec<TileJSON>)
    )
)]
pub async fn get_index() {}

/// List all data sources
///
/// Returns TileJSON metadata for all available tile sources.
/// The optional `key` parameter is appended to all tile URLs in the response.
#[utoipa::path(
    get,
    path = "/data.json",
    tag = "Data",
    params(
        ("key" = Option<String>, Query, description = "API key to include in tile URLs")
    ),
    responses(
        (status = 200, description = "List of data sources", body = Vec<TileJSON>)
    )
)]
pub async fn list_data_sources() {}

/// Get data source TileJSON
///
/// Returns TileJSON metadata for a specific tile source.
/// The optional `key` parameter is appended to all tile URLs in the response.
#[utoipa::path(
    get,
    path = "/data/{source}",
    tag = "Data",
    params(
        ("source" = String, Path, description = "Source ID (with or without .json extension)"),
        ("key" = Option<String>, Query, description = "API key to include in tile URLs")
    ),
    responses(
        (status = 200, description = "TileJSON metadata", body = TileJSON),
        (status = 404, description = "Source not found", body = ApiError)
    )
)]
pub async fn get_data_source() {}

/// Get a tile from any data source
///
/// Returns a tile from the specified source. The response format depends on the source type:
///
/// **Vector sources** (PMTiles, MBTiles, PostgreSQL functions):
/// - Formats: `pbf`, `mvt`, `geojson`
/// - Returns Mapbox Vector Tiles or GeoJSON
///
/// **Raster/COG sources** (Cloud Optimized GeoTIFF):
/// - Formats: `png`, `jpg`, `jpeg`, `webp`
/// - Query param `resampling`: nearest, bilinear, cubic, lanczos, average, etc.
///
/// **PostgreSQL Out-of-Database raster sources** (VRT/COG via PostGIS):
/// - Formats: `png`, `jpg`, `jpeg`, `webp`
/// - Query params are passed to the PostgreSQL function for dynamic filtering
/// - See: https://postgis.net/docs/using_raster_dataman.html#RT_Cloud_Rasters
#[utoipa::path(
    get,
    path = "/data/{source}/{z}/{x}/{y}.{format}",
    tag = "Data",
    params(
        ("source" = String, Path, description = "Source ID"),
        ("z" = u8, Path, description = "Zoom level (0-22)"),
        ("x" = u32, Path, description = "Tile X coordinate"),
        ("y" = u32, Path, description = "Tile Y coordinate"),
        ("format" = String, Path, description = "Tile format: pbf, mvt, geojson (vector) or png, jpg, webp (raster)"),
        ("resampling" = Option<String>, Query, description = "Resampling method for COG sources: nearest, bilinear, cubic, cubicspline, lanczos, average, mode, max, min, med, q1, q3")
    ),
    responses(
        (status = 200, description = "Vector tile data", content_type = "application/x-protobuf"),
        (status = 200, description = "GeoJSON tile data", body = GeoJSON, content_type = "application/geo+json"),
        (status = 200, description = "Raster tile image", content_type = "image/png"),
        (status = 404, description = "Tile or source not found")
    )
)]
pub async fn get_tile() {}

/// List all styles
///
/// Returns metadata for all available map styles.
/// The optional `key` parameter is appended to all style URLs in the response.
#[utoipa::path(
    get,
    path = "/styles.json",
    tag = "Styles",
    params(
        ("key" = Option<String>, Query, description = "API key to include in style URLs")
    ),
    responses(
        (status = 200, description = "List of styles", body = Vec<StyleInfo>)
    )
)]
pub async fn list_styles() {}

/// Get style TileJSON
///
/// Returns TileJSON for raster tiles rendered from this style.
/// The optional `key` parameter is appended to all tile URLs in the response.
#[utoipa::path(
    get,
    path = "/styles/{style}.json",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("key" = Option<String>, Query, description = "API key to include in tile URLs")
    ),
    responses(
        (status = 200, description = "TileJSON for raster tiles", body = TileJSON),
        (status = 404, description = "Style not found", body = ApiError)
    )
)]
pub async fn get_style_tilejson() {}

/// Get MapLibre style JSON
///
/// Returns the full MapLibre GL style specification.
/// The optional `key` parameter is appended to all URLs in the style (sources, glyphs, sprites).
#[utoipa::path(
    get,
    path = "/styles/{style}/style.json",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("key" = Option<String>, Query, description = "API key to include in all URLs within the style")
    ),
    responses(
        (status = 200, description = "MapLibre style specification", content_type = "application/json"),
        (status = 404, description = "Style not found", body = ApiError)
    )
)]
pub async fn get_style_json() {}

/// Get a raster tile
///
/// Returns a raster tile rendered from the style. Supports retina with @2x suffix.
#[utoipa::path(
    get,
    path = "/styles/{style}/{z}/{x}/{y}.{format}",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("z" = u8, Path, description = "Zoom level (0-22)"),
        ("x" = u32, Path, description = "Tile X coordinate"),
        ("y" = String, Path, description = "Tile Y coordinate (optionally with @2x for retina)", example = "123"),
        ("format" = String, Path, description = "Image format (png, jpg, jpeg, webp)")
    ),
    responses(
        (status = 200, description = "Raster tile image", content_type = "image/png"),
        (status = 404, description = "Style not found", body = ApiError)
    )
)]
pub async fn get_raster_tile() {}

/// Get a raster tile with custom size
///
/// Returns a raster tile with specified tile size (256 or 512 pixels)
#[utoipa::path(
    get,
    path = "/styles/{style}/{tileSize}/{z}/{x}/{y}.{format}",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("tileSize" = u16, Path, description = "Tile size in pixels (256 or 512)"),
        ("z" = u8, Path, description = "Zoom level"),
        ("x" = u32, Path, description = "Tile X coordinate"),
        ("y" = String, Path, description = "Tile Y coordinate"),
        ("format" = String, Path, description = "Image format (png, jpg, jpeg, webp)")
    ),
    responses(
        (status = 200, description = "Raster tile image", content_type = "image/png")
    )
)]
pub async fn get_raster_tile_with_size() {}

/// Get a static map image
///
/// Renders a static map image centered at the specified location
#[utoipa::path(
    get,
    path = "/styles/{style}/static/{center}/{size}.{format}",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("center" = String, Path, description = "Center point as lon,lat,zoom or 'auto'", example = "-122.4194,37.7749,12"),
        ("size" = String, Path, description = "Image size as WIDTHxHEIGHT, optionally with @2x", example = "800x600"),
        ("format" = String, Path, description = "Image format (png, jpg, jpeg, webp)"),
        ("bearing" = Option<f64>, Query, description = "Map bearing in degrees"),
        ("pitch" = Option<f64>, Query, description = "Map pitch in degrees"),
        ("markers" = Option<String>, Query, description = "Markers to add (format: pin-s+color(lon,lat))"),
        ("path" = Option<String>, Query, description = "Path to draw (format: path-width+color(lon,lat|lon,lat))")
    ),
    responses(
        (status = 200, description = "Static map image", content_type = "image/png")
    )
)]
pub async fn get_static_image() {}

/// Get sprite image or JSON
///
/// Returns sprite image (PNG) or metadata (JSON) for the style
#[utoipa::path(
    get,
    path = "/styles/{style}/sprite.{ext}",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("ext" = String, Path, description = "File extension (png or json, optionally with @2x)", example = "png")
    ),
    responses(
        (status = 200, description = "Sprite image", content_type = "image/png"),
        (status = 200, description = "Sprite metadata", content_type = "application/json"),
        (status = 404, description = "Sprite not found", body = ApiError)
    )
)]
pub async fn get_sprite() {}

/// Get WMTS capabilities
///
/// Returns OGC WMTS GetCapabilities document for the style
#[utoipa::path(
    get,
    path = "/styles/{style}/wmts.xml",
    tag = "Styles",
    params(
        ("style" = String, Path, description = "Style ID"),
        ("key" = Option<String>, Query, description = "API key to include in all tile URLs")
    ),
    responses(
        (status = 200, description = "WMTS capabilities XML", content_type = "application/xml")
    )
)]
pub async fn get_wmts_capabilities() {}

/// List available fonts
///
/// Returns a list of available font families
#[utoipa::path(
    get,
    path = "/fonts.json",
    tag = "Fonts",
    responses(
        (status = 200, description = "List of font names", body = Vec<String>,
            example = json!(["Noto Sans Regular", "Noto Sans Bold"]))
    )
)]
pub async fn list_fonts() {}

/// Get font glyphs
///
/// Returns PBF-encoded font glyphs for a character range
#[utoipa::path(
    get,
    path = "/fonts/{fontstack}/{range}",
    tag = "Fonts",
    params(
        ("fontstack" = String, Path, description = "Font stack (comma-separated font names)", example = "Noto Sans Regular"),
        ("range" = String, Path, description = "Character range (e.g., 0-255.pbf)", example = "0-255.pbf")
    ),
    responses(
        (status = 200, description = "Font glyph data", content_type = "application/x-protobuf"),
        (status = 404, description = "Font not found", body = ApiError)
    )
)]
pub async fn get_font_glyphs() {}

/// Get static file
///
/// Serves static files from the configured files directory
#[utoipa::path(
    get,
    path = "/files/{filepath}",
    tag = "Files",
    params(
        ("filepath" = String, Path, description = "Path to the file")
    ),
    responses(
        (status = 200, description = "File content"),
        (status = 404, description = "File not found", body = ApiError)
    )
)]
pub async fn get_static_file() {}

/// Upload a geospatial file
///
/// Streams a file to disk and registers it as a temporary tile source.
/// Supports MBTiles, SQLite, and COG formats.
#[utoipa::path(
    post,
    path = "/api/upload",
    tag = "Upload",
    request_body(content_type = "multipart/form-data", description = "Geospatial file to upload"),
    responses(
        (status = 200, description = "File uploaded and source registered", body = UploadResponse),
        (status = 400, description = "Unsupported format or invalid file", body = ApiError),
        (status = 413, description = "File exceeds size limit")
    )
)]
pub async fn upload_file() {}

/// List uploaded sources
///
/// Returns all currently uploaded file sources
#[utoipa::path(
    get,
    path = "/api/upload",
    tag = "Upload",
    responses(
        (status = 200, description = "List of uploaded sources", body = Vec<UploadInfo>)
    )
)]
pub async fn list_uploads() {}

/// Delete an uploaded source
///
/// Removes an uploaded source and deletes the file from disk
#[utoipa::path(
    delete,
    path = "/api/upload/{id}",
    tag = "Upload",
    params(
        ("id" = String, Path, description = "Upload UUID or full source ID (upload-{uuid})")
    ),
    responses(
        (status = 204, description = "Upload deleted successfully"),
        (status = 404, description = "Upload not found", body = ApiError)
    )
)]
pub async fn delete_upload() {}

/// Get source schema
///
/// Returns vector layer info, field types, and metadata for a source.
/// Used by LLM tools for understanding tile source structure.
#[utoipa::path(
    get,
    path = "/api/spatial/schema/{source}",
    tag = "Spatial",
    params(
        ("source" = String, Path, description = "Source ID")
    ),
    responses(
        (status = 200, description = "Source schema", body = SpatialSchemaResponse),
        (status = 404, description = "Source not found", body = ApiError)
    )
)]
pub async fn get_spatial_schema() {}

/// Get source statistics
///
/// Returns bounds, zoom range, layer count, and other statistics for a source
#[utoipa::path(
    get,
    path = "/api/spatial/stats/{source}",
    tag = "Spatial",
    params(
        ("source" = String, Path, description = "Source ID")
    ),
    responses(
        (status = 200, description = "Source statistics", body = SpatialStatsResponse),
        (status = 404, description = "Source not found", body = ApiError)
    )
)]
pub async fn get_spatial_stats() {}

/// Query features from a source
///
/// Decodes vector tiles at the requested location and returns features as JSON.
/// Supports bounding box filtering, layer filtering, and result limits.
#[utoipa::path(
    post,
    path = "/api/spatial/query",
    tag = "Spatial",
    request_body = SpatialQueryRequest,
    responses(
        (status = 200, description = "Query results", body = SpatialQueryResponse),
        (status = 400, description = "Invalid request (non-vector source)", body = ApiError),
        (status = 404, description = "Source not found", body = ApiError)
    )
)]
pub async fn post_spatial_query() {}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn test_openapi_spec_generates() {
        let spec = ApiDoc::openapi();

        // Check basic structure
        assert_eq!(spec.info.title, "tileserver-rs API");
        // The version in the derive macro is a placeholder; main.rs overrides it
        // at runtime with CARGO_PKG_VERSION. Here we just check the placeholder exists.
        assert!(!spec.info.version.is_empty());

        // Check that paths exist
        let paths = spec.paths.paths;
        assert!(paths.contains_key("/health"));
        assert!(paths.contains_key("/data.json"));
        assert!(paths.contains_key("/styles.json"));
        assert!(paths.contains_key("/fonts.json"));
    }

    #[test]
    fn test_openapi_spec_has_all_endpoints() {
        let spec = ApiDoc::openapi();
        let paths = spec.paths.paths;

        // All expected endpoints
        let expected_paths = [
            "/health",
            "/ping",
            "/index.json",
            "/data.json",
            "/data/{source}",
            "/data/{source}/{z}/{x}/{y}.{format}",
            "/styles.json",
            "/styles/{style}.json",
            "/styles/{style}/style.json",
            "/styles/{style}/{z}/{x}/{y}.{format}",
            "/styles/{style}/{tileSize}/{z}/{x}/{y}.{format}",
            "/styles/{style}/static/{center}/{size}.{format}",
            "/styles/{style}/sprite.{ext}",
            "/styles/{style}/wmts.xml",
            "/fonts.json",
            "/fonts/{fontstack}/{range}",
            "/files/{filepath}",
            "/api/upload",
            "/api/upload/{id}",
            "/api/spatial/schema/{source}",
            "/api/spatial/stats/{source}",
            "/api/spatial/query",
            "/health",
            "/ping",
            "/index.json",
            "/data.json",
            "/data/{source}",
            "/data/{source}/{z}/{x}/{y}.{format}",
            "/styles.json",
            "/styles/{style}.json",
            "/styles/{style}/style.json",
            "/styles/{style}/{z}/{x}/{y}.{format}",
            "/styles/{style}/{tileSize}/{z}/{x}/{y}.{format}",
            "/styles/{style}/static/{center}/{size}.{format}",
            "/styles/{style}/sprite.{ext}",
            "/styles/{style}/wmts.xml",
            "/fonts.json",
            "/fonts/{fontstack}/{range}",
            "/files/{filepath}",
        ];

        for path in expected_paths {
            assert!(
                paths.contains_key(path),
                "Missing path in OpenAPI spec: {}",
                path
            );
        }
    }

    #[test]
    fn test_openapi_has_tags() {
        let spec = ApiDoc::openapi();
        assert!(spec.tags.is_some(), "Tags should be defined");
        assert_eq!(
            spec.tags.as_ref().unwrap().len(),
            7,
            "Should have 5 tags defined"
        );
    }

    #[test]
    fn test_openapi_has_schemas() {
        let spec = ApiDoc::openapi();
        let schemas = &spec.components.as_ref().unwrap().schemas;

        assert!(schemas.contains_key("TileJSON"));
        assert!(schemas.contains_key("StyleInfo"));
        assert!(schemas.contains_key("VectorLayer"));
        assert!(schemas.contains_key("GeoJSON"));
        assert!(schemas.contains_key("ApiError"));
        assert!(schemas.contains_key("PingResponse"));
        assert!(schemas.contains_key("UploadResponse"));
        assert!(schemas.contains_key("UploadInfo"));
        assert!(schemas.contains_key("SpatialSchemaResponse"));
        assert!(schemas.contains_key("SpatialStatsResponse"));
        assert!(schemas.contains_key("SpatialQueryRequest"));
        assert!(schemas.contains_key("SpatialQueryResponse"));
    }
}
