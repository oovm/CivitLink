use gg_runtime::{SourceMap, SourceMapEntry};

#[test]
fn test_source_map_new() {
    let map = SourceMap::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn test_source_map_add_and_lookup() {
    let mut map = SourceMap::new();
    map.add_entry(SourceMapEntry { bytecode_offset: 0, source_file: "test.gg".to_string(), source_line: 1, source_column: 1 });
    map.add_entry(SourceMapEntry { bytecode_offset: 10, source_file: "test.gg".to_string(), source_line: 5, source_column: 3 });

    assert_eq!(map.len(), 2);

    let entry = map.lookup(0).unwrap();
    assert_eq!(entry.source_line, 1);

    let entry = map.lookup(10).unwrap();
    assert_eq!(entry.source_line, 5);

    let entry = map.lookup(5).unwrap();
    assert_eq!(entry.source_line, 1);
}

#[test]
fn test_source_map_lookup_empty() {
    let map = SourceMap::new();
    assert!(map.lookup(0).is_none());
}

#[test]
fn test_source_map_lookup_beyond_last() {
    let mut map = SourceMap::new();
    map.add_entry(SourceMapEntry { bytecode_offset: 0, source_file: "test.gg".to_string(), source_line: 1, source_column: 1 });

    let entry = map.lookup(100).unwrap();
    assert_eq!(entry.source_line, 1);
}

#[test]
fn test_source_map_insertion_order() {
    let mut map = SourceMap::new();
    map.add_entry(SourceMapEntry { bytecode_offset: 20, source_file: "test.gg".to_string(), source_line: 3, source_column: 1 });
    map.add_entry(SourceMapEntry { bytecode_offset: 5, source_file: "test.gg".to_string(), source_line: 1, source_column: 1 });
    map.add_entry(SourceMapEntry { bytecode_offset: 10, source_file: "test.gg".to_string(), source_line: 2, source_column: 1 });

    assert_eq!(map.entries[0].bytecode_offset, 5);
    assert_eq!(map.entries[1].bytecode_offset, 10);
    assert_eq!(map.entries[2].bytecode_offset, 20);
}
