use std::collections::HashMap;

use gg_core::GResult;

/// 音频分组
///
/// 将声音按逻辑分组管理，支持分组级别的音量控制和静音控制。
/// 分组音量与声音自身音量为乘法关系：实际音量 = 声音音量 × 分组音量。
#[derive(Debug, Clone)]
pub struct AudioGroup {
    /// 分组名称
    pub name: String,
    /// 分组音量 [0.0, 1.0]
    pub volume: f32,
    /// 是否静音
    pub mute: bool,
    /// 优先级，数值越大优先级越高
    pub priority: i32,
}

impl AudioGroup {
    /// 创建新的音频分组
    ///
    /// 默认音量 1.0，未静音，优先级 0。
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), volume: 1.0, mute: false, priority: 0 }
    }

    /// 设置分组音量
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// 设置分组静音状态
    pub fn set_mute(&mut self, mute: bool) {
        self.mute = mute;
    }

    /// 获取有效音量
    ///
    /// 静音时返回 0.0，非静音时返回分组音量。
    pub fn effective_volume(&self) -> f32 {
        if self.mute { 0.0 } else { self.volume }
    }
}

/// 音频分组管理器
///
/// 管理所有音频分组的创建、查询、修改和删除操作。
#[derive(Debug, Clone, Default)]
pub struct AudioGroupManager {
    /// 分组映射表
    groups: HashMap<String, AudioGroup>,
}

impl AudioGroupManager {
    /// 创建新的音频分组管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建音频分组
    ///
    /// 如果分组已存在，返回错误。
    pub fn create_group(&mut self, name: &str) -> GResult<()> {
        if self.groups.contains_key(name) {
            return Err(gg_core::GError::with_kind(
                gg_core::GErrorKind::Runtime,
                &format!("Audio group already exists: {}", name),
            ));
        }
        self.groups.insert(name.to_string(), AudioGroup::new(name));
        Ok(())
    }

    /// 获取音频分组
    pub fn get_group(&self, name: &str) -> Option<&AudioGroup> {
        self.groups.get(name)
    }

    /// 获取音频分组的可变引用
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut AudioGroup> {
        self.groups.get_mut(name)
    }

    /// 设置分组音量
    ///
    /// 如果分组不存在，返回 None。
    pub fn set_group_volume(&mut self, name: &str, volume: f32) -> Option<()> {
        self.groups.get_mut(name).map(|g| g.set_volume(volume))
    }

    /// 设置分组静音状态
    ///
    /// 如果分组不存在，返回 None。
    pub fn set_group_mute(&mut self, name: &str, mute: bool) -> Option<()> {
        self.groups.get_mut(name).map(|g| g.set_mute(mute))
    }

    /// 移除音频分组
    ///
    /// 返回被移除的分组，如果分组不存在则返回 None。
    pub fn remove_group(&mut self, name: &str) -> Option<AudioGroup> {
        self.groups.remove(name)
    }

    /// 获取分组有效音量
    ///
    /// 如果分组不存在，返回 None。
    pub fn effective_volume(&self, name: &str) -> Option<f32> {
        self.groups.get(name).map(|g| g.effective_volume())
    }

    /// 检查分组是否静音
    ///
    /// 如果分组不存在，返回 None。
    pub fn is_muted(&self, name: &str) -> Option<bool> {
        self.groups.get(name).map(|g| g.mute)
    }

    /// 获取分组音量
    ///
    /// 如果分组不存在，返回 None。
    pub fn group_volume(&self, name: &str) -> Option<f32> {
        self.groups.get(name).map(|g| g.volume)
    }
}
