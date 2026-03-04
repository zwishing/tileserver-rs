//! MLT (MapLibre Tiles) transcoding module.
//!
//! Provides conversion between MVT (Mapbox Vector Tiles, protobuf)
//! and MLT (MapLibre Tiles) formats. This enables:
//!
//! - **Phase 2**: Serve existing MVT/PBF sources as MLT tiles (MVT→MLT encoding)
//! - **Phase 3**: Serve MLT sources as MVT/PBF for backward compatibility with legacy clients
//!
//! Gated behind the `mlt` cargo feature.

use bytes::Bytes;
use flate2::read::GzDecoder;
use std::io::Read;

use crate::error::{Result, TileServerError};
use crate::sources::{TileCompression, TileData, TileFormat};

// ---------------------------------------------------------------------------
// MVT Protobuf types (minimal prost-generated structs for encoding MVT tiles)
// ---------------------------------------------------------------------------

/// Minimal MVT protobuf types for encoding vector tiles.
///
/// These mirror the Mapbox Vector Tile specification v2.1 protobuf schema.
/// Used only for MLT→MVT reverse transcoding (Phase 3).
#[allow(non_snake_case)]
pub mod MvtProto {
    /// MVT geometry type.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, prost::Enumeration)]
    #[repr(i32)]
    pub enum GeomType {
        Unknown = 0,
        Point = 1,
        Linestring = 2,
        Polygon = 3,
    }

    /// MVT property value.
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Value {
        #[prost(string, optional, tag = "1")]
        pub string_value: Option<String>,
        #[prost(float, optional, tag = "2")]
        pub float_value: Option<f32>,
        #[prost(double, optional, tag = "3")]
        pub double_value: Option<f64>,
        #[prost(int64, optional, tag = "4")]
        pub int_value: Option<i64>,
        #[prost(uint64, optional, tag = "5")]
        pub uint_value: Option<u64>,
        #[prost(sint64, optional, tag = "6")]
        pub sint_value: Option<i64>,
        #[prost(bool, optional, tag = "7")]
        pub bool_value: Option<bool>,
    }

    /// MVT feature within a layer.
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Feature {
        #[prost(uint64, optional, tag = "1")]
        pub id: Option<u64>,
        #[prost(uint32, repeated, packed = "true", tag = "2")]
        pub tags: Vec<u32>,
        #[prost(enumeration = "GeomType", optional, tag = "3")]
        pub r#type: Option<i32>,
        #[prost(uint32, repeated, packed = "true", tag = "4")]
        pub geometry: Vec<u32>,
    }

    /// MVT layer within a tile.
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Layer {
        #[prost(uint32, required, tag = "15")]
        pub version: u32,
        #[prost(string, required, tag = "1")]
        pub name: String,
        #[prost(message, repeated, tag = "2")]
        pub features: Vec<Feature>,
        #[prost(string, repeated, tag = "3")]
        pub keys: Vec<String>,
        #[prost(message, repeated, tag = "4")]
        pub values: Vec<Value>,
        #[prost(uint32, optional, tag = "5")]
        pub extent: Option<u32>,
    }

