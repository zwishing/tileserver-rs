//! In-process tile cache for all source types.
//!
//! Provides a byte-weighted, TTL-evicting cache backed by `moka`.
//! All source types (PMTiles, MBTiles, etc.) share this cache.

use moka::future::Cache;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use crate::sources::TileData;

/// Cache key: uniquely identifies a tile across all sources.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TileCacheKey {
    pub source_id: Arc<str>,
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

impl Hash for TileCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source_id.hash(state);
        self.z.hash(state);
        self.x.hash(state);
        self.y.hash(state);
    }
}

/// Byte-weighted, TTL-evicting tile cache.
#[derive(Clone)]
pub struct TileCache {
    cache: Cache<TileCacheKey, TileData>,
}

impl TileCache {
    /// Create a new cache capped at `max_size_mb` megabytes with per-entry TTL.
    #[must_use]
    pub fn new(max_size_mb: u64, ttl_seconds: u64) -> Self {
        let max_bytes = max_size_mb * 1024 * 1024;
        let cache = Cache::builder()
            .max_capacity(max_bytes)
            .weigher(|_k: &TileCacheKey, v: &TileData| -> u32 {
                v.data.len().try_into().unwrap_or(u32::MAX)
            })
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();
        Self { cache }
    }

    /// Look up a tile. Returns `None` on miss or after TTL expiry.
    pub async fn get(&self, key: &TileCacheKey) -> Option<TileData> {
        self.cache.get(key).await
    }

    /// Insert a tile. Eviction happens asynchronously in the background.
    pub async fn insert(&self, key: TileCacheKey, value: TileData) {
        self.cache.insert(key, value).await;
    }

    /// Invalidate all entries. Eviction is eventually consistent.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    /// Current number of cached entries (approximate).
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Current weighted size in bytes (approximate).
    #[must_use]
    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }
}

impl std::fmt::Debug for TileCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileCache")
            .field("entry_count", &self.cache.entry_count())
            .field("weighted_size_bytes", &self.cache.weighted_size())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::{TileCompression, TileFormat};
    use bytes::Bytes;

    fn make_tile(size: usize) -> TileData {
        TileData {
            data: Bytes::from(vec![0u8; size]),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        }
    }

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = TileCache::new(1, 3600);
        let key = TileCacheKey {
            source_id: "test".into(),
            z: 14,
            x: 8580,
            y: 5737,
        };
        cache.insert(key.clone(), make_tile(1024)).await;
        let result = cache.get(&key).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().data.len(), 1024);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = TileCache::new(1, 3600);
        let key = TileCacheKey {
            source_id: "test".into(),
            z: 14,
            x: 8580,
            y: 5737,
        };
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_weighted_size() {
        let cache = TileCache::new(10, 3600);
        for i in 0..5 {
            let key = TileCacheKey {
                source_id: "test".into(),
                z: 14,
                x: i,
                y: 0,
            };
            cache.insert(key, make_tile(1000)).await;
        }
        cache.cache.run_pending_tasks().await;
        assert!(cache.weighted_size() >= 4000);
    }

    #[tokio::test]
    async fn test_cache_different_sources_do_not_collide() {
        let cache = TileCache::new(10, 3600);
        let k1 = TileCacheKey {
            source_id: "source-a".into(),
            z: 1,
            x: 0,
            y: 0,
        };
        let k2 = TileCacheKey {
            source_id: "source-b".into(),
            z: 1,
            x: 0,
            y: 0,
        };
        cache.insert(k1.clone(), make_tile(100)).await;
        assert!(
            cache.get(&k2).await.is_none(),
            "different source must not collide"
        );
    }

    #[tokio::test]
    async fn test_cache_invalidate_all() {
        let cache = TileCache::new(10, 3600);
        let key = TileCacheKey {
            source_id: "src".into(),
            z: 0,
            x: 0,
            y: 0,
        };
        cache.insert(key.clone(), make_tile(512)).await;
        cache.cache.run_pending_tasks().await;
        assert!(cache.get(&key).await.is_some());

        cache.invalidate_all();
        cache.cache.run_pending_tasks().await;
        assert!(
            cache.get(&key).await.is_none(),
            "entry should be gone after invalidate_all"
        );
    }
}
