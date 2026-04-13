//! 预览会话模块
//! 管理游戏预览的生命周期，封装 GalgameEngine 的编辑器模式实例

use gg_core::GResult;
use gg_galgame::{config::GalgameConfig, engine::GalgameEngine};
use gg_render::TextureId;
use std::path::PathBuf;

/// 预览状态
///
/// 描述游戏预览的当前运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewState {
    /// 空闲状态
    Idle,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
}

/// 游戏预览会话
///
/// 封装 GalgameEngine 的编辑器模式实例，
/// 管理游戏预览的启动、暂停、恢复和停止生命周期。
/// 支持从指定节点开始预览，以及渲染纹理管理。
pub struct PreviewSession {
    /// Galgame 引擎实例
    engine: GalgameEngine,
    /// 当前预览状态
    state: PreviewState,
    /// 预览起始节点 ID
    start_node_id: Option<String>,
    /// 渲染纹理标识
    render_texture_id: Option<TextureId>,
}

impl PreviewSession {
    /// 创建新的预览会话
    ///
    /// 创建 GalgameEngine 编辑器模式实例并加载项目脚本。
    /// 初始状态为 Idle，无起始节点和渲染纹理。
    pub fn new(project_path: Option<PathBuf>) -> GResult<Self> {
        let config = GalgameConfig::default();
        let mut engine = GalgameEngine::new(config, true, project_path);
        engine.initialize()?;
        Ok(Self { engine, state: PreviewState::Idle, start_node_id: None, render_texture_id: None })
    }

    /// 创建从指定节点开始的预览会话
    ///
    /// 设置起始节点 ID，预览启动时将从该节点开始执行游戏逻辑。
    pub fn with_start_node(node_id: String, project_path: Option<PathBuf>) -> GResult<Self> {
        let config = GalgameConfig::default();
        let mut engine = GalgameEngine::new(config, true, project_path);
        engine.initialize()?;
        engine.set_start_node(&node_id)?;
        Ok(Self { engine, state: PreviewState::Idle, start_node_id: Some(node_id), render_texture_id: None })
    }

    /// 启动预览
    ///
    /// 将状态设置为 Running，引擎开始推进游戏逻辑。
    /// 如果设置了起始节点，从该节点开始执行。
    pub fn start(&mut self) {
        if let Some(ref node_id) = self.start_node_id {
            let _ = self.engine.set_start_node(node_id);
        }
        self.state = PreviewState::Running;
    }

    /// 停止预览
    ///
    /// 将状态设置为 Idle，引擎停止推进。
    pub fn stop(&mut self) {
        self.state = PreviewState::Idle;
    }

    /// 暂停预览
    ///
    /// 将状态设置为 Paused，引擎暂停推进。
    pub fn pause(&mut self) {
        if self.state == PreviewState::Running {
            self.state = PreviewState::Paused;
        }
    }

    /// 恢复预览
    ///
    /// 将状态从 Paused 恢复为 Running。
    pub fn resume(&mut self) {
        if self.state == PreviewState::Paused {
            self.state = PreviewState::Running;
        }
    }

    /// 推进一帧
    ///
    /// 如果状态为 Running，调用 GalgameEngine::tick() 推进游戏逻辑。
    pub fn tick(&mut self) -> GResult<()> {
        if self.state == PreviewState::Running {
            self.engine.tick()?;
        }
        Ok(())
    }

    /// 推进一帧并渲染
    ///
    /// 如果状态为 Running，推进游戏逻辑并执行渲染。
    /// 返回 true 表示成功渲染了一帧，false 表示未渲染（非运行状态）。
    pub fn tick_and_render(&mut self) -> GResult<bool> {
        if self.state != PreviewState::Running {
            return Ok(false);
        }
        self.engine.tick()?;
        self.engine.render()?;
        Ok(true)
    }

    /// 获取当前预览状态
    pub fn state(&self) -> PreviewState {
        self.state
    }

    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        self.state == PreviewState::Running
    }

    /// 是否已暂停
    pub fn is_paused(&self) -> bool {
        self.state == PreviewState::Paused
    }

    /// 获取 GalgameEngine 的可变引用
    pub fn engine_mut(&mut self) -> &mut GalgameEngine {
        &mut self.engine
    }

    /// 获取 GalgameEngine 的引用
    pub fn engine(&self) -> &GalgameEngine {
        &self.engine
    }

    /// 获取渲染纹理标识
    pub fn render_texture_id(&self) -> Option<TextureId> {
        self.render_texture_id
    }

    /// 设置渲染纹理标识
    ///
    /// 由 ViewportRenderer 在创建渲染纹理后调用，
    /// 将纹理标识关联到当前预览会话。
    pub fn set_render_texture_id(&mut self, id: TextureId) {
        self.render_texture_id = Some(id);
    }

    /// 获取起始节点 ID
    pub fn start_node_id(&self) -> Option<&str> {
        self.start_node_id.as_deref()
    }

    /// 设置起始节点 ID
    pub fn set_start_node_id(&mut self, node_id: String) {
        self.start_node_id = Some(node_id);
    }
}
