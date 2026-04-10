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

#[cfg(test)]
mod tests {
    use crate::SoundId;

    use super::AudioCommand;

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
}
