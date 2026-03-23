//! Build script for `mbgl-sys` — FFI bindings to MapLibre GL Native.
//!
//! # Feature resolution order
//!
//! The build script tries the following strategies in order:
//!
//! 1. **`MBGL_SYS_LIB_DIR`** env var — link pre-compiled `.a` files from a custom directory
//! 2. **`prebuilt`** feature — download pre-compiled libraries from GitHub Releases
//! 3. **`bundled`** (default) — compile the C++ wrapper against the vendored MapLibre Native
//!    source tree (requires prior `cmake --build` of MapLibre Native)
//! 4. **Stub** fallback — compile a minimal stub that returns error codes for all operations

// ---------------------------------------------------------------------------
// Imports
// ---------------------------------------------------------------------------

#[cfg(feature = "prebuilt")]
use sha2::{Digest, Sha256};

use std::env;
#[cfg(feature = "prebuilt")]
use std::fs;
#[cfg(feature = "prebuilt")]
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Crate version — used to construct download URLs and cache directory names.
#[cfg(feature = "prebuilt")]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Placeholder hash used when real checksums have not been computed yet.
/// Verification is skipped when the expected hash matches this value.
#[cfg(feature = "prebuilt")]
const PLACEHOLDER_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// SHA-256 checksums for pre-built tarballs, keyed by target triple.
///
/// After running the `build-mbgl-native` CI workflow, replace the placeholder
/// hashes below with the real values printed by each build job.
#[cfg(feature = "prebuilt")]
const CHECKSUMS: &[(&str, &str)] = &[
    (
        "aarch64-apple-darwin",
        "0000000000000000000000000000000000000000000000000000000000000000",
    ),
    (
        "x86_64-unknown-linux-gnu",
        "0000000000000000000000000000000000000000000000000000000000000000",
    ),
];

// ===========================================================================
// Entry point
// ===========================================================================

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

    emit_rerun_directives();

    // -----------------------------------------------------------------------
    // 1. MBGL_SYS_LIB_DIR escape hatch
    // -----------------------------------------------------------------------
    if let Ok(lib_dir) = env::var("MBGL_SYS_LIB_DIR") {
        let lib_dir = PathBuf::from(lib_dir);
        assert!(
            lib_dir.is_dir(),
            "MBGL_SYS_LIB_DIR={} does not exist or is not a directory",
            lib_dir.display()
        );
        println!(
            "cargo:warning=Using pre-built libraries from MBGL_SYS_LIB_DIR={}",
            lib_dir.display()
        );
        link_prebuilt_libs(&lib_dir);
        return;
    }

    // -----------------------------------------------------------------------
    // 2. Prebuilt download (feature-gated)
    // -----------------------------------------------------------------------
    #[cfg(feature = "prebuilt")]
    {
        if try_prebuilt() {
            return;
        }
    }

    // -----------------------------------------------------------------------
    // 3. Bundled — check vendor/ for pre-built MapLibre Native .a files
    // -----------------------------------------------------------------------
    let maplibre_base = manifest_dir.join("vendor/maplibre-native");

    #[cfg(target_os = "macos")]
    let build_dir_candidates = &["build-macos-metal"];
    #[cfg(target_os = "linux")]
    let build_dir_candidates = &["build-linux-opengl", "build-linux"];
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let build_dir_candidates: &[&str] = &[];

    let build_dir = build_dir_candidates
        .iter()
        .map(|d| maplibre_base.join(d))
        .find(|d| d.join("libmbgl-core.a").exists());

    if let Some(build_dir) = build_dir {
        println!("cargo:warning=Building with real MapLibre Native renderer");
        build_with_maplibre_native(&manifest_dir, &build_dir);
        return;
    }

    // -----------------------------------------------------------------------
    // 4. Stub fallback
    // -----------------------------------------------------------------------
    println!("cargo:warning=MapLibre Native not built — using stub implementation");
    emit_build_instructions();
    build_stub();
}

// ===========================================================================
// Cargo metadata directives
// ===========================================================================

/// Emit `rerun-if-changed` and `rerun-if-env-changed` directives so Cargo
/// knows when to re-run this build script.
fn emit_rerun_directives() {
    println!("cargo:rerun-if-changed=cpp/maplibre_c.h");
    println!("cargo:rerun-if-changed=cpp/maplibre_c.cpp");
    println!("cargo:rerun-if-changed=cpp/maplibre_c_stub.c");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=MBGL_SYS_LIB_DIR");

    #[cfg(target_os = "macos")]
    println!("cargo:rerun-if-changed=vendor/maplibre-native/build-macos-metal/libmbgl-core.a");

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rerun-if-changed=vendor/maplibre-native/build-linux-opengl/libmbgl-core.a");
        println!("cargo:rerun-if-changed=vendor/maplibre-native/build-linux/libmbgl-core.a");
    }
}

