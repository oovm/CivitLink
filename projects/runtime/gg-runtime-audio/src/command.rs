use crate::SoundId;

/// 音频命令
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
    },
    /// 停止声音
    Stop {
        /// 声音标识
        sound_id: SoundId,
    },
    /// 设置音量
    SetVolume {
        /// 声音标识
        sound_id: SoundId,
        /// 音量 [0.0, 1.0]
        volume: f32,
    },
    /// 暂停声音
    Pause {
        /// 声音标识
        sound_id: SoundId,
    },
    /// 恢复声音
    Resume {
        /// 声音标识
        sound_id: SoundId,
    },
}


