//! BitTorrent support through libtorrent-rasterbar.
//!
//! libtorrent itself lives behind the C API in `cpp/torrent_c_api.{h,cpp}` (compiled by
//! build.rs). This module is the only Rust code that calls it. When the crate is built
//! without libtorrent (cfg `idr_libtorrent` unset), every call reports that torrent
//! support is unavailable instead of failing to link.

use crate::exception::{IdmException, NetworkException, Result};
use crate::service::torrent::{TorrentService, TorrentSnapshot, TorrentStatus};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

#[repr(C)]
#[allow(dead_code)]
struct CTorrentSnapshot {
    name: [std::os::raw::c_char; 256],
    total_bytes: u64,
    downloaded_bytes: u64,
    uploaded_bytes: u64,
    download_speed: f64,
    upload_speed: f64,
    peers: u32,
    seeds: u32,
    status: i32,
}

#[cfg(idr_libtorrent)]
mod ffi {
    use super::CTorrentSnapshot;
    use std::os::raw::c_char;

    extern "C" {
        pub fn idr_torrent_add(source: *const c_char, destination_dir: *const c_char, auto_start: i32) -> i32;
        pub fn idr_torrent_get_snapshot(id: i32, out: *mut CTorrentSnapshot) -> i32;
        pub fn idr_torrent_pause(id: i32);
        pub fn idr_torrent_resume(id: i32);
        pub fn idr_torrent_remove_files(id: i32, delete_files: i32);
        pub fn idr_torrent_error(id: i32, buf: *mut c_char, buf_len: i32) -> i32;
    }
}

#[allow(dead_code)]
fn unavailable() -> IdmException {
    IdmException::Network(NetworkException::ConnectionFailed(
        "torrent support isn't included in this build (libtorrent wasn't found when it was compiled)".into(),
    ))
}

/// Thin handle to one torrent inside the libtorrent session.
pub struct TorrentTask {
    pub id: i64,
    pub source: String,
    pub destination_dir: PathBuf,
}

impl TorrentTask {
    pub fn get_snapshot(&self) -> TorrentSnapshot {
        TorrentManager::snapshot(self.id as i32).unwrap_or(TorrentSnapshot {
            name: self.source.clone(),
            total_bytes: 0,
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
            peers: 0,
            seeds: 0,
            status: TorrentStatus::Error,
        })
    }

    pub fn pause(&self) {
        TorrentManager::pause(self.id as i32);
    }

    pub fn resume(&self) {
        TorrentManager::resume(self.id as i32);
    }

    pub fn remove(&self) {
        TorrentManager::remove(self.id as i32, false);
    }
}

/// Entry point for torrent operations. libtorrent keeps one process-wide session,
/// created on first use (so the app opens no BitTorrent ports until a torrent is added).
pub struct TorrentManager {
    _private: (),
}

impl TorrentManager {
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// True when this build links libtorrent.
    pub fn available() -> bool {
        cfg!(idr_libtorrent)
    }

    /// Magnet links, http(s) links to .torrent files, and local .torrent files.
    pub fn is_torrent_url(url: &str) -> bool {
        let u = url.trim();
        if u.starts_with("magnet:?") {
            return true;
        }
        let path_part = u.split(['?', '#']).next().unwrap_or(u);
        path_part.to_ascii_lowercase().ends_with(".torrent")
    }

