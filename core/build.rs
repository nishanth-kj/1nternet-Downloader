fn main() {
    println!("cargo:rerun-if-changed=c/fast_ops.c");
    println!("cargo:rerun-if-changed=c/fast_ops.h");
    println!("cargo:rerun-if-changed=../.version");
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/icons/app.ico");

    // Embed the app icon into 1idm.exe (Explorer, taskbar, Alt-Tab).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("app.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to compile app.rc");
    }

    // Read and parse root .version file
    let version_path = std::path::Path::new("../.version");
    let mut version = "0.1.0".to_string();
    let mut version_code = "1".to_string();
    let mut sprint = "0".to_string();

    if let Ok(content) = std::fs::read_to_string(version_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim().to_uppercase();
                let raw_val = v.trim().trim_matches('"').trim_matches('\'');
                let val = raw_val.rsplit_once('=').map(|(_, v)| v.trim()).unwrap_or(raw_val).to_string();
                match key.as_str() {
                    "VERSION" => version = val,
                    "VERSION_CODE" => version_code = val,
                    "SPRINT" | "MAJOR" => sprint = val,
                    _ => {}
                }
            }
        }
    }

    let parts: Vec<&str> = version.split('.').collect();
    if !parts.is_empty() && sprint == "0" {
        sprint = parts[0].to_string();
    }

    // Derive version_code from VERSION if not explicitly provided
    if version_code == "1" && parts.len() >= 3 {
        if let (Ok(maj), Ok(min), Ok(pat)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            version_code = (maj * 10000 + min * 100 + pat).to_string();
        }
    }

    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rustc-env=APP_VERSION_CODE={}", version_code);
    println!("cargo:rustc-env=APP_SPRINT={}", sprint);

    let pkg_version = env!("CARGO_PKG_VERSION");
    if pkg_version != version {
        println!(
            "cargo:warning=Cargo.toml version ({}) does not match root .version ({})",
            pkg_version, version
        );
    }

    // Compile fast C operations library
    cc::Build::new()
        .file("c/fast_ops.c")
        .include("c")
        .warnings(true)
        .extra_warnings(true)
        .compile("fast_ops");

    build_libtorrent_bridge();
}

/// Compiles the C++ libtorrent bridge (cpp/) and links libtorrent-rasterbar from the
/// vcpkg tree, when that tree exists. Without it the crate still builds and torrent
/// support reports itself as unavailable (cfg `idr_libtorrent` is not set).
fn build_libtorrent_bridge() {
    println!("cargo::rustc-check-cfg=cfg(idr_libtorrent)");
    for f in ["cpp/torrent_manager.cpp", "cpp/torrent_c_api.cpp", "cpp/torrent_c_api.h", "cpp/manager.h"] {
        println!("cargo:rerun-if-changed={}", f);
    }
    println!("cargo:rerun-if-env-changed=IDR_VCPKG_INSTALLED");

    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let vcpkg = std::env::var("IDR_VCPKG_INSTALLED")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| manifest.join("vcpkg_installed").join("x64-windows"));
    let include = vcpkg.join("include");
    let lib = vcpkg.join("lib");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || !include.join("libtorrent").is_dir()
        || !lib.join("torrent-rasterbar.lib").is_file()
    {
        println!(
            "cargo:warning=libtorrent not found under {} - building without torrent support",
            vcpkg.display()
        );
        return;
    }

    // Must match INTERFACE_COMPILE_DEFINITIONS of vcpkg's LibtorrentRasterbar target
    // (release build), or the ABI won't line up with torrent-rasterbar.dll.
    let defines = [
        ("TORRENT_LINKING_SHARED", None),
        ("BOOST_ASIO_ENABLE_CANCELIO", None),
        ("BOOST_ASIO_NO_DEPRECATED", None),
        ("BOOST_SYSTEM_USE_UTF8", None),
        ("_SILENCE_CXX17_ALLOCATOR_VOID_DEPRECATION_WARNING", None),
        ("WIN32_LEAN_AND_MEAN", None),
        ("BOOST_ALL_NO_LIB", None),
        ("_SCL_SECURE_NO_DEPRECATE", None),
        ("_CRT_SECURE_NO_DEPRECATE", None),
        ("TORRENT_ABI_VERSION", Some("100")),
        ("TORRENT_USE_RTC", Some("0")),
        ("TORRENT_USE_OPENSSL", None),
        ("TORRENT_USE_LIBCRYPTO", None),
        ("TORRENT_SSL_PEERS", None),
        ("OPENSSL_NO_SSL2", None),
        ("OPENSSL_NO_SSL3", None),
        ("OPENSSL_NO_TLS1", None),
        ("OPENSSL_NO_TLS1_1", None),
        ("OPENSSL_NO_DTLS1", None),
        ("_WIN32_WINNT", Some("0x0A00")),
    ];
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .files(["cpp/torrent_manager.cpp", "cpp/torrent_c_api.cpp"])
        .include("cpp")
        .include(&include)
        .flag_if_supported("/EHsc")
        .flag_if_supported("/bigobj")
        .flag_if_supported("/utf-8")
        .warnings(false);
    for (k, v) in defines {
        build.define(k, v);
    }
    build.compile("idr_torrent_bridge");

    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=dylib=torrent-rasterbar");
    println!("cargo:rustc-cfg=idr_libtorrent");

    // Put the runtime DLLs next to 1idm.exe and the test binaries. torrent-rasterbar.dll
    // imports libssl, which imports libcrypto (checked with `dumpbin /dependents`).
    const RUNTIME_DLLS: [&str; 3] = ["torrent-rasterbar.dll", "libssl-3-x64.dll", "libcrypto-3-x64.dll"];
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    if let Some(profile_dir) = out_dir.ancestors().nth(3) {
        let bin = vcpkg.join("bin");
        {
            for name in RUNTIME_DLLS {
                let p = bin.join(name);
                if !p.is_file() {
                    println!("cargo:warning=missing runtime DLL {}", p.display());
                    continue;
                }
                for dest_dir in [profile_dir.to_path_buf(), profile_dir.join("deps")] {
                    let dest = dest_dir.join(name);
                    let stale = std::fs::metadata(&dest)
                        .and_then(|m| m.modified())
                        .ok()
                        .zip(std::fs::metadata(&p).and_then(|m| m.modified()).ok())
                        .map_or(true, |(d, s)| d < s);
                    if stale {
                        let _ = std::fs::copy(&p, &dest);
                    }
                }
            }
        }
    }
}
