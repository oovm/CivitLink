use crate::SoundId;

/// 播放实例标识符
///
/// 每次播放音频时分配的唯一标识，支持同一声音并发播放多个实例。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaybackId(pub u64);

/// 播放实例状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// 正在播放
    Playing,
    /// 已暂停
    Paused,
    /// 已停止
    Stopped,
}

/// 播放实例信息
///
/// 描述一个活跃播放实例的完整状态信息。
#[derive(Debug, Clone)]
pub struct PlaybackInfo {
    /// 播放实例标识
    pub playback_id: PlaybackId,
    /// 声音资源标识
    pub sound_id: SoundId,
    /// 播放音量 [0.0, 1.0]
    pub volume: f32,
    /// 是否循环播放
    pub looped: bool,
    /// 所属音频分组
    pub group: Option<String>,
    /// 播放状态
    pub state: PlaybackState,
}
