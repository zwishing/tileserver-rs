//! Data/tile route handlers.
//!
//! Endpoints for listing tile sources, fetching TileJSON metadata,
//! and serving individual vector tiles.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{
        HeaderMap, HeaderValue,
        header::{CACHE_CONTROL, CONTENT_ENCODING, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};

use crate::cache_control;
use crate::error::TileServerError;
use crate::reload::SharedState;
use crate::sources::{self, TileJson};

#[cfg(feature = "raster")]
use crate::config;

/// Query parameters for data source endpoints
#[derive(Debug, serde::Deserialize, Default)]
pub(crate) struct DataSourceQueryParams {
    /// API key to append to tile URLs
    key: Option<String>,
}

/// Tile request parameters (raw from URL)
#[derive(serde::Deserialize)]
pub(crate) struct TileParams {
    source: String,
    z: u8,
    x: u32,
    y_fmt: String, // e.g., "123.pbf" or "123.mvt"
}

impl TileParams {
    fn parse_y_and_format(&self) -> Option<(u32, &str)> {
        let (y_str, format) = self.y_fmt.rsplit_once('.')?;
        let y = y_str.parse().ok()?;
        Some((y, format))
    }
}

/// Get all available tile sources
/// Route: GET /data.json
/// Query parameters:
/// - `key`: Optional API key to append to tile URLs
pub(crate) async fn get_all_sources(
    State(shared): State<SharedState>,
    Query(query): Query<DataSourceQueryParams>,
) -> Json<Vec<TileJson>> {
    let state = shared.load();
    let sources: Vec<TileJson> = state
        .sources
        .all_metadata()
        .iter()
        .map(|m| m.to_tilejson_with_key(&state.base_url, query.key.as_deref()))
        .collect();

    Json(sources)
}

/// Get TileJSON for a specific source
/// Route: GET /data/{source}
/// Query parameters:
/// - `key`: Optional API key to append to tile URLs
pub(crate) async fn get_source_tilejson(
    State(shared): State<SharedState>,
    Path(source): Path<String>,
    Query(query): Query<DataSourceQueryParams>,
) -> Result<Json<TileJson>, TileServerError> {
    let state = shared.load();

    // Strip .json extension if present
    let source_id = source.strip_suffix(".json").unwrap_or(&source);

    let source_ref = state
        .sources
        .get(source_id)
        .ok_or_else(|| TileServerError::SourceNotFound(source_id.to_string()))?;

    let tilejson = source_ref
        .metadata()
        .to_tilejson_with_key(&state.base_url, query.key.as_deref());
    Ok(Json(tilejson))
}

