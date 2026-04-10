/// 声音资源标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub u64);

/// 声音格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundFormat {
    /// WAV 格式
    Wav,
    /// OGG Vorbis 格式
    Ogg,
    /// MP3 格式
    Mp3,
    /// FLAC 格式
    Flac,
}

/// 声音资源描述符
#[derive(Debug, Clone)]
pub struct SoundDescriptor {
    /// 声音格式
    pub format: SoundFormat,
    /// 时长（秒）
    pub duration_secs: f64,
    /// 声道数
    pub channels: u16,
    /// 采样率
    pub sample_rate: u32,
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use super::{SoundDescriptor, SoundFormat, SoundId};

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
}
