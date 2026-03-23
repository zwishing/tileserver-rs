# mbgl-sys

Low-level FFI bindings to [MapLibre GL Native](https://github.com/maplibre/maplibre-native) for server-side map rendering.

## Features

- `bundled` (default) — Build from vendored MapLibre Native source tree
- `system` — Link against system-installed MapLibre Native libraries

When neither native libraries nor source are available, a stub implementation is compiled that returns error codes for all operations. This allows the crate to compile on any platform for development without requiring the full native build.

## Building with native rendering

### macOS (Metal)

```bash
cd vendor/maplibre-native
git submodule update --init --recursive
cmake --preset macos-metal
cmake --build build-macos-metal --target mbgl-core mlt-cpp -j8
```

### Linux (OpenGL)

```bash
cd vendor/maplibre-native
git submodule update --init --recursive
cmake --preset linux-opengl
cmake --build build-linux-opengl --target mbgl-core mlt-cpp -j$(nproc)
```

## License

MIT OR Apache-2.0
