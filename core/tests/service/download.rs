use idm_lib::service::{DynamicScheduler, SpeedMeter};

#[test]
fn test_scheduler_no_gaps_no_overlaps() {
    let total_bytes: u64 = 100 * 1024 * 1024; // 100 MB
    let initial_workers = 4;
    let sched = DynamicScheduler::new(total_bytes, initial_workers);

    assert_eq!(sched.segments.len(), 4);
    assert_eq!(sched.segments[0].start_offset, 0);

    let mut previous_end = 0;
    for (i, seg) in sched.segments.iter().enumerate() {
        if i > 0 {
            assert_eq!(
                seg.start_offset,
                previous_end + 1,
                "Gap or overlap detected between segments"
            );
        }
        assert!(
            seg.end_offset >= seg.start_offset,
            "Segment end must be >= start"
        );
        previous_end = seg.end_offset;
    }

    assert_eq!(
        previous_end,
        total_bytes - 1,
        "Last segment must reach total_bytes - 1"
    );
}

#[test]
fn test_dynamic_work_stealing_split() {
    let total_bytes: u64 = 20 * 1024 * 1024; // 20 MB
    let mut sched = DynamicScheduler::new(total_bytes, 1);

    let work1 = sched
        .request_work()
        .expect("Worker 0 should get initial segment");
    assert_eq!(work1.segment_id, 0);
    assert_eq!(work1.resume_offset, 0);
    assert_eq!(work1.end_offset, total_bytes - 1);

    let work2 = sched
        .request_work()
        .expect("Worker 1 should steal second half");
    assert_eq!(work2.segment_id, 1);
    assert!(work2.resume_offset > 0);
    assert_eq!(work2.end_offset, total_bytes - 1);

    assert_eq!(sched.segments[0].end_offset, work2.resume_offset - 1);
    assert_eq!(
        sched.segments[0].end_offset + 1,
        sched.segments[1].start_offset
    );
}

#[test]
fn test_split_threshold_enforcement() {
    let total_bytes: u64 = 1024 * 1024; // 1 MB (< 2MB threshold)
    let mut sched = DynamicScheduler::new(total_bytes, 1);

    let _work1 = sched.request_work().expect("Worker 0 claims work");
    let work2 = sched.request_work();
    assert!(
        work2.is_none(),
        "Should not split segment smaller than minimum threshold"
    );
}

#[test]
fn test_speed_meter() {
    let mut meter = SpeedMeter::new(0);
    std::thread::sleep(std::time::Duration::from_millis(50));
    let speed = meter.update(1024 * 512); // 512 KB
    assert!(speed > 0.0, "Speed must be calculated as > 0 bytes/sec");
}
