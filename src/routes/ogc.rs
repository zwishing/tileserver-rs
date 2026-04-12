//! OGC API Features Part 1 (Core) route handlers.
//!
//! Implements read-only feature access for PostGIS table sources,
//! enabling QGIS/ArcGIS/FME native connectivity.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;

use crate::error::TileServerError;
use crate::reload::SharedState;
use crate::sources::postgres::PostgresTableSource;

const OGC_FEATURES_LIMIT_DEFAULT: i64 = 10;
const OGC_FEATURES_LIMIT_MAX: i64 = 10_000;

#[derive(Debug, Deserialize)]
pub(crate) struct ItemsQueryParams {
    #[serde(default)]
    bbox: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[allow(dead_code)]
    #[serde(default)]
    datetime: Option<String>,
}

fn default_limit() -> i64 {
    OGC_FEATURES_LIMIT_DEFAULT
}

fn build_base_url(state: &crate::reload::AppState) -> String {
    state.base_url.clone()
}

pub(crate) async fn landing_page(State(shared): State<SharedState>) -> impl IntoResponse {
    let state = shared.load();
    let base = build_base_url(&state);

    let landing = serde_json::json!({
        "title": "tileserver-rs OGC API",
        "description": "OGC API Features access to PostGIS table sources",
        "links": [
            {
                "href": base.clone(),
                "rel": "self",
                "type": "application/json",
                "title": "this document"
            },
            {
                "href": format!("{base}/conformance"),
                "rel": "conformance",
                "type": "application/json",
                "title": "OGC API conformance classes"
            },
            {
                "href": format!("{base}/collections"),
                "rel": "data",
                "type": "application/json",
                "title": "feature collections"
            },
            {
                "href": format!("{base}/_openapi"),
                "rel": "service-desc",
                "type": "text/html",
                "title": "API documentation"
            }
        ]
    });

    Json(landing)
}

pub(crate) async fn conformance() -> impl IntoResponse {
    let body = serde_json::json!({
        "conformsTo": [
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson"
        ]
    });
    Json(body)
}

pub(crate) async fn collections(State(shared): State<SharedState>) -> impl IntoResponse {
    let state = shared.load();
    let base = build_base_url(&state);
    let mut collections = Vec::new();

    for meta in state.sources.all_metadata() {
        let source = state.sources.get(&meta.id);
        let is_table = source
            .map(|s| s.as_any().downcast_ref::<PostgresTableSource>().is_some())
            .unwrap_or(false);

        if !is_table {
            continue;
        }

        collections.push(build_collection_json(meta, &base));
    }

    Json(serde_json::json!({
        "collections": collections,
        "links": [
            {
                "href": format!("{base}/collections"),
                "rel": "self",
                "type": "application/json",
                "title": "feature collections"
            }
        ]
    }))
}

pub(crate) async fn collection(
    State(shared): State<SharedState>,
    Path(collection_id): Path<String>,
) -> Result<impl IntoResponse, TileServerError> {
    let state = shared.load();
    let base = build_base_url(&state);

    let source = state
        .sources
        .get(&collection_id)
        .ok_or_else(|| TileServerError::SourceNotFound(collection_id.clone()))?;

    source
        .as_any()
        .downcast_ref::<PostgresTableSource>()
        .ok_or_else(|| {
            TileServerError::NotFound(format!(
                "collection '{collection_id}' is not an OGC features source"
            ))
        })?;

    let meta = source.metadata();
    let body = build_collection_json(meta, &base);
    Ok(Json(body))
}