    /// Adds a magnet URI or a local .torrent path. Returns libtorrent's task id.
    pub fn add(source: &str, destination_dir: &Path, auto_start: bool) -> Result<i32> {
        #[cfg(idr_libtorrent)]
        {
            use std::ffi::CString;
            let _ = std::fs::create_dir_all(destination_dir);
            let src = CString::new(source)
                .map_err(|_| IdmException::Network(NetworkException::ConnectionFailed("invalid torrent source".into())))?;
            let dest = CString::new(destination_dir.to_string_lossy().as_bytes())
                .map_err(|_| IdmException::Network(NetworkException::ConnectionFailed("invalid folder".into())))?;
            // SAFETY: both pointers are valid NUL-terminated strings for the call's duration.
            let id = unsafe { ffi::idr_torrent_add(src.as_ptr(), dest.as_ptr(), auto_start as i32) };
            if id < 0 {
                return Err(IdmException::Network(NetworkException::ConnectionFailed(
                    "libtorrent rejected this magnet link or .torrent file".into(),
                )));
            }
            info!("Added torrent #{} into {}", id, destination_dir.display());
            Ok(id)
        }
        #[cfg(not(idr_libtorrent))]
        {
            let _ = (source, destination_dir, auto_start);
            Err(unavailable())
        }
    }

    pub fn snapshot(id: i32) -> Option<TorrentSnapshot> {
        #[cfg(idr_libtorrent)]
        {
            let mut raw = std::mem::MaybeUninit::<CTorrentSnapshot>::zeroed();
            // SAFETY: raw points to a zeroed, correctly laid out struct the callee fills.
            let ok = unsafe { ffi::idr_torrent_get_snapshot(id, raw.as_mut_ptr()) };
            if ok == 0 {
                return None;
            }
            // SAFETY: zero-initialised and (when ok) fully written by the callee.
            let raw = unsafe { raw.assume_init() };
            let bytes: Vec<u8> = raw.name.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
            Some(TorrentSnapshot {
                name: String::from_utf8_lossy(&bytes).into_owned(),
                total_bytes: raw.total_bytes,
                downloaded_bytes: raw.downloaded_bytes,
                uploaded_bytes: raw.uploaded_bytes,
                download_speed: raw.download_speed,
                upload_speed: raw.upload_speed,
                peers: raw.peers,
                seeds: raw.seeds,
                status: TorrentStatus::from_i32(raw.status),
            })
        }
        #[cfg(not(idr_libtorrent))]
        {
            let _ = id;
            None
        }
    }

    pub fn error(id: i32) -> Option<String> {
        #[cfg(idr_libtorrent)]
        {
            let mut buf = [0 as std::os::raw::c_char; 512];
            // SAFETY: buf is writable for buf.len() bytes.
            let has = unsafe { ffi::idr_torrent_error(id, buf.as_mut_ptr(), buf.len() as i32) };
            if has == 0 {
                return None;
            }
            let bytes: Vec<u8> = buf.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
            Some(String::from_utf8_lossy(&bytes).into_owned())
        }
        #[cfg(not(idr_libtorrent))]
        {
            let _ = id;
            None
        }
    }

    pub fn pause(id: i32) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call; unknown ids are ignored by the callee.
        unsafe {
            ffi::idr_torrent_pause(id)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = id;
    }

    pub fn resume(id: i32) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call; unknown ids are ignored by the callee.
        unsafe {
            ffi::idr_torrent_resume(id)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = id;
    }

    pub fn remove(id: i32, delete_files: bool) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call; unknown ids are ignored by the callee.
        unsafe {
            ffi::idr_torrent_remove_files(id, delete_files as i32)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = (id, delete_files);
    }

    /// Downloads an http(s) .torrent file into `%APPDATA%\InternetDownloader\torrents\`
    /// and returns its local path, so libtorrent can load it.
    pub async fn fetch_torrent_file(url: &str) -> Result<PathBuf> {
        let dir = crate::config::Config::app_data_dir().join("torrents");
        std::fs::create_dir_all(&dir).map_err(|e| IdmException::Storage(crate::exception::StorageException::Io(e)))?;
        let resp = reqwest::get(url)
            .await
            .map_err(|e| IdmException::Network(NetworkException::ConnectionFailed(e.to_string())))?;
        if !resp.status().is_success() {
            return Err(IdmException::Network(NetworkException::ConnectionFailed(format!(
                "server answered {} for the .torrent file",
                resp.status()
            ))));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| IdmException::Network(NetworkException::TransferInterrupted(e.to_string())))?;
        if bytes.len() > 20 * 1024 * 1024 || bytes.first() != Some(&b'd') {
            warn!("{} doesn't look like a .torrent file", url);
            return Err(IdmException::Network(NetworkException::ConnectionFailed(
                "the link didn't return a .torrent file".into(),
            )));
        }
        let name = format!("{:x}.torrent", sha2_digest(&bytes));
        let path = dir.join(name);
        std::fs::write(&path, &bytes).map_err(|e| IdmException::Storage(crate::exception::StorageException::Io(e)))?;
        Ok(path)
    }

