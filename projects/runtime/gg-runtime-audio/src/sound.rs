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