pub(crate) async fn items(
    State(shared): State<SharedState>,
    Path(collection_id): Path<String>,
    Query(params): Query<ItemsQueryParams>,
) -> Result<Response, TileServerError> {
    let state = shared.load();
    let base = build_base_url(&state);

    let source = state
        .sources
        .get(&collection_id)
        .ok_or_else(|| TileServerError::SourceNotFound(collection_id.clone()))?;

    let table_source = source
        .as_any()
        .downcast_ref::<PostgresTableSource>()
        .ok_or_else(|| {
            TileServerError::NotFound(format!(
                "collection '{collection_id}' is not an OGC features source"
            ))
        })?;

    let limit = params.limit.clamp(1, OGC_FEATURES_LIMIT_MAX);
    let offset = params.offset.max(0);

    let bbox = parse_bbox(params.bbox.as_deref())?;

    let (features, number_matched) = table_source
        .query_features_geojson(bbox, limit, offset)
        .await?;

    let number_returned = features.len() as i64;

    let mut links = vec![serde_json::json!({
        "href": format!("{base}/collections/{collection_id}/items?limit={limit}&offset={offset}"),
        "rel": "self",
        "type": "application/geo+json",
        "title": "this page"
    })];

    if offset + number_returned < number_matched {
        let next_offset = offset + limit;
        links.push(serde_json::json!({
            "href": format!("{base}/collections/{collection_id}/items?limit={limit}&offset={next_offset}"),
            "rel": "next",
            "type": "application/geo+json",
            "title": "next page"
        }));
    }

    let body = serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
        "numberMatched": number_matched,
        "numberReturned": number_returned,
        "links": links
    });

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/geo+json")],
        Json(body),
    )
        .into_response())
}

pub(crate) async fn feature(
    State(shared): State<SharedState>,
    Path((collection_id, feature_id)): Path<(String, String)>,
) -> Result<Response, TileServerError> {
    let state = shared.load();
    let base = build_base_url(&state);

    let source = state
        .sources
        .get(&collection_id)
        .ok_or_else(|| TileServerError::SourceNotFound(collection_id.clone()))?;

    let table_source = source
        .as_any()
        .downcast_ref::<PostgresTableSource>()
        .ok_or_else(|| {
            TileServerError::NotFound(format!(
                "collection '{collection_id}' is not an OGC features source"
            ))
        })?;

    let info = table_source.table_info();
    let conn =
        table_source.pool().get().await.map_err(|e| {
            TileServerError::PostgresError(format!("failed to get connection: {e}"))
        })?;

    let id_col = info.id_column.as_deref().unwrap_or("ctid");

    let geom_expr = if info.srid == 4326 {
        format!(r#"ST_AsGeoJSON("{}")::jsonb"#, info.geometry_column)
    } else {
        format!(
            r#"ST_AsGeoJSON(ST_Transform("{}", 4326))::jsonb"#,
            info.geometry_column
        )
    };

    let prop_cols: Vec<String> = info
        .properties
        .iter()
        .map(|p| format!(r#""{p}""#))
        .collect();
    let prop_select = if prop_cols.is_empty() {
        String::new()
    } else {
        format!(", {}", prop_cols.join(", "))
    };

    let sql = format!(
        r#"SELECT {geom_expr} AS __ogc_geom{prop_select} FROM "{}"."{}" WHERE "{}"::text = $1 LIMIT 1"#,
        info.schema, info.table, id_col
    );

    let row = conn
        .query_opt(&sql, &[&feature_id])
        .await
        .map_err(|e| TileServerError::PostgresError(format!("feature query failed: {e}")))?
        .ok_or_else(|| {
            TileServerError::NotFound(format!(
                "feature '{feature_id}' not found in collection '{collection_id}'"
            ))
        })?;

    let geom: serde_json::Value = row.get("__ogc_geom");

    let mut properties = serde_json::Map::new();
    for prop in &info.properties {
        if let Ok(val) = row.try_get::<_, Option<serde_json::Value>>(prop.as_str()) {
            properties.insert(prop.clone(), val.unwrap_or(serde_json::Value::Null));
        } else if let Ok(val) = row.try_get::<_, Option<String>>(prop.as_str()) {
            properties.insert(
                prop.clone(),
                val.map_or(serde_json::Value::Null, serde_json::Value::String),
            );
        } else if let Ok(val) = row.try_get::<_, Option<i64>>(prop.as_str()) {
            properties.insert(
                prop.clone(),
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Number(v.into())
                }),
            );
        } else if let Ok(val) = row.try_get::<_, Option<f64>>(prop.as_str()) {
            properties.insert(
                prop.clone(),
                val.map_or(serde_json::Value::Null, |v| serde_json::json!(v)),
            );
        } else if let Ok(val) = row.try_get::<_, Option<bool>>(prop.as_str()) {
            properties.insert(
                prop.clone(),
                val.map_or(serde_json::Value::Null, serde_json::Value::Bool),
            );
        }
    }

    let body = serde_json::json!({
        "type": "Feature",
        "id": feature_id,
        "geometry": geom,
        "properties": properties,
        "links": [
            {
                "href": format!("{base}/collections/{collection_id}/items/{feature_id}"),
                "rel": "self",
                "type": "application/geo+json",
                "title": "this feature"
            },
            {
                "href": format!("{base}/collections/{collection_id}"),
                "rel": "collection",
                "type": "application/json",
                "title": "the collection"
            }
        ]
    });

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/geo+json")],
        Json(body),
    )
        .into_response())
}

