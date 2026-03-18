//! 模板渲染模块
//!
//! 提供基于 Askama 模板引擎的渲染支持。
//!
//! # 特性
//!
//! - 编译时模板检查，类型安全
//! - 自动 HTML 转义，防止 XSS 攻击
//! - 可配置模板目录和扩展名

use http::{Response, StatusCode, header};
use std::path::PathBuf;

use crate::{Body, full_body};

pub use askama;

/// 模板配置
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// 模板根目录
    pub template_dir: PathBuf,
    /// 模板文件扩展名
    pub extension: String,
    /// 是否启用缓存（生产环境建议启用）
    pub enable_cache: bool,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self { template_dir: PathBuf::from("templates"), extension: "html".to_string(), enable_cache: true }
    }
}

impl TemplateConfig {
    /// 创建新的模板配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置模板目录
    pub fn with_template_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.template_dir = dir.into();
        self
    }

    /// 设置模板扩展名
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = ext.into();
        self
    }

    /// 设置是否启用缓存
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }
}

/// 模板渲染器
///
/// 提供模板渲染的核心功能。
///
/// # 示例
///
/// ```ignore
/// use wae_https::template::{TemplateConfig, TemplateRenderer};
///
/// let config = TemplateConfig::new()
///     .with_template_dir("views");
///
/// let renderer = TemplateRenderer::new(config);
/// ```
#[derive(Debug, Clone)]
pub struct TemplateRenderer {
    config: TemplateConfig,
}

impl TemplateRenderer {
    /// 创建新的模板渲染器
    pub fn new(config: TemplateConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建模板渲染器
    pub fn default_renderer() -> Self {
        Self::new(TemplateConfig::default())
    }

    /// 获取配置引用
    pub fn config(&self) -> &TemplateConfig {
        &self.config
    }

    /// 获取模板目录路径
    pub fn template_dir(&self) -> &PathBuf {
        &self.config.template_dir
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::default_renderer()
    }
}

/// 模板渲染错误
#[derive(Debug)]
pub enum TemplateError {
    /// 模板未找到
    NotFound(String),
    /// 渲染错误
    RenderError(String),
    /// IO 错误
    IoError(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::NotFound(name) => write!(f, "Template not found: {}", name),
            TemplateError::RenderError(msg) => write!(f, "Template render error: {}", msg),
            TemplateError::IoError(msg) => write!(f, "Template IO error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}

/// HTML 模板响应包装器
///
/// 用于将 Askama 模板包装为 HTTP 响应。
/// 支持自动设置 Content-Type 头。
pub struct HtmlTemplate<T> {
    /// 内部模板
    inner: T,
}

impl<T> HtmlTemplate<T> {
    /// 创建新的 HTML 模板包装器
    pub fn new(template: T) -> Self {
        Self { inner: template }
    }

    /// 获取内部模板引用
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// 消费包装器，返回内部模板
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> From<T> for HtmlTemplate<T> {
    fn from(template: T) -> Self {
        Self::new(template)
    }
}

impl<T> HtmlTemplate<T>
where
    T: askama::Template,
{
    /// 转换为 HTTP 响应
    pub fn into_response(self) -> Response<Body> {
        match self.inner.render() {
            Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(full_body(html))
                .unwrap(),
            Err(e) => {
                tracing::error!("Template render error: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(full_body(format!("Template error: {}", e)))
                    .unwrap()
            }
        }
    }
}

/// 模板响应构建器
///
/// 提供便捷的模板响应构建方法。
pub struct TemplateResponse;

impl TemplateResponse {
    /// 返回 HTML 模板响应
    pub fn html<T: askama::Template>(template: T) -> Response<Body> {
        HtmlTemplate::new(template).into_response()
    }

    /// 返回带自定义状态码的 HTML 模板响应
    pub fn html_with_status<T: askama::Template>(template: T, status: StatusCode) -> Response<Body> {
        match template.render() {
            Ok(html) => Response::builder()
                .status(status)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(full_body(html))
                .unwrap(),
            Err(e) => {
                tracing::error!("Template render error: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(full_body(format!("Template error: {}", e)))
                    .unwrap()
            }
        }
    }
}
