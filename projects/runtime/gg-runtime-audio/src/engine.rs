use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};

use crate::{AudioCommand, SoundDescriptor, SoundId};

/// 音频引擎 trait
///
/// 定义音频后端的统一接口，各音频后端实现此 trait 以提供实际的音频播放能力。
pub trait AudioEngine {
    /// 从文件加载声音资源
    fn load_sound(&mut self, path: &Path) -> GResult<SoundId>;

    /// 播放指定声音
    fn play(&mut self, sound_id: SoundId, volume: f32, looped: bool) -> GResult<()>;

    /// 停止指定声音
    fn stop(&mut self, sound_id: SoundId) -> GResult<()>;

    /// 设置声音音量
    fn set_volume(&mut self, sound_id: SoundId, volume: f32) -> GResult<()>;

    /// 暂停指定声音
    fn pause(&mut self, sound_id: SoundId) -> GResult<()>;

    /// 恢复指定声音
    fn resume(&mut self, sound_id: SoundId) -> GResult<()>;

    /// 处理音频命令队列
    fn update(&mut self, context: &AudioContext) -> GResult<()>;

    /// 获取声音描述符
    fn sound_descriptor(&self, sound_id: SoundId) -> Option<&SoundDescriptor>;

    /// 重新加载音频（用于 HMR 热更新）
    ///
    /// 从指定路径重新加载音频数据并更新音频对象。
    /// 默认实现返回不支持错误。
    ///
    /// # 参数
    ///
    /// - `path` - 音频文件路径
    fn reload_sound(&mut self, path: &str) -> GResult<()> {
        Err(GError { kind: GErrorKind::Runtime, message: format!("Sound reload not supported: {}", path) })
    }
}

/// 音频上下文
///
/// 管理待处理的音频命令列表，供音频引擎在 update 时消费。
#[derive(Debug, Clone, Default)]
pub struct AudioContext {
    /// 待处理的音频命令列表
    commands: Vec<AudioCommand>,
}

impl AudioContext {
    /// 创建新的音频上下文
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    /// 添加音频命令
    pub fn submit(&mut self, command: AudioCommand) {
        self.commands.push(command);
    }

    /// 清空命令列表
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 获取命令列表的引用
    pub fn commands(&self) -> &[AudioCommand] {
        &self.commands
    }
}
