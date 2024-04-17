use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};

use crate::{AudioCommand, PlaybackId, SoundDescriptor, SoundId};

/// 音频引擎 trait
///
/// 定义音频后端的统一接口，各音频后端实现此 trait 以提供实际的音频播放能力。
/// 支持声音加载、播放控制、分组管理、效果器管理和播放实例查询。
pub trait AudioEngine {
    /// 从文件加载声音资源
    fn load_sound(&mut self, path: &Path) -> GResult<SoundId>;

    /// 播放指定声音
    ///
    /// 返回播放实例标识 PlaybackId，支持同一声音并发播放多个实例。
    /// group 参数指定所属音频分组，为 None 时使用默认分组。
    fn play(&mut self, sound_id: SoundId, volume: f32, looped: bool, group: Option<&str>) -> GResult<PlaybackId>;

    /// 停止指定声音的所有活跃实例
    fn stop(&mut self, sound_id: SoundId) -> GResult<()>;

    /// 设置指定声音所有活跃实例的音量
    fn set_volume(&mut self, sound_id: SoundId, volume: f32) -> GResult<()>;

    /// 暂停指定声音的所有活跃实例
    fn pause(&mut self, sound_id: SoundId) -> GResult<()>;

    /// 恢复指定声音的所有活跃实例
    fn resume(&mut self, sound_id: SoundId) -> GResult<()>;

    /// 处理音频命令队列
    fn update(&mut self, context: &AudioContext) -> GResult<()>;

    /// 获取声音描述符
    fn sound_descriptor(&self, sound_id: SoundId) -> Option<&SoundDescriptor>;

    /// 重新加载音频（用于 HMR 热更新）
    ///
    /// 从指定路径重新加载音频数据并更新音频对象。
    /// 默认实现返回不支持错误。
    fn reload_sound(&mut self, path: &str) -> GResult<()> {
        Err(GError { kind: GErrorKind::Runtime, message: format!("Sound reload not supported: {}", path) })
    }

    /// 停止指定播放实例
    fn stop_playback(&mut self, playback_id: PlaybackId) -> GResult<()>;

    /// 设置播放实例音量
    fn set_playback_volume(&mut self, playback_id: PlaybackId, volume: f32) -> GResult<()>;

    /// 暂停指定播放实例
    fn pause_playback(&mut self, playback_id: PlaybackId) -> GResult<()>;

    /// 恢复指定播放实例
    fn resume_playback(&mut self, playback_id: PlaybackId) -> GResult<()>;

    /// 创建音频分组
    fn create_group(&mut self, name: &str) -> GResult<()>;

    /// 设置分组音量
    fn set_group_volume(&mut self, name: &str, volume: f32) -> GResult<()>;

    /// 设置分组静音状态
    fn set_group_mute(&mut self, name: &str, mute: bool) -> GResult<()>;

    /// 获取分组音量
    fn group_volume(&self, name: &str) -> Option<f32>;

    /// 检查分组是否静音
    fn is_group_muted(&self, name: &str) -> Option<bool>;

    /// 获取活跃播放实例数量
    fn active_playback_count(&self) -> usize;

    /// 检查指定播放实例是否正在播放
    fn is_playing(&self, playback_id: PlaybackId) -> bool;
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

    /// 提交播放声音命令
    pub fn submit_play(&mut self, sound_id: SoundId, volume: f32, looped: bool, group: Option<String>) {
        self.commands.push(AudioCommand::Play { sound_id, volume, looped, group });
    }

    /// 提交停止声音命令
    pub fn submit_stop(&mut self, sound_id: SoundId) {
        self.commands.push(AudioCommand::Stop { sound_id });
    }

    /// 提交设置音量命令
    pub fn submit_set_volume(&mut self, sound_id: SoundId, volume: f32) {
        self.commands.push(AudioCommand::SetVolume { sound_id, volume });
    }

    /// 提交暂停声音命令
    pub fn submit_pause(&mut self, sound_id: SoundId) {
        self.commands.push(AudioCommand::Pause { sound_id });
    }

    /// 提交恢复声音命令
    pub fn submit_resume(&mut self, sound_id: SoundId) {
        self.commands.push(AudioCommand::Resume { sound_id });
    }

    /// 提交设置分组音量命令
    pub fn submit_set_group_volume(&mut self, group: &str, volume: f32) {
        self.commands.push(AudioCommand::SetGroupVolume { group: group.to_string(), volume });
    }

    /// 提交设置分组静音命令
    pub fn submit_set_group_mute(&mut self, group: &str, mute: bool) {
        self.commands.push(AudioCommand::SetGroupMute { group: group.to_string(), mute });
    }

    /// 提交停止播放实例命令
    pub fn submit_stop_playback(&mut self, playback_id: PlaybackId) {
        self.commands.push(AudioCommand::StopPlayback { playback_id });
    }

    /// 提交设置播放实例音量命令
    pub fn submit_set_playback_volume(&mut self, playback_id: PlaybackId, volume: f32) {
        self.commands.push(AudioCommand::SetPlaybackVolume { playback_id, volume });
    }

    /// 提交暂停播放实例命令
    pub fn submit_pause_playback(&mut self, playback_id: PlaybackId) {
        self.commands.push(AudioCommand::PausePlayback { playback_id });
    }

    /// 提交恢复播放实例命令
    pub fn submit_resume_playback(&mut self, playback_id: PlaybackId) {
        self.commands.push(AudioCommand::ResumePlayback { playback_id });
    }

    /// 提交附加效果器命令
    pub fn submit_attach_effect(&mut self, playback_id: PlaybackId, effect: crate::effect::AudioEffectKind) {
        self.commands.push(AudioCommand::AttachEffect { playback_id, effect });
    }

    /// 提交移除效果器命令
    pub fn submit_detach_effect(&mut self, playback_id: PlaybackId, effect_type: crate::effect::EffectType) {
        self.commands.push(AudioCommand::DetachEffect { playback_id, effect_type });
    }

    /// 提交设置效果器参数命令
    pub fn submit_set_effect_param(
        &mut self,
        playback_id: PlaybackId,
        effect_type: crate::effect::EffectType,
        param: &str,
        value: f32,
    ) {
        self.commands.push(AudioCommand::SetEffectParam { playback_id, effect_type, param: param.to_string(), value });
    }
}