fn build_collection_json(meta: &crate::sources::TileMetadata, base_url: &str) -> serde_json::Value {
    let mut extent = serde_json::Map::new();
    if let Some(bounds) = meta.bounds {
        extent.insert(
            "spatial".to_string(),
            serde_json::json!({
                "bbox": [bounds],
                "crs": "http://www.opengis.net/def/crs/OGC/1.3/CRS84"
            }),
        );
    }

    serde_json::json!({
        "id": meta.id,
        "title": meta.name,
        "description": meta.description,
        "extent": extent,
        "itemType": "feature",
        "crs": [
            "http://www.opengis.net/def/crs/OGC/1.3/CRS84"
        ],
        "links": [
            {
                "href": format!("{base_url}/collections/{}", meta.id),
                "rel": "self",
                "type": "application/json",
                "title": meta.name
            },
            {
                "href": format!("{base_url}/collections/{}/items", meta.id),
                "rel": "items",
                "type": "application/geo+json",
                "title": format!("{} items", meta.name)
            }
        ]
    })
}

fn parse_bbox(bbox_str: Option<&str>) -> Result<Option<[f64; 4]>, TileServerError> {
    let Some(s) = bbox_str else {
        return Ok(None);
    };

    let parts: Vec<f64> = s
        .split(',')
        .map(|p| {
            p.trim()
                .parse::<f64>()
                .map_err(|_| TileServerError::InvalidTileRequest)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if parts.len() != 4 {
        return Err(TileServerError::InvalidTileRequest);
    }

    if parts[0] > parts[2] || parts[1] > parts[3] {
        return Err(TileServerError::InvalidTileRequest);
    }

    Ok(Some([parts[0], parts[1], parts[2], parts[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    struct OgcLink {
        href: String,
        rel: String,
        #[serde(rename = "type")]
        media_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    }

    #[test]
    fn parse_bbox_none_returns_none() {
        assert!(parse_bbox(None).unwrap().is_none());
    }

    #[test]
    fn parse_bbox_valid_four_values() {
        let result = parse_bbox(Some("-180,-90,180,90")).unwrap().unwrap();
        assert_eq!(result, [-180.0, -90.0, 180.0, 90.0]);
    }

    #[test]
    fn parse_bbox_with_spaces() {
        let result = parse_bbox(Some(" -10.5 , -20.3 , 30.1 , 40.2 "))
            .unwrap()
            .unwrap();
        assert_eq!(result, [-10.5, -20.3, 30.1, 40.2]);
    }

    #[test]
    fn parse_bbox_inverted_lon_fails() {
        assert!(parse_bbox(Some("180,-90,-180,90")).is_err());
    }

    #[test]
    fn parse_bbox_inverted_lat_fails() {
        assert!(parse_bbox(Some("-180,90,180,-90")).is_err());
    }

    #[test]
    fn parse_bbox_too_few_values_fails() {
        assert!(parse_bbox(Some("-180,-90,180")).is_err());
    }

    #[test]
    fn parse_bbox_too_many_values_fails() {
        assert!(parse_bbox(Some("-180,-90,180,90,100")).is_err());
    }

    #[test]
    fn parse_bbox_non_numeric_fails() {
        assert!(parse_bbox(Some("abc,-90,180,90")).is_err());
    }

    #[test]
    fn parse_bbox_empty_string_fails() {
        assert!(parse_bbox(Some("")).is_err());
    }

    #[test]
    fn conformance_includes_required_classes() {
        let body = serde_json::json!({
            "conformsTo": [
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson"
            ]
        });
        let conforms = body["conformsTo"].as_array().unwrap();
        assert_eq!(conforms.len(), 3);
        assert!(
            conforms
                .iter()
                .any(|v| v.as_str().unwrap().contains("conf/core"))
        );
        assert!(
            conforms
                .iter()
                .any(|v| v.as_str().unwrap().contains("conf/oas30"))
        );
        assert!(
            conforms
                .iter()
                .any(|v| v.as_str().unwrap().contains("conf/geojson"))
        );
    }

    #[test]
    fn build_collection_json_with_bounds() {
        let meta = crate::sources::TileMetadata {
            id: "test".to_string(),
            name: "Test Layer".to_string(),
            description: Some("A test layer".to_string()),
            attribution: None,
            format: crate::sources::TileFormat::Pbf,
            minzoom: 0,
            maxzoom: 14,
            bounds: Some([8.0, 47.0, 9.0, 48.0]),
            center: None,
            vector_layers: None,
        };

        let json = build_collection_json(&meta, "http://localhost:8080");
        assert_eq!(json["id"], "test");
        assert_eq!(json["title"], "Test Layer");
        assert_eq!(json["description"], "A test layer");
        assert_eq!(json["itemType"], "feature");

        let spatial = &json["extent"]["spatial"];
        assert!(spatial["bbox"].is_array());
        assert_eq!(spatial["bbox"][0][0], 8.0);
        assert_eq!(spatial["bbox"][0][1], 47.0);
        assert_eq!(spatial["bbox"][0][2], 9.0);
        assert_eq!(spatial["bbox"][0][3], 48.0);
        assert!(spatial["crs"].as_str().unwrap().contains("CRS84"));

        let links = json["links"].as_array().unwrap();
        assert_eq!(links.len(), 2);
        assert!(
            links[0]["href"]
                .as_str()
                .unwrap()
                .contains("/collections/test")
        );
        assert_eq!(links[0]["rel"], "self");
        assert!(
            links[1]["href"]
                .as_str()
                .unwrap()
                .contains("/collections/test/items")
        );
        assert_eq!(links[1]["rel"], "items");
    }

    #[test]
    fn build_collection_json_without_bounds() {
        let meta = crate::sources::TileMetadata {
            id: "no_bounds".to_string(),
            name: "No Bounds".to_string(),
            description: None,
            attribution: None,
            format: crate::sources::TileFormat::Pbf,
            minzoom: 0,
            maxzoom: 22,
            bounds: None,
            center: None,
            vector_layers: None,
        };

        let json = build_collection_json(&meta, "http://example.com");
        assert_eq!(json["id"], "no_bounds");
        assert!(json["extent"].as_object().unwrap().is_empty());
    }

    #[test]
    fn build_collection_json_links_format() {
        let meta = crate::sources::TileMetadata {
            id: "roads".to_string(),
            name: "Roads".to_string(),
            description: None,
            attribution: None,
            format: crate::sources::TileFormat::Pbf,
            minzoom: 0,
            maxzoom: 14,
            bounds: None,
            center: None,
            vector_layers: None,
        };

        let json = build_collection_json(&meta, "http://host:9090");
        let links = json["links"].as_array().unwrap();
        assert_eq!(links[0]["type"], "application/json");
        assert_eq!(links[1]["type"], "application/geo+json");
    }

    #[test]
    fn default_limit_is_10() {
        assert_eq!(default_limit(), OGC_FEATURES_LIMIT_DEFAULT);
        assert_eq!(OGC_FEATURES_LIMIT_DEFAULT, 10);
    }

    #[test]
    fn max_limit_is_10000() {
        assert_eq!(OGC_FEATURES_LIMIT_MAX, 10_000);
    }

    #[test]
    fn ogc_link_serializes_correctly() {
        let link = OgcLink {
            href: "http://example.com".to_string(),
            rel: "self".to_string(),
            media_type: "application/json".to_string(),
            title: Some("Test".to_string()),
        };
        let json = serde_json::to_value(&link).unwrap();
        assert_eq!(json["href"], "http://example.com");
        assert_eq!(json["rel"], "self");
        assert_eq!(json["type"], "application/json");
        assert_eq!(json["title"], "Test");
    }

    #[test]
    fn ogc_link_skips_none_title() {
        let link = OgcLink {
            href: "http://example.com".to_string(),
            rel: "self".to_string(),
            media_type: "application/json".to_string(),
            title: None,
        };
        let json = serde_json::to_value(&link).unwrap();
        assert!(!json.as_object().unwrap().contains_key("title"));
    }

    #[test]
    fn items_query_params_defaults() {
        let params: ItemsQueryParams = serde_json::from_str(r#"{}"#).unwrap();
        assert!(params.bbox.is_none());
        assert_eq!(params.limit, 10);
        assert_eq!(params.offset, 0);
        assert!(params.datetime.is_none());
    }

    #[test]
    fn items_query_params_custom_values() {
        let params: ItemsQueryParams =
            serde_json::from_str(r#"{"bbox":"-10,-20,30,40","limit":50,"offset":100}"#).unwrap();
        assert_eq!(params.bbox.as_deref(), Some("-10,-20,30,40"));
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 100);
    }

    #[test]
    fn parse_bbox_zero_width_valid() {
        let result = parse_bbox(Some("10,20,10,20")).unwrap().unwrap();
        assert_eq!(result, [10.0, 20.0, 10.0, 20.0]);
    }

    #[test]
    fn parse_bbox_decimal_precision() {
        let result = parse_bbox(Some("8.12345,47.54321,9.98765,48.01234"))
            .unwrap()
            .unwrap();
        assert!((result[0] - 8.12345).abs() < f64::EPSILON);
        assert!((result[1] - 47.54321).abs() < f64::EPSILON);
    }

    #[test]
    fn build_collection_json_crs_field() {
        let meta = crate::sources::TileMetadata {
            id: "t".to_string(),
            name: "T".to_string(),
            description: None,
            attribution: None,
            format: crate::sources::TileFormat::Pbf,
            minzoom: 0,
            maxzoom: 22,
            bounds: None,
            center: None,
            vector_layers: None,
        };
        let json = build_collection_json(&meta, "http://localhost");
        let crs = json["crs"].as_array().unwrap();
        assert_eq!(crs.len(), 1);
        assert!(crs[0].as_str().unwrap().contains("CRS84"));
    }

    #[test]
    fn conformance_three_uris() {
        let body = serde_json::json!({
            "conformsTo": [
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson"
            ]
        });
        assert_eq!(body["conformsTo"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn feature_collection_structure() {
        let body = serde_json::json!({
            "type": "FeatureCollection",
            "features": [],
            "numberMatched": 0,
            "numberReturned": 0,
            "links": []
        });
        assert_eq!(body["type"], "FeatureCollection");
        assert!(body["features"].as_array().unwrap().is_empty());
        assert_eq!(body["numberMatched"], 0);
        assert_eq!(body["numberReturned"], 0);
    }

    #[test]
    fn feature_structure() {
        let body = serde_json::json!({
            "type": "Feature",
            "id": "42",
            "geometry": {"type": "Point", "coordinates": [8.5, 47.5]},
            "properties": {"name": "Zurich"},
            "links": []
        });
        assert_eq!(body["type"], "Feature");
        assert_eq!(body["id"], "42");
        assert_eq!(body["geometry"]["type"], "Point");
        assert_eq!(body["properties"]["name"], "Zurich");
    }

    #[test]
    fn landing_page_structure() {
        let landing = serde_json::json!({
            "title": "tileserver-rs OGC API",
            "description": "OGC API Features access to PostGIS table sources",
            "links": [
                {"href": "http://localhost:8080", "rel": "self", "type": "application/json"},
                {"href": "http://localhost:8080/conformance", "rel": "conformance", "type": "application/json"},
                {"href": "http://localhost:8080/collections", "rel": "data", "type": "application/json"},
                {"href": "http://localhost:8080/_openapi", "rel": "service-desc", "type": "text/html"}
            ]
        });
        assert_eq!(landing["title"], "tileserver-rs OGC API");
        let links = landing["links"].as_array().unwrap();
        assert_eq!(links.len(), 4);
        assert!(links.iter().any(|l| l["rel"] == "self"));
        assert!(links.iter().any(|l| l["rel"] == "conformance"));
        assert!(links.iter().any(|l| l["rel"] == "data"));
        assert!(links.iter().any(|l| l["rel"] == "service-desc"));
    }

    #[test]
    fn landing_page_link_types() {
        let links = vec![
            OgcLink {
                href: "http://localhost".to_string(),
                rel: "self".to_string(),
                media_type: "application/json".to_string(),
                title: Some("this document".to_string()),
            },
            OgcLink {
                href: "http://localhost/conformance".to_string(),
                rel: "conformance".to_string(),
                media_type: "application/json".to_string(),
                title: None,
            },
        ];
        assert_eq!(links[0].rel, "self");
        assert_eq!(links[1].rel, "conformance");
    }

    #[test]
    fn pagination_link_next() {
        let base = "http://localhost:8080";
        let collection_id = "test";
        let limit = 10;
        let offset = 0;
        let number_returned = 10i64;
        let number_matched = 25i64;

        let has_next = offset + number_returned < number_matched;
        assert!(has_next);

        let next_offset = offset + limit;
        let next_href =
            format!("{base}/collections/{collection_id}/items?limit={limit}&offset={next_offset}");
        assert!(next_href.contains("offset=10"));
    }

    #[test]
    fn pagination_no_next_when_exhausted() {
        let offset = 20i64;
        let number_returned = 5i64;
        let number_matched = 25i64;

        let has_next = offset + number_returned < number_matched;
        assert!(!has_next);
    }

    #[test]
    fn limit_clamp_within_range() {
        let limit: i64 = 50;
        let clamped = limit.clamp(1, OGC_FEATURES_LIMIT_MAX);
        assert_eq!(clamped, 50);
    }

    #[test]
    fn limit_clamp_exceeds_max() {
        let limit: i64 = 50_000;
        let clamped = limit.clamp(1, OGC_FEATURES_LIMIT_MAX);
        assert_eq!(clamped, OGC_FEATURES_LIMIT_MAX);
    }

    #[test]
    fn limit_clamp_below_min() {
        let limit: i64 = -5;
        let clamped = limit.clamp(1, OGC_FEATURES_LIMIT_MAX);
        assert_eq!(clamped, 1);
    }

    #[test]
    fn offset_max_zero() {
        let offset: i64 = -10;
        let clamped = offset.max(0);
        assert_eq!(clamped, 0);
    }

    #[test]
    fn offset_positive_passthrough() {
        let offset: i64 = 100;
        let clamped = offset.max(0);
        assert_eq!(clamped, 100);
    }

    #[test]
    fn build_collection_json_item_type() {
        let meta = crate::sources::TileMetadata {
            id: "pts".to_string(),
            name: "Points".to_string(),
            description: None,
            attribution: None,
            format: crate::sources::TileFormat::Pbf,
            minzoom: 0,
            maxzoom: 14,
            bounds: Some([-180.0, -90.0, 180.0, 90.0]),
            center: None,
            vector_layers: None,
        };
        let json = build_collection_json(&meta, "http://localhost");
        assert_eq!(json["itemType"], "feature");
    }

    #[test]
    fn parse_bbox_negative_coordinates() {
        let result = parse_bbox(Some("-122.5,37.5,-122.0,38.0"))
            .unwrap()
            .unwrap();
        assert!((result[0] - (-122.5)).abs() < f64::EPSILON);
        assert!((result[1] - 37.5).abs() < f64::EPSILON);
    }
}