    pub fn add_torrent<P: AsRef<Path>>(&self, source: &str, destination_dir: P) -> Result<Arc<TorrentTask>> {
        let dest = destination_dir.as_ref();
        let id = Self::add(source, dest, true)?;
        Ok(Arc::new(TorrentTask {
            id: id as i64,
            source: source.to_string(),
            destination_dir: dest.to_path_buf(),
        }))
    }
}

fn sha2_digest(bytes: &[u8]) -> u128 {
    use sha2::{Digest, Sha256};
    let d = Sha256::digest(bytes);
    u128::from_be_bytes(d[..16].try_into().unwrap_or([0; 16]))
}

impl Default for TorrentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TorrentService for TorrentManager {
    fn is_torrent_url(&self, url: &str) -> bool {
        TorrentManager::is_torrent_url(url)
    }

    fn add_torrent(&self, source: &str, destination_dir: &Path) -> Result<Arc<TorrentTask>> {
        TorrentManager::add_torrent(self, source, destination_dir)
    }
}


// ---------------------------------------------------------------------------------------
// Details, per-torrent actions, session settings, fast-resume and the torrent creator.
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct TorrentFileEntry {
    pub index: i32,
    pub path: String,
    pub size: u64,
    pub done: u64,
    /// 0 = skip, 1..7 (4 = normal, 7 = top).
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerState {
    NotContacted,
    Working,
    Updating,
    Error,
}

#[derive(Debug, Clone)]
pub struct TorrentTrackerEntry {
    pub url: String,
    pub tier: i32,
    pub state: TrackerState,
    pub seeds: i32,
    pub peers: i32,
    pub downloaded: i32,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentPeerEntry {
    pub address: String,
    pub client: String,
    pub down_speed: f64,
    pub up_speed: f64,
    pub progress: f32,
    pub downloaded: u64,
    pub uploaded: u64,
    pub is_seed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentExtra {
    pub all_time_upload: u64,
    pub all_time_download: u64,
    pub seeding_seconds: i64,
    pub active_seconds: i64,
    pub num_pieces: i32,
    pub pieces_have: i32,
    pub piece_length: i32,
    pub info_hash: String,
    pub save_path: String,
}

/// Everything the torrent details window shows, read in one go.
#[derive(Debug, Clone, Default)]
pub struct TorrentDetails {
    pub extra: TorrentExtra,
    pub files: Vec<TorrentFileEntry>,
    pub trackers: Vec<TorrentTrackerEntry>,
    pub peers: Vec<TorrentPeerEntry>,
    /// One byte per piece: 1 = downloaded.
    pub pieces: Vec<u8>,
    pub upload_speed: f64,
    pub uploaded: u64,
    pub sequential: bool,
}

impl TorrentDetails {
    /// Upload divided by download (all time). 0 when nothing was downloaded yet.
    pub fn ratio(&self) -> f64 {
        if self.extra.all_time_download == 0 {
            0.0
        } else {
            self.extra.all_time_upload as f64 / self.extra.all_time_download as f64
        }
    }
}

#[repr(C)]
#[allow(dead_code)]
struct CTorrentFile {
    index: i32,
    priority: i32,
    size: u64,
    done: u64,
    path: [c_char; 520],
}

#[repr(C)]
#[allow(dead_code)]
struct CTorrentTracker {
    tier: i32,
    status: i32,
    seeds: i32,
    peers: i32,
    downloaded: i32,
    url: [c_char; 520],
    message: [c_char; 256],
}

#[repr(C)]
#[allow(dead_code)]
struct CTorrentPeer {
    down_speed: f64,
    up_speed: f64,
    progress: f32,
    is_seed: i32,
    downloaded: u64,
    uploaded: u64,
    address: [c_char; 64],
    client: [c_char; 96],
}

#[repr(C)]
#[allow(dead_code)]
struct CTorrentExtra {
    all_time_upload: u64,
    all_time_download: u64,
    seeding_seconds: i64,
    active_seconds: i64,
    num_pieces: i32,
    pieces_have: i32,
    piece_length: i32,
    info_hash: [c_char; 72],
    save_path: [c_char; 520],
    comment: [c_char; 256],
    creator: [c_char; 128],
}

#[repr(C)]
#[allow(dead_code)]
struct CTorrentSessionSettings {
    listen_port: i32,
    random_port: i32,
    dht: i32,
    lsd: i32,
    pex: i32,
    utp: i32,
    upnp: i32,
    natpmp: i32,
    encryption: i32,
    max_connections: i32,
    max_peers_per_torrent: i32,
    download_limit_kbps: i32,
    upload_limit_kbps: i32,
    announce_all_trackers: i32,
    validate_https_trackers: i32,
    user_agent: [c_char; 128],
}

use std::os::raw::c_char;

#[cfg(idr_libtorrent)]
mod ffi2 {
    use super::*;
    extern "C" {
        pub fn idr_torrent_files(id: i32, out: *mut CTorrentFile, max: i32) -> i32;
        pub fn idr_torrent_trackers(id: i32, out: *mut CTorrentTracker, max: i32) -> i32;
        pub fn idr_torrent_peers(id: i32, out: *mut CTorrentPeer, max: i32) -> i32;
        pub fn idr_torrent_pieces(id: i32, out: *mut u8, max: i32) -> i32;
        pub fn idr_torrent_extra(id: i32, out: *mut CTorrentExtra) -> i32;
        pub fn idr_torrent_set_file_priority(id: i32, index: i32, priority: i32);
        pub fn idr_torrent_set_resume_dir(dir: *const c_char);
        pub fn idr_torrent_save_resume(timeout_ms: i32) -> i32;
        pub fn idr_torrent_apply_settings(settings: *const CTorrentSessionSettings);
        pub fn idr_torrent_set_sequential(id: i32, on: i32);
        pub fn idr_torrent_is_sequential(id: i32) -> i32;
        pub fn idr_torrent_add_tracker(id: i32, url: *const c_char);
        pub fn idr_torrent_force_recheck(id: i32);
        pub fn idr_torrent_force_reannounce(id: i32);
        pub fn idr_torrent_create(
            source_path: *const c_char,
            output_file: *const c_char,
            trackers: *const c_char,
            web_seeds: *const c_char,
            comment: *const c_char,
            piece_size_kib: i32,
            is_private: i32,
            err: *mut c_char,
            err_len: i32,
        ) -> i32;
    }
}

fn cstr_field(buf: &[c_char]) -> String {
    let bytes: Vec<u8> = buf.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg_attr(not(idr_libtorrent), allow(dead_code))]
fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s.replace('\0', "")).unwrap_or_default()
}

/// Reads a variable-length list from the C API: ask once, grow the buffer if needed.
#[cfg(idr_libtorrent)]
fn read_list<T>(id: i32, call: unsafe extern "C" fn(i32, *mut T, i32) -> i32) -> Vec<T> {
    let mut cap: usize = 64;
    loop {
        let mut buf: Vec<T> = Vec::with_capacity(cap);
        // SAFETY: the callee writes at most `cap` entries into the buffer and returns the
        // total count; we only expose min(total, cap) initialised entries.
        let total = unsafe { call(id, buf.as_mut_ptr(), cap as i32) };
        if total < 0 {
            return Vec::new();
        }
        let total = total as usize;
        if total <= cap {
            // SAFETY: entries 0..total were written by the callee.
            unsafe { buf.set_len(total) };
            return buf;
        }
        cap = total;
    }
}

impl TorrentManager {
    pub fn details(id: i32) -> Option<TorrentDetails> {
        #[cfg(idr_libtorrent)]
        {
            let mut extra_raw = std::mem::MaybeUninit::<CTorrentExtra>::zeroed();
            // SAFETY: zeroed, correctly laid out struct filled by the callee.
            if unsafe { ffi2::idr_torrent_extra(id, extra_raw.as_mut_ptr()) } == 0 {
                return None;
            }
            // SAFETY: zero-initialised and written by the callee.
            let e = unsafe { extra_raw.assume_init() };
            let extra = TorrentExtra {
                all_time_upload: e.all_time_upload,
                all_time_download: e.all_time_download,
                seeding_seconds: e.seeding_seconds,
                active_seconds: e.active_seconds,
                num_pieces: e.num_pieces,
                pieces_have: e.pieces_have,
                piece_length: e.piece_length,
                info_hash: cstr_field(&e.info_hash),
                save_path: cstr_field(&e.save_path),
            };
            let files = read_list(id, ffi2::idr_torrent_files)
                .into_iter()
                .map(|f| TorrentFileEntry { index: f.index, path: cstr_field(&f.path), size: f.size, done: f.done, priority: f.priority })
                .collect();
            let trackers = read_list(id, ffi2::idr_torrent_trackers)
                .into_iter()
                .map(|t| TorrentTrackerEntry {
                    url: cstr_field(&t.url),
                    tier: t.tier,
                    state: match t.status {
                        1 => TrackerState::Working,
                        2 => TrackerState::Updating,
                        3 => TrackerState::Error,
                        _ => TrackerState::NotContacted,
                    },
                    seeds: t.seeds,
                    peers: t.peers,
                    downloaded: t.downloaded,
                    message: cstr_field(&t.message),
                })
                .collect();
            let peers = read_list(id, ffi2::idr_torrent_peers)
                .into_iter()
                .map(|p| TorrentPeerEntry {
                    address: cstr_field(&p.address),
                    client: cstr_field(&p.client),
                    down_speed: p.down_speed,
                    up_speed: p.up_speed,
                    progress: p.progress,
                    downloaded: p.downloaded,
                    uploaded: p.uploaded,
                    is_seed: p.is_seed != 0,
                })
                .collect();
            let pieces = read_list(id, ffi2::idr_torrent_pieces);
            let snap = Self::snapshot(id);
            Some(TorrentDetails {
                extra,
                files,
                trackers,
                peers,
                pieces,
                upload_speed: snap.as_ref().map(|s| s.upload_speed).unwrap_or(0.0),
                uploaded: snap.as_ref().map(|s| s.uploaded_bytes).unwrap_or(0),
                // SAFETY: plain value call.
                sequential: unsafe { ffi2::idr_torrent_is_sequential(id) } != 0,
            })
        }
        #[cfg(not(idr_libtorrent))]
        {
            let _ = id;
            None
        }
    }

    pub fn set_file_priority(id: i32, index: i32, priority: i32) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call; invalid ids/indexes are ignored by the callee.
        unsafe {
            ffi2::idr_torrent_set_file_priority(id, index, priority)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = (id, index, priority);
    }

    pub fn set_sequential(id: i32, on: bool) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call.
        unsafe {
            ffi2::idr_torrent_set_sequential(id, on as i32)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = (id, on);
    }

    pub fn add_tracker(id: i32, url: &str) {
        #[cfg(idr_libtorrent)]
        {
            let c = to_cstring(url);
            // SAFETY: valid NUL-terminated string for the call's duration.
            unsafe { ffi2::idr_torrent_add_tracker(id, c.as_ptr()) };
        }
        #[cfg(not(idr_libtorrent))]
        let _ = (id, url);
    }

    pub fn force_recheck(id: i32) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call.
        unsafe {
            ffi2::idr_torrent_force_recheck(id)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = id;
    }

    pub fn force_reannounce(id: i32) {
        #[cfg(idr_libtorrent)]
        // SAFETY: plain value call.
        unsafe {
            ffi2::idr_torrent_force_reannounce(id)
        };
        #[cfg(not(idr_libtorrent))]
        let _ = id;
    }

    pub fn set_resume_dir(dir: &Path) {
        #[cfg(idr_libtorrent)]
        {
            let c = to_cstring(&dir.to_string_lossy());
            // SAFETY: valid NUL-terminated string for the call's duration.
            unsafe { ffi2::idr_torrent_set_resume_dir(c.as_ptr()) };
        }
        #[cfg(not(idr_libtorrent))]
        let _ = dir;
    }

    /// Writes fast-resume files for torrents with unsaved changes. Blocks up to `timeout_ms`.
    pub fn save_resume(timeout_ms: i32) -> i32 {
        #[cfg(idr_libtorrent)]
        {
            // SAFETY: plain value call.
            unsafe { ffi2::idr_torrent_save_resume(timeout_ms) }
        }
        #[cfg(not(idr_libtorrent))]
        {
            let _ = timeout_ms;
            0
        }
    }

    pub fn apply_settings(s: &crate::config::TorrentSettings) {
        #[cfg(idr_libtorrent)]
        {
            let mut ua = [0 as c_char; 128];
            for (dst, b) in ua.iter_mut().zip(s.user_agent.bytes().take(127)) {
                *dst = b as c_char;
            }
            let c = CTorrentSessionSettings {
                listen_port: s.listen_port as i32,
                random_port: s.random_port as i32,
                dht: s.dht as i32,
                lsd: s.lsd as i32,
                pex: s.pex as i32,
                utp: s.utp as i32,
                upnp: s.upnp as i32,
                natpmp: s.natpmp as i32,
                encryption: s.encryption.code(),
                max_connections: s.max_connections as i32,
                max_peers_per_torrent: s.max_peers_per_torrent as i32,
                download_limit_kbps: s.download_limit_kbps as i32,
                upload_limit_kbps: s.upload_limit_kbps as i32,
                announce_all_trackers: s.announce_to_all_trackers as i32,
                validate_https_trackers: s.validate_https_trackers as i32,
                user_agent: ua,
            };
            // SAFETY: pointer to a fully initialised struct, valid for the call.
            unsafe { ffi2::idr_torrent_apply_settings(&c) };
        }
        #[cfg(not(idr_libtorrent))]
        let _ = s;
    }

    /// Builds a .torrent file. Blocking (hashes every file); call from a worker thread.
    pub fn create_torrent_file(
        source: &Path,
        output: &Path,
        trackers: &[String],
        web_seeds: &[String],
        comment: &str,
        piece_size_kib: i32,
        is_private: bool,
    ) -> std::result::Result<(), String> {
        #[cfg(idr_libtorrent)]
        {
            let src = to_cstring(&source.to_string_lossy());
            let out = to_cstring(&output.to_string_lossy());
            let tr = to_cstring(&trackers.join("\n"));
            let ws = to_cstring(&web_seeds.join("\n"));
            let cm = to_cstring(comment);
            let mut err = [0 as c_char; 512];
            // SAFETY: all pointers are valid NUL-terminated strings / a writable buffer.
            let ok = unsafe {
                ffi2::idr_torrent_create(
                    src.as_ptr(),
                    out.as_ptr(),
                    tr.as_ptr(),
                    ws.as_ptr(),
                    cm.as_ptr(),
                    piece_size_kib,
                    is_private as i32,
                    err.as_mut_ptr(),
                    err.len() as i32,
                )
            };
            if ok == 1 {
                Ok(())
            } else {
                Err(cstr_field(&err))
            }
        }
        #[cfg(not(idr_libtorrent))]
        {
            let _ = (source, output, trackers, web_seeds, comment, piece_size_kib, is_private);
            Err(unavailable().to_string())
        }
    }
}
