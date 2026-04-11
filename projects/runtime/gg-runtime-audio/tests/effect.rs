use gg_runtime_audio::{AudioEffectChain, AudioEffectKind, EffectType};

#[test]
fn test_effect_type_variants() {
    assert_eq!(EffectType::Reverb, EffectType::Reverb);
    assert_ne!(EffectType::LowPass, EffectType::PitchShift);
}

#[test]
fn test_effect_kind_type() {
    let reverb = AudioEffectKind::Reverb { decay: 0.5, mix: 0.3 };
    assert_eq!(reverb.effect_type(), EffectType::Reverb);

    let lowpass = AudioEffectKind::LowPass { cutoff_frequency: 1000.0 };
    assert_eq!(lowpass.effect_type(), EffectType::LowPass);

    let pitch = AudioEffectKind::PitchShift { pitch: 1.5 };
    assert_eq!(pitch.effect_type(), EffectType::PitchShift);
}

#[test]
fn test_effect_kind_set_param() {
    let mut reverb = AudioEffectKind::Reverb { decay: 0.5, mix: 0.3 };
    reverb.set_param("decay", 0.8);
    match reverb {
        AudioEffectKind::Reverb { decay, .. } => assert!((decay - 0.8).abs() < f32::EPSILON),
        _ => panic!("Expected Reverb"),
    }

    let mut lowpass = AudioEffectKind::LowPass { cutoff_frequency: 1000.0 };
    lowpass.set_param("cutoff_frequency", 2000.0);
    match lowpass {
        AudioEffectKind::LowPass { cutoff_frequency } => assert!((cutoff_frequency - 2000.0).abs() < f32::EPSILON),
        _ => panic!("Expected LowPass"),
    }

    let mut pitch = AudioEffectKind::PitchShift { pitch: 1.0 };
    pitch.set_param("pitch", 2.0);
    match pitch {
        AudioEffectKind::PitchShift { pitch } => assert!((pitch - 2.0).abs() < f32::EPSILON),
        _ => panic!("Expected PitchShift"),
    }
}

#[test]
fn test_effect_chain_attach() {
    let mut chain = AudioEffectChain::new();
    assert!(chain.is_empty());

    chain.attach(AudioEffectKind::LowPass { cutoff_frequency: 1000.0 });
    assert_eq!(chain.len(), 1);
    assert!(chain.has_effect(EffectType::LowPass));

    chain.attach(AudioEffectKind::Reverb { decay: 0.5, mix: 0.3 });
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_effect_chain_detach() {
    let mut chain = AudioEffectChain::new();
    chain.attach(AudioEffectKind::LowPass { cutoff_frequency: 1000.0 });
    chain.attach(AudioEffectKind::Reverb { decay: 0.5, mix: 0.3 });

    assert!(chain.detach(EffectType::LowPass));
    assert_eq!(chain.len(), 1);
    assert!(!chain.has_effect(EffectType::LowPass));
    assert!(chain.has_effect(EffectType::Reverb));
}

#[test]
fn test_effect_chain_detach_nonexistent() {
    let mut chain = AudioEffectChain::new();
    assert!(!chain.detach(EffectType::Reverb));
}

#[test]
fn test_effect_chain_set_param() {
    let mut chain = AudioEffectChain::new();
    chain.attach(AudioEffectKind::Reverb { decay: 0.5, mix: 0.3 });

    assert!(chain.set_param(EffectType::Reverb, "decay", 0.9));
    match chain.effects()[0] {
        AudioEffectKind::Reverb { decay, .. } => assert!((decay - 0.9).abs() < f32::EPSILON),
        _ => panic!("Expected Reverb"),
    }

    assert!(!chain.set_param(EffectType::LowPass, "cutoff_frequency", 1000.0));
}
