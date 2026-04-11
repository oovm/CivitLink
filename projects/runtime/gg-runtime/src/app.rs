//! 应用程序结构，提供插件系统集成的流式 API
//!
//! App 封装阶段调度器和 ECS 世界，提供便捷的系统注册和插件管理接口。
//! RuntimePlugin trait 扩展了 Plugin，允许插件通过 configure 方法注册系统到指定阶段。

use crate::{
    scheduler::StageScheduler,
    stage::{Stage, SystemFn, SystemSetId},
};
use gg_core::{GError, GErrorKind, GResult, plugin::Plugin};
use gg_ecs::World;
use std::{sync::Arc, time::Duration};

/// 运行时插件 trait
///
/// 扩展 `Plugin` trait，增加 `configure` 方法允许插件
/// 通过 `App` 注册系统到指定阶段。
///
/// # 示例
///
/// ```ignore
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn name(&self) -> &str { "my_plugin" }
///     fn initialize(&self) -> GResult<()> { Ok(()) }
///     fn shutdown(&self) -> GResult<()> { Ok(()) }
/// }
///
/// impl RuntimePlugin for MyPlugin {
///     fn configure(&self, app: &mut App) -> GResult<()> {
///         app.add_update_system("my_system", Box::new(my_system_fn));
///         Ok(())
///     }
/// }
/// ```
pub trait RuntimePlugin: Plugin {
    /// 配置插件，注册系统到应用程序
    ///
    /// 此方法在插件初始化后调用，插件可通过 `app` 参数
    /// 注册系统到指定阶段。
    fn configure(&self, _app: &mut App) -> GResult<()> {
        Ok(())
    }
}

/// 应用程序结构
///
/// 封装阶段调度器和 ECS 世界，提供流式 API 用于
/// 系统注册、插件管理和游戏循环控制。
pub struct App {
    /// 阶段调度器
    scheduler: StageScheduler,
    /// ECS 世界
    world: World,
    /// 已注册的运行时插件
    plugins: Vec<Arc<dyn RuntimePlugin>>,
}

impl App {
    /// 创建新的应用程序
    pub fn new() -> Self {
        Self { scheduler: StageScheduler::new(), world: World::new(), plugins: Vec::new() }
    }

    /// 添加系统到指定阶段
    ///
    /// 返回 `&mut Self` 以支持链式调用。
    /// 如需指定排序约束，请通过 `scheduler_mut()` 使用构建器 API。
    pub fn add_systems(&mut self, name: impl Into<String>, system: SystemFn, stage: Stage) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, stage);
        self
    }

    /// 添加启动系统（Startup 阶段）
    ///
    /// 启动系统仅在首次 tick 时执行一次。
    pub fn add_startup_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::Startup);
        self
    }

    /// 添加预更新系统（PreUpdate 阶段）
    pub fn add_pre_update_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::PreUpdate);
        self
    }

    /// 添加固定更新系统（FixedUpdate 阶段）
    pub fn add_fixed_update_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::FixedUpdate);
        self
    }

    /// 添加更新系统（Update 阶段）
    pub fn add_update_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::Update);
        self
    }

    /// 添加后更新系统（PostUpdate 阶段）
    pub fn add_post_update_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::PostUpdate);
        self
    }

    /// 添加渲染系统（Render 阶段）
    pub fn add_render_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::Render);
        self
    }

    /// 添加退出系统（Exit 阶段）
    pub fn add_exit_system(&mut self, name: impl Into<String>, system: SystemFn) -> &mut Self {
        self.scheduler.add_system_to_stage(name, system, Stage::Exit);
        self
    }

    /// 配置系统集合的排序约束
    pub fn configure_set(&mut self, set: SystemSetId) -> &mut Self {
        self.scheduler.configure_set(set);
        self
    }

    /// 添加运行时插件
    ///
    /// 依次调用插件的 `initialize()` 和 `configure()` 方法。
    /// `initialize()` 用于插件初始化，`configure()` 用于注册系统到指定阶段。
    pub fn add_plugin(&mut self, plugin: Arc<dyn RuntimePlugin>) -> GResult<()> {
        plugin.initialize().map_err(|e| GError {
            kind: GErrorKind::Plugin,
            message: format!("Plugin '{}' initialize failed: {}", plugin.name(), e),
        })?;
        RuntimePlugin::configure(&*plugin, self).map_err(|e| GError {
            kind: GErrorKind::Plugin,
            message: format!("Plugin '{}' configure failed: {}", plugin.name(), e),
        })?;
        self.plugins.push(plugin);
        Ok(())
    }

    /// 执行一帧
    ///
    /// 按阶段顺序执行系统，首次调用包含 Startup 阶段。
    pub fn tick(&mut self, delta: Duration) -> GResult<()> {
        self.scheduler.tick(&mut self.world, delta)
    }

    /// 停止应用程序
    ///
    /// 执行 Exit 阶段系统，然后关闭所有插件。
    pub fn stop(&mut self) -> GResult<()> {
        self.scheduler.stop(&mut self.world)?;
        for plugin in &self.plugins {
            plugin.shutdown().map_err(|e| GError {
                kind: GErrorKind::Plugin,
                message: format!("Plugin '{}' shutdown failed: {}", plugin.name(), e),
            })?;
        }
        Ok(())
    }

    /// 获取阶段调度器的不可变引用
    pub fn scheduler(&self) -> &StageScheduler {
        &self.scheduler
    }

    /// 获取阶段调度器的可变引用
    pub fn scheduler_mut(&mut self) -> &mut StageScheduler {
        &mut self.scheduler
    }

    /// 获取世界的不可变引用
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 获取世界的可变引用
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
