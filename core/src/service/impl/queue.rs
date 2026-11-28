use crate::config::Settings;
use crate::exception::{DatabaseException, Result};
use crate::model::database::segments::SegmentItem;
use crate::model::database::{downloads, segments};
use crate::model::{DownloadId, DownloadItem, DownloadSnapshot, DownloadStatus};
use crate::service::queue::QueueService;
use crate::service::r#impl::download::DownloadTask;
use crate::config::TorrentSettings;
use crate::service::r#impl::torrent::{TorrentDetails, TorrentManager};
use crate::service::torrent::TorrentStatus;
use parking_lot::RwLock;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::info;

struct ManagedTask {
    task: Arc<DownloadTask>,
    #[allow(dead_code)]
    handle: Option<JoinHandle<()>>,
}

/// MIME type stored in the downloads table to mark a row as a torrent.
pub const TORRENT_MIME: &str = "application/x-bittorrent";

/// A torrent in the list: our download id maps to libtorrent's task id.
struct TorrentEntry {
    /// libtorrent task id, or -1 when libtorrent couldn't load it.
    lt_id: i32,
    /// Magnet URI or local .torrent path (also what's stored in the database).
    source: String,
    dest_dir: PathBuf,
    /// Shown until libtorrent has the metadata (magnet "dn" or the .torrent file name).
    fallback_name: String,
    last_status: Option<DownloadStatus>,
    load_error: Option<String>,
}

pub struct QueueManager {
    conn: Arc<Mutex<Connection>>,
    tasks: Arc<RwLock<HashMap<DownloadId, ManagedTask>>>,
    torrents: Arc<RwLock<HashMap<DownloadId, TorrentEntry>>>,
    torrent_settings: Arc<RwLock<TorrentSettings>>,
    settings: Arc<RwLock<Settings>>,
    event_tx: broadcast::Sender<DownloadSnapshot>,
}

impl QueueManager {
    pub fn new(conn: Arc<Mutex<Connection>>, settings: Settings) -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(64);
        let qm = Arc::new(Self {
            conn,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            torrents: Arc::new(RwLock::new(HashMap::new())),
            torrent_settings: Arc::new(RwLock::new(TorrentSettings::load())),
            settings: Arc::new(RwLock::new(settings)),
            event_tx,
        });

