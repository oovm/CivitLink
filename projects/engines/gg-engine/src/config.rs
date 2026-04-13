#![warn(missing_docs)]

//! GG 引擎配置模块

use serde::{Deserialize, Serialize};

/// 引擎配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// 窗口配置
    pub window: WindowConfig,
    /// 渲染配置
    pub render: RenderConfig,
    /// 资产配置
    pub asset: AssetConfig,
    /// 性能配置
    pub performance: PerformanceConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 是否可调整大小
    pub resizable: bool,
}

/// 渲染配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// 渲染后端
    pub backend: String,
    /// 抗锯齿设置
    pub anti_aliasing: bool,
    /// 最大帧率
    pub max_fps: u32,
}

/// 资产配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    /// 资产目录
    pub asset_dir: String,
    /// 纹理配置
    pub texture: TextureConfig,
    /// 音频配置
    pub audio: AudioConfig,
}

/// 纹理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureConfig {
    /// 默认纹理过滤模式
    pub default_filter: String,
    /// 纹理压缩设置
    pub compression: bool,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 默认音量
    pub default_volume: f32,
    /// 音频设备
    pub device: Option<String>,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 是否启用多线程
    pub multithreading: bool,
    /// 线程池大小
    pub thread_pool_size: usize,
    /// 是否启用帧时间统计
    pub frame_time_stats: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig { title: "Pleroma".to_string(), width: 1280, height: 720, fullscreen: false, resizable: true },
            render: RenderConfig { backend: "wgpu".to_string(), anti_aliasing: true, max_fps: 60 },
            asset: AssetConfig {
                asset_dir: "assets".to_string(),
                texture: TextureConfig { default_filter: "linear".to_string(), compression: true },
                audio: AudioConfig { default_volume: 0.8, device: None },
            },
            performance: PerformanceConfig { multithreading: true, thread_pool_size: num_cpus::get(), frame_time_stats: false },
        }
    }
}
