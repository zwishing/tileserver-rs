//! Worker-thread renderer pool for concurrent tile rendering
//!
//! Each worker thread owns its own MapLibre Native instance (EGL context,
//! HeadlessFrontend, Map). Work is distributed via a shared crossbeam channel
//! and results return through tokio oneshot channels.

use std::thread;
use std::time::Duration;

use crossbeam_channel::{self, Sender};
use tokio::sync::oneshot;

use super::native::{MapMode, NativeMap, RenderOptions, RenderedImage, Size};
use crate::error::{Result, TileServerError};

/// Configuration for the renderer pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub tile_size: u32,
    pub pool_size: usize,
    pub render_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            tile_size: 512,
            pool_size: 4,
            render_timeout: Duration::from_secs(30),
        }
    }
}

enum RenderJob {
    Tile {
        style_json: String,
        z: u8,
        x: u32,
        y: u32,
        tile_size: u32,
        scale: f32,
    },
    Static {
        style_json: String,
        options: RenderOptions,
    },
}

enum RenderOutput {
    Tile(Result<Vec<u8>>),
    Static(Result<RenderedImage>),
}

struct RenderRequest {
    job: RenderJob,
    response: oneshot::Sender<RenderOutput>,
}

/// Pool of native MapLibre renderer worker threads.
///
/// Each worker receives jobs from a shared crossbeam channel,
/// creates a fresh NativeMap per render (for EGL context isolation),
/// and sends results back via tokio oneshot channels.
pub struct RendererPool {
    sender: Option<Sender<RenderRequest>>,
    config: PoolConfig,
    max_scale: u8,
    worker_handles: Vec<thread::JoinHandle<()>>,
}

impl RendererPool {
    pub fn new(config: PoolConfig, max_scale: u8) -> Result<Self> {
        super::native::init()?;

        let pool_size = config.pool_size.max(1);
        let (sender, receiver) = crossbeam_channel::unbounded::<RenderRequest>();
        let mut handles = Vec::with_capacity(pool_size);

        for i in 0..pool_size {
            let rx = receiver.clone();
            let handle = thread::Builder::new()
                .name(format!("renderer-{i}"))
                .spawn(move || {
                    tracing::info!(worker = i, "renderer worker started");
                    while let Ok(request) = rx.recv() {
                        let output = match request.job {
                            RenderJob::Tile {
                                ref style_json,
                                z,
                                x,
                                y,
                                tile_size,
                                scale,
                            } => RenderOutput::Tile(execute_tile_render(
                                style_json, z, x, y, tile_size, scale,
                            )),
                            RenderJob::Static {
                                ref style_json,
                                ref options,
                            } => RenderOutput::Static(execute_static_render(style_json, options)),
                        };
                        let _ = request.response.send(output);
                    }
                    tracing::info!(worker = i, "renderer worker stopped");
                })
                .map_err(|e| {
                    TileServerError::RenderError(format!(
                        "failed to spawn renderer thread {i}: {e}"
                    ))
                })?;
            handles.push(handle);
        }

        tracing::info!(
            pool_size,
            tile_size = config.tile_size,
            max_scale,
            timeout_secs = config.render_timeout.as_secs(),
            "renderer pool initialized"
        );

        Ok(Self {
            sender: Some(sender),
            config,
            max_scale,
            worker_handles: handles,
        })
    }

    /// Render a raster tile via a worker thread
    pub async fn render_tile(
        &self,
        style_json: &str,
        z: u8,
        x: u32,
        y: u32,
        scale: u8,
    ) -> Result<Vec<u8>> {
        let scale = scale.min(self.max_scale).max(1);
        let (tx, rx) = oneshot::channel();

        self.dispatch(RenderRequest {
            job: RenderJob::Tile {
                style_json: style_json.to_string(),
                z,
                x,
                y,
                tile_size: self.config.tile_size,
                scale: scale as f32,
            },
            response: tx,
        })?;

        match tokio::time::timeout(self.config.render_timeout, rx).await {
            Ok(Ok(RenderOutput::Tile(result))) => result,
            Ok(Ok(_)) => Err(TileServerError::RenderError(
                "unexpected render output type".to_string(),
            )),
            Ok(Err(_)) => Err(TileServerError::RenderError(
                "renderer worker dropped response".to_string(),
            )),
            Err(_) => Err(TileServerError::RenderError(format!(
                "render timed out after {}s",
                self.config.render_timeout.as_secs()
            ))),
        }
    }

    /// Render a static map image via a worker thread
    pub async fn render_static(
        &self,
        style_json: &str,
        options: RenderOptions,
    ) -> Result<RenderedImage> {
        let (tx, rx) = oneshot::channel();

        self.dispatch(RenderRequest {
            job: RenderJob::Static {
                style_json: style_json.to_string(),
                options,
            },
            response: tx,
        })?;

        match tokio::time::timeout(self.config.render_timeout, rx).await {
            Ok(Ok(RenderOutput::Static(result))) => result,
            Ok(Ok(_)) => Err(TileServerError::RenderError(
                "unexpected render output type".to_string(),
            )),
            Ok(Err(_)) => Err(TileServerError::RenderError(
                "renderer worker dropped response".to_string(),
            )),
            Err(_) => Err(TileServerError::RenderError(format!(
                "static render timed out after {}s",
                self.config.render_timeout.as_secs()
            ))),
        }
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            max_scale: self.max_scale,
            pool_size: self.config.pool_size,
        }
    }

    fn dispatch(&self, request: RenderRequest) -> Result<()> {
        self.sender
            .as_ref()
            .ok_or_else(|| TileServerError::RenderError("renderer pool is shut down".to_string()))?
            .send(request)
            .map_err(|_| {
                TileServerError::RenderError("all renderer workers have stopped".to_string())
            })
    }
}

impl Drop for RendererPool {
    fn drop(&mut self) {
        // Close the channel so workers exit their recv loop
        self.sender.take();
        for handle in self.worker_handles.drain(..) {
            if let Err(e) = handle.join() {
                tracing::error!("renderer worker panicked during shutdown: {e:?}");
            }
        }
        tracing::info!("renderer pool shut down");
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_scale: u8,
    pub pool_size: usize,
}

fn execute_tile_render(
    style_json: &str,
    z: u8,
    x: u32,
    y: u32,
    tile_size: u32,
    scale: f32,
) -> Result<Vec<u8>> {
    let mut map = NativeMap::new(Size::new(tile_size, tile_size), scale, MapMode::Tile)?;
    map.load_style(style_json)?;
    let image = map.render_tile(z, x, y, tile_size, scale)?;
    image.to_png()
}

fn execute_static_render(style_json: &str, options: &RenderOptions) -> Result<RenderedImage> {
    let mut map = NativeMap::new(options.size, options.pixel_ratio, MapMode::Static)?;
    map.load_style(style_json)?;
    map.render(Some(RenderOptions {
        size: options.size,
        pixel_ratio: options.pixel_ratio,
        camera: options.camera,
        mode: options.mode,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = PoolConfig::default();
        let pool = RendererPool::new(config, 3);
        assert!(pool.is_ok());
    }
}