        TorrentManager::apply_settings(&qm.torrent_settings.read());
        TorrentManager::set_resume_dir(&crate::config::Config::app_data_dir().join("torrents").join("resume"));
        qm.load_history();
        if TorrentManager::available() {
            // Fast-resume: save changed torrents every 30 s so a crash loses little.
            let weak = Arc::downgrade(&qm);
            std::thread::Builder::new()
                .name("torrent-resume".into())
                .spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    let Some(qm) = weak.upgrade() else { break };
                    if !qm.torrents.read().is_empty() {
                        TorrentManager::save_resume(5_000);
                    }
                })
                .ok();
        }
        qm
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot> {
        QueueService::subscribe(self)
    }

    fn load_history(&self) {
        let items_res = {
            let conn_guard = match self.conn.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            downloads::select_all_downloads(&conn_guard)
        };

        if let Ok(items) = items_res {
            info!("Loaded {} items from database history", items.len());
            let mut tasks = self.tasks.write();
            let settings = self.settings.read().clone();

            for item in items {
                if item.mime_type.as_deref() == Some(TORRENT_MIME) {
                    self.restore_torrent(&item);
                    continue;
                }
                let dest = PathBuf::from(&item.destination);
                if let Ok(task) = DownloadTask::new(item.id, item.url, dest, settings.clone()) {
                    tasks.insert(
                        item.id,
                        ManagedTask {
                            task: Arc::new(task),
                            handle: None,
                        },
                    );
                }
            }
        }
    }

    pub async fn add_download(
        self: &Arc<Self>,
        url: &str,
        destination: Option<PathBuf>,
        auto_start: bool,
    ) -> Result<DownloadId> {
        self.add_download_as(url, destination, None, auto_start).await
    }

    /// Same as [`add_download`](Self::add_download), with an optional "Save As" file name.
    /// A blank or path-like override falls back to the name derived from the URL.
    pub async fn add_download_as(
        self: &Arc<Self>,
        url: &str,
        destination: Option<PathBuf>,
        filename_override: Option<String>,
        auto_start: bool,
    ) -> Result<DownloadId> {
        if TorrentManager::is_torrent_url(url) || is_local_torrent(url) {
            return self.add_torrent_download(url, destination, auto_start).await;
        }
        let settings = self.settings.read().clone();
        let filename = filename_override
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty() && !f.contains(['/', '\\', ':']) && f != "." && f != "..")
            .unwrap_or_else(|| crate::service::network::NetworkClient::extract_filename_from_url(url));
        let dest_dir = destination.unwrap_or_else(|| PathBuf::from(&settings.download_dir));
        let full_path = dest_dir.join(&filename);

        let now = chrono::Utc::now().timestamp();
        let item = DownloadItem {
            id: 0,
            url: url.to_string(),
            destination: full_path.to_string_lossy().to_string(),
            filename: filename.clone(),
            total_bytes: 0,
            downloaded_bytes: 0,
            status: if auto_start {
                DownloadStatus::Downloading
            } else {
                DownloadStatus::Queued
            },
            mime_type: None,
            checksum: None,
            created_at: now,
            updated_at: now,
        };

        let id = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
            downloads::insert_download(&conn, &item)?
        };

        let task = Arc::new(DownloadTask::new(
            id,
            url.to_string(),
            full_path,
            settings,
        )?);

        let mut sub = task.subscribe();
        let event_tx_clone = self.event_tx.clone();
        let conn_clone = Arc::clone(&self.conn);

        crate::service::spawn(async move {
            let mut last_saved = std::time::Instant::now();
            while let Ok(snap) = sub.recv().await {
                let _ = event_tx_clone.send(snap.clone());
                let is_terminal = snap.status == DownloadStatus::Completed
                    || snap.status == DownloadStatus::Error
                    || snap.status == DownloadStatus::Paused;
                let is_periodic = last_saved.elapsed().as_millis() >= 500;

                if is_terminal || is_periodic {
                    last_saved = std::time::Instant::now();
                    if let Ok(conn) = conn_clone.lock() {
                        let _ = downloads::update_download_progress(
                            &conn,
                            snap.id,
                            snap.downloaded_bytes,
                            snap.total_bytes,
                            snap.status,
                        );
                    }
                }
            }
        });

        let mut handle = None;
        if auto_start {
            let task_clone = Arc::clone(&task);
            handle = Some(crate::service::spawn(async move {
                let _ = task_clone.run().await;
            }));
        }

        self.tasks.write().insert(
            id,
            ManagedTask {
                task,
                handle,
            },
        );

        Ok(id)
    }

    /// Adds a magnet link, an http(s) .torrent link (fetched first) or a local .torrent file.
    async fn add_torrent_download(
        self: &Arc<Self>,
        url: &str,
        destination: Option<PathBuf>,
        auto_start: bool,
    ) -> Result<DownloadId> {
        let settings = self.settings.read().clone();
        let dest_dir = destination.unwrap_or_else(|| PathBuf::from(&settings.download_dir));
        let url = url.trim();
        let source = if url.starts_with("magnet:") || is_local_torrent(url) {
            url.to_string()
        } else {
            TorrentManager::fetch_torrent_file(url).await?.to_string_lossy().into_owned()
        };
        let fallback_name = torrent_display_name(&source, url);

        let now = chrono::Utc::now().timestamp();
        let item = DownloadItem {
            id: 0,
            url: source.clone(),
            destination: dest_dir.to_string_lossy().to_string(),
            filename: fallback_name.clone(),
            total_bytes: 0,
            downloaded_bytes: 0,
            status: if auto_start { DownloadStatus::Downloading } else { DownloadStatus::Paused },
            mime_type: Some(TORRENT_MIME.to_string()),
            checksum: None,
            created_at: now,
            updated_at: now,
        };

        let lt_id = TorrentManager::add(&source, &dest_dir, auto_start)?;
        {
            let ts = self.torrent_settings.read();
            for tracker in ts.extra_tracker_list() {
                TorrentManager::add_tracker(lt_id, &tracker);
            }
            if ts.sequential_by_default {
                TorrentManager::set_sequential(lt_id, true);
            }
        }
        let id = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
            match downloads::insert_download(&conn, &item) {
                Ok(id) => id,
                Err(e) => {
                    TorrentManager::remove(lt_id, false);
                    return Err(e);
                }
            }
        };
        self.torrents.write().insert(
            id,
            TorrentEntry { lt_id, source, dest_dir, fallback_name, last_status: None, load_error: None },
        );
        info!("Torrent download #{} added (libtorrent #{})", id, lt_id);
        Ok(id)
    }

    /// Re-adds a saved torrent at startup; libtorrent re-checks any data already on disk.
    fn restore_torrent(&self, item: &DownloadItem) {
        let dest_dir = PathBuf::from(&item.destination);
        let start = item.status == DownloadStatus::Downloading;
        let (lt_id, load_error) = match TorrentManager::add(&item.url, &dest_dir, start) {
            Ok(id) => (id, None),
            Err(e) => (-1, Some(e.to_string())),
        };
        self.torrents.write().insert(
            item.id,
            TorrentEntry {
                lt_id,
                source: item.url.clone(),
                dest_dir,
                fallback_name: item.filename.clone(),
                last_status: Some(item.status),
                load_error,
            },
        );
    }

    /// Current state of every torrent, in the same shape as HTTP downloads. Status
    /// changes are written back to the database so the list survives a restart.
    fn torrent_snapshots(&self) -> Vec<DownloadSnapshot> {
        let mut out = Vec::new();
        let mut changed: Vec<(DownloadId, u64, u64, DownloadStatus)> = Vec::new();
        {
            let mut torrents = self.torrents.write();
            for (id, t) in torrents.iter_mut() {
                let snap = if t.lt_id >= 0 { TorrentManager::snapshot(t.lt_id) } else { None };
                let (name, total, done, speed, peers, seeds, status) = match &snap {
                    Some(s) => (
                        if s.name.is_empty() { t.fallback_name.clone() } else { s.name.clone() },
                        s.total_bytes,
                        s.downloaded_bytes,
                        s.download_speed,
                        s.peers,
                        s.seeds,
                        match s.status {
                            TorrentStatus::Queued => DownloadStatus::Queued,
                            TorrentStatus::Downloading => DownloadStatus::Downloading,
                            TorrentStatus::Paused => DownloadStatus::Paused,
                            TorrentStatus::Completed | TorrentStatus::Seeding => DownloadStatus::Completed,
                            TorrentStatus::Error => DownloadStatus::Error,
                        },
                    ),
                    None => (t.fallback_name.clone(), 0, 0, 0.0, 0, 0, DownloadStatus::Error),
                };
                let error_message = if status == DownloadStatus::Error {
                    t.load_error.clone().or_else(|| TorrentManager::error(t.lt_id)).or(Some("libtorrent lost this torrent".into()))
                } else {
                    None
                };
                if status == DownloadStatus::Completed && t.lt_id >= 0 && snap.is_some() {
                    self.enforce_seeding_limits(t.lt_id, t.last_status != Some(DownloadStatus::Completed));
                }
                if t.last_status != Some(status) {
                    t.last_status = Some(status);
                    changed.push((*id, done, total, status));
                }
                let mut s = DownloadSnapshot {
                    id: *id,
                    url: t.source.clone(),
                    destination: t.dest_dir.join(&name).to_string_lossy().to_string(),
                    filename: name,
                    total_bytes: total,
                    downloaded_bytes: done,
                    speed_bytes_per_sec: if status == DownloadStatus::Downloading { speed } else { 0.0 },
                    status,
                    segment_count: 1,
                    segments: Vec::new(),
                    is_torrent: true,
                    peers,
                    seeds,
                    error_message,
                    progress_percent: 0.0,
                    eta_seconds: None,
                };
                s.progress_percent = s.calculate_progress();
                s.eta_seconds = s.calculate_eta();
                out.push(s);
            }
        }
        if !changed.is_empty() {
            if let Ok(conn) = self.conn.lock() {
                for (id, done, total, status) in changed {
                    let _ = downloads::update_download_progress(&conn, id, done, total, status);
                }
            }
        }
        out
    }

    fn torrent_id(&self, id: DownloadId) -> Option<i32> {
        self.torrents.read().get(&id).map(|t| t.lt_id)
    }

    /// Stops seeding when the user's limits are reached (or right away if seeding is off).
    fn enforce_seeding_limits(&self, lt_id: i32, just_finished: bool) {
        let ts = self.torrent_settings.read().clone();
        if !ts.seed_finished {
            if just_finished {
                TorrentManager::pause(lt_id);
            }
            return;
        }
        if ts.seed_ratio_limit <= 0.0 && ts.seed_time_limit_minutes == 0 {
            return;
        }
        if let Some(d) = TorrentManager::details(lt_id) {
            let ratio_hit = ts.seed_ratio_limit > 0.0 && d.ratio() >= ts.seed_ratio_limit;
            let time_hit = ts.seed_time_limit_minutes > 0
                && d.extra.seeding_seconds >= i64::from(ts.seed_time_limit_minutes) * 60;
            if ratio_hit || time_hit {
                TorrentManager::pause(lt_id);
            }
        }
    }

    /// libtorrent id for one of our torrent downloads.
    fn live_torrent(&self, id: DownloadId) -> Option<i32> {
        self.torrent_id(id).filter(|lt| *lt >= 0)
    }

    pub fn is_torrent(&self, id: DownloadId) -> bool {
        self.torrents.read().contains_key(&id)
    }

    /// Files, trackers, peers, pieces and totals for the details window.
    pub fn torrent_details(&self, id: DownloadId) -> Option<TorrentDetails> {
        self.live_torrent(id).and_then(TorrentManager::details)
    }

    pub fn set_torrent_file_priority(&self, id: DownloadId, file_index: i32, priority: i32) {
        if let Some(lt) = self.live_torrent(id) {
            TorrentManager::set_file_priority(lt, file_index, priority);
        }
    }

    pub fn set_torrent_sequential(&self, id: DownloadId, on: bool) {
        if let Some(lt) = self.live_torrent(id) {
            TorrentManager::set_sequential(lt, on);
        }
    }

    pub fn add_torrent_tracker(&self, id: DownloadId, url: &str) {
        if let Some(lt) = self.live_torrent(id) {
            TorrentManager::add_tracker(lt, url);
        }
    }

    pub fn force_torrent_recheck(&self, id: DownloadId) {
        if let Some(lt) = self.live_torrent(id) {
            TorrentManager::force_recheck(lt);
        }
    }

    pub fn force_torrent_reannounce(&self, id: DownloadId) {
        if let Some(lt) = self.live_torrent(id) {
            TorrentManager::force_reannounce(lt);
        }
    }

    pub fn torrent_settings(&self) -> TorrentSettings {
        self.torrent_settings.read().clone()
    }

    /// Saves and applies new torrent options to the running session.
    pub fn update_torrent_settings(&self, settings: TorrentSettings) -> std::result::Result<(), String> {
        settings.save()?;
        TorrentManager::apply_settings(&settings);
        *self.torrent_settings.write() = settings;
        Ok(())
    }

    /// Writes fast-resume data now (used before the app exits).
    pub fn save_torrent_state(&self) {
        if TorrentManager::available() && !self.torrents.read().is_empty() {
            let n = TorrentManager::save_resume(5_000);
            info!("Saved fast-resume data for {} torrent(s)", n);
        }
    }

    /// Applies new settings to downloads added from now on (running tasks keep theirs).
    pub fn update_settings(&self, mut settings: Settings) {
        settings.normalize();
        *self.settings.write() = settings;
    }

    pub fn pause_download(&self, id: DownloadId) {
        QueueService::pause_download(self, id);
    }

    pub fn resume_download(self: &Arc<Self>, id: DownloadId) {
        QueueService::resume_download(self, id);
    }

    pub fn delete_download(&self, id: DownloadId, delete_file: bool) {
        QueueService::delete_download(self, id, delete_file);
    }

    pub fn pause_all(&self) {
        QueueService::pause_all(self);
    }

    pub fn resume_all(self: &Arc<Self>) {
        QueueService::resume_all(self);
    }

    pub fn get_all_snapshots(&self) -> Vec<DownloadSnapshot> {
        QueueService::get_all_snapshots(self)
    }

    pub fn save_segments(&self, download_id: DownloadId, segments_data: &[SegmentItem]) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        segments::save_segments(&mut conn, download_id, segments_data)
    }

    pub fn get_segments(&self, download_id: DownloadId) -> Result<Vec<SegmentItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        segments::select_segments_by_download_id(&conn, download_id)
    }
}

