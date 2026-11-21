use idm_lib::service::decode_idr_url;
use idm_lib::utils::format::{format_bytes, format_eta, format_percent, format_speed};

#[test]
fn test_utils_formatting() {
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.00 GB");
    assert_eq!(format_speed(1024.0 * 1024.0 * 5.0), "5.00 MB/s");
    assert_eq!(format_eta(65), "01:05");
    assert_eq!(format_eta(3665), "1h 01m");
    assert_eq!(format_percent(50, 100), "50.0%");
}

#[test]
fn test_idr_url_decoding() {
    let idr = "idr://https%3A%2F%2Fexample.com%2Finstaller.exe";
    let decoded = decode_idr_url(idr);
    assert_eq!(
        decoded,
        Some("https://example.com/installer.exe".to_string())
    );

    let raw = "https://example.com/file.zip";
    let unchanged = decode_idr_url(raw);
    assert_eq!(unchanged, Some("https://example.com/file.zip".to_string()));
}
