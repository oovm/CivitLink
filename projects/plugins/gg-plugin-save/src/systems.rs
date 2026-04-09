//! 存档系统模块
//! 实现存档系统，负责处理保存和加载请求

use gg_core::GResult;
use gg_ecs::{Component, System, World};
use serde::{Deserialize, Serialize};

use crate::manager::SaveManager;

/// 存档请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveRequestType {
    /// 保存
    Save,
    /// 加载
    Load,
}

/// 存档请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    /// 请求类型
    pub request_type: SaveRequestType,
    /// 存档槽位
    pub slot: u32,
}

impl Component for SaveRequest {}

/// 存档系统
///
/// 负责检查 World 中的 SaveRequest 资源，
/// 根据请求类型执行保存或加载操作。
pub struct SaveSystem;

impl SaveSystem {
    /// 创建新的存档系统
    pub fn new() -> Self {
        Self
    }
}

impl System for SaveSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "save"
    }

    /// 执行存档系统逻辑
    ///
    /// 检查 World 中的 SaveRequest 资源，
    /// 如果存在保存请求，执行保存操作；
    /// 如果存在加载请求，执行加载操作。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let request = match world.get_component_mut::<SaveRequest>(0) {
            Some(req) => req.clone(),
            None => return Ok(()),
        };

        match request.request_type {
            SaveRequestType::Save => {
                let save_data = SaveManager::save(world, request.slot, None)?;

                let path = std::path::PathBuf::from(format!("saves/save_{}.json", request.slot));
                SaveManager::save_to_file(&save_data, &path)?;
            }
            SaveRequestType::Load => {
                let path = std::path::PathBuf::from(format!("saves/save_{}.json", request.slot));
                let save_data = SaveManager::load_from_file(&path)?;
                SaveManager::load(&save_data, world)?;
            }
        }

        world.remove_component::<SaveRequest>(0);

        Ok(())
    }
}
