use gg_asset::{
    AssetChangeKind, AssetServer, AssetWatcher, AudioAsset, AudioFormat, BinaryAsset, BinaryLoader, FontAsset, TextAsset,
    TextLoader,
};
use std::sync::{Arc, Mutex};

#[test]
fn test_audio_format_from_extension() {
    assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
    assert_eq!(AudioFormat::from_extension("ogg"), Some(AudioFormat::Ogg));
    assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
    assert_eq!(AudioFormat::from_extension("flac"), Some(AudioFormat::Flac));
    assert_eq!(AudioFormat::from_extension("WAV"), Some(AudioFormat::Wav));
    assert_eq!(AudioFormat::from_extension("Mp3"), Some(AudioFormat::Mp3));
    assert_eq!(AudioFormat::from_extension("txt"), None);
    assert_eq!(AudioFormat::from_extension(""), None);
}

#[test]
fn test_audio_asset_creation() {
    let data = vec![1u8, 2, 3, 4];
    let asset = AudioAsset::new(data.clone(), AudioFormat::Wav, 2, 44100);

    assert_eq!(asset.data(), &data);
    assert_eq!(asset.format(), AudioFormat::Wav);
    assert_eq!(asset.channels(), 2);
    assert_eq!(asset.sample_rate(), 44100);
}

#[test]
fn test_audio_asset_default_values() {
    let asset = AudioAsset::new(vec![], AudioFormat::Ogg, 2, 44100);
    assert_eq!(asset.channels(), 2);
    assert_eq!(asset.sample_rate(), 44100);
}

#[test]
fn test_on_load_callback_triggered() {
    let mut server = AssetServer::new();
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    server.on_load::<TextAsset>(move |_id| {
        *called_clone.lock().unwrap() = true;
    });

    server.register_loader::<TextAsset, TextLoader>(TextLoader);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(server.load::<TextAsset>("nonexistent.txt"));

    assert!(result.is_err());
    assert!(!*called.lock().unwrap());
}

#[test]
fn test_on_load_callback_with_add_asset_not_triggered() {
    let mut server = AssetServer::new();
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    server.on_load::<TextAsset>(move |_id| {
        *called_clone.lock().unwrap() = true;
    });

    let _handle = server.add_asset("test.txt", TextAsset::new("test", "hello".to_string()));

    assert!(!*called.lock().unwrap());
}

#[test]
fn test_multiple_on_load_callbacks() {
    let mut server = AssetServer::new();
    let count = Arc::new(Mutex::new(0u32));
    let count_clone1 = count.clone();
    let count_clone2 = count.clone();

    server.on_load::<BinaryAsset>(move |_id| {
        *count_clone1.lock().unwrap() += 1;
    });
    server.on_load::<BinaryAsset>(move |_id| {
        *count_clone2.lock().unwrap() += 10;
    });

    server.register_loader::<BinaryAsset, BinaryLoader>(BinaryLoader);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(server.load::<BinaryAsset>("nonexistent.bin"));

    assert!(result.is_err());
    assert_eq!(*count.lock().unwrap(), 0);
}

#[test]
fn test_font_asset_invalid_data() {
    let result = FontAsset::new(vec![0u8; 4]);
    assert!(result.is_err());
}

#[test]
fn test_asset_watcher_creation() {
    let mut watcher = AssetWatcher::new();
    assert!(watcher.poll_changes().is_empty());
}

#[test]
fn test_asset_watcher_default() {
    let mut watcher = AssetWatcher::default();
    assert!(watcher.poll_changes().is_empty());
}

#[test]
fn test_asset_change_kind_equality() {
    assert_eq!(AssetChangeKind::Modified, AssetChangeKind::Modified);
    assert_ne!(AssetChangeKind::Modified, AssetChangeKind::Created);
    assert_ne!(AssetChangeKind::Created, AssetChangeKind::Deleted);
}

#[tokio::test]
async fn test_manual_reload() {
    let temp_dir = std::env::temp_dir().join("gg_asset_test_reload");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("test_reload.txt");
    std::fs::write(&file_path, "original content").unwrap();

    let mut server = AssetServer::new();
    server.register_loader::<TextAsset, TextLoader>(TextLoader);

    let path = file_path.to_str().unwrap();
    let handle = server.load::<TextAsset>(path).await.unwrap();

    let asset = server.cache().get(&handle).unwrap();
    assert_eq!(asset.content(), "original content");

    std::fs::write(&file_path, "updated content").unwrap();

    let handle = server.reload::<TextAsset>(path).await.unwrap();
    let asset = server.cache().get(&handle).unwrap();
    assert_eq!(asset.content(), "updated content");

    assert_eq!(server.load_state(&handle), gg_asset::LoadState::Loaded);

    std::fs::remove_dir_all(&temp_dir).ok();
}

#[tokio::test]
async fn test_reload_fallback_to_load() {
    let temp_dir = std::env::temp_dir().join("gg_asset_test_reload_fallback");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("test_fallback.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    let mut server = AssetServer::new();
    server.register_loader::<TextAsset, TextLoader>(TextLoader);

    let path = file_path.to_str().unwrap();
    let handle = server.reload::<TextAsset>(path).await.unwrap();

    let asset = server.cache().get(&handle).unwrap();
    assert_eq!(asset.content(), "hello world");

    std::fs::remove_dir_all(&temp_dir).ok();
}

#[tokio::test]
async fn test_on_reload_callback() {
    let temp_dir = std::env::temp_dir().join("gg_asset_test_on_reload");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("test_callback.txt");
    std::fs::write(&file_path, "original").unwrap();

    let mut server = AssetServer::new();
    server.register_loader::<TextAsset, TextLoader>(TextLoader);

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();
    let reloaded_path = Arc::new(Mutex::new(String::new()));
    let reloaded_path_clone = reloaded_path.clone();
    server.on_reload::<TextAsset>(move |path| {
        *called_clone.lock().unwrap() = true;
        *reloaded_path_clone.lock().unwrap() = path.to_string();
    });

    let path = file_path.to_str().unwrap();
    let _handle = server.load::<TextAsset>(path).await.unwrap();

    assert!(!*called.lock().unwrap());

    std::fs::write(&file_path, "updated").unwrap();
    let _handle = server.reload::<TextAsset>(path).await.unwrap();

    assert!(*called.lock().unwrap());
    assert_eq!(*reloaded_path.lock().unwrap(), path);

    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_on_reload_callback_not_triggered_by_add_asset() {
    let mut server = AssetServer::new();
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    server.on_reload::<TextAsset>(move |_path| {
        *called_clone.lock().unwrap() = true;
    });

    let _handle = server.add_asset("test.txt", TextAsset::new("test", "hello".to_string()));

    assert!(!*called.lock().unwrap());
}

#[test]
fn test_watch_directory() {
    let temp_dir = std::env::temp_dir().join("gg_asset_test_watch");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let mut server = AssetServer::new();
    let path = temp_dir.to_str().unwrap();
    assert!(server.watch_directory(path).is_ok());

    server.unwatch();

    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_process_watcher_events_no_changes() {
    let mut server = AssetServer::new();
    server.process_watcher_events();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(server.process_pending_reloads());
}
