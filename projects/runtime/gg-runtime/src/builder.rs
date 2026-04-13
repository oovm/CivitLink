//! 运行时构建器模块
//!
//! 提供 builder 模式构建 Runtime 实例，
//! 支持动态注入渲染后端、音频后端、平台服务和 HMR 支持。

use crate::Runtime;
use gg_core::{GResult, platform::PlatformServices};
use gg_render::Renderer;
use gg_runtime_audio::AudioEngine;
use std::time::Duration;

/// 运行时构建器
///
/// 提供 builder 模式构建 Runtime 实例，
/// 支持动态注入渲染后端、音频后端、平台服务和 HMR 支持。
pub struct RuntimeBuilder {
    /// 渲染后端实例
    pub(crate) renderer: Option<Box<dyn Renderer>>,
    /// 音频后端实例
    pub(crate) audio_engine: Option<Box<dyn AudioEngine>>,
    /// 平台服务
    pub(crate) platform_services: Option<PlatformServices>,
    /// 是否启用 HMR
    pub(crate) hmr_enabled: bool,
    /// 固定更新时间步长
    pub(crate) fixed_timestep: Option<Duration>,
    /// 目标帧率
    pub(crate) target_fps: Option<u32>,
    /// 是否启用帧性能分析器
    pub(crate) enable_frame_profiler: bool,
}

impl RuntimeBuilder {
    /// 创建新的运行时构建器
    pub fn new() -> Self {
        Self {
            renderer: None,
            audio_engine: None,
            platform_services: None,
            hmr_enabled: false,
            fixed_timestep: None,
            target_fps: None,
            enable_frame_profiler: false,
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

    /// 设置平台服务
    pub fn with_platform(mut self, services: PlatformServices) -> Self {
        self.platform_services = Some(services);
        self
    }

    /// 使用桌面平台服务
    #[cfg(feature = "desktop")]
    pub fn desktop(mut self) -> Self {
        self.platform_services = Some(gg_platform_desktop::DesktopPlatformServices::create());
        self
    }

    /// 使用 Web 平台服务
    #[cfg(feature = "web")]
    pub fn web(mut self) -> Self {
        self.platform_services = Some(gg_platform_web::WebPlatformServices::create("/assets"));
        self
    }

    /// 启用 HMR 热更新支持
    pub fn hmr_enabled(mut self) -> Self {
        self.hmr_enabled = true;
        self
    }

    /// 设置固定更新时间步长
    pub fn fixed_timestep(mut self, timestep: Duration) -> Self {
        self.fixed_timestep = Some(timestep);
        self
    }

    /// 设置目标帧率
    pub fn target_fps(mut self, fps: u32) -> Self {
        self.target_fps = Some(fps);
        self
    }

    /// 设置是否启用帧性能分析器
    pub fn with_frame_profiler(mut self, enabled: bool) -> Self {
        self.enable_frame_profiler = enabled;
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
