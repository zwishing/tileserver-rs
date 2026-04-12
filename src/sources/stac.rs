//! STAC catalog-driven COG discovery and tile serving.
//!
//! This module implements a tile source that discovers Cloud-Optimized GeoTIFF
//! (COG) assets from STAC API catalogs and serves them as raster tiles.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::SourceConfig;
use crate::error::{Result, TileServerError};
use crate::sources::cog::CogSource;
use crate::sources::{TileData, TileFormat, TileMetadata, TileSource};

/// MIME type for Cloud-Optimized GeoTIFF files.
const COG_MIME_TYPE: &str = "image/tiff; application=geotiff; profile=cloud-optimized";

/// MIME type prefix for standard GeoTIFF files.
const GEOTIFF_MIME_TYPE: &str = "image/tiff";

/// A discovered COG asset from a STAC catalog item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StacAsset {
    /// Unique identifier from the STAC item.
    pub id: String,
    /// URL to the COG file.
    pub href: String,
    /// Bounding box as `[west, south, east, north]`.
    pub bbox: [f64; 4],
    /// Human-readable title of the asset.
    pub title: String,
}

/// A tile source backed by STAC API catalog discovery.
///
/// Queries a STAC API for items in a collection, extracts COG asset URLs,
/// and delegates tile serving to an underlying [`CogSource`].
pub struct StacSource {
    id: String,
    metadata: TileMetadata,
    cog_source: Arc<Mutex<Option<CogSource>>>,
    discovered_assets: Vec<StacAsset>,
}

impl StacSource {
    /// Create a new STAC source from configuration.
    ///
    /// Discovers COG assets from the configured STAC API endpoint and
    /// initializes a [`CogSource`] for the first discovered asset.
    ///
    /// # Errors
    ///
    /// Returns [`TileServerError::StacError`] if:
    /// - The `collection` field is missing from config
    /// - The STAC API request fails or returns invalid JSON
    /// - No features are found in the STAC response
    pub async fn new(config: &SourceConfig) -> Result<Self> {
        let api_url = &config.path;
        let collection = config.collection.as_deref().ok_or_else(|| {
            TileServerError::StacError("'collection' is required for stac sources".to_string())
        })?;
        let asset_role = &config.asset_role;
        let max_items = config.max_items;

        let assets = discover_assets(api_url, collection, asset_role, max_items).await?;

        let bounds = compute_merged_bounds(&assets);
        let name = config
            .name
            .clone()
            .unwrap_or_else(|| format!("STAC: {collection}"));

        let cog_source = if let Some(first_asset) = assets.first() {
            let cog_config = SourceConfig {
                id: config.id.clone(),
                source_type: crate::config::SourceType::Cog,
                path: first_asset.href.clone(),
                name: config.name.clone(),
                attribution: config.attribution.clone(),
                description: config.description.clone(),
                resampling: config.resampling,
                layer_name: None,
                geometry_column: None,
                query: None,
                minzoom: config.minzoom,
                maxzoom: config.maxzoom,
                serve_as: None,
                #[cfg(feature = "raster")]
                colormap: config.colormap.clone(),
                options: None,
                collection: config.collection.clone(),
                asset_role: config.asset_role.clone(),
                dynamic: config.dynamic,
                max_items: config.max_items,
            };

            match CogSource::from_file(&cog_config).await {
                Ok(source) => Some(source),
                Err(e) => {
                    tracing::warn!(
                        "failed to create CogSource for first STAC asset {}: {e}",
                        first_asset.href
                    );
                    None
                }
            }
        } else {
            None
        };

        let metadata = TileMetadata {
            id: config.id.clone(),
            name,
            description: config.description.clone(),
            attribution: config.attribution.clone(),
            format: TileFormat::Png,
            minzoom: config.minzoom.unwrap_or(0),
            maxzoom: config.maxzoom.unwrap_or(18),
            bounds,
            center: bounds.map(|b| [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0, 10.0]),
            vector_layers: None,
        };

        Ok(Self {
            id: config.id.clone(),
            metadata,
            cog_source: Arc::new(Mutex::new(cog_source)),
            discovered_assets: assets,
        })
    }

