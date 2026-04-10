use gg_asset::{AssetCache, AssetServer, Handle, LoadState, TextAsset};
use std::{
    sync::{Arc, Barrier},
    thread,
};

#[test]
fn test_cache_insert_and_get() {
    let cache = AssetCache::new();
    let handle = cache.insert("test.txt", TextAsset::new("test", "hello".to_string()));
    let asset = cache.get(&handle).unwrap();
    assert_eq!(asset.content(), "hello");
    assert_eq!(asset.name(), "test");
}

#[test]
fn test_cache_contains_and_remove() {
    let cache = AssetCache::new();
    cache.insert("test.txt", TextAsset::new("test", "hello".to_string()));
    assert!(cache.contains("test.txt"));
    assert!(cache.remove("test.txt"));
    assert!(!cache.contains("test.txt"));
    assert!(!cache.remove("test.txt"));
}

#[test]
fn test_handle_equality_and_cloning() {
    let cache = AssetCache::new();
    let handle1 = cache.insert("test.txt", TextAsset::new("test", "hello".to_string()));
    let handle2 = handle1.clone();
    assert_eq!(handle1, handle2);
    assert_eq!(handle1.id(), handle2.id());
    assert_eq!(handle1.path(), handle2.path());
}

#[test]
fn test_server_add_asset() {
    let server = AssetServer::new();
    let handle = server.add_asset("test.txt", TextAsset::new("test", "hello".to_string()));
    let asset = server.cache().get(&handle).unwrap();
    assert_eq!(asset.content(), "hello");
    assert_eq!(server.load_state(&handle), LoadState::Loaded);
}

#[test]
fn test_load_state_transitions() {
    let server = AssetServer::new();

    let untracked_handle: Handle<TextAsset> = Handle::new(999, Arc::from("nonexistent.txt"));
    assert_eq!(server.load_state(&untracked_handle), LoadState::NotLoaded);

    let loaded_handle = server.add_asset("loaded.txt", TextAsset::new("loaded", "data".to_string()));
    assert_eq!(server.load_state(&loaded_handle), LoadState::Loaded);

    let failed_id = 777;
    server.load_states.insert(failed_id, LoadState::Failed("io error".to_string()));
    let failed_handle: Handle<TextAsset> = Handle::new(failed_id, Arc::from("failed.txt"));
    assert_eq!(server.load_state(&failed_handle), LoadState::Failed("io error".to_string()));

    let loading_id = 888;
    server.load_states.insert(loading_id, LoadState::Loading);
    let loading_handle: Handle<TextAsset> = Handle::new(loading_id, Arc::from("loading.txt"));
    assert_eq!(server.load_state(&loading_handle), LoadState::Loading);
}

#[test]
fn test_concurrent_access() {
    let cache = Arc::new(AssetCache::new());
    let barrier = Arc::new(Barrier::new(4));

    let mut handles = vec![];
    for i in 0..4 {
        let cache = Arc::clone(&cache);
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            barrier.wait();
            let path = format!("file_{}.txt", i);
            let h = cache.insert(&path, TextAsset::new(&path, format!("content_{}", i)));
            let asset = cache.get(&h).unwrap();
            assert_eq!(asset.content(), format!("content_{}", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for i in 0..4 {
        let path = format!("file_{}.txt", i);
        assert!(cache.contains(&path));
    }
}
