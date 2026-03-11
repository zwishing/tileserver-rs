# Release Architecture

This document describes the build and release architecture for tileserver-rs.

## Single Binary Design

tileserver-rs is distributed as a **single self-contained binary**, similar to [martin](https://maplibre.org/martin/) and [tileserver-gl](https://tileserver.readthedocs.io/). The web UI (Nuxt SPA) is embedded directly into the Rust binary at compile time using `rust-embed`.

```
┌──────────────────────────────────────────────────────┐
│                  tileserver-rs binary                 │
├──────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌────────────────┐               │
│  │  Tile Server  │  │  Embedded SPA  │               │
│  │    (Axum)     │  │    (Nuxt)      │               │
│  │               │  │                │               │
│  │  /health      │  │  /             │               │
│  │  /ping        │  │  /styles/:id   │               │
│  │  /data.json   │  │  /data/:id     │               │
│  │  /data/:id/.. │  │                │               │
│  │  /data/:id/.mlt│  │                │               │
│  └───────────────┘  └────────────────┘               │
│         Main port (default: 8080)                    │
│                                                      │
│  ┌──────────────────────────────────┐                │
│  │  Admin Server (optional)        │                │
│  │  POST /__admin/reload           │                │
│  │  Separate port (admin_bind)     │                │
│  └──────────────────────────────────┘                │
│                                                      │
│  ┌──────────────────────────────────┐                │
│  │  Hot Reload (SIGHUP handler)    │                │
│  │  ArcSwap-based state swap       │                │
│  └──────────────────────────────────┘                │
└──────────────────────────────────────────────────────┘
│                                                      │
│  ┌──────────────────────────────────┐                │
│  │  MLT Transcoding (optional)    │                │
│  │  MLT→MVT reverse transcoding  │                │
│  │  cargo feature: mlt            │                │
│  └──────────────────────────────────┘                │
```

## Supported Platforms

| Platform | Architecture | Runner | Target Triple | Status |
|----------|-------------|--------|---------------|--------|
| macOS | ARM64 (Apple Silicon) | `macos-14` | `aarch64-apple-darwin` | ✅ |
| macOS | x86_64 (Intel) | `macos-13` | `x86_64-apple-darwin` | ✅ |
| Linux | x86_64 | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | ✅ |
| Linux | ARM64 | `ubuntu-latest` + cross | `aarch64-unknown-linux-gnu` | ✅ |
| Windows | x86_64 | `windows-latest` | `x86_64-pc-windows-msvc` | 🚧 Planned |
| Windows | ARM64 | `windows-latest` | `aarch64-pc-windows-msvc` | 🚧 Planned |

## Build Process

### 1. Generate Nuxt Static SPA

The frontend is generated as a static SPA (`ssr: false` in nuxt.config.ts):

```bash
bun install --frozen-lockfile
bun run --filter @tileserver-rs/client generate
```

Output: `apps/client/.output/public/` (static HTML + JS bundles)

See: [Nuxt Static Hosting](https://nuxt.com/docs/getting-started/deployment#static-hosting)

### 2. Build Rust Binary

The Rust binary embeds the SPA at compile time via `rust-embed`:

```rust
#[derive(Embed)]
#[folder = "apps/client/.output/public"]
struct Assets;
```

```bash
cargo build --release --target <target-triple>
```

Release profile optimizations (`Cargo.toml`):
- `lto = true` - Link-time optimization
- `codegen-units = 1` - Single codegen unit
- `opt-level = 3` - Maximum optimization
- `strip = true` - Strip debug symbols

### 3. Package

The final artifact is just the binary:

```bash
# macOS/Linux
tileserver-rs-macos-arm64.tar.gz
tileserver-rs-macos-arm64.tar.gz.sha256

# Windows
tileserver-rs-windows-x64.zip
tileserver-rs-windows-x64.zip.sha256
```

## Usage

```bash
# Download and extract
curl -LO https://github.com/.../tileserver-rs-macos-arm64.tar.gz
tar -xzf tileserver-rs-macos-arm64.tar.gz

# Zero-config: auto-detect sources from a directory
./tileserver-rs /path/to/data

# Run with explicit config (UI enabled by default)
./tileserver-rs --config config.toml

# Run without UI (API only)
./tileserver-rs --no-ui --config config.toml
```

## CLI Options

```
tileserver-rs [PATH] [OPTIONS]

Arguments:
  [PATH]  Path to a tile file or directory to auto-detect sources/styles from

Options:
  -c, --config <FILE>       Path to configuration file [env: TILESERVER_CONFIG]
      --host <HOST>         Host to bind to [env: TILESERVER_HOST]
  -p, --port <PORT>         Port to bind to [env: TILESERVER_PORT]
      --public-url <URL>    Public URL for tile URLs in TileJSON [env: TILESERVER_PUBLIC_URL]
      --ui                  Enable the web UI (default: true) [env: TILESERVER_UI]
      --no-ui               Disable the web UI
  -v, --verbose             Enable verbose logging
  -h, --help                Print help
  -V, --version             Print version
```

### Config Resolution Priority

1. `--config <FILE>` — explicit config path (fail-fast if missing)
2. Positional `PATH` — auto-detect from that path
3. Default locations: `./config.toml`, `/etc/tileserver-rs/config.toml`
4. CWD auto-detect — scan current directory

## Workflow Files

### CI & Quality

| Workflow | File | Purpose | Trigger |
|----------|------|---------|---------|
| Pipeline | `pipeline.yml` | Orchestrator — runs lint + CI jobs | Push to `main`, PRs |
| CI Node | `ci-node.yml` | Lint, typecheck, build Nuxt client | Called by pipeline |
| CI Rust | `ci-rust.yml` | Format, clippy, check, test, build Rust | Called by pipeline |
| Lint Branch | `lint-branch.yml` | Validate branch naming convention | Called by pipeline |
| Lint PR | `lint-pr.yml` | Validate PR title (conventional commits) | Called by pipeline |

### Releases

| Workflow | File | Purpose | Trigger |
|----------|------|---------|---------|
| Release Please | `release-please.yml` | Version bumps, changelog, GitHub Release | Push to `main` |
| macOS ARM64 | `release-macos-arm64.yml` | Apple Silicon binary | Tags `v*`, manual |
| macOS AMD64 | `release-macos-amd64.yml` | Intel binary | Tags `v*`, manual |
| Linux | `release-linux.yml` | x86_64 + ARM64 (full & headless variants) | Tags `v*`, manual |
| Docker | `release-docker.yml` | Multi-arch Docker image (amd64/arm64) | Push to `main` (path-filtered), tags `v*`, manual |
| Homebrew | `release-homebrew.yml` | Update Homebrew formula | Tags `v*` |
| Sample Data | `release-sample-data.yml` | Package sample tile data | Manual |

### Deployment

| Workflow | File | Purpose | Trigger |
|----------|------|---------|---------|
| Deploy Docs | `deploy-docs.yml` | Build & deploy docs to CF Pages | Push to `main` (`apps/docs/**`), manual |
| Deploy Marketing | `deploy-marketing.yml` | Build & deploy marketing to CF Pages | Push to `main` (`apps/marketing/**`), manual |

### Utilities

| Workflow | File | Purpose | Trigger |
|----------|------|---------|---------|
| Automerger | `automerger.yml` | Auto-merge Dependabot PRs | PR events |

## GitHub Actions Runners

### macOS
- **macos-14**: M1/M2 (Apple Silicon) - ARM64 native
- **macos-13**: Intel - x86_64 native

### Linux
- **ubuntu-latest**: x86_64 native
- For ARM64: Use cross-compilation or self-hosted ARM runners

### Windows
- **windows-latest**: x86_64 native
- For ARM64: Cross-compilation from x86_64

## Version Tagging

Release triggers on tags matching `v*`:

- `v0.1.0` - Stable release
- `v0.1.0-beta.1` - Pre-release (marked on GitHub)
- `v0.1.0-rc.1` - Release candidate

## Manual Releases

Trigger via `workflow_dispatch`:

1. Go to Actions tab
2. Select release workflow
3. Click "Run workflow"
4. Enter version tag

## Cross-Compilation

### Linux ARM64 from x86_64

```bash
rustup target add aarch64-unknown-linux-gnu
apt-get install gcc-aarch64-linux-gnu
```

### Windows from macOS/Linux

```bash
rustup target add x86_64-pc-windows-msvc
# Requires cargo-xwin or cross
```

## Comparison with Similar Tools

| Feature | tileserver-rs | martin | tileserver-gl |
|---------|--------------|--------|---------------|
| Single binary | ✅ | ✅ | ❌ (Node.js) |
| Embedded UI | ✅ | ✅ | ✅ |
| PMTiles | ✅ | ✅ | ❌ |
| MBTiles | ✅ | ✅ | ✅ |
| MLT (MapLibre Tiles) | ✅ (passthrough + transcode) | ✅ (passthrough only) | ❌ |
| MLT↔MVT Transcoding | ✅ (feature-gated) | ❌ | ❌ |
| Language | Rust | Rust | JavaScript |
## Future Improvements

- [x] ~~Homebrew formula~~ (done — `release-homebrew.yml`)
- [x] ~~Multi-platform release builds~~ (done — macOS ARM64/AMD64, Linux x86_64/ARM64)
- [x] ~~Consolidated CI pipeline~~ (done — `pipeline.yml` orchestrator)
- [ ] Automatic changelog generation (beyond Release Please)
- [ ] Code signing (macOS notarization, Windows signing)
- [ ] APT/RPM packages
- [ ] MSI/PKG installers
- [ ] Windows builds