    /// Returns the list of discovered STAC assets.
    #[must_use]
    pub fn discovered_assets(&self) -> &[StacAsset] {
        &self.discovered_assets
    }

    /// Returns the source identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[async_trait]
impl TileSource for StacSource {
    async fn get_tile(&self, z: u8, x: u32, y: u32) -> Result<Option<TileData>> {
        let guard = self.cog_source.lock().await;
        if let Some(ref cog) = *guard {
            cog.get_tile(z, x, y).await
        } else {
            Ok(None)
        }
    }

    fn metadata(&self) -> &TileMetadata {
        &self.metadata
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Discover COG assets from a STAC API catalog.
///
/// Sends a POST search request to the STAC API and extracts COG assets
/// from the returned item collection.
///
/// # Errors
///
/// Returns [`TileServerError::StacError`] if the HTTP request fails,
/// the response status is non-2xx, or the response JSON is malformed.
pub async fn discover_assets(
    api_url: &str,
    collection: &str,
    asset_role: &str,
    max_items: usize,
) -> Result<Vec<StacAsset>> {
    let search_url = build_search_url(api_url);
    let client = reqwest::Client::builder()
        .user_agent("tileserver-rs/stac")
        .build()
        .map_err(|e| TileServerError::StacError(format!("failed to create HTTP client: {e}")))?;

    let search_body = serde_json::json!({
        "collections": [collection],
        "limit": max_items
    });

    let response = client
        .post(&search_url)
        .json(&search_body)
        .send()
        .await
        .map_err(|e| TileServerError::StacError(format!("stac api search failed: {e}")))?;

    if !response.status().is_success() {
        return Err(TileServerError::StacError(format!(
            "stac api returned status {}",
            response.status()
        )));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| TileServerError::StacError(format!("failed to parse stac response: {e}")))?;

    extract_assets_from_item_collection(&body, asset_role)
}

/// Build the STAC API search endpoint URL from a base API URL.
#[must_use]
pub fn build_search_url(api_url: &str) -> String {
    let base = api_url.trim_end_matches('/');
    format!("{base}/search")
}

/// Extract COG assets from a STAC item collection JSON response.
///
/// # Errors
///
/// Returns [`TileServerError::StacError`] if the `features` array is missing.
pub fn extract_assets_from_item_collection(
    body: &serde_json::Value,
    asset_role: &str,
) -> Result<Vec<StacAsset>> {
    let features = body
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or_else(|| {
            TileServerError::StacError("stac response missing 'features' array".to_string())
        })?;

    let mut assets = Vec::with_capacity(features.len());

    for feature in features {
        if let Some(asset) = extract_cog_asset_from_item(feature, asset_role) {
            assets.push(asset);
        }
    }

    Ok(assets)
}

/// Extract a single COG asset from a STAC item, first by role then by MIME type.
pub fn extract_cog_asset_from_item(
    item: &serde_json::Value,
    asset_role: &str,
) -> Option<StacAsset> {
    let item_id = item.get("id")?.as_str()?;
    let bbox = extract_bbox(item)?;
    let assets_obj = item.get("assets")?.as_object()?;

    if let Some(asset) = find_asset_by_role(assets_obj, asset_role) {
        return Some(StacAsset {
            id: item_id.to_string(),
            href: asset.get("href")?.as_str()?.to_string(),
            bbox,
            title: asset
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or(item_id)
                .to_string(),
        });
    }

    if let Some(asset) = find_asset_by_cog_mime(assets_obj) {
        return Some(StacAsset {
            id: item_id.to_string(),
            href: asset.get("href")?.as_str()?.to_string(),
            bbox,
            title: asset
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or(item_id)
                .to_string(),
        });
    }

    None
}

fn find_asset_by_role<'a>(
    assets: &'a serde_json::Map<String, serde_json::Value>,
    role: &str,
) -> Option<&'a serde_json::Value> {
    for asset in assets.values() {
        if let Some(roles) = asset.get("roles").and_then(|r| r.as_array())
            && roles.iter().any(|r| r.as_str() == Some(role))
        {
            return Some(asset);
        }
    }

