#!/bin/sh
# Docker entrypoint script for tileserver-rs
# Starts Xvfb for headless OpenGL rendering (required by MapLibre Native)

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
