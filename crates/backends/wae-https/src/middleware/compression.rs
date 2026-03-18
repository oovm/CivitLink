//! 压缩中间件
//!
//! 提供响应压缩功能，支持 gzip、brotli、deflate 压缩算法。
//! 根据 Accept-Encoding 请求头自动选择最优压缩算法。

use tower_http::compression::CompressionLayer as TowerCompressionLayer;

/// 压缩算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// gzip 压缩算法
    Gzip,

    /// brotli 压缩算法
    Brotli,

    /// deflate 压缩算法
    Deflate,
}

/// 压缩配置
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// 是否启用 gzip 压缩
    pub enable_gzip: bool,

    /// 是否启用 brotli 压缩
    pub enable_brotli: bool,

    /// 是否启用 deflate 压缩
    pub enable_deflate: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self { enable_gzip: true, enable_brotli: true, enable_deflate: true }
    }
}

impl CompressionConfig {
    /// 创建默认压缩配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 gzip 压缩开关
    pub fn with_gzip(mut self, enable: bool) -> Self {
        self.enable_gzip = enable;
        self
    }

    /// 设置 brotli 压缩开关
    pub fn with_brotli(mut self, enable: bool) -> Self {
        self.enable_brotli = enable;
        self
    }

    /// 设置 deflate 压缩开关
    pub fn with_deflate(mut self, enable: bool) -> Self {
        self.enable_deflate = enable;
        self
    }
}

/// 压缩中间件层
///
/// 封装 tower-http 的 CompressionLayer，提供响应压缩功能。
/// 根据 Accept-Encoding 请求头自动选择客户端支持的压缩算法。
pub struct Compression {
    config: CompressionConfig,
}

impl Compression {
    /// 使用默认配置创建压缩中间件
    pub fn new() -> Self {
        Self { config: CompressionConfig::new() }
    }

    /// 使用指定配置创建压缩中间件
    pub fn with_config(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// 获取当前配置
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }

    /// 构建 CompressionLayer
    pub fn build(&self) -> TowerCompressionLayer {
        let mut layer = TowerCompressionLayer::new();

        if self.config.enable_gzip {
            layer = layer.gzip(true);
        }

        if self.config.enable_brotli {
            layer = layer.br(true);
        }

        if self.config.enable_deflate {
            layer = layer.deflate(true);
        }

        layer
    }
}

impl Default for Compression {
    fn default() -> Self {
        Self::new()
    }
}

/// 压缩中间件构建器
///
/// 提供链式 API 构建压缩中间件配置。
pub struct CompressionBuilder {
    config: CompressionConfig,
}

impl CompressionBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self { config: CompressionConfig::new() }
    }

    /// 启用 gzip 压缩
    pub fn gzip(mut self) -> Self {
        self.config.enable_gzip = true;
        self
    }

    /// 禁用 gzip 压缩
    pub fn no_gzip(mut self) -> Self {
        self.config.enable_gzip = false;
        self
    }

    /// 启用 brotli 压缩
    pub fn brotli(mut self) -> Self {
        self.config.enable_brotli = true;
        self
    }

    /// 禁用 brotli 压缩
    pub fn no_brotli(mut self) -> Self {
        self.config.enable_brotli = false;
        self
    }

    /// 启用 deflate 压缩
    pub fn deflate(mut self) -> Self {
        self.config.enable_deflate = true;
        self
    }

    /// 禁用 deflate 压缩
    pub fn no_deflate(mut self) -> Self {
        self.config.enable_deflate = false;
        self
    }

    /// 构建压缩中间件
    pub fn build(self) -> Compression {
        Compression::with_config(self.config)
    }
}

impl Default for CompressionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
