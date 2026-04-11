use gg_runtime::{
    DiagnosticBuilder, DiagnosticCollector, DiagnosticLevel, DiagnosticSource, RuntimeDiagnostic, VecDiagnosticCollector,
};

#[test]
fn test_diagnostic_level_ordering() {
    assert!(DiagnosticLevel::Info < DiagnosticLevel::Warning);
    assert!(DiagnosticLevel::Warning < DiagnosticLevel::Error);
}

#[test]
fn test_diagnostic_builder_basic() {
    let d = DiagnosticBuilder::new(DiagnosticLevel::Error, DiagnosticSource::Script, "test error").build(1000);
    assert_eq!(d.level, DiagnosticLevel::Error);
    assert_eq!(d.source, DiagnosticSource::Script);
    assert_eq!(d.message, "test error");
    assert_eq!(d.timestamp_ns, 1000);
    assert!(d.stack_trace.is_none());
    assert!(d.source_location.is_none());
}

#[test]
fn test_diagnostic_builder_with_stack_trace() {
    let d = DiagnosticBuilder::new(DiagnosticLevel::Warning, DiagnosticSource::Hmr, "rollback")
        .stack_trace("frame1\nframe2")
        .build(2000);
    assert_eq!(d.stack_trace.unwrap(), "frame1\nframe2");
}

#[test]
fn test_diagnostic_builder_with_source_location() {
    let d = DiagnosticBuilder::new(DiagnosticLevel::Error, DiagnosticSource::Vm, "crash")
        .source_location("test.gg", 10, 5)
        .build(3000);
    let loc = d.source_location.unwrap();
    assert_eq!(loc.file, "test.gg");
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
}

#[test]
fn test_vec_diagnostic_collector_collect_and_drain() {
    let mut collector = VecDiagnosticCollector::new();
    collector.collect(DiagnosticBuilder::new(DiagnosticLevel::Info, DiagnosticSource::Runtime, "msg1").build(100));
    collector.collect(DiagnosticBuilder::new(DiagnosticLevel::Error, DiagnosticSource::Script, "msg2").build(200));

    let diagnostics = collector.drain();
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "msg1");
    assert_eq!(diagnostics[1].message, "msg2");

    assert!(collector.drain().is_empty());
}

#[test]
fn test_vec_diagnostic_collector_capacity() {
    let mut collector = VecDiagnosticCollector::with_capacity(3);
    for i in 0..5 {
        collector.collect(
            DiagnosticBuilder::new(DiagnosticLevel::Info, DiagnosticSource::Runtime, format!("msg{}", i)).build(i as u64),
        );
    }

    let diagnostics = collector.drain();
    assert_eq!(diagnostics.len(), 3);
    assert_eq!(diagnostics[0].message, "msg2");
    assert_eq!(diagnostics[2].message, "msg4");
}

#[test]
fn test_vec_diagnostic_collector_clear() {
    let mut collector = VecDiagnosticCollector::new();
    collector.collect(DiagnosticBuilder::new(DiagnosticLevel::Info, DiagnosticSource::Runtime, "msg").build(0));
    collector.clear();
    assert!(collector.drain().is_empty());
}
