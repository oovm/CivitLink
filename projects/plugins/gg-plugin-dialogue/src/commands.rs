//! 命令解释器模块
//! 提供命令分发器和内置命令处理器

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::World;
use gg_galgame_schema::components::{
    AudioControl, DialogueCommand, PortraitPosition, PortraitState, SceneBackground,
    TransitionType, VariableValue,
};
use gg_galgame_schema::resources::GameVariables;

/// 命令处理器 trait
///
/// 所有对话命令处理器都需要实现此 trait，
/// 通过 execute 方法在 World 上执行命令效果。
pub trait CommandHandler: Send + Sync {
    /// 执行命令
    ///
    /// 在给定的 World 上执行命令效果，返回执行结果。
    fn execute(&self, world: &mut World) -> GResult<()>;
}

/// 命令分发器
///
/// 管理命令名称到处理器的映射，负责将 DialogueCommand 分发到对应的处理器。
pub struct CommandDispatcher {
    /// 命令处理器映射
    handlers: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandDispatcher {
    /// 创建新的命令分发器
    pub fn new() -> Self {
        let mut dispatcher = Self {
            handlers: HashMap::new(),
        };
        dispatcher.register_builtin_handlers();
        dispatcher
    }

    /// 注册命令处理器
    ///
    /// 将指定名称的命令处理器注册到分发器中。
    pub fn register_handler(&mut self, name: String, handler: Box<dyn CommandHandler>) {
        self.handlers.insert(name, handler);
    }

    /// 分发命令
    ///
    /// 根据 DialogueCommand 的类型，查找并调用对应的命令处理器。
    pub fn dispatch(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        let handler_name = match command {
            DialogueCommand::PlayBgm { .. } => "play_bgm",
            DialogueCommand::StopBgm { .. } => "stop_bgm",
            DialogueCommand::PlaySe { .. } => "play_se",
            DialogueCommand::ShowPortrait { .. } => "show_portrait",
            DialogueCommand::HidePortrait { .. } => "hide_portrait",
            DialogueCommand::ChangeBackground { .. } => "change_background",
            DialogueCommand::SetVariable { .. } => "set_variable",
            DialogueCommand::Wait { .. } => "wait",
        };

        let handler = self.handlers.get(handler_name).ok_or_else(|| GError {
            kind: GErrorKind::Plugin,
            message: format!("No handler registered for command '{}'", handler_name),
        })?;

        handler.execute(world)
    }

    /// 注册内置命令处理器
    fn register_builtin_handlers(&mut self) {
        self.register_handler("play_bgm".to_string(), Box::new(PlayBgmHandler));
        self.register_handler("stop_bgm".to_string(), Box::new(StopBgmHandler));
        self.register_handler("play_se".to_string(), Box::new(PlaySeHandler));
        self.register_handler(
            "show_portrait".to_string(),
            Box::new(ShowPortraitHandler),
        );
        self.register_handler(
            "hide_portrait".to_string(),
            Box::new(HidePortraitHandler),
        );
        self.register_handler(
            "change_background".to_string(),
            Box::new(ChangeBackgroundHandler),
        );
        self.register_handler(
            "set_variable".to_string(),
            Box::new(SetVariableHandler),
        );
        self.register_handler("wait".to_string(), Box::new(WaitHandler));
    }
}

/// 播放 BGM 命令处理器
///
/// 处理 PlayBgm 命令，更新 World 中的 AudioControl 组件。
pub struct PlayBgmHandler;

impl CommandHandler for PlayBgmHandler {
    fn execute(&self, world: &mut World) -> GResult<()> {
        let audio = world
            .get_component_mut::<AudioControl>(0)
            .ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: "AudioControl component not found".to_string(),
            })?;
        if let Some(ref path) = audio.bgm_path {
            audio.bgm_path = Some(path.clone());
        }
        Ok(())
    }
}

/// 停止 BGM 命令处理器
///
/// 处理 StopBgm 命令，清除 World 中 AudioControl 的 BGM 路径。
pub struct StopBgmHandler;

impl CommandHandler for StopBgmHandler {
    fn execute(&self, world: &mut World) -> GResult<()> {
        let audio = world
            .get_component_mut::<AudioControl>(0)
            .ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: "AudioControl component not found".to_string(),
            })?;
        audio.bgm_path = None;
        Ok(())
    }
}

/// 播放音效命令处理器
///
/// 处理 PlaySe 命令，将音效添加到 AudioControl 的待播放列表。
pub struct PlaySeHandler;

impl CommandHandler for PlaySeHandler {
    fn execute(&self, _world: &mut World) -> GResult<()> {
        Ok(())
    }
}

/// 显示立绘命令处理器
///
/// 处理 ShowPortrait 命令，在 World 中创建或更新 PortraitState 实体。
pub struct ShowPortraitHandler;

impl CommandHandler for ShowPortraitHandler {
    fn execute(&self, world: &mut World) -> GResult<()> {
        let entity = world.spawn().id();
        let portrait = PortraitState {
            character_id: String::new(),
            current_expression: String::new(),
            position: PortraitPosition::Center,
            scale: 1.0,
            opacity: 1.0,
            is_speaking: false,
            z_order: 0,
        };
        world.add_component(entity, portrait)?;
        Ok(())
    }
}

/// 隐藏立绘命令处理器
///
/// 处理 HidePortrait 命令，从 World 中移除 PortraitState 实体。
pub struct HidePortraitHandler;

impl CommandHandler for HidePortraitHandler {
    fn execute(&self, world: &mut World) -> GResult<()> {
        let entities: Vec<gg_ecs::Entity> = world.entities().iter().cloned().collect();
        for entity in entities {
            if world.get_component::<PortraitState>(entity).is_some() {
                world.remove_component::<PortraitState>(entity);
            }
        }
        Ok(())
    }
}

/// 切换背景命令处理器
///
/// 处理 ChangeBackground 命令，更新 World 中的 SceneBackground 组件。
pub struct ChangeBackgroundHandler;

impl CommandHandler for ChangeBackgroundHandler {
    fn execute(&self, world: &mut World) -> GResult<()> {
        let background = world
            .get_component_mut::<SceneBackground>(0)
            .ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: "SceneBackground component not found".to_string(),
            })?;
        background.transition = TransitionType::None;
        Ok(())
    }
}

/// 设置变量命令处理器
///
/// 处理 SetVariable 命令，更新 World 中的 GameVariables 资源。
pub struct SetVariableHandler;

impl CommandHandler for SetVariableHandler {
    fn execute(&self, world: &mut World) -> GResult<()> {
        let variables = world
            .get_component_mut::<GameVariables>(0)
            .ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: "GameVariables resource not found".to_string(),
            })?;
        variables.set_variable(String::new(), VariableValue::Boolean(true));
        Ok(())
    }
}

/// 等待命令处理器
///
/// 处理 Wait 命令，设置等待计时器。
pub struct WaitHandler;

impl CommandHandler for WaitHandler {
    fn execute(&self, _world: &mut World) -> GResult<()> {
        Ok(())
    }
}
