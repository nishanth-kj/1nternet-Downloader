use crate::exception::Result;
use crate::model::{DownloadSegment, DownloadSnapshot};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

pub const MIN_SEGMENT_SIZE_BYTES: u64 = 1024 * 1024; // 1 MB minimum segment
pub const MIN_SPLIT_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024; // 2 MB minimum to allow work-stealing split

#[derive(Debug)]
pub struct WorkerAssignment {
    pub segment_id: u32,
    pub resume_offset: u64,
    pub end_offset: u64,
}

/// Service trait defining active download lifecycle contract
pub trait DownloadService: Send + Sync {
    fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot>;
    fn get_snapshot(&self) -> DownloadSnapshot;
    fn pause(&self);
    fn resume(&self);
    fn run(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
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
