use gg_core::platform::{PlatformServices, WindowConfig};

use crate::render::RenderBackendType;
use crate::{WebFileSystem, WebInput, WebThread, WebTime, WebWindow};

/// Web 平台服务工厂
///
/// 创建 Web 平台的 `PlatformServices` 实例。
pub struct WebPlatformServices;

impl WebPlatformServices {
    /// 创建 Web 平台的平台服务
    ///
    /// # 参数
    ///
    /// - `base_url` - 资源文件的基础 URL 路径
    pub fn create(base_url: impl Into<String>) -> PlatformServices {
        Self::create_with_config(base_url, WindowConfig::default())
    }

    /// 使用指定窗口配置创建 Web 平台的平台服务
    pub fn create_with_config(base_url: impl Into<String>, window_config: WindowConfig) -> PlatformServices {
        PlatformServices::new(
            Box::new(WebFileSystem::new(base_url)),
            Box::new(WebInput::new()),
            Box::new(WebTime::new()),
            Box::new(WebWindow::new(window_config)),
            Box::new(WebThread::new()),
        )
    }

    /// 检测 Web 平台的最佳渲染后端
    ///
    /// 优先返回 WebGPU，若不可用则回退到 WebGL2。
    /// 若两者均不可用则返回 `None`。
    pub fn detect_render_backend() -> Option<RenderBackendType> {
        crate::render::detect_best_backend()
    }
}
