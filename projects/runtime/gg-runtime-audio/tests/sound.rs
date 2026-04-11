use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use gg_runtime_audio::{SoundDescriptor, SoundFormat, SoundId};

fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_sound_id_copy_eq_hash() {
    let a = SoundId(42);
    let b = a;
    assert_eq!(a, b);
    assert_eq!(calculate_hash(&a), calculate_hash(&b));
}

#[test]
fn test_sound_id_different() {
    let a = SoundId(1);
    let b = SoundId(2);
    assert_ne!(a, b);
}

#[test]
fn test_sound_format_variants() {
    let wav = SoundFormat::Wav;
    let ogg = SoundFormat::Ogg;
    let mp3 = SoundFormat::Mp3;
    let flac = SoundFormat::Flac;
    assert_eq!(wav, SoundFormat::Wav);
    assert_eq!(ogg, SoundFormat::Ogg);
    assert_eq!(mp3, SoundFormat::Mp3);
    assert_eq!(flac, SoundFormat::Flac);
    assert_ne!(wav, ogg);
    assert_ne!(mp3, flac);
}

#[test]
fn test_sound_descriptor_construction() {
    let descriptor = SoundDescriptor {
        format: SoundFormat::Ogg,
        duration_secs: 3.5,
        channels: 2,
        sample_rate: 44100,
    };
    assert_eq!(descriptor.format, SoundFormat::Ogg);
    assert!((descriptor.duration_secs - 3.5).abs() < f64::EPSILON);
    assert_eq!(descriptor.channels, 2);
    assert_eq!(descriptor.sample_rate, 44100);
}
