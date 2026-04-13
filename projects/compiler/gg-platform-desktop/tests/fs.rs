use gg_core::platform::FileSystem;
use gg_platform_desktop::fs::DesktopFileSystem;
use std::path::{Path, PathBuf};

fn temp_dir() -> PathBuf {
    std::env::temp_dir().join("gg_platform_desktop_test")
}

fn setup() {
    let _ = std::fs::create_dir_all(temp_dir());
}

fn cleanup() {
    let _ = std::fs::remove_dir_all(temp_dir());
}

#[test]
fn test_write_and_read() {
    setup();
    let fs = DesktopFileSystem::new();
    let path = temp_dir().join("test_write.txt");
    let content = b"Hello, GG!";
    fs.write(&path, content).unwrap();
    let read = fs.read(&path).unwrap();
    assert_eq!(read, content);
    cleanup();
}

#[test]
fn test_exists() {
    setup();
    let fs = DesktopFileSystem::new();
    let path = temp_dir().join("test_exists.txt");
    assert!(!fs.exists(&path));
    fs.write(&path, b"test").unwrap();
    assert!(fs.exists(&path));
    cleanup();
}

#[test]
fn test_create_dir_all() {
    setup();
    let fs = DesktopFileSystem::new();
    let dir = temp_dir().join("nested/dir");
    fs.create_dir_all(&dir).unwrap();
    assert!(dir.exists());
    cleanup();
}

#[test]
fn test_read_to_string() {
    setup();
    let fs = DesktopFileSystem::new();
    let path = temp_dir().join("test_string.txt");
    fs.write(&path, "Hello String".as_bytes()).unwrap();
    let content = fs.read_to_string(&path).unwrap();
    assert_eq!(content, "Hello String");
    cleanup();
}

#[test]
fn test_remove_file() {
    setup();
    let fs = DesktopFileSystem::new();
    let path = temp_dir().join("test_remove.txt");
    fs.write(&path, b"temp").unwrap();
    assert!(fs.exists(&path));
    fs.remove_file(&path).unwrap();
    assert!(!fs.exists(&path));
    cleanup();
}