pub(crate) async fn get_tile(
    State(shared): State<SharedState>,
    Path(params): Path<TileParams>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, TileServerError> {
    let state = shared.load();

    let (y, format) = params
        .parse_y_and_format()
        .ok_or(TileServerError::InvalidTileRequest)?;

    if format == "geojson" {
        return get_tile_as_geojson(&state, &params.source, params.z, params.x, y).await;
    }

    #[cfg(feature = "raster")]
    let tile = {
        #[cfg(feature = "postgres")]
        if state.sources.is_postgres_function_source(&params.source) {
            let query_params = serde_json::to_value(&query).unwrap_or_default();
            state
                .sources
                .get_vector_tile_with_query_params(
                    &params.source,
                    params.z,
                    params.x,
                    y,
                    &query_params,
                )
                .await?
                .ok_or(TileServerError::TileNotFound {
                    z: params.z,
                    x: params.x,
                    y,
                })?
        } else {
            let resampling = query
                .get("resampling")
                .and_then(|s| s.parse::<config::ResamplingMethod>().ok());

            #[cfg(all(feature = "postgres", feature = "raster"))]
            let query_params = if state.sources.is_outdb_raster_source(&params.source) {
                Some(serde_json::to_value(&query).unwrap_or_default())
            } else {
                None
            };

            #[cfg(not(all(feature = "postgres", feature = "raster")))]
            let query_params: Option<serde_json::Value> = None;

            state
                .sources
                .get_raster_tile_with_params(
                    &params.source,
                    params.z,
                    params.x,
                    y,
                    256,
                    resampling,
                    query_params,
                )
                .await?
                .ok_or(TileServerError::TileNotFound {
                    z: params.z,
                    x: params.x,
                    y,
                })?
        }

        #[cfg(not(feature = "postgres"))]
        {
            let resampling = query
                .get("resampling")
                .and_then(|s| s.parse::<config::ResamplingMethod>().ok());

            state
                .sources
                .get_raster_tile_with_params(
                    &params.source,
                    params.z,
                    params.x,
                    y,
                    256,
                    resampling,
                    None,
                )
                .await?
                .ok_or(TileServerError::TileNotFound {
                    z: params.z,
                    x: params.x,
                    y,
                })?
        }
    };

    #[cfg(not(feature = "raster"))]
    let tile = {
        #[cfg(feature = "postgres")]
        let tile = if state.sources.is_postgres_function_source(&params.source) {
            let query_params: serde_json::Value = serde_json::to_value(&query).unwrap_or_default();
            state
                .sources
                .get_vector_tile_with_query_params(
                    &params.source,
                    params.z,
                    params.x,
                    y,
                    &query_params,
                )
                .await?
                .ok_or(TileServerError::TileNotFound {
                    z: params.z,
                    x: params.x,
                    y,
                })?
        } else {
            let source = state
                .sources
                .get(&params.source)
                .ok_or_else(|| TileServerError::SourceNotFound(params.source.clone()))?;
            source
                .get_tile(params.z, params.x, y)
                .await?
                .ok_or(TileServerError::TileNotFound {
                    z: params.z,
                    x: params.x,
                    y,
                })?
        };

        #[cfg(not(feature = "postgres"))]
        let tile = {
            let _ = query;
            let source = state
                .sources
                .get(&params.source)
                .ok_or_else(|| TileServerError::SourceNotFound(params.source.clone()))?;

            source
                .get_tile(params.z, params.x, y)
                .await?
                .ok_or(TileServerError::TileNotFound {
                    z: params.z,
                    x: params.x,
                    y,
                })?
        };

        tile
    };

    // MLT transcoding: if the requested format differs from the source format
    // and the `mlt` feature is enabled, attempt on-the-fly transcoding.
    #[cfg(feature = "mlt")]
    let tile = {
        let requested_format = format.parse::<crate::sources::TileFormat>().ok();
        if let Some(target) = requested_format {
            if tile.format != target && tile.format.is_vector() && target.is_vector() {
                match crate::transcode::transcode_tile(&tile, target) {
                    Ok(transcoded) => transcoded,
                    Err(e) => {
                        // Fall back to serving the original tile on any transcode
                        // error. This ensures clients always get usable data even
                        // when mlt-core cannot handle certain geometries or edge
                        // cases. The original tile (MVT or MLT) is still valid —
                        // only the on-the-fly format conversion failed.
                        tracing::warn!(
                            "transcoding {:?} -> {:?} failed for {}/{}/{}/{}, serving original tile: {}",
                            tile.format,
                            target,
                            params.source,
                            params.z,
                            params.x,
                            y,
                            e
                        );
                        tile
                    }
                }
            } else {
                tile
            }
        } else {
            tile
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static(tile.format.content_type()),
    );
    headers.insert(CACHE_CONTROL, cache_control::tile_cache_headers());

    if let Some(encoding) = tile.compression.content_encoding() {
        headers.insert(CONTENT_ENCODING, HeaderValue::from_static(encoding));
    }

    Ok((headers, tile.data).into_response())
}

/// Get a tile as GeoJSON (helper function)
async fn get_tile_as_geojson(
    state: &crate::reload::AppState,
    source_id: &str,
    z: u8,
    x: u32,
    y: u32,
) -> Result<Response, TileServerError> {
    use flate2::read::GzDecoder;
    use geozero::ProcessToJson;
    use geozero::mvt::{Message, Tile};
    use sources::TileCompression;
    use std::io::Read;

    let source = state
        .sources
        .get(source_id)
        .ok_or_else(|| TileServerError::SourceNotFound(source_id.to_string()))?;

    // Check if source is vector format
    if source.metadata().format != sources::TileFormat::Pbf {
        return Err(TileServerError::RenderError(
            "GeoJSON conversion only supported for vector tiles (PBF)".to_string(),
        ));
    }

    let tile = source
        .get_tile(z, x, y)
        .await?
        .ok_or(TileServerError::TileNotFound { z, x, y })?;

    // Decompress if needed
    let raw_data = match tile.compression {
        TileCompression::Gzip => {
            let mut decoder = GzDecoder::new(&tile.data[..]);
            let mut decompressed = Vec::with_capacity(tile.data.len() * 4);
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                TileServerError::RenderError(format!("Failed to decompress tile: {}", e))
            })?;
            decompressed
        }
        TileCompression::None => tile.data.to_vec(),
        _ => {
            return Err(TileServerError::RenderError(format!(
                "Unsupported compression: {:?}",
                tile.compression
            )));
        }
    };

    // Parse MVT tile using prost
    let mvt_tile = Tile::decode(raw_data.as_slice())
        .map_err(|e| TileServerError::RenderError(format!("Failed to decode MVT tile: {}", e)))?;

    // Convert each layer to GeoJSON and combine into a FeatureCollection
    let mut all_features: Vec<serde_json::Value> = Vec::with_capacity(mvt_tile.layers.len() * 64);

    for mut layer in mvt_tile.layers {
        // Each layer implements GeozeroDatasource which can convert to JSON
        if let Ok(layer_json) = layer.to_json() {
            // Parse the layer GeoJSON (it's a FeatureCollection)
            if let Ok(fc) = serde_json::from_str::<serde_json::Value>(&layer_json) {
                if let Some(features) = fc.get("features").and_then(|f| f.as_array()) {
                    // Add layer name to each feature's properties
                    for feature in features {
                        let mut feature = feature.clone();
                        if let Some(props) = feature.get_mut("properties") {
                            if let Some(props_obj) = props.as_object_mut() {
                                props_obj.insert(
                                    "_layer".to_string(),
                                    serde_json::Value::String(layer.name.clone()),
                                );
                            }
                        }
                        all_features.push(feature);
                    }
                }
            }
        }
    }

    // Build final FeatureCollection
    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": all_features
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/geo+json"),
    );
    headers.insert(CACHE_CONTROL, cache_control::tile_cache_headers());

    Ok((headers, geojson.to_string()).into_response())
}
