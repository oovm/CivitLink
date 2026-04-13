use gg_vm::{HotspotDetector, InstructionCache, instruction_cache::FunctionKey};

#[test]
fn test_instruction_cache_new() {
    let cache = InstructionCache::new();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.hits(), 0);
    assert_eq!(cache.misses(), 0);
}

#[test]
fn test_instruction_cache_get_or_decode() {
    let mut cache = InstructionCache::new();
    let key = FunctionKey { module_name: "test".to_string(), function_name: "main".to_string() };

    let instructions = cache.get_or_decode(key.clone(), || {
        vec![gg_vm::instruction_cache::CachedInstruction { opcode: 1, operands: vec![42], byte_offset: 0 }]
    });

    assert_eq!(instructions.len(), 1);
    assert_eq!(instructions[0].opcode, 1);
    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.hits(), 0);

    let instructions2 = cache.get_or_decode(key.clone(), || panic!("should not call"));
    assert_eq!(instructions2.len(), 1);
    assert_eq!(cache.hits(), 1);
}

#[test]
fn test_instruction_cache_invalidate() {
    let mut cache = InstructionCache::new();
    let key = FunctionKey { module_name: "test".to_string(), function_name: "main".to_string() };

    cache.get_or_decode(key.clone(), || vec![]);
    assert_eq!(cache.len(), 1);

    assert!(cache.invalidate(&key));
    assert!(cache.is_empty());
    assert!(!cache.invalidate(&key));
}

#[test]
fn test_instruction_cache_invalidate_module() {
    let mut cache = InstructionCache::new();
    let key1 = FunctionKey { module_name: "mod_a".to_string(), function_name: "fn1".to_string() };
    let key2 = FunctionKey { module_name: "mod_a".to_string(), function_name: "fn2".to_string() };
    let key3 = FunctionKey { module_name: "mod_b".to_string(), function_name: "fn1".to_string() };

    cache.get_or_decode(key1, || vec![]);
    cache.get_or_decode(key2, || vec![]);
    cache.get_or_decode(key3, || vec![]);
    assert_eq!(cache.len(), 3);

    let removed = cache.invalidate_module("mod_a");
    assert_eq!(removed, 2);
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_instruction_cache_hit_rate() {
    let mut cache = InstructionCache::new();
    let key = FunctionKey { module_name: "test".to_string(), function_name: "main".to_string() };

    cache.get_or_decode(key.clone(), || vec![]);
    cache.get_or_decode(key, || vec![]);

    assert_eq!(cache.hit_rate(), 0.5);
}

#[test]
fn test_instruction_cache_clear() {
    let mut cache = InstructionCache::new();
    let key = FunctionKey { module_name: "test".to_string(), function_name: "main".to_string() };

    cache.get_or_decode(key, || vec![]);
    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.hits(), 0);
    assert_eq!(cache.misses(), 0);
}

#[test]
fn test_hotspot_detector_new() {
    let detector = HotspotDetector::new();
    assert!(!detector.is_enabled());
    assert_eq!(detector.tracked_count(), 0);
}

#[test]
fn test_hotspot_detector_enabled() {
    let detector = HotspotDetector::enabled();
    assert!(detector.is_enabled());
}

#[test]
fn test_hotspot_detector_enter_exit() {
    let mut detector = HotspotDetector::enabled();
    detector.on_function_enter("main");
    detector.on_function_exit("main");

    assert_eq!(detector.tracked_count(), 1);
    let report = detector.report();
    assert_eq!(report[0].name, "main");
    assert_eq!(report[0].call_count, 1);
}

#[test]
fn test_hotspot_detector_multiple_calls() {
    let mut detector = HotspotDetector::enabled();
    for _ in 0..5 {
        detector.on_function_enter("update");
        detector.on_function_exit("update");
    }

    let report = detector.report();
    assert_eq!(report[0].call_count, 5);
    assert!(report[0].avg_ns > 0 || report[0].total_ns == 0);
}

#[test]
fn test_hotspot_detector_disabled() {
    let mut detector = HotspotDetector::new();
    detector.on_function_enter("main");
    detector.on_function_exit("main");
    assert_eq!(detector.tracked_count(), 0);
}

#[test]
fn test_hotspot_detector_report_sorted() {
    let mut detector = HotspotDetector::enabled();

    detector.on_function_enter("fast");
    detector.on_function_exit("fast");

    std::thread::sleep(std::time::Duration::from_micros(100));

    detector.on_function_enter("slow");
    detector.on_function_exit("slow");

    let report = detector.report();
    assert_eq!(report.len(), 2);
    assert!(report[0].total_ns >= report[1].total_ns);
}

#[test]
fn test_hotspot_detector_clear() {
    let mut detector = HotspotDetector::enabled();
    detector.on_function_enter("main");
    detector.on_function_exit("main");
    assert_eq!(detector.tracked_count(), 1);

    detector.clear();
    assert_eq!(detector.tracked_count(), 0);
}
