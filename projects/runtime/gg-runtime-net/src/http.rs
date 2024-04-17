//! HTTP 网络驱动模块
//! 提供 HTTP 协议的请求/响应模型和路由驱动实现

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};

use crate::NetDriver;

/// HTTP 请求方法枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET 请求
    Get,
    /// POST 请求
    Post,
    /// PUT 请求
    Put,
    /// DELETE 请求
    Delete,
    /// PATCH 请求
    Patch,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

/// HTTP 请求
///
/// 封装 HTTP 请求的所有信息，包括方法、路径、头部和请求体。
pub struct HttpRequest {
    /// 请求方法
    pub method: HttpMethod,
    /// 请求路径
    pub path: String,
    /// 请求头部
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Vec<u8>,
    /// 路由参数
    pub params: HashMap<String, String>,
}

impl HttpRequest {
    /// 创建新的 HTTP 请求
    ///
    /// # 参数
    /// - `method`: 请求方法
    /// - `path`: 请求路径
    pub fn new(method: HttpMethod, path: &str) -> Self {
        Self { method, path: path.to_string(), headers: HashMap::new(), body: Vec::new(), params: HashMap::new() }
    }

    /// 添加请求头部
    ///
    /// # 参数
    /// - `key`: 头部名称
    /// - `value`: 头部值
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置请求体
    ///
    /// # 参数
    /// - `body`: 请求体字节数据
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}

/// HTTP 响应
///
/// 封装 HTTP 响应的所有信息，包括状态码、头部和响应体。
pub struct HttpResponse {
    /// HTTP 状态码
    pub status_code: u16,
    /// 响应头部
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// 创建新的 HTTP 响应
    ///
    /// # 参数
    /// - `status_code`: HTTP 状态码
    pub fn new(status_code: u16) -> Self {
        Self { status_code, headers: HashMap::new(), body: Vec::new() }
    }

    /// 创建 200 OK 响应
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// 创建 404 Not Found 响应
    pub fn not_found() -> Self {
        Self::new(404)
    }

    /// 创建 500 Internal Server Error 响应
    pub fn internal_error() -> Self {
        Self::new(500)
    }

    /// 添加响应头部
    ///
    /// # 参数
    /// - `key`: 头部名称
    /// - `value`: 头部值
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置响应体
    ///
    /// # 参数
    /// - `body`: 响应体字节数据
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}

/// HTTP 路由处理器 trait
///
/// 实现此 trait 以处理匹配到的 HTTP 请求，返回对应的响应。
pub trait RouteHandler {
    /// 处理 HTTP 请求
    ///
    /// # 参数
    /// - `request`: 收到的 HTTP 请求
    ///
    /// # 返回
    /// 处理后的 HTTP 响应
    fn handle(&self, request: &HttpRequest) -> GResult<HttpResponse>;
}

/// HTTP 网络驱动
///
/// 提供 HTTP 服务器功能，支持路由注册和请求分发。
/// 路由以 "METHOD /path" 格式注册，匹配对应的处理器。
pub struct HttpDriver {
    /// 已注册的路由处理器映射
    routes: HashMap<String, Box<dyn RouteHandler>>,
}

impl HttpDriver {
    /// 创建新的 HTTP 驱动
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    /// 注册路由处理器
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn register_route(&mut self, method: &HttpMethod, path: &str, handler: Box<dyn RouteHandler>) {
        let key = format!("{} {}", method, path);
        self.routes.insert(key, handler);
    }

    /// 处理 HTTP 请求
    ///
    /// 根据请求的方法和路径查找匹配的路由处理器并调用。
    /// 如果没有匹配的路由，返回 404 响应。
    ///
    /// # 参数
    /// - `request`: 收到的 HTTP 请求
    pub fn handle_request(&self, request: &HttpRequest) -> GResult<HttpResponse> {
        let key = format!("{} {}", request.method, request.path);
        match self.routes.get(&key) {
            Some(handler) => handler.handle(request),
            None => Ok(HttpResponse::not_found()),
        }
    }
}

impl NetDriver for HttpDriver {
    fn listen(&mut self, addr: &str) -> GResult<()> {
        let _ = addr;
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "HTTP 驱动需要 tokio 异步运行时支持，请使用 handle_request 处理请求".to_string(),
        })
    }

    fn accept(&mut self) -> GResult<Box<dyn crate::Connection>> {
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "HTTP 驱动不支持 accept 操作，请使用 handle_request 处理请求".to_string(),
        })
    }

    fn shutdown(&mut self) -> GResult<()> {
        self.routes.clear();
        Ok(())
    }
}
