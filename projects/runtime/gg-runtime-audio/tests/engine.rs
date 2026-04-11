use gg_runtime_audio::{AudioCommand, AudioContext, SoundId};

#[test]
fn test_audio_context_new() {
    let ctx = AudioContext::new();
    assert!(ctx.commands().is_empty());
}

#[test]
fn test_audio_context_submit() {
    let mut ctx = AudioContext::new();
    ctx.submit(AudioCommand::Play {
        sound_id: SoundId(1),
        volume: 0.5,
        looped: false,
    });
    assert_eq!(ctx.commands().len(), 1);
}

#[test]
fn test_audio_context_clear() {
    let mut ctx = AudioContext::new();
    ctx.submit(AudioCommand::Play {
        sound_id: SoundId(1),
        volume: 0.5,
        looped: false,
    });
    assert_eq!(ctx.commands().len(), 1);
    ctx.clear();
    assert!(ctx.commands().is_empty());
}

#[test]
fn test_audio_context_multiple_commands() {
    let mut ctx = AudioContext::new();
    ctx.submit(AudioCommand::Play {
        sound_id: SoundId(1),
        volume: 0.8,
        looped: true,
    });
    ctx.submit(AudioCommand::Stop {
        sound_id: SoundId(2),
    });
    ctx.submit(AudioCommand::SetVolume {
        sound_id: SoundId(3),
        volume: 0.3,
    });
    assert_eq!(ctx.commands().len(), 3);
}
