//! 运行时构建器模块
//!
//! 提供 builder 模式构建 Runtime 实例，
//! 支持动态注入渲染后端、音频后端和 HMR 支持。

use gg_core::GResult;
use gg_render::Renderer;
use gg_runtime_audio::AudioEngine;
use crate::Runtime;

/// 运行时构建器
///
/// 提供 builder 模式构建 Runtime 实例，
/// 支持动态注入渲染后端、音频后端和 HMR 支持。
pub struct RuntimeBuilder {
    /// 渲染后端实例
    pub(crate) renderer: Option<Box<dyn Renderer>>,
    /// 音频后端实例
    pub(crate) audio_engine: Option<Box<dyn AudioEngine>>,
    /// 是否启用 HMR
    pub(crate) hmr_enabled: bool,
}

impl RuntimeBuilder {
    /// 创建新的运行时构建器
    pub fn new() -> Self {
        Self {
            renderer: None,
            audio_engine: None,
            hmr_enabled: false,
        }
    }

    /// 设置渲染后端
    pub fn renderer(mut self, renderer: Box<dyn Renderer>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// 设置音频后端
    pub fn audio_engine(mut self, audio_engine: Box<dyn AudioEngine>) -> Self {
        self.audio_engine = Some(audio_engine);
        self
    }

    /// 启用 HMR 热更新支持
    pub fn hmr_enabled(mut self) -> Self {
        self.hmr_enabled = true;
        self
    }

    /// 构建 Runtime 实例
    pub fn build(self) -> GResult<Runtime> {
        Runtime::from_builder(self)
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
