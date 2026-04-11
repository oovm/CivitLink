//! Spine 皮肤切换模块
//! 提供运行时皮肤切换功能

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::SlotAttachment;

/// Spine 皮肤定义
///
/// 每个皮肤包含一组插槽到附件的映射，切换皮肤时替换对应插槽的附件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    /// 皮肤名称
    pub name: String,
    /// 插槽附件映射（插槽名 → 附件列表）
    pub attachments: HashMap<String, Vec<SlotAttachment>>,
}

/// 皮肤管理器
///
/// 管理所有可用皮肤和当前激活的皮肤。
#[derive(Debug, Clone, Default)]
pub struct SkinManager {
    /// 可用皮肤列表
    pub skins: Vec<Skin>,
    /// 当前激活的皮肤名称
    pub active_skin: Option<String>,
}

impl SkinManager {
    /// 创建新的皮肤管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 根据名称查找皮肤
    pub fn find_skin(&self, name: &str) -> Option<&Skin> {
        self.skins.iter().find(|s| s.name == name)
    }

    /// 切换到指定皮肤
    ///
    /// 返回 Some(()) 如果皮肤存在且切换成功，否则返回 None。
    pub fn set_active(&mut self, name: &str) -> Option<()> {
        if self.find_skin(name).is_some() {
            self.active_skin = Some(name.to_string());
            Some(())
        }
        else {
            None
        }
    }

    /// 获取当前激活皮肤的附件映射
    pub fn active_attachments(&self) -> Option<&HashMap<String, Vec<SlotAttachment>>> {
        self.active_skin.as_ref().and_then(|name| self.find_skin(name).map(|s| &s.attachments))
    }
}
