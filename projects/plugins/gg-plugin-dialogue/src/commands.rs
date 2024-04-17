//! 命令解释器模块
//! 提供命令分发器和内置命令处理器

use std::collections::HashMap;

use crate::schema::{AudioControl, DialogueCommand, GameVariables, PortraitState, SceneBackground, SeTrigger, WaitTimer};
use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::{Entity, World};
use gg_render::TextureId;

/// 命令处理器 trait
///
/// 所有对话命令处理器都需要实现此 trait，
/// 通过 execute 方法接收 DialogueCommand 引用并在 World 上执行命令效果。
pub trait CommandHandler: Send + Sync {
    /// 执行命令
    ///
    /// 根据给定的 DialogueCommand 引用提取数据，
    /// 在给定的 World 上执行命令效果，返回执行结果。
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()>;
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
        let mut dispatcher = Self { handlers: HashMap::new() };
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
    /// 根据 DialogueCommand 的类型，查找并调用对应的命令处理器，
    /// 将命令引用传递给处理器以提取数据。
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

        handler.execute(command, world)
    }

    /// 注册内置命令处理器
    fn register_builtin_handlers(&mut self) {
        self.register_handler("play_bgm".to_string(), Box::new(PlayBgmHandler));
        self.register_handler("stop_bgm".to_string(), Box::new(StopBgmHandler));
        self.register_handler("play_se".to_string(), Box::new(PlaySeHandler));
        self.register_handler("show_portrait".to_string(), Box::new(ShowPortraitHandler));
        self.register_handler("hide_portrait".to_string(), Box::new(HidePortraitHandler));
        self.register_handler("change_background".to_string(), Box::new(ChangeBackgroundHandler));
        self.register_handler("set_variable".to_string(), Box::new(SetVariableHandler));
        self.register_handler("wait".to_string(), Box::new(WaitHandler));
    }
}

/// 播放 BGM 命令处理器
///
/// 处理 PlayBgm 命令，从 DialogueCommand 中提取资源路径、音量和淡入时长，
/// 更新 World 中 AudioControl 组件的 BGM 相关字段。
pub struct PlayBgmHandler;

impl CommandHandler for PlayBgmHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::PlayBgm { asset_path, volume, fade_in_secs } = command {
            let audio = world
                .get_component_mut::<AudioControl>(Entity::new(0, 0))
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "AudioControl component not found".to_string() })?;
            audio.bgm_path = Some(asset_path.clone());
            audio.bgm_volume = *volume;
            audio.bgm_fade_in_secs = *fade_in_secs;
        }
        Ok(())
    }
}

/// 停止 BGM 命令处理器
///
/// 处理 StopBgm 命令，从 DialogueCommand 中提取淡出时长，
/// 清除 World 中 AudioControl 的 BGM 路径并设置淡出时长。
pub struct StopBgmHandler;

impl CommandHandler for StopBgmHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::StopBgm { fade_out_secs } = command {
            let audio = world
                .get_component_mut::<AudioControl>(Entity::new(0, 0))
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "AudioControl component not found".to_string() })?;
            audio.bgm_path = None;
            audio.bgm_fade_out_secs = *fade_out_secs;
        }
        Ok(())
    }
}

/// 播放音效命令处理器
///
/// 处理 PlaySe 命令，从 DialogueCommand 中提取资源路径和音量，
/// 将音效触发器添加到 AudioControl 的待播放列表。
pub struct PlaySeHandler;

impl CommandHandler for PlaySeHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::PlaySe { asset_path, volume } = command {
            let audio = world
                .get_component_mut::<AudioControl>(Entity::new(0, 0))
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "AudioControl component not found".to_string() })?;
            audio.pending_se.push(SeTrigger { asset_path: asset_path.clone(), volume: *volume, timestamp: None });
        }
        Ok(())
    }
}

/// 显示立绘命令处理器
///
/// 处理 ShowPortrait 命令，从 DialogueCommand 中提取角色 ID、表情和位置，
/// 在 World 中创建新实体并添加 PortraitState 组件。
pub struct ShowPortraitHandler;

impl CommandHandler for ShowPortraitHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::ShowPortrait { character_id, expression, position, .. } = command {
            let entity = world.spawn().id();
            let portrait = PortraitState {
                character_id: character_id.clone(),
                current_expression: expression.clone(),
                position: position.clone(),
                scale: 1.0,
                opacity: 1.0,
                is_speaking: false,
                z_order: 0,
                texture_id: TextureId::INVALID,
            };
            world.add_component(entity, portrait)?;
        }
        Ok(())
    }
}

/// 隐藏立绘命令处理器
///
/// 处理 HidePortrait 命令，从 DialogueCommand 中提取角色 ID，
/// 在 World 中查找匹配的 PortraitState 实体并移除其组件。
pub struct HidePortraitHandler;

impl CommandHandler for HidePortraitHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::HidePortrait { character_id, .. } = command {
            let entities = world.entities();
            for entity in entities {
                if let Some(portrait) = world.get_component::<PortraitState>(entity) {
                    if portrait.character_id == *character_id {
                        world.remove_component::<PortraitState>(entity);
                    }
                }
            }
        }
        Ok(())
    }
}

/// 切换背景命令处理器
///
/// 处理 ChangeBackground 命令，从 DialogueCommand 中提取资源路径和过渡动画类型，
/// 更新 World 中 SceneBackground 组件的对应字段。
pub struct ChangeBackgroundHandler;

impl CommandHandler for ChangeBackgroundHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::ChangeBackground { asset_path, transition } = command {
            let background = world
                .get_component_mut::<SceneBackground>(Entity::new(0, 0))
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "SceneBackground component not found".to_string() })?;
            background.asset_path = Some(asset_path.clone());
            background.transition = transition.clone();
        }
        Ok(())
    }
}

/// 设置变量命令处理器
///
/// 处理 SetVariable 命令，从 DialogueCommand 中提取变量名和变量值，
/// 调用 GameVariables 的 set_variable 方法更新游戏变量。
pub struct SetVariableHandler;

impl CommandHandler for SetVariableHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::SetVariable { name, value } = command {
            let variables = world
                .get_resource_mut::<GameVariables>()
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "GameVariables resource not found".to_string() })?;
            variables.set_variable(name.clone(), value.clone());
        }
        Ok(())
    }
}

/// 等待命令处理器
///
/// 处理 Wait 命令，从 DialogueCommand 中提取等待时长，
/// 将 WaitTimer 资源插入 World 中以启动等待计时。
pub struct WaitHandler;

impl CommandHandler for WaitHandler {
    fn execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()> {
        if let DialogueCommand::Wait { duration_secs } = command {
            world.insert_resource(WaitTimer { remaining_secs: *duration_secs });
        }
        Ok(())
    }
}
