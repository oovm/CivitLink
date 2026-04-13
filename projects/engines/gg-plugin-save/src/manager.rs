//! 存档管理器模块
//! 提供存档的保存、加载和文件操作功能

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::{Entity, World};
use pleroma::{GameVariables, VariableValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

/// 存档数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// 当前对话节点
    pub current_node_id: String,
    /// 游戏变量快照
    pub variables: HashMap<String, VariableValue>,
    /// 立绘状态快照
    pub portrait_states: Vec<PortraitState>,
    /// 当前背景路径
    pub background_path: Option<String>,
    /// 当前 BGM 路径
    pub bgm_path: Option<String>,
    /// 截图数据
    pub screenshot: Option<Vec<u8>>,
    /// 保存时间戳
    pub timestamp: f64,
}

/// 立绘状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortraitState {
    /// 关联角色 ID
    pub character_id: String,
    /// 当前表情标签
    pub current_expression: String,
    /// 当前位置
    pub position: PortraitPosition,
    /// 缩放比例（默认 1.0）
    pub scale: f32,
    /// 透明度（默认 1.0）
    pub opacity: f32,
    /// 是否正在说话（用于高亮）
    pub is_speaking: bool,
    /// Z 轴排序
    pub z_order: i32,
}

/// 立绘位置枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortraitPosition {
    /// 左侧
    Left,
    /// 中央
    Center,
    /// 右侧
    Right,
    /// 自定义坐标
    Custom {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
}

/// 存档槽位信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    /// 存档槽位
    pub slot: u32,
    /// 保存时间戳
    pub timestamp: f64,
    /// 当前节点 ID
    pub current_node_id: String,
    /// 是否有截图
    pub has_screenshot: bool,
}

/// 存档管理器
///
/// 负责游戏存档的保存和加载操作，
/// 包括从 World 中提取游戏状态、恢复游戏状态、
/// 以及存档文件的读写。
pub struct SaveManager;

impl SaveManager {
    /// 保存存档
    ///
    /// 从 World 中提取当前游戏状态，组装为 SaveData。
    ///
    /// # 参数
    ///
    /// - `world` - 游戏世界
    /// - `slot` - 存档槽位
    /// - `screenshot` - 可选的截图数据
    pub fn save(world: &World, _slot: u32, screenshot: Option<Vec<u8>>) -> GResult<SaveData> {
        // 尝试获取游戏变量
        let variables = if let Some(game_vars) = world.get_resource::<GameVariables>() {
            game_vars.variables.clone()
        }
        else {
            HashMap::new()
        };

        // 尝试获取立绘状态
        let portrait_states = {
            let entities: Vec<Entity> = world.entities().iter().copied().collect();
            let mut states = Vec::new();
            for entity in entities {
                if let Some(state) = world.get_component::<PortraitState>(entity) {
                    states.push(state.clone());
                }
            }
            states
        };

        // 背景路径和 BGM 路径暂时设为 None，因为它们依赖于特定的组件
        let background_path = None;
        let bgm_path = None;

        // 使用空字符串作为默认的当前节点 ID
        let current_node_id = String::new();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to get timestamp: {}", e) })?
            .as_secs_f64();

        Ok(SaveData { current_node_id, variables, portrait_states, background_path, bgm_path, screenshot, timestamp })
    }

    /// 加载存档
    ///
    /// 将 SaveData 中的游戏状态恢复到 World 中。
    ///
    /// # 参数
    ///
    /// - `save_data` - 存档数据
    /// - `world` - 游戏世界
    pub fn load(save_data: &SaveData, world: &mut World) -> GResult<()> {
        // 尝试恢复游戏变量
        if let Some(game_vars) = world.get_resource_mut::<GameVariables>() {
            game_vars.variables = save_data.variables.clone();
        }

        // 尝试恢复立绘状态
        {
            let all_entities: Vec<Entity> = world.entities().iter().copied().collect();
            let portrait_entities: Vec<Entity> =
                all_entities.into_iter().filter(|&entity| world.get_component::<PortraitState>(entity).is_some()).collect();
            for entity in portrait_entities {
                world.despawn(entity)?;
            }
        }

        for portrait_state in &save_data.portrait_states {
            let entity = world.spawn().id();
            world.add_component(entity, portrait_state.clone())?;
        }

        // 背景路径和 BGM 路径暂时不处理，因为它们依赖于特定的组件

        Ok(())
    }

    /// 保存存档到文件
    ///
    /// 将 SaveData 序列化为 JSON 并写入指定路径。
    ///
    /// # 参数
    ///
    /// - `data` - 存档数据
    /// - `path` - 文件路径
    pub fn save_to_file(data: &SaveData, path: &Path) -> GResult<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to serialize save data: {}", e) })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create save directory: {}", e) })?;
        }

        fs::write(path, json).map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write save file: {}", e) })
    }

    /// 从文件加载存档
    ///
    /// 从指定路径读取 JSON 文件并反序列化为 SaveData。
    ///
    /// # 参数
    ///
    /// - `path` - 文件路径
    pub fn load_from_file(path: &Path) -> GResult<SaveData> {
        let json = fs::read_to_string(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read save file: {}", e) })?;

        serde_json::from_str(&json)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to deserialize save data: {}", e) })
    }

    /// 列出存档目录中的所有存档
    ///
    /// 扫描指定目录中的存档文件，返回每个存档的摘要信息。
    /// 存档文件命名格式为 `save_N.json`，其中 N 为槽位号。
    ///
    /// # 参数
    ///
    /// - `dir` - 存档目录路径
    pub fn list_saves(dir: &Path) -> GResult<Vec<SaveSlotInfo>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut saves = Vec::new();

        let entries = fs::read_dir(dir)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read save directory: {}", e) })?;

        for entry in entries {
            let entry = entry
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let slot = if file_name.starts_with("save_") && file_name.ends_with(".json") {
                let trimmed = &file_name[5..file_name.len() - 5];
                match trimmed.parse::<u32>() {
                    Ok(s) => s,
                    Err(_) => continue,
                }
            }
            else {
                continue;
            };

            match Self::load_from_file(&path) {
                Ok(data) => {
                    saves.push(SaveSlotInfo {
                        slot,
                        timestamp: data.timestamp,
                        current_node_id: data.current_node_id,
                        has_screenshot: data.screenshot.is_some(),
                    });
                }
                Err(_) => continue,
            }
        }

        saves.sort_by_key(|info| info.slot);

        Ok(saves)
    }
}
