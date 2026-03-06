#!/bin/sh
# Docker entrypoint script for tileserver-rs
# - Downloads sample data if /data is empty (for one-click deploys)
# - Starts Xvfb for headless OpenGL rendering (required by MapLibre Native)

set -e

# ============================================================================
# Sample data download (for one-click cloud deploys)
# ============================================================================
# If /data has no tile files, download sample data from the GitHub release.
# This enables zero-config deploys on Railway, Render, Fly.io, etc.
# Users can skip this by mounting their own data volume at /data.

SAMPLE_DATA_VERSION="${SAMPLE_DATA_VERSION:-latest}"
REPO="vinayakkulkarni/tileserver-rs"

download_sample_data() {
  if [ "${SAMPLE_DATA_VERSION}" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/tileserver-rs-sample-data.tar.gz"
  else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${SAMPLE_DATA_VERSION}/tileserver-rs-sample-data.tar.gz"
  fi

  echo "[DEMO] No tile data found in /data"
  echo "[DEMO] Downloading sample data from ${DOWNLOAD_URL}"
  echo "[DEMO] Set SAMPLE_DATA_VERSION=v2.12.1 to pin a specific release"

  if curl -fsSL --retry 3 --retry-delay 2 "${DOWNLOAD_URL}" | tar -xz -C /data; then
    echo "[DEMO] Sample data downloaded successfully"
    echo "[DEMO] Contents:"
    ls -la /data/tiles/ 2>/dev/null || true
    ls -la /data/styles/ 2>/dev/null || true
    ls -la /data/fonts/ 2>/dev/null || true
  else
    echo "[WARN] Failed to download sample data. Server will start without tile data."
    echo "[WARN] Mount your own data at /data or check network connectivity."
  fi
}

# Check if /data has any tile files (.pmtiles, .mbtiles)
has_tile_data() {
  # Look for tile files anywhere under /data
  find /data -name '*.pmtiles' -o -name '*.mbtiles' 2>/dev/null | head -1 | grep -q .
}

if ! has_tile_data; then
  download_sample_data
fi

# ============================================================================
# Xvfb setup for headless OpenGL rendering
# ============================================================================

# Clean up stale lock file if it exists (handles container restarts)
if [ -e /tmp/.X99-lock ]; then
  rm -f /tmp/.X99-lock
fi

# Set display for headless OpenGL rendering
export DISPLAY=:99

# Start Xvfb in background (no unix socket for security)
Xvfb "${DISPLAY}" -nolisten unix &

# Small delay to ensure Xvfb is ready
sleep 0.5

exec "$@"
