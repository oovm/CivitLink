use gg_runtime_audio::{command::AudioCommand, SoundId};

#[test]
fn test_audio_command_play() {
    let cmd = AudioCommand::Play {
        sound_id: SoundId(10),
        volume: 0.75,
        looped: true,
    };
    match cmd {
        AudioCommand::Play { sound_id, volume, looped } => {
            assert_eq!(sound_id, SoundId(10));
            assert!((volume - 0.75).abs() < f32::EPSILON);
            assert!(looped);
        }
        _ => panic!("Expected Play command"),
    }
}

#[test]
fn test_audio_command_stop() {
    let cmd = AudioCommand::Stop {
        sound_id: SoundId(20),
    };
    match cmd {
        AudioCommand::Stop { sound_id } => {
            assert_eq!(sound_id, SoundId(20));
        }
        _ => panic!("Expected Stop command"),
    }
}

#[test]
fn test_audio_command_set_volume() {
    let cmd = AudioCommand::SetVolume {
        sound_id: SoundId(30),
        volume: 0.5,
    };
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
    let pause = AudioCommand::Pause {
        sound_id: SoundId(40),
    };
    let resume = AudioCommand::Resume {
        sound_id: SoundId(40),
    };
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
