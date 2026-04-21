# =============================================================================
# Stage 1: Download pre-built MapLibre Native static libraries
# =============================================================================
FROM ubuntu:24.04 AS maplibre-downloader

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH

COPY crates/mbgl-sys/Cargo.toml /tmp/mbgl-sys-cargo.toml

WORKDIR /build
RUN MBGL_VERSION=$(grep '^version' /tmp/mbgl-sys-cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/') && \
    case "$TARGETARCH" in \
      amd64) MBGL_TARGET="x86_64-unknown-linux-gnu" ;; \
      arm64) MBGL_TARGET="aarch64-unknown-linux-gnu" ;; \
      *) echo "Unsupported arch: $TARGETARCH" && exit 1 ;; \
    esac && \
    echo "Downloading mbgl-sys v${MBGL_VERSION} for ${MBGL_TARGET}..." && \
    curl -fSL "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/mbgl-sys-v${MBGL_VERSION}/mbgl-native-${MBGL_TARGET}.tar.gz" | tar xz

# =============================================================================
# Stage 2: Build Nuxt frontend (SPA) - skipped for headless builds
# =============================================================================
FROM oven/bun:1 AS node-builder

ARG FEATURES="frontend geoparquet"

# ulimit -s unlimited increases the POSIX thread stack size so Bun's JSC engine
# can handle the deep CJS resolver recursion caused by @mlc-ai/web-llm in Vite builds

WORKDIR /app

# Copy workspace files
COPY package.json bun.lock ./
COPY apps/client ./apps/client

# Only build frontend when the "frontend" feature is enabled
# For headless builds (FEATURES=""), create an empty output directory
# so the COPY --from=node-builder in the rust-builder stage still works
RUN if echo "$FEATURES" | grep -q "frontend"; then \
      bun install --filter '@tileserver-rs/client' && \
      cd apps/client && ulimit -s unlimited && bun run generate; \
    else \
      mkdir -p apps/client/.output/public; \
    fi

# =============================================================================
# Stage 3: Build Rust backend
# =============================================================================
# Use Ubuntu 24.04 to match glibc version with maplibre-builder stage
# (Ubuntu 24.04 has glibc 2.39 with C23 functions like __isoc23_strtol)
FROM ubuntu:24.04 AS rust-builder

ENV DEBIAN_FRONTEND=noninteractive

# Install Rust and deps needed for linking
RUN apt-get update && apt-get install -y --no-install-recommends --fix-missing \
    ca-certificates \
    curl \
    git \
    build-essential \
    pkg-config \
    libcurl4-openssl-dev \
    libpng-dev \
    libicu-dev \
    libjpeg-dev \
    libwebp-dev \
    libsqlite3-dev \
    libuv1-dev \
    libglfw3-dev \
    libgdal-dev \
    libclang-dev \
    libopengl-dev \
    libgl-dev \
    libegl-dev \
    libx11-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

ENV PATH="/root/.cargo/bin:${PATH}"
ENV MBGL_SYS_LIB_DIR=/app/lib

# Compiler optimisation tier.  Per-architecture defaults:
#   - amd64 → `x86-64-v3` (AVX2 + BMI2; every x86 CPU since 2013).
#   - arm64 → `neoverse-n1` (Graviton2 / Ampere Altra / Cobalt, default
#     in every major cloud ARM VM as of 2025).
# Override `TARGET_CPU` explicitly for the `:fast` variant (AVX-512 / SVE2)
# or to `native` for on-prem builds pinned to specific host hardware.
# See `release-docker-images.yml` for how the release matrix passes this.
# NOTE: `TARGET_CPU=x86-64-v3` is ONLY valid on amd64; on arm64 it triggers
# a ring-crate static-NEON assert and kills the build.  Always branch on
# `TARGETARCH` before using a target-cpu string as the fallback.
# Cargo reads a per-user config at `~/.cargo/config.toml`; write the
# selected flags there so both the pre-built dependency cache layer
# AND the final source build pick them up automatically.
ARG TARGETARCH
ARG TARGET_CPU
# Optional extra rustflags appended after `-C target-cpu=<CPU>`.  Used by the
# `:fast` amd64 variant to add `+aes,+sha,+rdrnd` on top of x86-64-v3 for
# crypto/rand speedups (without AVX-512 — GitHub runners lack it; see
# release-docker-images.yml HISTORY comment).  Format: pass flags as a single
# space-separated token pair per -C arg, e.g. `-C target-feature=+aes,+sha`.
# Cargo's rustflags array requires `-C` and its value as SEPARATE elements
# (not "-C target-feature=..." as one string — that fails "failed to run
# rustc to learn about target-specific info").  The awk below splits the
# input on whitespace into consecutive pairs and emits each as two JSON
# array elements: `"-C", "target-feature=+aes,+sha,+rdrnd"`.
ARG RUSTFLAGS_EXTRA=""
RUN mkdir -p /root/.cargo && \
    if [ -n "${TARGET_CPU}" ]; then \
      CPU="${TARGET_CPU}"; \
    elif [ "${TARGETARCH}" = "arm64" ]; then \
      CPU="neoverse-n1"; \
    else \
      CPU="x86-64-v3"; \
    fi && \
    echo "[build]" > /root/.cargo/config.toml && \
    EXTRA_FLAGS_JSON=""; \
    if [ -n "${RUSTFLAGS_EXTRA}" ]; then \
      EXTRA_FLAGS_JSON=$(echo "${RUSTFLAGS_EXTRA}" | awk '{ \
        for (i=1; i<=NF; i+=2) printf ", \"%s\", \"%s\"", $i, $(i+1) \
      }'); \
    fi && \
    echo "rustflags = [\"-C\", \"target-cpu=${CPU}\"${EXTRA_FLAGS_JSON}]" >> /root/.cargo/config.toml && \
    cat /root/.cargo/config.toml

WORKDIR /app

COPY --from=maplibre-downloader /build/lib ./lib
COPY crates/mbgl-sys ./crates/mbgl-sys

# Copy Cargo files and build script for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source and bench files for dependency caching
RUN mkdir -p src benches && echo "fn main() {}" > src/main.rs && echo "fn main() {}" > benches/mlt.rs

# Copy the embedded SPA
COPY --from=node-builder /app/apps/client/.output/public ./apps/client/.output/public

ARG FEATURES="frontend geoparquet"

# Build dependencies only (may fail on first try, that's ok)
RUN if [ -n "$FEATURES" ]; then \
      cargo build --release --features "$FEATURES" 2>/dev/null || true; \
    else \
      cargo build --release 2>/dev/null || true; \
    fi
RUN rm -rf src

# Copy actual source code and benchmarks
COPY src ./src
COPY benches ./benches

RUN touch src/main.rs && \
    if [ -n "$FEATURES" ]; then \
      cargo build --release --features "$FEATURES"; \
    else \
      cargo build --release; \
    fi

# =============================================================================
# Stage 4: Runtime (must match glibc version from build stage)
# =============================================================================
FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends --no-install-suggests --fix-missing \
    ca-certificates \
    curl \
    xvfb \
    libglfw3 \
    libuv1 \
    libjpeg-turbo8 \
    libicu74 \
    libcurl4t64 \
    libpng16-16t64 \
    libwebp7 \
    libsqlite3-0 \
    libopengl0 \
    libgles2 \
    libx11-6 \
    libegl1 \
    libgdal34t64 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary
COPY --from=rust-builder /app/target/release/tileserver-rs ./tileserver-rs

# Copy entrypoint script
COPY deploy/docker-entrypoint.sh ./docker-entrypoint.sh
RUN chmod +x ./docker-entrypoint.sh

# Copy example config
COPY configs/example.toml ./config.toml

# Create data directory
RUN mkdir -p /data

# Environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Expose port
EXPOSE 8080

# Volume for tile data
VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Use entrypoint script to handle Xvfb setup
ENTRYPOINT ["./docker-entrypoint.sh"]
CMD ["./tileserver-rs", "--config", "/app/config.toml"]
