use crate::config::Settings;
use crate::exception::{DownloadException, IdmException, Result, StorageException};
use crate::model::{DownloadId, DownloadSegment, DownloadSnapshot, DownloadStatus};
use crate::service::network::NetworkClient;
use futures::StreamExt;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub const MIN_SEGMENT_SIZE_BYTES: u64 = 1024 * 1024; // 1 MB minimum segment
pub const MIN_SPLIT_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024; // 2 MB minimum to allow work-stealing split

#[derive(Debug)]
pub struct WorkerAssignment {
    pub segment_id: u32,
    pub resume_offset: u64,
    pub end_offset: u64,
}

/// Dynamic Work-Stealing Segment Scheduler inspired by IDM
pub struct DynamicScheduler {
    pub total_bytes: u64,
    pub segments: Vec<DownloadSegment>,
    pub max_segments: usize,
    next_segment_id: u32,
}

impl DynamicScheduler {
    pub fn new(total_bytes: u64, initial_connections: usize) -> Self {
        let n = initial_connections.clamp(1, 16);
        let mut segments = Vec::with_capacity(n);
        let chunk_size = total_bytes / (n as u64);

        for i in 0..n {
            let start = (i as u64) * chunk_size;
            let end = if i == n - 1 {
                total_bytes - 1
            } else {
                start + chunk_size - 1
            };

            segments.push(DownloadSegment {
                segment_id: i as u32,
                start_offset: start,
                end_offset: end,
                downloaded_bytes: 0,
                is_active: false,
                is_completed: false,
            });
        }

        Self {
            total_bytes,
            segments,
            max_segments: 32,
            next_segment_id: n as u32,
        }
    }

    /// Claim unworked segment or steal work from the largest remaining segment
    pub fn request_work(&mut self) -> Option<WorkerAssignment> {
        // 1. First priority: find an inactive, non-completed initial segment
        for seg in self.segments.iter_mut() {
            if !seg.is_active && !seg.is_completed {
                seg.is_active = true;
                return Some(WorkerAssignment {
                    segment_id: seg.segment_id,
                    resume_offset: seg.start_offset + seg.downloaded_bytes,
                    end_offset: seg.end_offset,
                });
            }
        }

        // 2. Second priority: work-stealing dynamic split
        if self.segments.len() >= self.max_segments {
            return None;
        }

        let mut candidate_idx = None;
        let mut max_remaining = 0;

        for (idx, seg) in self.segments.iter().enumerate() {
            if seg.is_active && !seg.is_completed {
                let remaining = seg.remaining_bytes();
                if remaining >= MIN_SPLIT_THRESHOLD_BYTES && remaining > max_remaining {
                    max_remaining = remaining;
                    candidate_idx = Some(idx);
                }
            }
        }

        let idx = candidate_idx?;
        let half = max_remaining / 2;

        let target_seg = &mut self.segments[idx];
        let original_end = target_seg.end_offset;
        let split_point = original_end - half;

        // Clamp the existing segment
        target_seg.end_offset = split_point;

        // Allocate the new stolen segment
        let new_id = self.next_segment_id;
        self.next_segment_id += 1;

        let new_seg = DownloadSegment {
            segment_id: new_id,
            start_offset: split_point + 1,
            end_offset: original_end,
            downloaded_bytes: 0,
            is_active: true,
            is_completed: false,
        };

        let assignment = WorkerAssignment {
            segment_id: new_id,
            resume_offset: split_point + 1,
            end_offset: original_end,
        };

        self.segments.push(new_seg);
        Some(assignment)
    }

    pub fn record_progress(&mut self, segment_id: u32, bytes_written: u64) {
        if let Some(seg) = self.segments.iter_mut().find(|s| s.segment_id == segment_id) {
            seg.downloaded_bytes += bytes_written;
            if seg.downloaded_bytes >= seg.total_bytes() {
                seg.is_completed = true;
                seg.is_active = false;
            }
        }
    }

    pub fn is_all_completed(&self) -> bool {
        self.segments.iter().all(|s| s.is_completed)
    }

    pub fn get_segments(&self) -> Vec<DownloadSegment> {
        self.segments.clone()
    }
}

/// Rolling Speed Meter calculation using a sliding window
pub struct SpeedMeter {
    samples: VecDeque<(Instant, u64)>,
    last_bytes: u64,
}

impl SpeedMeter {
    pub fn new(initial_bytes: u64) -> Self {
        let mut samples = VecDeque::with_capacity(16);
        samples.push_back((Instant::now(), initial_bytes));
        Self {
            samples,
            last_bytes: initial_bytes,
        }
    }

