fn main() {
    println!("cargo:rerun-if-changed=c/fast_ops.c");
    println!("cargo:rerun-if-changed=c/fast_ops.h");
    println!("cargo:rerun-if-changed=../.version");

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
                let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                match key.as_str() {
                    "VERSION" => version = val,
                    "VERSION_CODE" => version_code = val,
                    "SPRINT" => sprint = val,
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
}