    assets.get(role)
}

fn find_asset_by_cog_mime(
    assets: &serde_json::Map<String, serde_json::Value>,
) -> Option<&serde_json::Value> {
    for asset in assets.values() {
        if let Some(mime) = asset.get("type").and_then(|t| t.as_str())
            && (mime == COG_MIME_TYPE || mime.starts_with(GEOTIFF_MIME_TYPE))
        {
            return Some(asset);
        }
    }
    None
}

fn extract_bbox(item: &serde_json::Value) -> Option<[f64; 4]> {
    let bbox = item.get("bbox")?.as_array()?;
    if bbox.len() < 4 {
        return None;
    }
    Some([
        bbox[0].as_f64()?,
        bbox[1].as_f64()?,
        bbox[2].as_f64()?,
        bbox[3].as_f64()?,
    ])
}

/// Compute the merged bounding box across all assets.
#[must_use]
pub fn compute_merged_bounds(assets: &[StacAsset]) -> Option<[f64; 4]> {
    if assets.is_empty() {
        return None;
    }

    let mut merged = assets[0].bbox;
    for asset in &assets[1..] {
        merged[0] = merged[0].min(asset.bbox[0]);
        merged[1] = merged[1].min(asset.bbox[1]);
        merged[2] = merged[2].max(asset.bbox[2]);
        merged[3] = merged[3].max(asset.bbox[3]);
    }

    Some(merged)
}

/// Returns `true` if the MIME type indicates a COG or GeoTIFF file.
#[must_use]
pub fn is_cog_mime_type(mime: &str) -> bool {
    mime == COG_MIME_TYPE || mime.starts_with(GEOTIFF_MIME_TYPE)
}

/// Returns `true` if the URL uses an HTTP(S) scheme (likely a STAC API endpoint).
#[must_use]
pub fn is_stac_api_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item_json(item_id: &str, role: &str, mime: &str) -> serde_json::Value {
        serde_json::json!({
            "id": item_id,
            "type": "Feature",
            "bbox": [-122.5, 37.0, -122.0, 37.5],
            "geometry": {
                "type": "Polygon",
                "coordinates": [[[-122.5, 37.0], [-122.0, 37.0], [-122.0, 37.5], [-122.5, 37.5], [-122.5, 37.0]]]
            },
            "properties": {
                "datetime": "2023-01-15T00:00:00Z"
            },
            "assets": {
                "visual": {
                    "href": "https://example.com/cog.tif",
                    "type": mime,
                    "title": "Visual RGB",
                    "roles": [role]
                },
                "thumbnail": {
                    "href": "https://example.com/thumb.png",
                    "type": "image/png",
                    "title": "Thumbnail",
                    "roles": ["thumbnail"]
                }
            }
        })
    }