    pub fn update(&mut self, current_bytes: u64) -> f64 {
        let now = Instant::now();
        self.samples.push_back((now, current_bytes));
        self.last_bytes = current_bytes;

        // Keep 2-second sliding window
        while let Some(&(time, _)) = self.samples.front() {
            if now.duration_since(time) > Duration::from_secs(2) && self.samples.len() > 2 {
                self.samples.pop_front();
            } else {
                break;
            }
        }

        if self.samples.len() < 2 {
            return 0.0;
        }

        let (first_time, first_bytes) = *self.samples.front().unwrap();
        let (last_time, last_bytes) = *self.samples.back().unwrap();

        let elapsed = last_time.duration_since(first_time).as_secs_f64();
        if elapsed > 0.05 {
            let bytes_diff = (last_bytes.saturating_sub(first_bytes)) as f64;
            (bytes_diff / elapsed).max(0.0)
        } else {
            0.0
        }
    }
}

pub struct DownloadTask {
    pub id: DownloadId,
    pub url: String,
    pub destination: PathBuf,
    pub filename: String,
    pub total_bytes: Arc<AtomicU64>,
    pub downloaded_bytes: Arc<AtomicU64>,
    pub status: Arc<RwLock<DownloadStatus>>,
    pub stop_requested: Arc<AtomicBool>,
    pub error_message: Arc<RwLock<Option<String>>>,
    pub client: NetworkClient,
    pub settings: Settings,
    snapshot_tx: broadcast::Sender<DownloadSnapshot>,
}

impl DownloadTask {
    pub fn new(
        id: DownloadId,
        url: String,
        destination: PathBuf,
        settings: Settings,
    ) -> Result<Self> {
        let filename = destination
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download.bin")
            .to_string();

        let (snapshot_tx, _) = broadcast::channel(32);
        let client = NetworkClient::new()?;

        Ok(Self {
            id,
            url,
            destination,
            filename,
            total_bytes: Arc::new(AtomicU64::new(0)),
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
            status: Arc::new(RwLock::new(DownloadStatus::Queued)),
            stop_requested: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(RwLock::new(None)),
            client,
            settings,
            snapshot_tx,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot> {
        self.snapshot_tx.subscribe()
    }

    pub fn get_snapshot(&self) -> DownloadSnapshot {
        let total = self.total_bytes.load(Ordering::Relaxed);
        let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);
        let status = *self.status.read();
        let error_message = self.error_message.read().clone();

        let mut snapshot = DownloadSnapshot {
            id: self.id,
            url: self.url.clone(),
            destination: self.destination.to_string_lossy().to_string(),
            filename: self.filename.clone(),
            total_bytes: total,
            downloaded_bytes: downloaded,
            speed_bytes_per_sec: 0.0,
            status,
            segment_count: 1,
            segments: Vec::new(),
            is_torrent: false,
            peers: 0,
            seeds: 0,
            error_message,
            progress_percent: 0.0,
            eta_seconds: None,
        };
        snapshot.progress_percent = snapshot.calculate_progress();
        snapshot.eta_seconds = snapshot.calculate_eta();
        snapshot
    }

    pub fn pause(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        *self.status.write() = DownloadStatus::Paused;
        let _ = self.snapshot_tx.send(self.get_snapshot());
    }

    pub fn resume(&self) {
        self.stop_requested.store(false, Ordering::SeqCst);
        *self.status.write() = DownloadStatus::Downloading;
        let _ = self.snapshot_tx.send(self.get_snapshot());
    }

    pub async fn run(&self) -> Result<()> {
        self.stop_requested.store(false, Ordering::SeqCst);
        *self.status.write() = DownloadStatus::Downloading;
        *self.error_message.write() = None;

        info!("Starting download #{} for {}", self.id, self.url);

        if let Some(parent) = self.destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
        }

        let caps = match self.client.probe(&self.url).await {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Probe failed: {}", e);
                *self.error_message.write() = Some(err_msg.clone());
                *self.status.write() = DownloadStatus::Error;
                let _ = self.snapshot_tx.send(self.get_snapshot());
                return Err(e);
            }
        };

        if let Some(len) = caps.content_length {
            self.total_bytes.store(len, Ordering::SeqCst);
        }

        let res = if caps.is_segmented_download_supported() {
            let total_len = caps.content_length.unwrap();
            self.run_segmented(total_len).await
        } else {
            self.run_single_stream().await
        };

        match res {
            Ok(_) => {
                if !self.stop_requested.load(Ordering::SeqCst) {
                    *self.status.write() = DownloadStatus::Completed;
                    info!("Download #{} successfully completed", self.id);
                }
            }
            Err(e) => {
                if !self.stop_requested.load(Ordering::SeqCst) {
                    *self.error_message.write() = Some(e.to_string());
                    *self.status.write() = DownloadStatus::Error;
                    error!("Download #{} failed: {}", self.id, e);
                }
            }
        }

        let _ = self.snapshot_tx.send(self.get_snapshot());
        Ok(())
    }

    async fn run_segmented(&self, total_bytes: u64) -> Result<()> {
        let temp_path = self.destination.with_extension("idm_tmp");
        let worker_count = (self.settings.max_segments as usize).clamp(2, 16);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_path)
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;

        file.set_len(total_bytes)
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
        let file_shared = Arc::new(tokio::sync::Mutex::new(file));

        let scheduler = Arc::new(parking_lot::Mutex::new(DynamicScheduler::new(
            total_bytes,
            worker_count,
        )));

