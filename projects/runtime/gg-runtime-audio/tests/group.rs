use gg_runtime_audio::{AudioGroup, AudioGroupManager};

#[test]
fn test_audio_group_new() {
    let group = AudioGroup::new("bgm");
    assert_eq!(group.name, "bgm");
    assert!((group.volume - 1.0).abs() < f32::EPSILON);
    assert!(!group.mute);
    assert_eq!(group.priority, 0);
}

#[test]
fn test_audio_group_set_volume() {
    let mut group = AudioGroup::new("sfx");
    group.set_volume(0.5);
    assert!((group.volume - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_audio_group_volume_clamped() {
    let mut group = AudioGroup::new("test");
    group.set_volume(1.5);
    assert!((group.volume - 1.0).abs() < f32::EPSILON);
    group.set_volume(-0.5);
    assert!((group.volume - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_audio_group_set_mute() {
    let mut group = AudioGroup::new("bgm");
    assert!(!group.mute);
    group.set_mute(true);
    assert!(group.mute);
}

#[test]
fn test_audio_group_effective_volume() {
    let mut group = AudioGroup::new("bgm");
    assert!((group.effective_volume() - 1.0).abs() < f32::EPSILON);
    group.set_volume(0.5);
    assert!((group.effective_volume() - 0.5).abs() < f32::EPSILON);
    group.set_mute(true);
    assert!((group.effective_volume() - 0.0).abs() < f32::EPSILON);
    group.set_mute(false);
    assert!((group.effective_volume() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_group_manager_create() {
    let mut manager = AudioGroupManager::new();
    assert!(manager.create_group("bgm").is_ok());
    assert!(manager.get_group("bgm").is_some());
}

#[test]
fn test_group_manager_create_duplicate() {
    let mut manager = AudioGroupManager::new();
    assert!(manager.create_group("bgm").is_ok());
    assert!(manager.create_group("bgm").is_err());
}

#[test]
fn test_group_manager_set_volume() {
    let mut manager = AudioGroupManager::new();
    manager.create_group("sfx").unwrap();
    assert!(manager.set_group_volume("sfx", 0.5).is_some());
    assert!((manager.group_volume("sfx").unwrap() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_group_manager_set_mute() {
    let mut manager = AudioGroupManager::new();
    manager.create_group("bgm").unwrap();
    assert!(manager.set_group_mute("bgm", true).is_some());
    assert!(manager.is_muted("bgm").unwrap());
}

#[test]
fn test_group_manager_nonexistent() {
    let manager = AudioGroupManager::new();
    assert!(manager.get_group("nonexistent").is_none());
    assert!(manager.group_volume("nonexistent").is_none());
    assert!(manager.is_muted("nonexistent").is_none());
}

#[test]
fn test_group_manager_remove() {
    let mut manager = AudioGroupManager::new();
    manager.create_group("test").unwrap();
    let removed = manager.remove_group("test");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "test");
    assert!(manager.get_group("test").is_none());
}