impl QueueService for QueueManager {
    fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot> {
        self.event_tx.subscribe()
    }

    fn pause_download(&self, id: DownloadId) {
        if let Some(lt) = self.torrent_id(id) {
            TorrentManager::pause(lt);
            return;
        }
        if let Some(m) = self.tasks.read().get(&id) {
            m.task.pause();
            if let Ok(conn) = self.conn.lock() {
                let _ = downloads::update_download_status(&conn, id, DownloadStatus::Paused);
            }
        }
    }

    fn resume_download(self: &Arc<Self>, id: DownloadId) {
        if let Some(lt) = self.torrent_id(id) {
            TorrentManager::resume(lt);
            return;
        }
        let task_opt = self.tasks.read().get(&id).map(|m| Arc::clone(&m.task));
        if let Some(task) = task_opt {
            task.resume();
            if let Ok(conn) = self.conn.lock() {
                let _ = downloads::update_download_status(&conn, id, DownloadStatus::Downloading);
            }

            let task_clone = Arc::clone(&task);
            let handle = crate::service::spawn(async move {
                let _ = task_clone.run().await;
            });

            if let Some(m) = self.tasks.write().get_mut(&id) {
                m.handle = Some(handle);
            }
        }
    }

    fn delete_download(&self, id: DownloadId, delete_file: bool) {
        if let Some(t) = self.torrents.write().remove(&id) {
            if t.lt_id >= 0 {
                TorrentManager::remove(t.lt_id, delete_file);
            }
            if let Ok(conn) = self.conn.lock() {
                let _ = downloads::delete_download_by_id(&conn, id);
            }
            return;
        }
        let task_opt = self.tasks.write().remove(&id);
        if let Some(m) = task_opt {
            m.task.pause();
            if delete_file && m.task.destination.exists() {
                let _ = std::fs::remove_file(&m.task.destination);
            }
        }
        if let Ok(conn) = self.conn.lock() {
            let _ = downloads::delete_download_by_id(&conn, id);
        }
    }

