//! Galgame 引擎核心模块
//! 提供引擎初始化、主循环和插件管理功能

use crate::config::GalgameConfig;
use gg_asset::AssetManager;
use gg_core::plugin::Plugin;
use gg_core::GResult;
use gg_ecs::Scheduler;
use gg_plugin_dialogue::plugin::DialoguePlugin;
use gg_plugin_portrait::plugin::PortraitPlugin;
use gg_plugin_save::plugin::SavePlugin;
use gg_plugin_scene_transition::plugin::SceneTransitionPlugin;

/// Galgame 引擎
///
/// 负责管理游戏的生命周期，包括：
/// - 初始化所有插件
/// - 加载游戏资源
/// - 执行主循环
pub struct GalgameEngine {
    /// 游戏配置
    pub config: GalgameConfig,
    /// ECS 调度器
    pub scheduler: Scheduler,
    /// 资源管理器
    pub asset_manager: AssetManager,
    /// 是否编辑器模式
    pub is_editor_mode: bool,
}

impl GalgameEngine {
    /// 创建新的 Galgame 引擎实例
    ///
    /// # 参数
    ///
    /// - `config` - 游戏配置
    /// - `is_editor_mode` - 是否启用编辑器模式
    pub fn new(config: GalgameConfig, is_editor_mode: bool) -> Self {
        Self {
            config,
            scheduler: Scheduler::new(),
            asset_manager: AssetManager::new(),
            is_editor_mode,
        }
    }

    /// 初始化引擎
    ///
    /// 注册所有插件并加载游戏资源。
    pub fn initialize(&mut self) -> GResult<()> {
        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(DialoguePlugin),
            Box::new(PortraitPlugin),
            Box::new(SceneTransitionPlugin),
            Box::new(SavePlugin),
        ];

        for plugin in plugins {
            plugin.initialize()?;
        }

        Ok(())
    }

    /// 执行一帧
    ///
    /// 调度器执行所有已注册的系统。
    pub fn tick(&mut self) -> GResult<()> {
        self.scheduler.tick()
    }

    /// 运行主循环
    ///
    /// 持续执行 tick 直到引擎停止。
    /// 当前为简单实现，仅执行一次 tick。
    pub fn run(&mut self) -> GResult<()> {
        self.tick()
    }
}
