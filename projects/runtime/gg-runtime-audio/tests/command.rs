use gg_runtime_audio::{
    AudioCommand,
    effect::{AudioEffectKind, EffectType},
    playback::PlaybackId,
    sound::SoundId,
};

#[test]
fn test_audio_command_play() {
    let cmd = AudioCommand::Play { sound_id: SoundId(10), volume: 0.75, looped: true, group: Some("bgm".to_string()) };
    match cmd {
        AudioCommand::Play { sound_id, volume, looped, group } => {
            assert_eq!(sound_id, SoundId(10));
            assert!((volume - 0.75).abs() < f32::EPSILON);
            assert!(looped);
            assert_eq!(group, Some("bgm".to_string()));
        }
        _ => panic!("Expected Play command"),
    }
}

#[test]
fn test_audio_command_play_no_group() {
    let cmd = AudioCommand::Play { sound_id: SoundId(1), volume: 0.5, looped: false, group: None };
    match cmd {
        AudioCommand::Play { group, .. } => {
            assert!(group.is_none());
        }
        _ => panic!("Expected Play command"),
    }
}

#[test]
fn test_audio_command_stop() {
    let cmd = AudioCommand::Stop { sound_id: SoundId(20) };
    match cmd {
        AudioCommand::Stop { sound_id } => {
            assert_eq!(sound_id, SoundId(20));
        }
        _ => panic!("Expected Stop command"),
    }
}

#[test]
fn test_audio_command_set_volume() {
    let cmd = AudioCommand::SetVolume { sound_id: SoundId(30), volume: 0.5 };
    match cmd {
        AudioCommand::SetVolume { sound_id, volume } => {
            assert_eq!(sound_id, SoundId(30));
            assert!((volume - 0.5).abs() < f32::EPSILON);
        }
        _ => panic!("Expected SetVolume command"),
    }
}

#[test]
fn test_audio_command_pause_resume() {
    let pause = AudioCommand::Pause { sound_id: SoundId(40) };
    let resume = AudioCommand::Resume { sound_id: SoundId(40) };
    match pause {
        AudioCommand::Pause { sound_id } => {
            assert_eq!(sound_id, SoundId(40));
        }
        _ => panic!("Expected Pause command"),
    }
    match resume {
        AudioCommand::Resume { sound_id } => {
            assert_eq!(sound_id, SoundId(40));
        }
        _ => panic!("Expected Resume command"),
    }
}

#[test]
fn test_audio_command_set_group_volume() {
    let cmd = AudioCommand::SetGroupVolume { group: "bgm".to_string(), volume: 0.5 };
    match cmd {
        AudioCommand::SetGroupVolume { group, volume } => {
            assert_eq!(group, "bgm");
            assert!((volume - 0.5).abs() < f32::EPSILON);
        }
        _ => panic!("Expected SetGroupVolume command"),
    }
}

#[test]
fn test_audio_command_set_group_mute() {
    let cmd = AudioCommand::SetGroupMute { group: "sfx".to_string(), mute: true };
    match cmd {
        AudioCommand::SetGroupMute { group, mute } => {
            assert_eq!(group, "sfx");
            assert!(mute);
        }
        _ => panic!("Expected SetGroupMute command"),
    }
}

#[test]
fn test_audio_command_stop_playback() {
    let cmd = AudioCommand::StopPlayback { playback_id: PlaybackId(1) };
    match cmd {
        AudioCommand::StopPlayback { playback_id } => {
            assert_eq!(playback_id, PlaybackId(1));
        }
        _ => panic!("Expected StopPlayback command"),
    }
}

#[test]
fn test_audio_command_set_playback_volume() {
    let cmd = AudioCommand::SetPlaybackVolume { playback_id: PlaybackId(2), volume: 0.8 };
    match cmd {
        AudioCommand::SetPlaybackVolume { playback_id, volume } => {
            assert_eq!(playback_id, PlaybackId(2));
            assert!((volume - 0.8).abs() < f32::EPSILON);
        }
        _ => panic!("Expected SetPlaybackVolume command"),
    }
}

#[test]
fn test_audio_command_pause_resume_playback() {
    let pause = AudioCommand::PausePlayback { playback_id: PlaybackId(3) };
    let resume = AudioCommand::ResumePlayback { playback_id: PlaybackId(3) };
    match pause {
        AudioCommand::PausePlayback { playback_id } => {
            assert_eq!(playback_id, PlaybackId(3));
        }
        _ => panic!("Expected PausePlayback command"),
    }
    match resume {
        AudioCommand::ResumePlayback { playback_id } => {
            assert_eq!(playback_id, PlaybackId(3));
        }
        _ => panic!("Expected ResumePlayback command"),
    }
}

#[test]
fn test_audio_command_attach_effect() {
    let cmd =
        AudioCommand::AttachEffect { playback_id: PlaybackId(4), effect: AudioEffectKind::Reverb { decay: 0.8, mix: 0.3 } };
    match cmd {
        AudioCommand::AttachEffect { playback_id, effect } => {
            assert_eq!(playback_id, PlaybackId(4));
            assert_eq!(effect.effect_type(), EffectType::Reverb);
        }
        _ => panic!("Expected AttachEffect command"),
    }
}

#[test]
fn test_audio_command_detach_effect() {
    let cmd = AudioCommand::DetachEffect { playback_id: PlaybackId(5), effect_type: EffectType::LowPass };
    match cmd {
        AudioCommand::DetachEffect { playback_id, effect_type } => {
            assert_eq!(playback_id, PlaybackId(5));
            assert_eq!(effect_type, EffectType::LowPass);
        }
        _ => panic!("Expected DetachEffect command"),
    }
}

#[test]
fn test_audio_command_set_effect_param() {
    let cmd = AudioCommand::SetEffectParam {
        playback_id: PlaybackId(6),
        effect_type: EffectType::Reverb,
        param: "decay".to_string(),
        value: 0.9,
    };
    match cmd {
        AudioCommand::SetEffectParam { playback_id, effect_type, param, value } => {
            assert_eq!(playback_id, PlaybackId(6));
            assert_eq!(effect_type, EffectType::Reverb);
            assert_eq!(param, "decay");
            assert!((value - 0.9).abs() < f32::EPSILON);
        }
        _ => panic!("Expected SetEffectParam command"),
    }
}
