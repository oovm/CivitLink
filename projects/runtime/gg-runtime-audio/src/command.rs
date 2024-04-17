use crate::{
    SoundId,
    effect::{AudioEffectKind, EffectType},
    playback::PlaybackId,
};

/// 音频命令
///
/// 定义音频系统支持的所有操作命令，通过 AudioContext 提交，
/// 由音频引擎在 update 时统一消费处理。
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// 播放声音
    Play {
        /// 声音标识
        sound_id: SoundId,
        /// 音量 [0.0, 1.0]
        volume: f32,
        /// 是否循环播放
        looped: bool,
        /// 所属音频分组
        group: Option<String>,
    },
    /// 停止声音（所有活跃实例）
    Stop {
        /// 声音标识
        sound_id: SoundId,
    },
    /// 设置声音音量（所有活跃实例）
    SetVolume {
        /// 声音标识
        sound_id: SoundId,
        /// 音量 [0.0, 1.0]
        volume: f32,
    },
    /// 暂停声音（所有活跃实例）
    Pause {
        /// 声音标识
        sound_id: SoundId,
    },
    /// 恢复声音（所有活跃实例）
    Resume {
        /// 声音标识
        sound_id: SoundId,
    },
    /// 设置分组音量
    SetGroupVolume {
        /// 分组名称
        group: String,
        /// 音量 [0.0, 1.0]
        volume: f32,
    },
    /// 设置分组静音
    SetGroupMute {
        /// 分组名称
        group: String,
        /// 是否静音
        mute: bool,
    },
    /// 附加效果器到播放实例
    AttachEffect {
        /// 播放实例标识
        playback_id: PlaybackId,
        /// 效果器
        effect: AudioEffectKind,
    },
    /// 从播放实例移除效果器
    DetachEffect {
        /// 播放实例标识
        playback_id: PlaybackId,
        /// 效果器类型
        effect_type: EffectType,
    },
    /// 设置播放实例的效果器参数
    SetEffectParam {
        /// 播放实例标识
        playback_id: PlaybackId,
        /// 效果器类型
        effect_type: EffectType,
        /// 参数名称
        param: String,
        /// 参数值
        value: f32,
    },
    /// 停止指定播放实例
    StopPlayback {
        /// 播放实例标识
        playback_id: PlaybackId,
    },
    /// 设置播放实例音量
    SetPlaybackVolume {
        /// 播放实例标识
        playback_id: PlaybackId,
        /// 音量 [0.0, 1.0]
        volume: f32,
    },
    /// 暂停指定播放实例
    PausePlayback {
        /// 播放实例标识
        playback_id: PlaybackId,
    },
    /// 恢复指定播放实例
    ResumePlayback {
        /// 播放实例标识
        playback_id: PlaybackId,
    },
}
