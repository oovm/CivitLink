use gg_runtime::{FrameProfileReport, FrameProfiler};

#[test]
fn test_frame_profiler_new() {
    let profiler = FrameProfiler::new();
    assert!(!profiler.is_enabled());
}

#[test]
fn test_frame_profiler_enabled() {
    let profiler = FrameProfiler::enabled();
    assert!(profiler.is_enabled());
}

#[test]
fn test_frame_profiler_set_enabled() {
    let mut profiler = FrameProfiler::new();
    profiler.set_enabled(true);
    assert!(profiler.is_enabled());
    profiler.set_enabled(false);
    assert!(!profiler.is_enabled());
}

#[test]
fn test_frame_profiler_begin_end_frame() {
    let mut profiler = FrameProfiler::enabled();
    profiler.begin_frame();
    profiler.begin_stage("Update");
    profiler.end_stage();
    profiler.end_frame();

    let report = profiler.report();
    assert_eq!(report.frame_count, 1);
    assert!(report.total_frame_ns > 0 || report.avg_frame_ns == 0);
}

#[test]
fn test_frame_profiler_multiple_frames() {
    let mut profiler = FrameProfiler::enabled();
    for _ in 0..5 {
        profiler.begin_frame();
        profiler.begin_stage("Update");
        profiler.end_stage();
        profiler.end_frame();
    }

    let report = profiler.report();
    assert_eq!(report.frame_count, 5);
}

#[test]
fn test_frame_profiler_record_alloc() {
    let mut profiler = FrameProfiler::enabled();
    profiler.begin_frame();
    profiler.record_alloc(64);
    profiler.record_alloc(128);
    profiler.end_frame();
}

#[test]
fn test_frame_profiler_report_empty() {
    let profiler = FrameProfiler::enabled();
    let report = profiler.report();
    assert_eq!(report.frame_count, 0);
    assert_eq!(report.avg_frame_ns, 0);
}

#[test]
fn test_frame_profiler_disabled_noop() {
    let mut profiler = FrameProfiler::new();
    profiler.begin_frame();
    profiler.begin_stage("Update");
    profiler.end_stage();
    profiler.end_frame();

    let report = profiler.report();
    assert_eq!(report.frame_count, 0);
}

#[test]
fn test_frame_profiler_stage_avg() {
    let mut profiler = FrameProfiler::enabled();
    for _ in 0..3 {
        profiler.begin_frame();
        profiler.begin_stage("Update");
        profiler.end_stage();
        profiler.begin_stage("Render");
        profiler.end_stage();
        profiler.end_frame();
    }

    let report = profiler.report();
    assert!(report.stage_avg_ns.iter().any(|(name, _)| name == "Update"));
    assert!(report.stage_avg_ns.iter().any(|(name, _)| name == "Render"));
}
