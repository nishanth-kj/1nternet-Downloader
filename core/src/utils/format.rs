use crate::config::{GB, KB, MB, TB};

/// Human-readable byte formatting (e.g. "12.34 MB")
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Human-readable download speed formatting (e.g. "5.67 MB/s")
pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec <= 0.0 {
        return "0 B/s".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}

/// Human-readable ETA formatting (e.g. "02:15" or "1h 12m")
pub fn format_eta(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".to_string();
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else if minutes > 0 {
        format!("{:02}:{:02}", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Progress percentage formatting
pub fn format_percent(downloaded: u64, total: u64) -> String {
    if total == 0 {
        "0.0%".to_string()
    } else {
        let pct = (downloaded as f64 / total as f64) * 100.0;
        format!("{:.1}%", pct.clamp(0.0, 100.0))
    }
}
