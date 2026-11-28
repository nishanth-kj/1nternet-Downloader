//! Display helpers for the download list: sizes, rates, times and file categories.

use serde::{Deserialize, Serialize};

pub fn bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

/// Size column: "Unknown" until the server reports a length.
pub fn size(total: u64) -> String {
    if total == 0 {
        "Unknown".into()
    } else {
        bytes(total)
    }
}

pub fn rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec <= 0.0 {
        String::new()
    } else {
        format!("{}/s", bytes(bytes_per_sec as u64))
    }
}

pub fn eta(seconds: Option<u64>) -> String {
    match seconds {
        None => String::new(),
        Some(0) => String::new(),
        Some(s) if s >= 86_400 => format!("{}d {:02}h", s / 86_400, (s % 86_400) / 3600),
        Some(s) if s >= 3600 => format!("{}h {:02}m", s / 3600, (s % 3600) / 60),
        Some(s) if s >= 60 => format!("{}m {:02}s", s / 60, s % 60),
        Some(s) => format!("{}s", s),
    }
}

/// Folder of a saved file, shortened to its last two parts ("…\Downloads\Video") for narrow
/// columns. (The text shaper breaks lines after "C:", so full paths don't clip cleanly.)
pub fn short_folder(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split(['\\', '/']).filter(|s| !s.is_empty()).collect();
    match parts.len() {
        0 | 1 => file_path.to_string(),
        2 => parts[0].to_string(),
        3 => format!("{}\\{}", parts[0], parts[1]),
        n => format!("…\\{}\\{}", parts[n - 3], parts[n - 2]),
    }
}

/// Local date and time, or empty for "never".
pub fn datetime(unix_secs: Option<i64>) -> String {
    use chrono::{Local, TimeZone};
    match unix_secs.and_then(|t| Local.timestamp_opt(t, 0).single()) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => String::new(),
    }
}

/// File type groups shown under "All Downloads", decided from the file extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileCategory {
    Compressed,
    Documents,
    Music,
    Programs,
    Video,
    Images,
    Other,
}

impl FileCategory {
    pub const ALL: [FileCategory; 7] = [
        FileCategory::Compressed,
        FileCategory::Documents,
        FileCategory::Music,
        FileCategory::Programs,
        FileCategory::Video,
        FileCategory::Images,
        FileCategory::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FileCategory::Compressed => "Compressed",
            FileCategory::Documents => "Documents",
            FileCategory::Music => "Music",
            FileCategory::Programs => "Programs",
            FileCategory::Video => "Video",
            FileCategory::Images => "Images",
            FileCategory::Other => "Other",
        }
    }

    /// Icon codepoint in the Segoe MDL2 Assets font (see `ui::view::icon`).
    pub fn glyph(self) -> &'static str {
        match self {
            FileCategory::Compressed => "\u{E7B8}",
            FileCategory::Documents => "\u{E8A5}",
            FileCategory::Music => "\u{E8D6}",
            FileCategory::Programs => "\u{ECAA}",
            FileCategory::Video => "\u{E714}",
            FileCategory::Images => "\u{EB9F}",
            FileCategory::Other => "\u{E7C3}",
        }
    }

    pub fn from_filename(name: &str) -> Self {
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "zip" | "rar" | "7z" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "cab" | "lz" | "lzma" => {
                FileCategory::Compressed
            }
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" | "rtf" | "txt"
            | "csv" | "epub" | "md" | "chm" => FileCategory::Documents,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "oga" | "m4a" | "wma" | "opus" | "aiff" => FileCategory::Music,
            "exe" | "msi" | "msix" | "appx" | "apk" | "dmg" | "pkg" | "deb" | "rpm" | "appimage" | "bat" | "iso"
            | "img" => FileCategory::Programs,
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "ts" | "3gp" => {
                FileCategory::Video
            }
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "tif" | "tiff" | "heic" | "avif" | "ico" => {
                FileCategory::Images
            }
            _ => FileCategory::Other,
        }
    }
}

impl std::fmt::Display for FileCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// True for links the engine accepts: http(s) URLs with a host, or magnet links.
pub fn is_supported_link(s: &str) -> bool {
    let s = s.trim();
    if s.starts_with("magnet:?") {
        return true;
    }
    // A .torrent file on this computer.
    if s.to_ascii_lowercase().ends_with(".torrent") && std::path::Path::new(s).is_file() {
        return true;
    }
    match url::Url::parse(s) {
        Ok(u) => matches!(u.scheme(), "http" | "https") && u.host_str().is_some_and(|h| !h.is_empty()),
        Err(_) => false,
    }
}

/// Pulls every supported link out of free text (one per line or whitespace-separated).
pub fn extract_links(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for token in text.split_whitespace() {
        let token = token.trim_matches(|c| matches!(c, '"' | '\'' | '<' | '>' | '(' | ')' | ',' | ';'));
        if is_supported_link(token) && !out.iter().any(|u| u == token) {
            out.push(token.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_from_extension() {
        assert_eq!(FileCategory::from_filename("a.ZIP"), FileCategory::Compressed);
        assert_eq!(FileCategory::from_filename("movie.mkv"), FileCategory::Video);
        assert_eq!(FileCategory::from_filename("setup.exe"), FileCategory::Programs);
        assert_eq!(FileCategory::from_filename("noext"), FileCategory::Other);
    }

    #[test]
    fn link_extraction() {
        let t = "see https://a.com/x.zip and \"https://a.com/x.zip\"\nftp://no magnet:?xt=urn:btih:abc notaurl";
        assert_eq!(extract_links(t), vec!["https://a.com/x.zip", "magnet:?xt=urn:btih:abc"]);
        assert!(!is_supported_link("https://"));
    }

    #[test]
    fn eta_format() {
        assert_eq!(eta(Some(59)), "59s");
        assert_eq!(eta(Some(61)), "1m 01s");
        assert_eq!(eta(Some(3700)), "1h 01m");
        assert_eq!(eta(None), "");
    }
}