    fn sample_item_collection(items: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "type": "FeatureCollection",
            "features": items,
            "numberMatched": items.len(),
            "numberReturned": items.len()
        })
    }

    #[test]
    fn test_build_search_url_basic() {
        assert_eq!(
            build_search_url("https://earth-search.aws.element84.com/v1"),
            "https://earth-search.aws.element84.com/v1/search"
        );
    }

    #[test]
    fn test_build_search_url_strips_trailing_slash() {
        assert_eq!(
            build_search_url("https://example.com/stac/"),
            "https://example.com/stac/search"
        );
    }

    #[test]
    fn test_extract_bbox_valid() {
        let item = serde_json::json!({"bbox": [-122.5, 37.0, -122.0, 37.5]});
        let bbox = extract_bbox(&item).unwrap();
        assert_eq!(bbox, [-122.5, 37.0, -122.0, 37.5]);
    }

    #[test]
    fn test_extract_bbox_6_element() {
        let item = serde_json::json!({"bbox": [-122.5, 37.0, 0.0, -122.0, 37.5, 100.0]});
        let bbox = extract_bbox(&item).unwrap();
        assert_eq!(bbox, [-122.5, 37.0, 0.0, -122.0]);
    }

    #[test]
    fn test_extract_bbox_missing() {
        let item = serde_json::json!({"id": "test"});
        assert!(extract_bbox(&item).is_none());
    }

    #[test]
    fn test_extract_bbox_too_short() {
        let item = serde_json::json!({"bbox": [1.0, 2.0]});
        assert!(extract_bbox(&item).is_none());
    }

    #[test]
    fn test_extract_bbox_not_array() {
        let item = serde_json::json!({"bbox": "invalid"});
        assert!(extract_bbox(&item).is_none());
    }

    #[test]
    fn test_is_cog_mime_type_exact() {
        assert!(is_cog_mime_type(COG_MIME_TYPE));
    }

    #[test]
    fn test_is_cog_mime_type_geotiff() {
        assert!(is_cog_mime_type("image/tiff"));
    }

    #[test]
    fn test_is_cog_mime_type_geotiff_with_params() {
        assert!(is_cog_mime_type("image/tiff; application=geotiff"));
    }

    #[test]
    fn test_is_cog_mime_type_false() {
        assert!(!is_cog_mime_type("image/png"));
        assert!(!is_cog_mime_type("application/json"));
    }

    #[test]
    fn test_is_stac_api_url_https() {
        assert!(is_stac_api_url("https://earth-search.aws.element84.com/v1"));
    }

    #[test]
    fn test_is_stac_api_url_http() {
        assert!(is_stac_api_url("http://localhost:8080/stac"));
    }

    #[test]
    fn test_is_stac_api_url_false() {
        assert!(!is_stac_api_url("/local/path"));
        assert!(!is_stac_api_url("s3://bucket/path"));
    }

    #[test]
    fn test_find_asset_by_role_found() {
        let item = sample_item_json("s2-001", "visual", COG_MIME_TYPE);
        let assets = item["assets"].as_object().unwrap();
        let result = find_asset_by_role(assets, "visual");
        assert!(result.is_some());
        assert_eq!(
            result.unwrap()["href"].as_str().unwrap(),
            "https://example.com/cog.tif"
        );
    }

    #[test]
    fn test_find_asset_by_role_not_found() {
        let item = sample_item_json("s2-001", "visual", COG_MIME_TYPE);
        let assets = item["assets"].as_object().unwrap();
        let result = find_asset_by_role(assets, "data");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_asset_by_role_fallback_to_key() {
        let item = serde_json::json!({
            "assets": {
                "visual": {
                    "href": "https://example.com/file.tif",
                    "type": COG_MIME_TYPE
                }
            }
        });
        let assets = item["assets"].as_object().unwrap();
        let result = find_asset_by_role(assets, "visual");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_asset_by_cog_mime_found() {
        let item = sample_item_json("s2-001", "visual", COG_MIME_TYPE);
        let assets = item["assets"].as_object().unwrap();
        let result = find_asset_by_cog_mime(assets);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_asset_by_cog_mime_geotiff() {
        let item = sample_item_json("s2-001", "visual", "image/tiff; application=geotiff");
        let assets = item["assets"].as_object().unwrap();
        let result = find_asset_by_cog_mime(assets);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_asset_by_cog_mime_none() {
        let item = serde_json::json!({
            "assets": {
                "thumb": {
                    "href": "https://example.com/thumb.png",
                    "type": "image/png"
                }
            }
        });
        let assets = item["assets"].as_object().unwrap();
        let result = find_asset_by_cog_mime(assets);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_cog_asset_by_role() {
        let item = sample_item_json("s2-001", "visual", COG_MIME_TYPE);
        let asset = extract_cog_asset_from_item(&item, "visual").unwrap();
        assert_eq!(asset.id, "s2-001");
        assert_eq!(asset.href, "https://example.com/cog.tif");
        assert_eq!(asset.title, "Visual RGB");
        assert_eq!(asset.bbox, [-122.5, 37.0, -122.0, 37.5]);
    }

    #[test]
    fn test_extract_cog_asset_fallback_to_mime() {
        let item = serde_json::json!({
            "id": "landsat-001",
            "bbox": [10.0, 20.0, 11.0, 21.0],
            "assets": {
                "B04": {
                    "href": "https://example.com/B04.tif",
                    "type": COG_MIME_TYPE,
                    "title": "Band 4",
                    "roles": ["data"]
                }
            }
        });
        let asset = extract_cog_asset_from_item(&item, "visual").unwrap();
        assert_eq!(asset.id, "landsat-001");
        assert_eq!(asset.href, "https://example.com/B04.tif");
    }

    #[test]
    fn test_extract_cog_asset_no_match() {
        let item = serde_json::json!({
            "id": "empty",
            "bbox": [0.0, 0.0, 1.0, 1.0],
            "assets": {
                "thumb": {
                    "href": "https://example.com/thumb.png",
                    "type": "image/png",
                    "roles": ["thumbnail"]
                }
            }
        });
        assert!(extract_cog_asset_from_item(&item, "visual").is_none());
    }

    #[test]
    fn test_extract_cog_asset_missing_bbox() {
        let item = serde_json::json!({
            "id": "no-bbox",
            "assets": {
                "visual": {
                    "href": "https://example.com/cog.tif",
                    "type": COG_MIME_TYPE,
                    "roles": ["visual"]
                }
            }
        });
        assert!(extract_cog_asset_from_item(&item, "visual").is_none());
    }

    #[test]
    fn test_extract_cog_asset_missing_id() {
        let item = serde_json::json!({
            "bbox": [0.0, 0.0, 1.0, 1.0],
            "assets": {}
        });
        assert!(extract_cog_asset_from_item(&item, "visual").is_none());
    }

    #[test]
    fn test_extract_assets_from_item_collection() {
        let items = vec![
            sample_item_json("item-1", "visual", COG_MIME_TYPE),
            sample_item_json("item-2", "visual", COG_MIME_TYPE),
        ];
        let collection = sample_item_collection(items);
        let assets = extract_assets_from_item_collection(&collection, "visual").unwrap();
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].id, "item-1");
        assert_eq!(assets[1].id, "item-2");
    }

    #[test]
    fn test_extract_assets_empty_collection() {
        let collection = sample_item_collection(vec![]);
        let assets = extract_assets_from_item_collection(&collection, "visual").unwrap();
        assert!(assets.is_empty());
    }

    #[test]
    fn test_extract_assets_missing_features() {
        let body = serde_json::json!({"type": "FeatureCollection"});
        let result = extract_assets_from_item_collection(&body, "visual");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_assets_mixed_items() {
        let items = vec![
            sample_item_json("has-cog", "visual", COG_MIME_TYPE),
            serde_json::json!({
                "id": "no-cog",
                "bbox": [0.0, 0.0, 1.0, 1.0],
                "assets": {
                    "thumb": {
                        "href": "https://example.com/thumb.png",
                        "type": "image/png",
                        "roles": ["thumbnail"]
                    }
                }
            }),
        ];
        let collection = sample_item_collection(items);
        let assets = extract_assets_from_item_collection(&collection, "visual").unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, "has-cog");
    }

    #[test]
    fn test_compute_merged_bounds_empty() {
        assert!(compute_merged_bounds(&[]).is_none());
    }

    #[test]
    fn test_compute_merged_bounds_single() {
        let assets = vec![StacAsset {
            id: "a".to_string(),
            href: "https://example.com/a.tif".to_string(),
            bbox: [10.0, 20.0, 11.0, 21.0],
            title: "A".to_string(),
        }];
        let bounds = compute_merged_bounds(&assets).unwrap();
        assert_eq!(bounds, [10.0, 20.0, 11.0, 21.0]);
    }

    #[test]
    fn test_compute_merged_bounds_multiple() {
        let assets = vec![
            StacAsset {
                id: "a".to_string(),
                href: "https://example.com/a.tif".to_string(),
                bbox: [10.0, 20.0, 11.0, 21.0],
                title: "A".to_string(),
            },
            StacAsset {
                id: "b".to_string(),
                href: "https://example.com/b.tif".to_string(),
                bbox: [9.0, 19.0, 12.0, 22.0],
                title: "B".to_string(),
            },
        ];
        let bounds = compute_merged_bounds(&assets).unwrap();
        assert_eq!(bounds, [9.0, 19.0, 12.0, 22.0]);
    }

    #[test]
    fn test_compute_merged_bounds_non_overlapping() {
        let assets = vec![
            StacAsset {
                id: "west".to_string(),
                href: "https://example.com/w.tif".to_string(),
                bbox: [-10.0, 0.0, -5.0, 5.0],
                title: "West".to_string(),
            },
            StacAsset {
                id: "east".to_string(),
                href: "https://example.com/e.tif".to_string(),
                bbox: [5.0, 0.0, 10.0, 5.0],
                title: "East".to_string(),
            },
        ];
        let bounds = compute_merged_bounds(&assets).unwrap();
        assert_eq!(bounds, [-10.0, 0.0, 10.0, 5.0]);
    }

    #[test]
    fn test_stac_asset_serialization_roundtrip() {
        let asset = StacAsset {
            id: "test-id".to_string(),
            href: "https://example.com/cog.tif".to_string(),
            bbox: [-180.0, -90.0, 180.0, 90.0],
            title: "Test Asset".to_string(),
        };
        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: StacAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.href, "https://example.com/cog.tif");
        assert_eq!(deserialized.bbox, [-180.0, -90.0, 180.0, 90.0]);
        assert_eq!(deserialized.title, "Test Asset");
    }

    #[test]
    fn test_stac_asset_deserialization() {
        let json = r#"{"id":"s2","href":"https://ex.com/s2.tif","bbox":[0,0,1,1],"title":"S2"}"#;
        let asset: StacAsset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.id, "s2");
        assert_eq!(asset.bbox, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn test_extract_cog_asset_title_fallback_to_id() {
        let item = serde_json::json!({
            "id": "item-no-title",
            "bbox": [0.0, 0.0, 1.0, 1.0],
            "assets": {
                "visual": {
                    "href": "https://example.com/cog.tif",
                    "type": COG_MIME_TYPE,
                    "roles": ["visual"]
                }
            }
        });
        let asset = extract_cog_asset_from_item(&item, "visual").unwrap();
        assert_eq!(asset.title, "item-no-title");
    }

    #[test]
    fn test_extract_cog_asset_multiple_roles() {
        let item = serde_json::json!({
            "id": "multi-role",
            "bbox": [0.0, 0.0, 1.0, 1.0],
            "assets": {
                "B04": {
                    "href": "https://example.com/B04.tif",
                    "type": COG_MIME_TYPE,
                    "title": "Red Band",
                    "roles": ["data", "visual"]
                }
            }
        });
        let asset = extract_cog_asset_from_item(&item, "visual").unwrap();
        assert_eq!(asset.id, "multi-role");
        assert_eq!(asset.title, "Red Band");
    }

    #[test]
    fn test_config_stac_source_type_serialization() {
        assert_eq!(
            serde_json::to_string(&crate::config::SourceType::Stac).unwrap(),
            "\"stac\""
        );
    }

    #[test]
    fn test_config_stac_source_type_deserialization() {
        let parsed: crate::config::SourceType = serde_json::from_str("\"stac\"").unwrap();
        assert_eq!(parsed, crate::config::SourceType::Stac);
    }

    #[test]
    fn test_config_stac_toml_parsing() {
        let toml = r#"
            [[sources]]
            id = "sentinel2"
            type = "stac"
            path = "https://earth-search.aws.element84.com/v1"
            name = "Sentinel-2 L2A"
            collection = "sentinel-2-l2a"
            asset_role = "visual"
            dynamic = false
            max_items = 50
        "#;

        let config: crate::config::Config = toml::from_str(toml).unwrap();
        assert_eq!(config.sources.len(), 1);
        let src = &config.sources[0];
        assert_eq!(src.id, "sentinel2");
        assert_eq!(src.source_type, crate::config::SourceType::Stac);
        assert_eq!(src.path, "https://earth-search.aws.element84.com/v1");
        assert_eq!(src.collection.as_deref(), Some("sentinel-2-l2a"));
        assert_eq!(src.asset_role, "visual");
        assert!(!src.dynamic);
        assert_eq!(src.max_items, 50);
    }

    #[test]
    fn test_config_stac_defaults() {
        let toml = r#"
            [[sources]]
            id = "test-stac"
            type = "stac"
            path = "https://stac-api.example.com"
            collection = "my-collection"
        "#;

        let config: crate::config::Config = toml::from_str(toml).unwrap();
        let src = &config.sources[0];
        assert_eq!(src.asset_role, "visual");
        assert!(!src.dynamic);
        assert_eq!(src.max_items, 100);
    }

    #[test]
    fn test_config_stac_alongside_pmtiles() {
        let toml = r#"
            [[sources]]
            id = "osm"
            type = "pmtiles"
            path = "/data/osm.pmtiles"

            [[sources]]
            id = "sentinel"
            type = "stac"
            path = "https://earth-search.aws.element84.com/v1"
            collection = "sentinel-2-l2a"
        "#;

        let config: crate::config::Config = toml::from_str(toml).unwrap();
        assert_eq!(config.sources.len(), 2);
        assert_eq!(
            config.sources[0].source_type,
            crate::config::SourceType::PMTiles
        );
        assert_eq!(
            config.sources[1].source_type,
            crate::config::SourceType::Stac
        );
    }

    #[test]
    fn test_config_multiple_stac_sources() {
        let toml = r#"
            [[sources]]
            id = "sentinel"
            type = "stac"
            path = "https://earth-search.aws.element84.com/v1"
            collection = "sentinel-2-l2a"

            [[sources]]
            id = "landsat"
            type = "stac"
            path = "https://landsatlook.usgs.gov/stac-server"
            collection = "landsat-c2-l2"
            asset_role = "data"
            max_items = 200
        "#;

        let config: crate::config::Config = toml::from_str(toml).unwrap();
        assert_eq!(config.sources.len(), 2);
        assert_eq!(
            config.sources[0].collection.as_deref(),
            Some("sentinel-2-l2a")
        );
        assert_eq!(
            config.sources[1].collection.as_deref(),
            Some("landsat-c2-l2")
        );
        assert_eq!(config.sources[1].asset_role, "data");
        assert_eq!(config.sources[1].max_items, 200);
    }

    #[test]
    fn test_stac_error_display() {
        let err = TileServerError::StacError("connection timeout".to_string());
        assert_eq!(err.to_string(), "stac error: connection timeout");
    }

    #[test]
    fn test_stac_error_status_code() {
        use axum::response::IntoResponse;
        let err = TileServerError::StacError("api failure".to_string());
        let response = err.into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
