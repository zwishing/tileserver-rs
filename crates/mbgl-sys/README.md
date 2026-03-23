# mbgl-sys

[![crates.io](https://img.shields.io/crates/v/mbgl-sys.svg)](https://crates.io/crates/mbgl-sys)
[![docs.rs](https://docs.rs/mbgl-sys/badge.svg)](https://docs.rs/mbgl-sys)

Low-level FFI bindings to [MapLibre GL Native](https://github.com/maplibre/maplibre-native) for server-side map rendering.

## Installation

```toml
[dependencies]
mbgl-sys = "0.1"
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `bundled` | ✅ | Build from vendored MapLibre Native source tree (requires prior `cmake` build) |
| `prebuilt` | | Download pre-compiled libraries from [GitHub Releases](https://github.com/vinayakkulkarni/tileserver-rs/releases) |
| `system` | | Link against system-installed MapLibre Native libraries |

When neither native libraries nor source are available, a stub implementation is compiled that returns error codes for all operations. This allows the crate to compile on any platform for development without requiring the full native build.

### Using pre-built binaries (recommended for crates.io)

```toml
[dependencies]
mbgl-sys = { version = "0.1", default-features = false, features = ["prebuilt"] }
```

The `prebuilt` feature automatically downloads the correct platform binary during `cargo build`. Supported targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`.

### Environment variables

| Variable | Description |
|----------|-------------|
| `MBGL_SYS_LIB_DIR` | Path to a directory containing pre-compiled `.a` files — bypasses all other strategies |

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