        let mut speed_meter = SpeedMeter::new(0);
        let mut worker_handles = Vec::new();

        for worker_id in 0..worker_count {
            let scheduler_clone = Arc::clone(&scheduler);
            let client_clone = self.client.clone();
            let url_clone = self.url.clone();
            let file_clone = Arc::clone(&file_shared);
            let stop_flag = Arc::clone(&self.stop_requested);
            let downloaded_clone = Arc::clone(&self.downloaded_bytes);

            worker_handles.push(crate::service::spawn(async move {
                while !stop_flag.load(Ordering::Relaxed) {
                    let work = {
                        let mut sched = scheduler_clone.lock();
                        sched.request_work()
                    };

                    let work = match work {
                        Some(w) => w,
                        None => break,
                    };

                    let range_stream_res = client_clone
                        .stream_range(&url_clone, work.resume_offset, work.end_offset)
                        .await;

                    let mut stream = match range_stream_res {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("Worker #{} range request failed: {}", worker_id, e);
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            continue;
                        }
                    };

                    let mut current_offset = work.resume_offset;
                    while let Some(chunk_res) = stream.next().await {
                        if stop_flag.load(Ordering::Relaxed) {
                            return;
                        }

                        match chunk_res {
                            Ok(chunk) => {
                                let chunk_len = chunk.len() as u64;

                                {
                                    let mut f = file_clone.lock().await;
                                    if let Err(e) = f.seek(SeekFrom::Start(current_offset)).await {
                                        error!("File seek failed: {}", e);
                                        return;
                                    }
                                    if let Err(e) = f.write_all(&chunk).await {
                                        error!("File write failed: {}", e);
                                        return;
                                    }
                                }

                                current_offset += chunk_len;
                                scheduler_clone.lock().record_progress(work.segment_id, chunk_len);
                                downloaded_clone.fetch_add(chunk_len, Ordering::Relaxed);
                            }
                            Err(e) => {
                                warn!("Error reading stream chunk in worker #{}: {}", worker_id, e);
                                break;
                            }
                        }
                    }
                }
            }));
        }

        while !self.stop_requested.load(Ordering::Relaxed) {
            let is_done = scheduler.lock().is_all_completed();
            let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);
            let speed = speed_meter.update(downloaded);

            let mut snap = self.get_snapshot();
            snap.speed_bytes_per_sec = speed;
            snap.segments = scheduler.lock().get_segments();
            let _ = self.snapshot_tx.send(snap);

            if is_done || downloaded >= total_bytes {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        for handle in worker_handles {
            let _ = handle.await;
        }

        if self.stop_requested.load(Ordering::SeqCst) {
            return Ok(());
        }

        file_shared
            .lock()
            .await
            .flush()
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
        let final_size = tokio::fs::metadata(&temp_path)
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?
            .len();
        if final_size != total_bytes {
            return Err(IdmException::Download(DownloadException::ChecksumMismatch {
                expected: format!("{} bytes", total_bytes),
                actual: format!("{} bytes", final_size),
            }));
        }

        self.atomic_finalize(&temp_path).await?;
        Ok(())
    }

    async fn run_single_stream(&self) -> Result<()> {
        let temp_path = self.destination.with_extension("idm_tmp");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_path)
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;

        let resume_offset = file.metadata().await.map(|m| m.len()).unwrap_or(0);
        let (_, mut stream) = self.client.stream_full(&self.url, resume_offset).await?;

        file.seek(SeekFrom::Start(resume_offset))
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
        self.downloaded_bytes.store(resume_offset, Ordering::SeqCst);

        let mut speed_meter = SpeedMeter::new(resume_offset);

        while let Some(chunk_res) = stream.next().await {
            if self.stop_requested.load(Ordering::Relaxed) {
                return Ok(());
            }

            let chunk = chunk_res?;
            file.write_all(&chunk)
                .await
                .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
            let new_total = self.downloaded_bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed) + chunk.len() as u64;

            let speed = speed_meter.update(new_total);
            let mut snap = self.get_snapshot();
            snap.speed_bytes_per_sec = speed;
            let _ = self.snapshot_tx.send(snap);
        }

        file.flush()
            .await
            .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
        self.atomic_finalize(&temp_path).await?;
        Ok(())
    }

    async fn atomic_finalize(&self, temp_path: &Path) -> Result<()> {
        let backup_path = self.destination.with_extension("previous");
        let had_dest = self.destination.exists();

        if had_dest {
            let _ = tokio::fs::remove_file(&backup_path).await;
            tokio::fs::rename(&self.destination, &backup_path)
                .await
                .map_err(|e| IdmException::Storage(StorageException::Io(e)))?;
        }

        if let Err(e) = tokio::fs::rename(temp_path, &self.destination).await {
            if had_dest {
                let _ = tokio::fs::rename(&backup_path, &self.destination).await;
            }
            return Err(IdmException::Storage(StorageException::Io(e)));
        }

        if had_dest {
            let _ = tokio::fs::remove_file(&backup_path).await;
        }

        Ok(())
    }
}