/// Print helpful instructions when MapLibre Native is not available.
fn emit_build_instructions() {
    println!("cargo:warning=To build MapLibre Native, run:");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:warning=  cd crates/mbgl-sys/vendor/maplibre-native");
        println!("cargo:warning=  cmake --preset macos-metal");
        println!("cargo:warning=  cmake --build build-macos-metal --target mbgl-core mlt-cpp -j8");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:warning=  cd crates/mbgl-sys/vendor/maplibre-native");
        println!("cargo:warning=  cmake --preset linux-opengl");
        println!(
            "cargo:warning=  cmake --build build-linux-opengl --target mbgl-core mlt-cpp -j$(nproc)"
        );
    }
}

// ===========================================================================
// Linking helpers
// ===========================================================================

/// Link pre-built static libraries from a flat directory that contains all the
/// required `.a` files (including the pre-compiled C++ wrapper).
///
/// Used by both the `MBGL_SYS_LIB_DIR` escape hatch and the `prebuilt` feature.
fn link_prebuilt_libs(lib_dir: &Path) {
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Pre-compiled C++ wrapper
    println!("cargo:rustc-link-lib=static=maplibre_c");

    // Core MapLibre libraries
    println!("cargo:rustc-link-lib=static=mbgl-core");
    println!("cargo:rustc-link-lib=static=mlt-cpp");
    println!("cargo:rustc-link-lib=static=mbgl-freetype");
    println!("cargo:rustc-link-lib=static=mbgl-harfbuzz");
    println!("cargo:rustc-link-lib=static=mbgl-vendor-csscolorparser");
    println!("cargo:rustc-link-lib=static=mbgl-vendor-parsedate");

    // Platform-specific vendored libraries
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=static=mbgl-vendor-icu");

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=static=mbgl-vendor-nunicode");
        println!("cargo:rustc-link-lib=static=mbgl-vendor-sqlite");
    }

    link_system_libs();
}

/// Link platform-specific system libraries and frameworks required by
/// MapLibre Native at runtime.
fn link_system_libs() {
    #[cfg(target_os = "macos")]
    {
        // macOS frameworks
        for framework in &[
            "Metal",
            "MetalKit",
            "QuartzCore",
            "CoreFoundation",
            "CoreGraphics",
            "CoreText",
            "Foundation",
            "ImageIO",
            "Security",
            "SystemConfiguration",
            "CoreServices",
        ] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }

        // System libraries
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=sqlite3");

        // libuv — installed via Homebrew on macOS
        if let Ok(output) = Command::new("brew").args(["--prefix", "libuv"]).output()
            && output.status.success()
        {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-link-search=native={prefix}/lib");
            println!("cargo:rustc-link-lib=uv");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // System ICU (not vendored on Linux)
        println!("cargo:rustc-link-lib=icuuc");
        println!("cargo:rustc-link-lib=icui18n");
        println!("cargo:rustc-link-lib=icudata");

        // System libraries
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=curl");
        println!("cargo:rustc-link-lib=png");
        println!("cargo:rustc-link-lib=jpeg");
        println!("cargo:rustc-link-lib=webp");
        println!("cargo:rustc-link-lib=uv");

        // OpenGL / X11
        println!("cargo:rustc-link-lib=GL");
        println!("cargo:rustc-link-lib=EGL");
        println!("cargo:rustc-link-lib=X11");
    }
}

// ===========================================================================
// Build paths
// ===========================================================================