    fn pause_all(&self) {
        for t in self.torrents.read().values() {
            if t.lt_id >= 0 {
                TorrentManager::pause(t.lt_id);
            }
        }
        let tasks = self.tasks.read();
        for m in tasks.values() {
            m.task.pause();
            if let Ok(conn) = self.conn.lock() {
                let _ = downloads::update_download_status(&conn, m.task.id, DownloadStatus::Paused);
            }
        }
    }

    fn resume_all(self: &Arc<Self>) {
        for t in self.torrents.read().values() {
            if t.lt_id >= 0 {
                TorrentManager::resume(t.lt_id);
            }
        }
        let ids: Vec<DownloadId> = self.tasks.read().keys().copied().collect();
        for id in ids {
            self.resume_download(id);
        }
    }

    fn get_all_snapshots(&self) -> Vec<DownloadSnapshot> {
        let mut all: Vec<DownloadSnapshot> = self.tasks.read().values().map(|m| m.task.get_snapshot()).collect();
        all.extend(self.torrent_snapshots());
        all
    }
}

/// A path to an existing .torrent file on this computer.
fn is_local_torrent(s: &str) -> bool {
    let s = s.trim();
    s.to_ascii_lowercase().ends_with(".torrent") && std::path::Path::new(s).is_file()
}

/// Name to show before libtorrent has the metadata.
fn torrent_display_name(source: &str, original: &str) -> String {
    if let Some(dn) = source
        .strip_prefix("magnet:?")
        .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("dn=")))
    {
        let decoded = urlencoding::decode(&dn.replace('+', " ")).map(|c| c.into_owned()).unwrap_or_else(|_| dn.to_string());
        if !decoded.trim().is_empty() {
            return decoded;
        }
    }
    if source.starts_with("magnet:") {
        return "Magnet link (fetching metadata…)".to_string();
    }
    std::path::Path::new(original.split(['?', '#']).next().unwrap_or(original))
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Torrent".to_string())
}
