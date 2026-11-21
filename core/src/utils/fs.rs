use std::fs::{self, File};
use std::path::Path;

/// Clean and sanitize a filename for Windows filesystem
pub fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut cleaned: String = name
        .chars()
        .map(|c| if invalid_chars.contains(&c) || c.is_control() { '_' } else { c })
        .collect();

    // Prevent Windows reserved filenames (CON, PRN, AUX, NUL, COM1-9, LPT1-9)
    let reserved = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let stem = Path::new(&cleaned)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if reserved.iter().any(|&r| r.eq_ignore_ascii_case(stem)) {
        cleaned = format!("idm_{}", cleaned);
    }

    if cleaned.trim().is_empty() {
        "download.bin".to_string()
    } else {
        cleaned
    }
}

/// Ensure directory exists, creating all parent components
pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Allocate or expand file to target size using set_len (in-place sparse file on NTFS)
pub fn preallocate_file(path: &Path, total_bytes: u64) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;

    file.set_len(total_bytes)?;
    Ok(file)
}