/// Compile the C++ wrapper against the vendored MapLibre Native headers and
/// link all required static libraries from the cmake build directory.
fn build_with_maplibre_native(manifest_dir: &Path, build_dir: &Path) {
    let maplibre_src = manifest_dir.join("vendor/maplibre-native");

    // Build the C++ wrapper via cc::Build
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .file("cpp/maplibre_c.cpp")
        .include("cpp")
        // MapLibre Native include paths
        .include(maplibre_src.join("include"))
        .include(maplibre_src.join("platform/default/include"))
        .include(maplibre_src.join("src"))
        .include(maplibre_src.join("vendor/maplibre-native-base/extras/expected-lite/include"))
        .include(maplibre_src.join("vendor/maplibre-native-base/include"))
        .include(maplibre_src.join("vendor/maplibre-native-base/deps/geojson.hpp/include"))
        .include(maplibre_src.join("vendor/maplibre-native-base/deps/geometry.hpp/include"))
        .include(maplibre_src.join("vendor/maplibre-native-base/deps/variant/include"))
        .include(maplibre_src.join("vendor/maplibre-native-base/deps/optional/include"))
        .include(maplibre_src.join("vendor/rapidjson/include"))
        .flag("-std=c++20")
        .flag("-fPIC")
        .flag("-fvisibility=hidden")
        .warnings(false); // Suppress warnings from MapLibre Native headers

    // Platform-specific include paths and compiler flags
    #[cfg(target_os = "macos")]
    {
        build.include(maplibre_src.join("platform/darwin/include"));
        build.flag("-mmacosx-version-min=14.3");
    }

    #[cfg(target_os = "linux")]
    build.include(maplibre_src.join("platform/linux/include"));

    // cc::Build::compile() emits cargo:rustc-link-lib and cargo:rustc-link-search
    build.compile("maplibre_c");

    // Link MapLibre Native static libraries from the cmake build directory
    println!("cargo:rustc-link-search=native={}", build_dir.display());

    // MLT (MapLibre Tiles) library lives in a subdirectory
    println!(
        "cargo:rustc-link-search=native={}",
        build_dir.join("vendor/maplibre-tile-spec/cpp").display()
    );

    // Core MapLibre libraries (common to all platforms)
    println!("cargo:rustc-link-lib=static=mbgl-core");
    println!("cargo:rustc-link-lib=static=mlt-cpp");
    println!("cargo:rustc-link-lib=static=mbgl-freetype");
    println!("cargo:rustc-link-lib=static=mbgl-harfbuzz");
    println!("cargo:rustc-link-lib=static=mbgl-vendor-csscolorparser");
    println!("cargo:rustc-link-lib=static=mbgl-vendor-parsedate");

    // Platform-specific vendored libraries
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=static=mbgl-vendor-icu");

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=static=mbgl-vendor-nunicode");
        println!("cargo:rustc-link-lib=static=mbgl-vendor-sqlite");
    }

    // System libraries and frameworks
    link_system_libs();
}

/// Compile the minimal C stub that returns error codes for all MapLibre
/// operations. This allows the crate to compile without MapLibre Native.
fn build_stub() {
    let mut build = cc::Build::new();

    build
        .file("cpp/maplibre_c_stub.c")
        .include("cpp")
        .warnings(true)
        .extra_warnings(true)
        .opt_level(2);

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    build.flag("-std=c11");

    #[cfg(target_os = "linux")]
    build.flag("-D_GNU_SOURCE");

    // cc::Build::compile() emits cargo:rustc-link-lib and cargo:rustc-link-search
    build.compile("maplibre_c_stub");
}

// ===========================================================================
// Pre-built binary download (feature = "prebuilt")
// ===========================================================================

/// Attempt to use pre-built libraries from GitHub Releases.
///
/// Returns `true` if pre-built libraries were successfully linked, `false` if
/// the current target is not supported and the caller should fall through to
/// the bundled/stub paths.
#[cfg(feature = "prebuilt")]
fn try_prebuilt() -> bool {
    let Some(target) = resolve_target() else {
        println!(
            "cargo:warning=prebuilt feature enabled but no pre-built binary available for this target"
        );
        return false;
    };

    let cache = cache_dir(&target);

    if cache.join(".mbgl-sys-ok").exists() {
        println!("cargo:warning=Using cached pre-built MapLibre Native for {target}");
        link_prebuilt_libs(&cache.join("lib"));
        return true;
    }

    match fetch_and_extract_prebuilt(&target, &cache) {
        Ok(()) => {
            link_prebuilt_libs(&cache.join("lib"));
            true
        }
        Err(e) => {
            println!("cargo:warning=prebuilt download failed: {e}");
            println!("cargo:warning=Falling back to bundled/stub...");
            false
        }
    }
}

