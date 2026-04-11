use gg_runtime_audio::{AudioCommand, AudioContext, PlaybackId, SoundId};

#[test]
fn test_audio_context_new() {
    let ctx = AudioContext::new();
    assert!(ctx.commands().is_empty());
}

#[test]
fn test_audio_context_submit() {
    let mut ctx = AudioContext::new();
    ctx.submit(AudioCommand::Play { sound_id: SoundId(1), volume: 0.5, looped: false, group: None });
    assert_eq!(ctx.commands().len(), 1);
}

#[test]
fn test_audio_context_clear() {
    let mut ctx = AudioContext::new();
    ctx.submit(AudioCommand::Play { sound_id: SoundId(1), volume: 0.5, looped: false, group: None });
    assert_eq!(ctx.commands().len(), 1);
    ctx.clear();
    assert!(ctx.commands().is_empty());
}

#[test]
fn test_audio_context_multiple_commands() {
    let mut ctx = AudioContext::new();
    ctx.submit(AudioCommand::Play { sound_id: SoundId(1), volume: 0.8, looped: true, group: Some("bgm".to_string()) });
    ctx.submit(AudioCommand::Stop { sound_id: SoundId(2) });
    ctx.submit(AudioCommand::SetVolume { sound_id: SoundId(3), volume: 0.3 });
    assert_eq!(ctx.commands().len(), 3);
}

#[test]
fn test_audio_context_submit_play() {
    let mut ctx = AudioContext::new();
    ctx.submit_play(SoundId(1), 0.8, true, Some("bgm".to_string()));
    assert_eq!(ctx.commands().len(), 1);
    match &ctx.commands()[0] {
        AudioCommand::Play { sound_id, volume, looped, group } => {
            assert_eq!(*sound_id, SoundId(1));
            assert!((*volume - 0.8).abs() < f32::EPSILON);
            assert!(*looped);
            assert_eq!(group.as_deref(), Some("bgm"));
        }
        _ => panic!("Expected Play command"),
    }
}

#[test]
fn test_audio_context_submit_group_commands() {
    let mut ctx = AudioContext::new();
    ctx.submit_set_group_volume("bgm", 0.5);
    ctx.submit_set_group_mute("sfx", true);
    assert_eq!(ctx.commands().len(), 2);
}

#[test]
fn test_audio_context_submit_playback_commands() {
    let mut ctx = AudioContext::new();
    ctx.submit_stop_playback(PlaybackId(1));
    ctx.submit_set_playback_volume(PlaybackId(2), 0.7);
    ctx.submit_pause_playback(PlaybackId(3));
    ctx.submit_resume_playback(PlaybackId(4));
    assert_eq!(ctx.commands().len(), 4);
}