    /// MVT tile (top-level message).
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Tile {
        #[prost(message, repeated, tag = "3")]
        pub layers: Vec<Layer>,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Transcode a tile from its current format to a target format.
///
/// Handles decompression of source data if needed (gzip).
/// Returns a new `TileData` with the transcoded bytes and appropriate format.
///
/// # Supported conversions
///
/// | From | To  | Description |
/// |------|-----|-------------|
/// | PBF  | MLT | MVT→MLT encoding via mlt-core (Phase 2) |
/// | MLT  | PBF | MLT→MVT decoding (Phase 3) |
///
/// # Errors
///
/// Returns `TileServerError::TranscodeUnsupported` for unsupported format pairs.
/// Returns `TileServerError::MltEncodeError` or `MltDecodeError` on conversion failure.
pub fn transcode_tile(tile: &TileData, target_format: TileFormat) -> Result<TileData> {
    // No-op if formats already match
    if tile.format == target_format {
        return Ok(tile.clone());
    }

    match (tile.format, target_format) {
        (TileFormat::Pbf, TileFormat::Mlt) => {
            // Phase 2: MVT→MLT encoding using mlt-core's encoding API.
            // Wrap in catch_unwind because mlt-core can panic on certain
            // geometries (off-by-one in geometry encoder, see #651).
            let raw = decompress_tile_data(tile)?;
            let mlt_bytes = std::panic::catch_unwind(|| mvt_to_mlt(&raw)).map_err(|panic| {
                let msg = panic
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown panic");
                TileServerError::MltEncodeError(format!(
                    "mlt-core panicked during MVT→MLT encoding: {msg}"
                ))
            })??;
            Ok(TileData {
                data: mlt_bytes,
                format: TileFormat::Mlt,
                compression: TileCompression::None,
            })
        }
        (TileFormat::Mlt, TileFormat::Pbf) => {
            let raw = decompress_tile_data(tile)?;
            let mvt_bytes = mlt_to_mvt(&raw)?;
            Ok(TileData {
                data: mvt_bytes,
                format: TileFormat::Pbf,
                compression: TileCompression::None,
            })
        }
        (from, to) => Err(TileServerError::TranscodeUnsupported {
            from: format!("{from:?}"),
            to: format!("{to:?}"),
        }),
    }
}

// ---------------------------------------------------------------------------
// Internal: Decompression
// ---------------------------------------------------------------------------

/// Decompress tile data if compressed, returning raw bytes.
fn decompress_tile_data(tile: &TileData) -> Result<Vec<u8>> {
    match tile.compression {
        TileCompression::None => Ok(tile.data.to_vec()),
        TileCompression::Gzip => {
            let mut decoder = GzDecoder::new(tile.data.as_ref());
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                TileServerError::MltDecodeError(format!("gzip decompression failed: {e}"))
            })?;
            Ok(decompressed)
        }
        _ => Err(TileServerError::MltDecodeError(format!(
            "{:?} decompression not supported for transcoding",
            tile.compression
        ))),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: MVT → MLT encoding
// ---------------------------------------------------------------------------

/// Convert MVT (protobuf) bytes to MLT format.
///
/// Uses `mlt-core` to:
/// 1. Parse MVT binary into a `FeatureCollection`
/// 2. Group features by layer (via `_layer` property)
/// 3. Build decoded geometry, IDs, and column-oriented properties per layer
/// 4. Encode each column using mlt-core's encoding API
/// 5. Serialize encoded layers to MLT wire format
fn mvt_to_mlt(mvt_bytes: &[u8]) -> Result<Bytes> {
    use std::collections::BTreeMap;

    // Step 1: Parse MVT protobuf into FeatureCollection
    let fc = mlt_core::mvt::mvt_to_feature_collection(mvt_bytes.to_vec())
        .map_err(|e| TileServerError::MltEncodeError(format!("failed to parse MVT tile: {e}")))?;

    // Step 2: Group features by layer name
    let mut layer_map: BTreeMap<String, Vec<&mlt_core::geojson::Feature>> = BTreeMap::new();
    for feature in &fc.features {
        let layer_name = feature
            .properties
            .get("_layer")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        layer_map.entry(layer_name).or_default().push(feature);
    }

    // Step 3: Build and encode each layer, write to output buffer
    let mut output = Vec::new();
    for (layer_name, features) in &layer_map {
        let layer = build_mlt_layer(layer_name, features)?;
        layer.write_to(&mut output).map_err(|e| {
            TileServerError::MltEncodeError(format!("failed to write MLT layer: {e}"))
        })?;
    }

    Ok(Bytes::from(output))
}

/// Build an encoded `OwnedLayer` from a set of features belonging to one MVT layer.
fn build_mlt_layer(
    layer_name: &str,
    features: &[&mlt_core::geojson::Feature],
) -> Result<mlt_core::OwnedLayer> {
    use mlt_core::v01::{
        DecodedGeometry, DecodedId, Encoder, GeometryEncoder, IdEncoder, IdWidth, LogicalEncoder,
        OwnedGeometry, OwnedId, OwnedLayer01, OwnedProperty, PhysicalEncoder, PresenceStream,
        PropertyEncoder,
    };
    use mlt_core::Encodable as _;

    // Extract extent from first feature (injected by mvt_to_feature_collection as _extent)
    let extent = features
        .first()
        .and_then(|f| f.properties.get("_extent"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(4096);

    // --- Geometry ---
    let mut decoded_geom = DecodedGeometry::default();
    for feature in features {
        decoded_geom.push_geom(&feature.geometry);
    }
    let mut geometry = OwnedGeometry::Decoded(decoded_geom);
    let geom_encoder = GeometryEncoder::all(Encoder::varint());
    geometry.encode_with(geom_encoder).map_err(|e| {
        TileServerError::MltEncodeError(format!("failed to encode MLT geometry: {e}"))
    })?;

    // --- IDs ---
    let ids: Vec<Option<u64>> = features.iter().map(|f| f.id).collect();
    let has_ids = ids.iter().any(|id| id.is_some());
    let mut id = if has_ids {
        OwnedId::Decoded(DecodedId(Some(ids)))
    } else {
        OwnedId::None
    };
    if has_ids {
        let id_encoder = IdEncoder::new(LogicalEncoder::None, IdWidth::Id64);
        id.encode_with(id_encoder).map_err(|e| {
            TileServerError::MltEncodeError(format!("failed to encode MLT IDs: {e}"))
        })?;
    }

    // --- Properties (row-oriented → column-oriented) ---
    let properties = build_column_properties(features)?;
    let prop_encoder = PropertyEncoder::new(
        PresenceStream::Present,
        LogicalEncoder::None,
        PhysicalEncoder::VarInt,
    );
    let mut encoded_properties: Vec<OwnedProperty> = Vec::with_capacity(properties.len());
    for decoded_prop in properties {
        let mut prop = OwnedProperty::Decoded(decoded_prop);
        prop.encode_with(prop_encoder).map_err(|e| {
            TileServerError::MltEncodeError(format!("failed to encode MLT property: {e}"))
        })?;
        encoded_properties.push(prop);
    }

    // --- Assemble Layer ---
    let layer01 = OwnedLayer01 {
        name: layer_name.to_string(),
        extent,
        id,
        geometry,
        properties: encoded_properties,
    };

    Ok(mlt_core::OwnedLayer::Tag01(layer01))
}

/// Convert row-oriented feature properties to column-oriented `DecodedProperty` vectors.
///
/// MVT stores properties per-feature (row-oriented), but MLT stores them per-column.
/// This function collects all unique property keys, infers their types, and builds
/// a `PropValue` vector for each key across all features.
fn build_column_properties(
    features: &[&mlt_core::geojson::Feature],
) -> Result<Vec<mlt_core::v01::DecodedProperty>> {
    use mlt_core::v01::DecodedProperty;
    use std::collections::BTreeSet;

    // Collect all unique property keys (excluding internal _layer, _extent)
    let mut all_keys = BTreeSet::new();
    for feature in features {
        for key in feature.properties.keys() {
            if !key.starts_with('_') {
                all_keys.insert(key.clone());
            }
        }
    }

    let num_features = features.len();
    let mut result = Vec::with_capacity(all_keys.len());

    for key in &all_keys {
        // Determine the dominant type for this key by scanning feature values
        let prop_value = infer_column_values(features, key, num_features);
        result.push(DecodedProperty {
            name: key.clone(),
            values: prop_value,
        });
    }

    Ok(result)
}

/// Infer the column type and build a `PropValue` vector for a single property key.
///
/// Scans all features to determine the best type (String, i64, f64, bool),
/// inserting `None` for features where the property is absent.
fn infer_column_values(
    features: &[&mlt_core::geojson::Feature],
    key: &str,
    _num_features: usize,
) -> mlt_core::v01::PropValue {
    use mlt_core::v01::PropValue;

    // First pass: determine dominant type
    let mut has_string = false;
    let mut has_float = false;
    let mut has_int = false;
    let mut has_bool = false;

    for feature in features {
        if let Some(val) = feature.properties.get(key) {
            match val {
                serde_json::Value::String(_) => has_string = true,
                serde_json::Value::Bool(_) => has_bool = true,
                serde_json::Value::Number(n) => {
                    if n.is_f64() && !n.is_i64() && !n.is_u64() {
                        has_float = true;
                    } else {
                        has_int = true;
                    }
                }
                _ => {}
            }
        }
    }

    // If mixed types exist, prefer string (can represent anything)
    // Otherwise use the most specific type
    if has_string || (!has_int && !has_float && !has_bool) {
        let vals: Vec<Option<String>> = features
            .iter()
            .map(|f| {
                f.properties.get(key).and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Null => None,
                    other => Some(other.to_string()),
                })
            })
            .collect();
        PropValue::Str(vals)
    } else if has_bool && !has_int && !has_float {
        let vals: Vec<Option<bool>> = features
            .iter()
            .map(|f| f.properties.get(key).and_then(|v| v.as_bool()))
            .collect();
        PropValue::Bool(vals)
    } else if has_float {
        let vals: Vec<Option<f64>> = features
            .iter()
            .map(|f| f.properties.get(key).and_then(|v| v.as_f64()))
            .collect();
        PropValue::F64(vals)
    } else {
        // Integer — use i64 since MVT properties can be signed
        let vals: Vec<Option<i64>> = features
            .iter()
            .map(|f| f.properties.get(key).and_then(|v| v.as_i64()))
            .collect();
        PropValue::I64(vals)
    }
}

// ---------------------------------------------------------------------------
// Phase 3: MLT → MVT decoding
// ---------------------------------------------------------------------------

/// Convert MLT bytes to MVT (protobuf) format.
///
/// Uses `mlt-core` to:
/// 1. Parse MLT binary into layers
/// 2. Decode all columns (geometry, properties, IDs)
/// 3. Convert decoded features to intermediate `FeatureCollection`
/// 4. Build MVT protobuf from features using prost
fn mlt_to_mvt(mlt_bytes: &[u8]) -> Result<Bytes> {
    use prost::Message;

    // Step 1: Parse MLT layers (lazy — column data not yet decoded)
    let mut layers = mlt_core::parse_layers(mlt_bytes)
        .map_err(|e| TileServerError::MltDecodeError(format!("failed to parse MLT tile: {e}")))?;

    // Step 2: Decode all columns in each layer
    for layer in &mut layers {
        layer.decode_all().map_err(|e| {
            TileServerError::MltDecodeError(format!("failed to decode MLT layer: {e}"))
        })?;
    }

    // Step 3: Convert decoded MLT layers to FeatureCollection
    let fc = mlt_core::geojson::FeatureCollection::from_layers(&layers).map_err(|e| {
        TileServerError::MltDecodeError(format!("failed to convert MLT layers to features: {e}"))
    })?;

    // Step 4: Build MVT protobuf from FeatureCollection
    let mvt_tile = feature_collection_to_mvt(&fc)?;

    // Step 5: Encode MVT protobuf to bytes
    let encoded = mvt_tile.encode_to_vec();
    Ok(Bytes::from(encoded))
}

/// Convert an MLT `FeatureCollection` to an MVT protobuf `Tile`.
///
/// Maps the GeoJSON-like features (with i32 tile coordinates) back into
/// the MVT protobuf wire format with keys/values interning.
///
/// Features are grouped by the `_layer` property injected by `from_layers`.
fn feature_collection_to_mvt(fc: &mlt_core::geojson::FeatureCollection) -> Result<MvtProto::Tile> {
    use std::collections::HashMap;

    let mut mvt_layers: Vec<MvtProto::Layer> = Vec::new();

    // Group features by layer name (stored in _layer property by from_layers)
    let mut layer_map: HashMap<&str, Vec<&mlt_core::geojson::Feature>> = HashMap::new();
    for feature in &fc.features {
        let layer_name = feature
            .properties
            .get("_layer")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        layer_map.entry(layer_name).or_default().push(feature);
    }

    for (layer_name, features) in &layer_map {
        // Get extent from first feature (injected by from_layers as _extent)
        let extent = features
            .first()
            .and_then(|f| f.properties.get("_extent"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(4096);

        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<MvtProto::Value> = Vec::new();
        let mut key_index: HashMap<String, u32> = HashMap::new();
        let mut value_index: HashMap<String, u32> = HashMap::new();
        let mut mvt_features: Vec<MvtProto::Feature> = Vec::new();

        for feature in features {
            // Encode geometry to MVT command sequence
            let (geom_type, geometry) = encode_geometry_to_mvt(&feature.geometry);

            // Encode properties as interned key/value tags
            // Skip internal properties (_layer, _extent)
            let mut tags = Vec::new();
            for (key, val) in &feature.properties {
                if key.starts_with('_') {
                    continue; // Skip internal properties
                }

                let key_idx = *key_index.entry(key.clone()).or_insert_with(|| {
                    let idx = keys.len() as u32;
                    keys.push(key.clone());
                    idx
                });

                let val_str = format!("{val}");
                let val_idx = *value_index.entry(val_str.clone()).or_insert_with(|| {
                    let idx = values.len() as u32;
                    values.push(json_value_to_mvt(val));
                    idx
                });

                tags.push(key_idx);
                tags.push(val_idx);
            }

            mvt_features.push(MvtProto::Feature {
                id: feature.id,
                tags,
                r#type: Some(geom_type as i32),
                geometry,
            });
        }

        mvt_layers.push(MvtProto::Layer {
            version: 2,
            name: layer_name.to_string(),
            features: mvt_features,
            keys,
            values,
            extent: Some(extent),
        });
    }

    Ok(MvtProto::Tile { layers: mvt_layers })
}

// ---------------------------------------------------------------------------
// MVT Geometry encoding helpers
// ---------------------------------------------------------------------------

/// Encode geo_types Geometry<i32> to MVT command-encoded geometry.
///
/// Returns `(GeomType, Vec<u32>)` with the MVT command sequence.
fn encode_geometry_to_mvt(geometry: &mlt_core::geojson::Geom32) -> (MvtProto::GeomType, Vec<u32>) {
    use geo_types::Geometry;

    match geometry {
        Geometry::Point(point) => {
            let commands = encode_point(point.x(), point.y());
            (MvtProto::GeomType::Point, commands)
        }
        Geometry::MultiPoint(mp) => {
            let points = mp.0.as_slice();
            let mut commands = Vec::with_capacity(points.len() * 3);
            // MoveTo(count)
            commands.push(command_integer(1, points.len() as u32));
            let mut cx = 0i32;
            let mut cy = 0i32;
            for point in points {
                let dx = point.x() - cx;
                let dy = point.y() - cy;
                commands.push(zigzag_encode(dx));
                commands.push(zigzag_encode(dy));
                cx = point.x();
                cy = point.y();
            }
            (MvtProto::GeomType::Point, commands)
        }
        Geometry::LineString(ls) => {
            let commands = encode_linestring(&ls.0, false);
            (MvtProto::GeomType::Linestring, commands)
        }
        Geometry::MultiLineString(mls) => {
            let mut commands = Vec::new();
            for line in &mls.0 {
                commands.extend(encode_linestring(&line.0, false));
            }
            (MvtProto::GeomType::Linestring, commands)
        }
        Geometry::Polygon(poly) => {
            let mut commands = Vec::new();
            // Exterior ring
            commands.extend(encode_linestring(&poly.exterior().0, true));
            // Interior rings (holes)
            for ring in poly.interiors() {
                commands.extend(encode_linestring(&ring.0, true));
            }
            (MvtProto::GeomType::Polygon, commands)
        }
        Geometry::MultiPolygon(mp) => {
            let mut commands = Vec::new();
            for polygon in &mp.0 {
                commands.extend(encode_linestring(&polygon.exterior().0, true));
                for ring in polygon.interiors() {
                    commands.extend(encode_linestring(&ring.0, true));
                }
            }
            (MvtProto::GeomType::Polygon, commands)
        }
        // Unsupported geometry types produce empty geometry
        _ => (MvtProto::GeomType::Unknown, Vec::new()),
    }
}

/// Encode a single point as MVT commands: MoveTo(1) + dx, dy.
fn encode_point(x: i32, y: i32) -> Vec<u32> {
    vec![
        command_integer(1, 1), // MoveTo, count=1
        zigzag_encode(x),
        zigzag_encode(y),
    ]
}

/// Encode a linestring/ring as MVT commands.
///
/// If `close_path` is true, appends ClosePath command (for polygon rings).
fn encode_linestring(coords: &[geo_types::Coord<i32>], close_path: bool) -> Vec<u32> {
    if coords.is_empty() {
        return Vec::new();
    }

    let mut commands = Vec::with_capacity(coords.len() * 2 + 4);
    let mut cx = 0i32;
    let mut cy = 0i32;

    // MoveTo first point
    commands.push(command_integer(1, 1));
    let dx = coords[0].x - cx;
    let dy = coords[0].y - cy;
    commands.push(zigzag_encode(dx));
    commands.push(zigzag_encode(dy));
    cx = coords[0].x;
    cy = coords[0].y;

    // LineTo remaining points
    let line_count = if close_path {
        // For closed rings, skip the last point (ClosePath handles it)
        coords.len().saturating_sub(2)
    } else {
        coords.len() - 1
    };

    if line_count > 0 {
        commands.push(command_integer(2, line_count as u32));
        let end = if close_path {
            coords.len() - 1
        } else {
            coords.len()
        };
        for coord in &coords[1..end] {
            let dx = coord.x - cx;
            let dy = coord.y - cy;
            commands.push(zigzag_encode(dx));
            commands.push(zigzag_encode(dy));
            cx = coord.x;
            cy = coord.y;
        }
    }

    // ClosePath for polygon rings
    if close_path {
        commands.push(command_integer(7, 1));
    }

    commands
}

/// Encode an MVT command integer: `(id & 0x7) | (count << 3)`.
#[inline]
fn command_integer(id: u32, count: u32) -> u32 {
    (id & 0x7) | (count << 3)
}

/// Zigzag-encode a signed integer for protobuf.
#[inline]
fn zigzag_encode(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

// ---------------------------------------------------------------------------
// MVT Value encoding helpers
// ---------------------------------------------------------------------------

/// Convert a serde_json::Value to an MVT protobuf Value.
fn json_value_to_mvt(val: &serde_json::Value) -> MvtProto::Value {
    match val {
        serde_json::Value::String(s) => MvtProto::Value {
            string_value: Some(s.clone()),
            ..Default::default()
        },
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                MvtProto::Value {
                    int_value: Some(i),
                    ..Default::default()
                }
            } else if let Some(u) = n.as_u64() {
                MvtProto::Value {
                    uint_value: Some(u),
                    ..Default::default()
                }
            } else if let Some(f) = n.as_f64() {
                MvtProto::Value {
                    double_value: Some(f),
                    ..Default::default()
                }
            } else {
                MvtProto::Value::default()
            }
        }
        serde_json::Value::Bool(b) => MvtProto::Value {
            bool_value: Some(*b),
            ..Default::default()
        },
        _ => MvtProto::Value::default(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_encode() {
        assert_eq!(zigzag_encode(0), 0);
        assert_eq!(zigzag_encode(-1), 1);
        assert_eq!(zigzag_encode(1), 2);
        assert_eq!(zigzag_encode(-2), 3);
        assert_eq!(zigzag_encode(2), 4);
    }

    #[test]
    fn test_command_integer() {
        // MoveTo, count=1
        assert_eq!(command_integer(1, 1), 9);
        // LineTo, count=3
        assert_eq!(command_integer(2, 3), 26);
        // ClosePath, count=1
        assert_eq!(command_integer(7, 1), 15);
    }

    #[test]
    fn test_encode_point() {
        let commands = encode_point(25, 17);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], command_integer(1, 1)); // MoveTo(1)
        assert_eq!(commands[1], zigzag_encode(25)); // dx=25
        assert_eq!(commands[2], zigzag_encode(17)); // dy=17
    }

    #[test]
    fn test_transcode_same_format_is_noop() {
        let tile = TileData {
            data: Bytes::from_static(b"test"),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Pbf).unwrap();
        assert_eq!(result.data, tile.data);
        assert_eq!(result.format, TileFormat::Pbf);
    }

    #[test]
    fn test_transcode_unsupported_pair() {
        let tile = TileData {
            data: Bytes::from_static(b"test"),
            format: TileFormat::Png,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt);
        assert!(result.is_err());
        match result.unwrap_err() {
            TileServerError::TranscodeUnsupported { from, to } => {
                assert_eq!(from, "Png");
                assert_eq!(to, "Mlt");
            }
            e => panic!("expected TranscodeUnsupported, got: {e:?}"),
        }
    }

    #[test]
    fn test_mvt_to_mlt_invalid_input_returns_error() {
        // Invalid protobuf bytes should return an encode error, not panic
        let tile = TileData {
            data: Bytes::from_static(b"not valid protobuf"),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TileServerError::MltEncodeError(_)
        ));
    }

    #[test]
    fn test_mvt_to_mlt_catches_panic_from_mlt_core() {
        // Craft a tile with valid protobuf structure but geometry that
        // could trigger an mlt-core panic. Even if mlt-core panics,
        // transcode_tile should return an error, not crash the thread.
        // We verify the catch_unwind wrapper by ensuring any failure
        // on malformed geometry returns Err, not a panic propagation.
        use prost::Message;
        let tile_proto = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "test".to_string(),
                features: vec![MvtProto::Feature {
                    id: Some(1),
                    tags: vec![],
                    r#type: Some(3), // POLYGON
                    // Malformed geometry: ClosePath without preceding MoveTo/LineTo
                    geometry: vec![
                        command_integer(7, 1), // ClosePath x1 (invalid without MoveTo)
                    ],
                }],
                keys: vec![],
                values: vec![],
                extent: Some(4096),
            }],
        };
        let mut mvt_bytes = Vec::new();
        tile_proto.encode(&mut mvt_bytes).unwrap();

        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        // Should not panic — either succeeds or returns an error
        let result = transcode_tile(&tile, TileFormat::Mlt);
        // We don't assert is_err() because mlt-core may handle this
        // gracefully; the key assertion is that we reach this line
        // (no panic propagation).
        let _ = result;
    }

    #[test]
    fn test_json_value_to_mvt_string() {
        let val = serde_json::json!("hello");
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.string_value, Some("hello".to_string()));
    }

    #[test]
    fn test_json_value_to_mvt_number() {
        let val = serde_json::json!(42);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.int_value, Some(42));
    }

    #[test]
    fn test_json_value_to_mvt_bool() {
        let val = serde_json::json!(true);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.bool_value, Some(true));
    }

    // -------------------------------------------------------------------------
    // Helper: Build a valid MVT protobuf tile from layers of features
    // -------------------------------------------------------------------------

    /// Build a minimal valid MVT tile with one layer, one point feature.
    fn make_mvt_point_tile(layer_name: &str, x: i32, y: i32) -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: layer_name.to_string(),
                features: vec![MvtProto::Feature {
                    id: Some(1),
                    tags: vec![0, 0], // key[0] = "name", value[0] = "test"
                    r#type: Some(MvtProto::GeomType::Point as i32),
                    geometry: vec![
                        command_integer(1, 1), // MoveTo(1)
                        zigzag_encode(x),
                        zigzag_encode(y),
                    ],
                }],
                keys: vec!["name".to_string()],
                values: vec![MvtProto::Value {
                    string_value: Some("test".to_string()),
                    ..Default::default()
                }],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build a multi-feature MVT layer with various geometry types.
    fn make_mvt_multi_feature_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "buildings".to_string(),
                features: vec![
                    // Feature 1: Point
                    MvtProto::Feature {
                        id: Some(1),
                        tags: vec![0, 0, 1, 1], // name=building_a, height=10
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![
                            command_integer(1, 1),
                            zigzag_encode(100),
                            zigzag_encode(200),
                        ],
                    },
                    // Feature 2: Point
                    MvtProto::Feature {
                        id: Some(2),
                        tags: vec![0, 2, 1, 3], // name=building_b, height=25
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![
                            command_integer(1, 1),
                            zigzag_encode(300),
                            zigzag_encode(400),
                        ],
                    },
                    // Feature 3: Point with no ID
                    MvtProto::Feature {
                        id: None,
                        tags: vec![0, 4], // name=building_c
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![
                            command_integer(1, 1),
                            zigzag_encode(500),
                            zigzag_encode(600),
                        ],
                    },
                ],
                keys: vec!["name".to_string(), "height".to_string()],
                values: vec![
                    MvtProto::Value {
                        string_value: Some("building_a".to_string()),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        int_value: Some(10),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        string_value: Some("building_b".to_string()),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        int_value: Some(25),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        string_value: Some("building_c".to_string()),
                        ..Default::default()
                    },
                ],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with multiple layers.
    fn make_mvt_multi_layer_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![
                MvtProto::Layer {
                    version: 2,
                    name: "roads".to_string(),
                    features: vec![MvtProto::Feature {
                        id: Some(1),
                        tags: vec![0, 0],
                        r#type: Some(MvtProto::GeomType::Linestring as i32),
                        geometry: vec![
                            command_integer(1, 1), // MoveTo
                            zigzag_encode(0),
                            zigzag_encode(0),
                            command_integer(2, 1), // LineTo(1)
                            zigzag_encode(100),
                            zigzag_encode(0),
                        ],
                    }],
                    keys: vec!["class".to_string()],
                    values: vec![MvtProto::Value {
                        string_value: Some("highway".to_string()),
                        ..Default::default()
                    }],
                    extent: Some(4096),
                },
                MvtProto::Layer {
                    version: 2,
                    name: "water".to_string(),
                    features: vec![MvtProto::Feature {
                        id: Some(10),
                        tags: vec![0, 0],
                        r#type: Some(MvtProto::GeomType::Polygon as i32),
                        geometry: vec![
                            command_integer(1, 1), // MoveTo
                            zigzag_encode(10),
                            zigzag_encode(10),
                            command_integer(2, 3), // LineTo(3)
                            zigzag_encode(100),
                            zigzag_encode(0),
                            zigzag_encode(0),
                            zigzag_encode(100),
                            zigzag_encode(-100),
                            zigzag_encode(0),
                            command_integer(7, 1), // ClosePath
                        ],
                    }],
                    keys: vec!["type".to_string()],
                    values: vec![MvtProto::Value {
                        string_value: Some("lake".to_string()),
                        ..Default::default()
                    }],
                    extent: Some(4096),
                },
            ],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with an empty layer (no features).
    fn make_mvt_empty_layer_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "empty".to_string(),
                features: vec![],
                keys: vec![],
                values: vec![],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with completely empty layers list.
    fn make_mvt_no_layers_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile { layers: vec![] };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with mixed property types for testing type inference.
    fn make_mvt_mixed_props_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "mixed".to_string(),
                features: vec![
                    MvtProto::Feature {
                        id: Some(1),
                        tags: vec![0, 0, 1, 1, 2, 2], // str, int, bool
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(10), zigzag_encode(20)],
                    },
                    MvtProto::Feature {
                        id: Some(2),
                        tags: vec![0, 3, 1, 4, 2, 5], // str, float, bool
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(30), zigzag_encode(40)],
                    },
                ],
                keys: vec![
                    "label".to_string(),
                    "value".to_string(),
                    "active".to_string(),
                ],
                values: vec![
                    MvtProto::Value {
                        string_value: Some("alpha".to_string()),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        int_value: Some(42),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        bool_value: Some(true),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        string_value: Some("beta".to_string()),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        double_value: Some(2.72),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        bool_value: Some(false),
                        ..Default::default()
                    },
                ],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with linestring geometry.
    fn make_mvt_linestring_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "lines".to_string(),
                features: vec![MvtProto::Feature {
                    id: Some(1),
                    tags: vec![],
                    r#type: Some(MvtProto::GeomType::Linestring as i32),
                    geometry: vec![
                        command_integer(1, 1), // MoveTo(1)
                        zigzag_encode(0),
                        zigzag_encode(0),
                        command_integer(2, 2), // LineTo(2)
                        zigzag_encode(100),
                        zigzag_encode(0),
                        zigzag_encode(0),
                        zigzag_encode(100),
                    ],
                }],
                keys: vec![],
                values: vec![],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with polygon geometry (exterior + interior ring).
    fn make_mvt_polygon_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "polygons".to_string(),
                features: vec![MvtProto::Feature {
                    id: Some(1),
                    tags: vec![0, 0],
                    r#type: Some(MvtProto::GeomType::Polygon as i32),
                    geometry: vec![
                        // Exterior ring: square (0,0)-(100,0)-(100,100)-(0,100)
                        command_integer(1, 1), // MoveTo(1)
                        zigzag_encode(0),
                        zigzag_encode(0),
                        command_integer(2, 3), // LineTo(3)
                        zigzag_encode(100),
                        zigzag_encode(0),
                        zigzag_encode(0),
                        zigzag_encode(100),
                        zigzag_encode(-100),
                        zigzag_encode(0),
                        command_integer(7, 1), // ClosePath
                    ],
                }],
                keys: vec!["kind".to_string()],
                values: vec![MvtProto::Value {
                    string_value: Some("park".to_string()),
                    ..Default::default()
                }],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with features missing some properties (sparse columns).
    fn make_mvt_sparse_props_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "sparse".to_string(),
                features: vec![
                    // Feature 1: has both 'name' and 'pop'
                    MvtProto::Feature {
                        id: Some(1),
                        tags: vec![0, 0, 1, 1], // name="city_a", pop=1000
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(10), zigzag_encode(20)],
                    },
                    // Feature 2: has only 'name' (missing 'pop')
                    MvtProto::Feature {
                        id: Some(2),
                        tags: vec![0, 2], // name="city_b"
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(30), zigzag_encode(40)],
                    },
                    // Feature 3: has only 'pop' (missing 'name')
                    MvtProto::Feature {
                        id: Some(3),
                        tags: vec![1, 3], // pop=5000
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(50), zigzag_encode(60)],
                    },
                ],
                keys: vec!["name".to_string(), "pop".to_string()],
                values: vec![
                    MvtProto::Value {
                        string_value: Some("city_a".to_string()),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        int_value: Some(1000),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        string_value: Some("city_b".to_string()),
                        ..Default::default()
                    },
                    MvtProto::Value {
                        int_value: Some(5000),
                        ..Default::default()
                    },
                ],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    /// Build an MVT tile with large numeric IDs.
    fn make_mvt_large_id_tile() -> Vec<u8> {
        use prost::Message;
        let tile = MvtProto::Tile {
            layers: vec![MvtProto::Layer {
                version: 2,
                name: "large_ids".to_string(),
                features: vec![
                    MvtProto::Feature {
                        id: Some(u64::MAX),
                        tags: vec![],
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(10), zigzag_encode(20)],
                    },
                    MvtProto::Feature {
                        id: Some(0),
                        tags: vec![],
                        r#type: Some(MvtProto::GeomType::Point as i32),
                        geometry: vec![command_integer(1, 1), zigzag_encode(30), zigzag_encode(40)],
                    },
                ],
                keys: vec![],
                values: vec![],
                extent: Some(4096),
            }],
        };
        tile.encode_to_vec()
    }

    // -------------------------------------------------------------------------
    // MVT → MLT transcoding tests (Phase 2)
    // -------------------------------------------------------------------------

    #[test]
    fn test_mvt_to_mlt_point_tile() {
        let mvt_bytes = make_mvt_point_tile("places", 100, 200);
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt);
        assert!(
            result.is_ok(),
            "MVT→MLT transcoding should succeed: {:?}",
            result.err()
        );
        let mlt_tile = result.unwrap();
        assert_eq!(mlt_tile.format, TileFormat::Mlt);
        assert_eq!(mlt_tile.compression, TileCompression::None);
        assert!(!mlt_tile.data.is_empty(), "MLT output should not be empty");
        // Verify MLT data is valid by parsing it back
        let layers = mlt_core::parse_layers(&mlt_tile.data);
        assert!(
            layers.is_ok(),
            "MLT output should be parseable: {:?}",
            layers.err()
        );
    }

    #[test]
    fn test_mvt_to_mlt_multi_feature_tile() {
        let mvt_bytes = make_mvt_multi_feature_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
        // Verify MLT is parseable
        let layers = mlt_core::parse_layers(&result.data);
        assert!(
            layers.is_ok(),
            "MLT output should parse: {:?}",
            layers.err()
        );
    }

    #[test]
    fn test_mvt_to_mlt_multi_layer_tile() {
        let mvt_bytes = make_mvt_multi_layer_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
        // Verify we can parse back and get 2 layers
        let mut layers = mlt_core::parse_layers(&result.data).unwrap();
        assert_eq!(layers.len(), 2, "Should have 2 layers (roads + water)");
        // Verify layer names
        for layer in &mut layers {
            layer.decode_all().unwrap();
        }
    }

    #[test]
    fn test_mvt_to_mlt_empty_layer() {
        let mvt_bytes = make_mvt_empty_layer_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        // Empty layer might produce minimal output or skip entirely
        let result = transcode_tile(&tile, TileFormat::Mlt);
        // Whether it succeeds or fails is implementation-dependent;
        // the key is it should not panic
        if let Ok(mlt_tile) = result {
            assert_eq!(mlt_tile.format, TileFormat::Mlt);
        }
    }

    #[test]
    fn test_mvt_to_mlt_no_layers() {
        let mvt_bytes = make_mvt_no_layers_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt);
        // Empty tile should produce empty or minimal MLT output
        if let Ok(mlt_tile) = result {
            assert_eq!(mlt_tile.format, TileFormat::Mlt);
        }
    }

    #[test]
    fn test_mvt_to_mlt_linestring() {
        let mvt_bytes = make_mvt_linestring_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
        let layers = mlt_core::parse_layers(&result.data);
        assert!(layers.is_ok());
    }

    #[test]
    fn test_mvt_to_mlt_polygon() {
        let mvt_bytes = make_mvt_polygon_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
        let layers = mlt_core::parse_layers(&result.data);
        assert!(layers.is_ok());
    }

    #[test]
    fn test_mvt_to_mlt_mixed_property_types() {
        let mvt_bytes = make_mvt_mixed_props_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
        let layers = mlt_core::parse_layers(&result.data);
        assert!(layers.is_ok());
    }

    #[test]
    fn test_mvt_to_mlt_sparse_properties() {
        // Features with missing properties should produce None in column vectors
        let mvt_bytes = make_mvt_sparse_props_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
        let layers = mlt_core::parse_layers(&result.data);
        assert!(layers.is_ok());
    }

    #[test]
    fn test_mvt_to_mlt_large_ids() {
        let mvt_bytes = make_mvt_large_id_tile();
        let tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_mvt_to_mlt_gzip_compressed_input() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mvt_bytes = make_mvt_point_tile("compressed", 50, 50);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&mvt_bytes).unwrap();
        let compressed = encoder.finish().unwrap();

        let tile = TileData {
            data: Bytes::from(compressed),
            format: TileFormat::Pbf,
            compression: TileCompression::Gzip,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        assert_eq!(result.format, TileFormat::Mlt);
        assert_eq!(result.compression, TileCompression::None);
        assert!(!result.data.is_empty());
        let layers = mlt_core::parse_layers(&result.data);
        assert!(layers.is_ok());
    }

    #[test]
    fn test_mvt_to_mlt_output_differs_from_input() {
        // Ensure the transcoded output is actually different from the MVT input
        let mvt_bytes = make_mvt_point_tile("test_layer", 100, 200);
        let tile = TileData {
            data: Bytes::from(mvt_bytes.clone()),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt).unwrap();
        // MLT format is structurally different from MVT protobuf
        assert_ne!(
            result.data.as_ref(),
            mvt_bytes.as_slice(),
            "MLT output should differ from MVT input"
        );
    }

    // -------------------------------------------------------------------------
    // MVT → MLT → MVT roundtrip tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_roundtrip_mvt_mlt_mvt_point() {
        let mvt_bytes = make_mvt_point_tile("roundtrip", 100, 200);
        let original_tile = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        // MVT → MLT
        let mlt_tile = transcode_tile(&original_tile, TileFormat::Mlt).unwrap();
        assert_eq!(mlt_tile.format, TileFormat::Mlt);
        // MLT → MVT
        let roundtripped = transcode_tile(&mlt_tile, TileFormat::Pbf).unwrap();
        assert_eq!(roundtripped.format, TileFormat::Pbf);
        assert!(!roundtripped.data.is_empty());
    }

    #[test]
    fn test_roundtrip_mvt_mlt_mvt_multi_feature() {
        let mvt_bytes = make_mvt_multi_feature_tile();
        let original = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let mlt = transcode_tile(&original, TileFormat::Mlt).unwrap();
        let roundtripped = transcode_tile(&mlt, TileFormat::Pbf).unwrap();
        assert_eq!(roundtripped.format, TileFormat::Pbf);
        assert!(!roundtripped.data.is_empty());
    }

    #[test]
    fn test_roundtrip_mvt_mlt_mvt_multi_layer() {
        let mvt_bytes = make_mvt_multi_layer_tile();
        let original = TileData {
            data: Bytes::from(mvt_bytes),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let mlt = transcode_tile(&original, TileFormat::Mlt).unwrap();
        let roundtripped = transcode_tile(&mlt, TileFormat::Pbf).unwrap();
        assert_eq!(roundtripped.format, TileFormat::Pbf);
        // Verify roundtripped tile has valid protobuf structure
        use prost::Message;
        let tile = MvtProto::Tile::decode(roundtripped.data.as_ref());
        assert!(tile.is_ok(), "Roundtripped MVT should be valid protobuf");
        let tile = tile.unwrap();
        assert_eq!(tile.layers.len(), 2, "Should preserve 2 layers");
    }

    #[test]
    fn test_roundtrip_preserves_layer_count() {
        let mvt_bytes = make_mvt_multi_layer_tile();
        let original = TileData {
            data: Bytes::from(mvt_bytes.clone()),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        // Parse original MVT
        use prost::Message;
        let orig_tile = MvtProto::Tile::decode(mvt_bytes.as_slice()).unwrap();
        // Roundtrip
        let mlt = transcode_tile(&original, TileFormat::Mlt).unwrap();
        let roundtripped = transcode_tile(&mlt, TileFormat::Pbf).unwrap();
        let rt_tile = MvtProto::Tile::decode(roundtripped.data.as_ref()).unwrap();
        assert_eq!(
            orig_tile.layers.len(),
            rt_tile.layers.len(),
            "Roundtrip should preserve layer count"
        );
    }

    #[test]
    fn test_roundtrip_preserves_feature_count() {
        let mvt_bytes = make_mvt_multi_feature_tile();
        let original = TileData {
            data: Bytes::from(mvt_bytes.clone()),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        use prost::Message;
        let orig_tile = MvtProto::Tile::decode(mvt_bytes.as_slice()).unwrap();
        let orig_feature_count: usize = orig_tile.layers.iter().map(|l| l.features.len()).sum();
        // Roundtrip
        let mlt = transcode_tile(&original, TileFormat::Mlt).unwrap();
        let roundtripped = transcode_tile(&mlt, TileFormat::Pbf).unwrap();
        let rt_tile = MvtProto::Tile::decode(roundtripped.data.as_ref()).unwrap();
        let rt_feature_count: usize = rt_tile.layers.iter().map(|l| l.features.len()).sum();
        assert_eq!(
            orig_feature_count, rt_feature_count,
            "Roundtrip should preserve total feature count"
        );
    }

    // -------------------------------------------------------------------------
    // Internal function tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mvt_to_mlt_internal_single_point() {
        let mvt_bytes = make_mvt_point_tile("internal_test", 42, 84);
        let result = mvt_to_mlt(&mvt_bytes);
        assert!(
            result.is_ok(),
            "mvt_to_mlt should succeed: {:?}",
            result.err()
        );
        let mlt_bytes = result.unwrap();
        assert!(!mlt_bytes.is_empty());
    }

    #[test]
    fn test_mvt_to_mlt_internal_empty_input() {
        // Empty protobuf encodes as zero bytes
        let result = mvt_to_mlt(&[]);
        // Should either succeed (empty tile) or give a clean error
        if let Ok(mlt_bytes) = result {
            // Empty tile might produce empty output
            let _ = mlt_bytes;
        }
    }

    #[test]
    fn test_decompress_tile_data_none() {
        let tile = TileData {
            data: Bytes::from_static(b"raw bytes"),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = decompress_tile_data(&tile).unwrap();
        assert_eq!(result, b"raw bytes");
    }

    #[test]
    fn test_decompress_tile_data_gzip() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let original = b"hello world";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let tile = TileData {
            data: Bytes::from(compressed),
            format: TileFormat::Pbf,
            compression: TileCompression::Gzip,
        };
        let result = decompress_tile_data(&tile).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_decompress_tile_data_invalid_gzip() {
        let tile = TileData {
            data: Bytes::from_static(b"not gzip data"),
            format: TileFormat::Pbf,
            compression: TileCompression::Gzip,
        };
        let result = decompress_tile_data(&tile);
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Geometry encoding edge case tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_encode_point_zero_coords() {
        let commands = encode_point(0, 0);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[1], 0); // zigzag(0) = 0
        assert_eq!(commands[2], 0);
    }

    #[test]
    fn test_encode_point_negative_coords() {
        let commands = encode_point(-50, -75);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[1], zigzag_encode(-50));
        assert_eq!(commands[2], zigzag_encode(-75));
    }

    #[test]
    fn test_encode_linestring_empty() {
        let commands = encode_linestring(&[], false);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_encode_linestring_single_point() {
        let coords = vec![geo_types::Coord { x: 10, y: 20 }];
        let commands = encode_linestring(&coords, false);
        // MoveTo(1) + dx,dy = 3 commands, no LineTo
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], command_integer(1, 1));
    }

    #[test]
    fn test_encode_linestring_two_points() {
        let coords = vec![
            geo_types::Coord { x: 0, y: 0 },
            geo_types::Coord { x: 100, y: 50 },
        ];
        let commands = encode_linestring(&coords, false);
        // MoveTo(1) + dx,dy + LineTo(1) + dx,dy = 6 commands
        assert_eq!(commands.len(), 6);
        assert_eq!(commands[0], command_integer(1, 1)); // MoveTo
        assert_eq!(commands[3], command_integer(2, 1)); // LineTo(1)
    }

    #[test]
    fn test_encode_linestring_closed_ring() {
        // Closed ring: first = last point, ClosePath appended
        let coords = vec![
            geo_types::Coord { x: 0, y: 0 },
            geo_types::Coord { x: 100, y: 0 },
            geo_types::Coord { x: 100, y: 100 },
            geo_types::Coord { x: 0, y: 0 }, // closing point
        ];
        let commands = encode_linestring(&coords, true);
        // MoveTo(1) + dx,dy + LineTo(2) + 2*(dx,dy) + ClosePath = 3+1+4+1 = 9
        let last = *commands.last().unwrap();
        assert_eq!(last, command_integer(7, 1), "Should end with ClosePath");
    }

    #[test]
    fn test_encode_linestring_delta_encoding() {
        // Verify delta encoding: second point should be relative to first
        let coords = vec![
            geo_types::Coord { x: 10, y: 20 },
            geo_types::Coord { x: 30, y: 50 },
        ];
        let commands = encode_linestring(&coords, false);
        // MoveTo first point (absolute): dx=10, dy=20
        assert_eq!(commands[1], zigzag_encode(10));
        assert_eq!(commands[2], zigzag_encode(20));
        // LineTo second point (delta): dx=30-10=20, dy=50-20=30
        assert_eq!(commands[4], zigzag_encode(20));
        assert_eq!(commands[5], zigzag_encode(30));
    }

    // -------------------------------------------------------------------------
    // Zigzag encoding edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_zigzag_encode_min_max() {
        assert_eq!(zigzag_encode(i32::MAX), (u32::MAX - 1));
        assert_eq!(zigzag_encode(i32::MIN), u32::MAX);
    }

    #[test]
    fn test_zigzag_encode_symmetry() {
        // Positive and negative values should alternate
        for i in 0..100 {
            let pos = zigzag_encode(i);
            let neg = zigzag_encode(-i);
            if i == 0 {
                assert_eq!(pos, neg);
            } else {
                assert_eq!(pos, neg + 1, "zigzag({i}) should be zigzag(-{i}) + 1");
            }
        }
    }

    // -------------------------------------------------------------------------
    // Command integer encoding tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_command_integer_all_types() {
        // MoveTo = 1
        assert_eq!(command_integer(1, 1), 0b0000_1001); // 9
        assert_eq!(command_integer(1, 2), 0b0001_0001); // 17
                                                        // LineTo = 2
        assert_eq!(command_integer(2, 1), 0b0000_1010); // 10
        assert_eq!(command_integer(2, 5), 0b0010_1010); // 42
                                                        // ClosePath = 7
        assert_eq!(command_integer(7, 1), 0b0000_1111); // 15
    }

    // -------------------------------------------------------------------------
    // JSON value conversion tests (expanded)
    // -------------------------------------------------------------------------

    #[test]
    fn test_json_value_to_mvt_float() {
        let val = serde_json::json!(2.72);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.double_value, Some(2.72));
    }

    #[test]
    fn test_json_value_to_mvt_uint() {
        // Large unsigned value that doesn't fit i64
        let val = serde_json::json!(u64::MAX);
        let mvt = json_value_to_mvt(&val);
        // serde_json stores u64::MAX as u64, which as_i64 returns None,
        // so it should fall through to as_u64
        assert!(mvt.uint_value.is_some() || mvt.int_value.is_some());
    }

    #[test]
    fn test_json_value_to_mvt_negative_int() {
        let val = serde_json::json!(-42);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.int_value, Some(-42));
    }

    #[test]
    fn test_json_value_to_mvt_null() {
        let val = serde_json::Value::Null;
        let mvt = json_value_to_mvt(&val);
        // Null produces default/empty value
        assert_eq!(mvt, MvtProto::Value::default());
    }

    #[test]
    fn test_json_value_to_mvt_array() {
        // Arrays are not representable in MVT, should produce default
        let val = serde_json::json!([1, 2, 3]);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt, MvtProto::Value::default());
    }

    #[test]
    fn test_json_value_to_mvt_object() {
        // Objects are not representable in MVT, should produce default
        let val = serde_json::json!({"nested": true});
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt, MvtProto::Value::default());
    }

    #[test]
    fn test_json_value_to_mvt_empty_string() {
        let val = serde_json::json!("");
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.string_value, Some(String::new()));
    }

    #[test]
    fn test_json_value_to_mvt_zero() {
        let val = serde_json::json!(0);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.int_value, Some(0));
    }

    #[test]
    fn test_json_value_to_mvt_false() {
        let val = serde_json::json!(false);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.bool_value, Some(false));
    }

    // -------------------------------------------------------------------------
    // Type inference tests (infer_column_values)
    // -------------------------------------------------------------------------

    #[test]
    fn test_infer_column_values_all_strings() {
        let features = [
            make_test_feature(serde_json::json!({"k": "a"})),
            make_test_feature(serde_json::json!({"k": "b"})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::Str(vals) => {
                assert_eq!(vals.len(), 2);
                assert_eq!(vals[0], Some("a".to_string()));
                assert_eq!(vals[1], Some("b".to_string()));
            }
            other => panic!("Expected Str, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_all_ints() {
        let features = [
            make_test_feature(serde_json::json!({"k": 10})),
            make_test_feature(serde_json::json!({"k": 20})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::I64(vals) => {
                assert_eq!(vals, vec![Some(10), Some(20)]);
            }
            other => panic!("Expected I64, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_all_bools() {
        let features = [
            make_test_feature(serde_json::json!({"k": true})),
            make_test_feature(serde_json::json!({"k": false})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::Bool(vals) => {
                assert_eq!(vals, vec![Some(true), Some(false)]);
            }
            other => panic!("Expected Bool, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_all_floats() {
        let features = [
            make_test_feature(serde_json::json!({"k": 1.5})),
            make_test_feature(serde_json::json!({"k": 2.7})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::F64(vals) => {
                assert_eq!(vals, vec![Some(1.5), Some(2.7)]);
            }
            other => panic!("Expected F64, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_mixed_int_float_promotes_to_f64() {
        // When mixing int and float, should promote to f64
        let features = [
            make_test_feature(serde_json::json!({"k": 10})),
            make_test_feature(serde_json::json!({"k": 2.72})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::F64(vals) => {
                assert_eq!(vals.len(), 2);
                assert_eq!(vals[0], Some(10.0));
                assert_eq!(vals[1], Some(2.72));
            }
            other => panic!("Expected F64 for mixed int/float, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_mixed_types_falls_back_to_string() {
        // When mixing string and int, should fall back to String
        let features = [
            make_test_feature(serde_json::json!({"k": "hello"})),
            make_test_feature(serde_json::json!({"k": 42})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::Str(vals) => {
                assert_eq!(vals.len(), 2);
                assert_eq!(vals[0], Some("hello".to_string()));
                assert_eq!(vals[1], Some("42".to_string()));
            }
            other => panic!("Expected Str for mixed types, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_missing_key_produces_none() {
        let features = [
            make_test_feature(serde_json::json!({"k": "present"})),
            make_test_feature(serde_json::json!({"other": "no k"})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::Str(vals) => {
                assert_eq!(vals.len(), 2);
                assert_eq!(vals[0], Some("present".to_string()));
                assert_eq!(vals[1], None);
            }
            other => panic!("Expected Str with None for missing, got: {other:?}"),
        }
    }

    #[test]
    fn test_infer_column_values_null_values() {
        let features = [
            make_test_feature(serde_json::json!({"k": null})),
            make_test_feature(serde_json::json!({"k": "present"})),
        ];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let result = infer_column_values(&refs, "k", 2);
        match result {
            mlt_core::v01::PropValue::Str(vals) => {
                assert_eq!(vals.len(), 2);
                assert_eq!(vals[0], None); // null produces None
                assert_eq!(vals[1], Some("present".to_string()));
            }
            other => panic!("Expected Str with None for null, got: {other:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // build_column_properties tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_column_properties_skips_internal_keys() {
        let features = [make_test_feature(serde_json::json!({
            "_layer": "internal",
            "_extent": 4096,
            "name": "visible"
        }))];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let props = build_column_properties(&refs).unwrap();
        // Should only have "name", not _layer or _extent
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name, "name");
    }

    #[test]
    fn test_build_column_properties_no_properties() {
        let features = [make_test_feature(serde_json::json!({}))];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let props = build_column_properties(&refs).unwrap();
        assert!(props.is_empty());
    }

    #[test]
    fn test_build_column_properties_multiple_keys() {
        let features = [make_test_feature(
            serde_json::json!({"a": 1, "b": "two", "c": true}),
        )];
        let refs: Vec<&mlt_core::geojson::Feature> = features.iter().collect();
        let props = build_column_properties(&refs).unwrap();
        assert_eq!(props.len(), 3);
        // BTreeMap ordering: a, b, c
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    // -------------------------------------------------------------------------
    // MLT → MVT tests (Phase 3, existing path)
    // -------------------------------------------------------------------------

    #[test]
    fn test_mlt_to_mvt_from_valid_mlt() {
        // Create a valid MLT tile via MVT→MLT, then convert back
        let mvt_bytes = make_mvt_point_tile("phase3_test", 50, 75);
        let mlt_bytes = mvt_to_mlt(&mvt_bytes).unwrap();
        let mlt_tile = TileData {
            data: mlt_bytes,
            format: TileFormat::Mlt,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&mlt_tile, TileFormat::Pbf);
        assert!(result.is_ok(), "MLT→MVT should succeed: {:?}", result.err());
        let mvt_tile = result.unwrap();
        assert_eq!(mvt_tile.format, TileFormat::Pbf);
        // Verify it's valid protobuf
        use prost::Message;
        let decoded = MvtProto::Tile::decode(mvt_tile.data.as_ref());
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_mlt_to_mvt_invalid_input() {
        let tile = TileData {
            data: Bytes::from_static(b"not valid MLT"),
            format: TileFormat::Mlt,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Pbf);
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Property helpers for test fixtures
    // -------------------------------------------------------------------------

    /// Create a test Feature with given properties and a dummy point geometry.
    fn make_test_feature(properties: serde_json::Value) -> mlt_core::geojson::Feature {
        use std::collections::BTreeMap;
        let props: BTreeMap<String, serde_json::Value> = match properties {
            serde_json::Value::Object(map) => map.into_iter().collect(),
            _ => BTreeMap::new(),
        };
        mlt_core::geojson::Feature {
            geometry: geo_types::Geometry::Point(geo_types::Point::new(0, 0)),
            id: None,
            properties: props,
            ty: String::new(),
        }
    }
}