/// Download, verify, and extract pre-built libraries for the given target.
#[cfg(feature = "prebuilt")]
fn fetch_and_extract_prebuilt(target: &str, cache: &Path) -> Result<(), String> {
    let url = download_url(target);
    let tarball_name = format!("mbgl-native-{target}.tar.gz");
    let tarball = cache.join(&tarball_name);

    fs::create_dir_all(cache).map_err(|e| format!("failed to create cache directory: {e}"))?;

    println!("cargo:warning=Downloading pre-built MapLibre Native for {target}...");
    println!("cargo:warning=  URL: {url}");
    download(&url, &tarball)?;

    if let Some(expected) = expected_sha256(target) {
        println!("cargo:warning=Verifying SHA-256 checksum...");
        verify_sha256(&tarball, expected)?;
    } else {
        println!("cargo:warning=Skipping SHA-256 verification (placeholder hash)");
    }

    extract_tar_gz(&tarball, cache)?;
    let _ = fs::remove_file(&tarball);

    fs::write(cache.join(".mbgl-sys-ok"), "ok")
        .map_err(|e| format!("failed to write sentinel: {e}"))?;

    Ok(())
}

/// Map Cargo target environment variables to a supported target triple.
///
/// Returns `None` if the current target has no pre-built binary available.
#[cfg(feature = "prebuilt")]
#[must_use]
fn resolve_target() -> Option<String> {
    let os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    match (arch.as_str(), os.as_str(), target_env.as_str()) {
        ("aarch64", "macos", _) => Some("aarch64-apple-darwin".to_owned()),
        ("x86_64", "linux", "gnu") => Some("x86_64-unknown-linux-gnu".to_owned()),
        _ => None,
    }
}

/// Compute the cache directory path for a given target.
///
/// Layout: `$OUT_DIR/prebuilt-mbgl-sys-v{VERSION}-{TARGET}/`
#[cfg(feature = "prebuilt")]
#[must_use]
fn cache_dir(target: &str) -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    out_dir.join(format!("prebuilt-mbgl-sys-v{VERSION}-{target}"))
}

/// Build the GitHub Releases download URL for a given target.
#[cfg(feature = "prebuilt")]
#[must_use]
fn download_url(target: &str) -> String {
    format!(
        "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/mbgl-sys-v{VERSION}/mbgl-native-{target}.tar.gz"
    )
}

/// Look up the expected SHA-256 hash for a target triple.
///
/// Returns `None` if the target is not in the checksum table or the hash is
/// still a placeholder (all zeros).
#[cfg(feature = "prebuilt")]
#[must_use]
fn expected_sha256(target: &str) -> Option<&'static str> {
    CHECKSUMS
        .iter()
        .find(|(t, _)| *t == target)
        .map(|(_, hash)| *hash)
        .filter(|hash| *hash != PLACEHOLDER_HASH)
}

/// Download a URL to a local file using a temporary `.tmp` extension and
/// atomic rename.
///
/// Uses a 5-minute read timeout to accommodate large downloads on slow
/// connections.
#[cfg(feature = "prebuilt")]
fn download(url: &str, dest: &Path) -> Result<(), String> {
    let tmp = dest.with_extension("tmp");

    let response = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout_read(std::time::Duration::from_secs(300))
        .build()
        .get(url)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;

    let file =
        fs::File::create(&tmp).map_err(|e| format!("failed to create {}: {e}", tmp.display()))?;
    let mut writer = BufWriter::new(file);
    let mut reader = response.into_reader();

    io::copy(&mut reader, &mut writer)
        .map_err(|e| format!("failed to write to {}: {e}", tmp.display()))?;
    writer
        .flush()
        .map_err(|e| format!("failed to flush: {e}"))?;
    drop(writer);

    fs::rename(&tmp, dest).map_err(|e| {
        format!(
            "failed to rename {} → {}: {e}",
            tmp.display(),
            dest.display()
        )
    })?;

    Ok(())
}

/// Verify a file's SHA-256 checksum using streaming 64 KiB chunks.
///
/// # Panics
///
/// Panics if the computed hash does not match `expected`.
#[cfg(feature = "prebuilt")]
fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let file =
        fs::File::open(path).map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65_536];

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("read error during checksum: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let actual = format!("{:x}", hasher.finalize());
    if actual != expected {
        let _ = fs::remove_file(path);
        return Err(format!(
            "SHA-256 mismatch for {}: expected {expected}, got {actual}",
            path.display()
        ));
    }
    Ok(())
}

/// Extract a `.tar.gz` archive into a destination directory.
///
/// File permissions from the archive are NOT preserved (not meaningful for
/// static libraries on most platforms).
#[cfg(feature = "prebuilt")]
fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path)
        .map_err(|e| format!("failed to open {}: {e}", archive_path.display()))?;
    let gz = flate2::read::GzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(gz);
    archive.set_preserve_permissions(false);
    archive
        .unpack(dest)
        .map_err(|e| format!("failed to extract {}: {e}", archive_path.display()))?;
    Ok(())
}
