use gg_runtime_audio::{PlaybackId, PlaybackInfo, PlaybackState, SoundId};

#[test]
fn test_playback_id_copy_eq_hash() {
    let a = PlaybackId(42);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn test_playback_id_different() {
    let a = PlaybackId(1);
    let b = PlaybackId(2);
    assert_ne!(a, b);
}

#[test]
fn test_playback_state_variants() {
    assert_eq!(PlaybackState::Playing, PlaybackState::Playing);
    assert_ne!(PlaybackState::Playing, PlaybackState::Paused);
    assert_ne!(PlaybackState::Paused, PlaybackState::Stopped);
}

#[test]
fn test_playback_info_construction() {
    let info = PlaybackInfo {
        playback_id: PlaybackId(1),
        sound_id: SoundId(10),
        volume: 0.8,
        looped: true,
        group: Some("bgm".to_string()),
        state: PlaybackState::Playing,
    };
    assert_eq!(info.playback_id, PlaybackId(1));
    assert_eq!(info.sound_id, SoundId(10));
    assert!((info.volume - 0.8).abs() < f32::EPSILON);
    assert!(info.looped);
    assert_eq!(info.group, Some("bgm".to_string()));
    assert_eq!(info.state, PlaybackState::Playing);
}

#[test]
fn test_playback_info_no_group() {
    let info = PlaybackInfo {
        playback_id: PlaybackId(2),
        sound_id: SoundId(20),
        volume: 0.5,
        looped: false,
        group: None,
        state: PlaybackState::Paused,
    };
    assert!(info.group.is_none());
    assert_eq!(info.state, PlaybackState::Paused);
}
