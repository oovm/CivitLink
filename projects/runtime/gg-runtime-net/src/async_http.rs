//! 异步 HTTP 服务器模块
//! 提供基于 tokio 的异步 HTTP 服务器、路由前缀树和内置中间件实现

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use gg_core::{GError, GErrorKind, GResult};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::async_driver::{AsyncConnection, AsyncNetDriver, Middleware, NextMiddleware};
use crate::http::{HttpMethod, HttpRequest, HttpResponse};

/// 路由前缀树节点
///
/// 每个节点存储路径段名称和子节点映射，支持路径参数提取。
struct TrieNode {
    /// 子节点映射，键为路径段
    children: HashMap<String, TrieNode>,
    /// 路径参数段名称（如 "id" 对应 ":id"）
    param_name: Option<String>,
    /// 参数子节点
    param_child: Option<Box<TrieNode>>,
    /// 路由处理器，存储 "METHOD" -> handler 的映射
    handlers: HashMap<String, Box<dyn Fn(&HttpRequest, &HashMap<String, String>) -> GResult<HttpResponse> + Send + Sync>>,
}

impl TrieNode {
    /// 创建新的前缀树节点
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            param_name: None,
            param_child: None,
            handlers: HashMap::new(),
        }
    }
}

/// 路由前缀树
///
/// 支持路径参数的 HTTP 路由匹配树。
/// 路径参数使用 `:name` 语法，例如 `/api/v1/users/:id` 中的 `:id` 会被提取为参数。
pub struct RouteTrie {
    /// 根节点
    root: TrieNode,
}

impl RouteTrie {
    /// 创建新的路由前缀树
    pub fn new() -> Self {
        Self { root: TrieNode::new() }
    }

    /// 注册路由处理器
    ///
    /// 将指定方法和路径的处理器注册到路由树中。
    /// 路径参数使用 `:name` 语法标记。
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 路由路径，如 "/api/v1/users/:id"
    /// - `handler`: 路由处理函数
    pub fn insert(
        &mut self,
        method: HttpMethod,
        path: &str,
        handler: Box<dyn Fn(&HttpRequest, &HashMap<String, String>) -> GResult<HttpResponse> + Send + Sync>,
    ) {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut self.root;

        for segment in &segments {
            if segment.starts_with(':') {
                let param_name = segment[1..].to_string();
                if current.param_child.is_none() {
                    current.param_name = Some(param_name.clone());
                    current.param_child = Some(Box::new(TrieNode::new()));
                }
                current = current.param_child.as_mut().unwrap().as_mut();
            } else {
                current = current.children.entry(segment.to_string()).or_insert_with(TrieNode::new);
            }
        }

        current.handlers.insert(method.to_string(), handler);
    }

    /// 查找匹配的路由处理器
    ///
    /// 根据方法和路径在路由树中查找匹配的处理器，同时提取路径参数。
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 请求路径
    ///
    /// # 返回
    /// 匹配的处理器和提取的路径参数，未找到时返回 None
    pub fn find(
        &self,
        method: &HttpMethod,
        path: &str,
    ) -> Option<(&dyn Fn(&HttpRequest, &HashMap<String, String>) -> GResult<HttpResponse>, HashMap<String, String>)> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut params = HashMap::new();
        let mut current = &self.root;

        for segment in &segments {
            if let Some(child) = current.children.get(*segment) {
                current = child;
            } else if let Some(ref param_child) = current.param_child {
                if let Some(ref param_name) = current.param_name {
                    params.insert(param_name.clone(), segment.to_string());
                }
                current = param_child.as_ref();
            } else {
                return None;
            }
        }

        current.handlers.get(&method.to_string()).map(|h| (h.as_ref(), params))
    }
}

/// 解析原始 HTTP 请求数据
///
/// 将 TCP 流中读取的原始字节数据解析为结构化的 HttpRequest。
/// 支持解析请求行、头部和请求体。
///
/// # 参数
/// - `data`: 原始 HTTP 请求数据
fn parse_http_request(data: &[u8]) -> GResult<HttpRequest> {
    let text = std::str::from_utf8(data).map_err(|e| GError {
        kind: GErrorKind::Network,
        message: format!("HTTP 请求数据非有效 UTF-8: {}", e),
    })?;

    let mut parts = text.split("\r\n\r\n");
    let header_section = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or("").as_bytes().to_vec();

    let mut lines = header_section.split("\r\n");
    let request_line = lines.next().ok_or_else(|| GError {
        kind: GErrorKind::Network,
        message: "HTTP 请求行缺失".to_string(),
    })?;

    let mut request_parts = request_line.split_whitespace();
    let method_str = request_parts.next().ok_or_else(|| GError {
        kind: GErrorKind::Network,
        message: "HTTP 请求方法缺失".to_string(),
    })?;
    let path = request_parts.next().ok_or_else(|| GError {
        kind: GErrorKind::Network,
        message: "HTTP 请求路径缺失".to_string(),
    })?;

    let method = match method_str {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        _ => HttpMethod::Get,
    };

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let path_only = path.split('?').next().unwrap_or(path).to_string();

    Ok(HttpRequest { method, path: path_only, headers, body })
}

/// 格式化 HTTP 响应为原始字节数据
///
/// 将结构化的 HttpResponse 转换为可发送的原始 HTTP 响应字节数据。
///
/// # 参数
/// - `response`: HTTP 响应对象
fn format_http_response(response: &HttpResponse) -> Vec<u8> {
    let status_text = match response.status_code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let mut output = format!("HTTP/1.1 {} {}\r\n", response.status_code, status_text);

    let has_content_length = response.headers.contains_key("Content-Length");
    let has_content_type = response.headers.contains_key("Content-Type");

    for (key, value) in &response.headers {
        output.push_str(&format!("{}: {}\r\n", key, value));
    }

    if !has_content_type {
        output.push_str("Content-Type: application/octet-stream\r\n");
    }

    if !has_content_length {
        output.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    }

    output.push_str("Connection: close\r\n");
    output.push_str("\r\n");

    let mut bytes = output.into_bytes();
    bytes.extend_from_slice(&response.body);
    bytes
}

/// 异步 HTTP 服务器驱动
///
/// 基于 tokio TcpListener 的异步 HTTP 服务器实现。
/// 支持路由前缀树匹配、路径参数提取和中间件链处理。
pub struct AsyncHttpDriver {
    /// TCP 异步监听器
    listener: Option<TcpListener>,
    /// 路由前缀树
    router: RouteTrie,
    /// 中间件列表
    middlewares: Vec<Box<dyn Middleware>>,
}

impl AsyncHttpDriver {
    /// 创建新的异步 HTTP 驱动
    pub fn new() -> Self {
        Self {
            listener: None,
            router: RouteTrie::new(),
            middlewares: Vec::new(),
        }
    }

    /// 注册路由处理器
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 路由路径，支持 `:param` 路径参数
    /// - `handler`: 路由处理函数
    pub fn route(
        &mut self,
        method: HttpMethod,
        path: &str,
        handler: Box<dyn Fn(&HttpRequest, &HashMap<String, String>) -> GResult<HttpResponse> + Send + Sync>,
    ) {
        self.router.insert(method, path, handler);
    }

    /// 添加中间件
    ///
    /// 中间件按添加顺序依次执行。
    ///
    /// # 参数
    /// - `middleware`: 中间件实例
    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.middlewares.push(middleware);
    }

    /// 处理单个 HTTP 连接
    ///
    /// 从 TCP 流中读取 HTTP 请求，经过路由匹配和中间件链处理后返回响应。
    ///
    /// # 参数
    /// - `stream`: TCP 数据流
    pub async fn handle_connection(&self, stream: tokio::net::TcpStream) -> GResult<()> {
        let mut buf = vec![0u8; 8192];
        let n = {
            let n = stream.read(&mut buf).await.map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("HTTP 读取请求失败: {}", e),
            })?;
            n
        };

        if n == 0 {
            return Ok(());
        }

        let request = match parse_http_request(&buf[..n]) {
            Ok(req) => req,
            Err(e) => {
                let response = HttpResponse::new(400).with_body(format!("Bad Request: {}", e.message).into_bytes());
                let response_bytes = format_http_response(&response);
                let mut writer = stream;
                let _ = writer.write_all(&response_bytes).await;
                let _ = writer.shutdown().await;
                return Ok(());
            }
        };

        let response = match self.router.find(&request.method, &request.path) {
            Some((handler, params)) => {
                let final_handler = |req: &HttpRequest| handler(req, &params);
                if self.middlewares.is_empty() {
                    final_handler(&request)
                } else {
                    let next = NextMiddleware::new(&self.middlewares, &final_handler);
                    next.call(&request).await
                }
            }
            None => {
                let final_handler = |_req: &HttpRequest| Ok(HttpResponse::not_found());
                if self.middlewares.is_empty() {
                    final_handler(&request)
                } else {
                    let next = NextMiddleware::new(&self.middlewares, &final_handler);
                    next.call(&request).await
                }
            }
        };

        let response = response.unwrap_or_else(|_| HttpResponse::internal_error());
        let response_bytes = format_http_response(&response);

        let mut writer = stream;
        writer.write_all(&response_bytes).await.map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("HTTP 写入响应失败: {}", e),
        })?;
        let _ = writer.shutdown().await;

        Ok(())
    }
}

impl AsyncNetDriver for AsyncHttpDriver {
    fn listen<'a>(&'a mut self, addr: &'a str) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let listener = TcpListener::bind(addr).await.map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("HTTP 服务器监听绑定失败: {}", e),
            })?;
            self.listener = Some(listener);
            Ok(())
        })
    }

    fn accept(&mut self) -> Pin<Box<dyn Future<Output = GResult<Box<dyn AsyncConnection>>> + Send + '_>> {
        Box::pin(async {
            let listener = self.listener.as_ref().ok_or_else(|| GError {
                kind: GErrorKind::Runtime,
                message: "HTTP 服务器尚未开始监听".to_string(),
            })?;
            let (stream, _) = listener.accept().await.map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("HTTP 服务器接受连接失败: {}", e),
            })?;
            self.handle_connection(stream).await?;
            Err(GError {
                kind: GErrorKind::Runtime,
                message: "HTTP 驱动 accept 不返回连接对象，请使用 handle_connection".to_string(),
            })
        })
    }

    fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>> {
        Box::pin(async {
            self.listener = None;
            Ok(())
        })
    }
}

/// 日志中间件
///
/// 记录每个 HTTP 请求的方法、路径和响应状态码。
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        request: &'a HttpRequest,
        next: NextMiddleware<'a>,
    ) -> Pin<Box<dyn Future<Output = GResult<HttpResponse>> + Send + 'a>> {
        let method = request.method.to_string();
        let path = request.path.clone();
        Box::pin(async move {
            log::info!("HTTP 请求: {} {}", method, path);
            let response = next.call(request).await;
            match &response {
                Ok(resp) => log::info!("HTTP 响应: {} {} -> {}", method, path, resp.status_code),
                Err(e) => log::error!("HTTP 错误: {} {} -> {}", method, path, e.message),
            }
            response
        })
    }
}

/// CORS 中间件
///
/// 为 HTTP 响应添加跨域资源共享（CORS）头部。
pub struct CorsMiddleware {
    /// 允许的源
    pub allow_origin: String,
    /// 允许的方法
    pub allow_methods: String,
    /// 允许的头部
    pub allow_headers: String,
}

impl CorsMiddleware {
    /// 创建新的 CORS 中间件，使用默认配置
    pub fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }
}

impl Middleware for CorsMiddleware {
    fn handle<'a>(
        &'a self,
        request: &'a HttpRequest,
        next: NextMiddleware<'a>,
    ) -> Pin<Box<dyn Future<Output = GResult<HttpResponse>> + Send + 'a>> {
        let origin = self.allow_origin.clone();
        let methods = self.allow_methods.clone();
        let headers = self.allow_headers.clone();
        Box::pin(async move {
            let mut response = next.call(request).await?;
            response.headers.insert("Access-Control-Allow-Origin".to_string(), origin);
            response.headers.insert("Access-Control-Allow-Methods".to_string(), methods);
            response.headers.insert("Access-Control-Allow-Headers".to_string(), headers);
            Ok(response)
        })
    }
}

/// 认证中间件
///
/// 检查 HTTP 请求中的 Authorization 头部，未提供时返回 401 响应。
pub struct AuthMiddleware {
    /// 认证类型前缀（如 "Bearer"）
    pub auth_prefix: String,
}

impl AuthMiddleware {
    /// 创建新的认证中间件
    ///
    /// # 参数
    /// - `auth_prefix`: 认证类型前缀，如 "Bearer"
    pub fn new(auth_prefix: &str) -> Self {
        Self { auth_prefix: auth_prefix.to_string() }
    }
}

impl Middleware for AuthMiddleware {
    fn handle<'a>(
        &'a self,
        request: &'a HttpRequest,
        next: NextMiddleware<'a>,
    ) -> Pin<Box<dyn Future<Output = GResult<HttpResponse>> + Send + 'a>> {
        let has_auth = request.headers.get("Authorization").map(|v| v.starts_with(&self.auth_prefix)).unwrap_or(false);
        let prefix = self.auth_prefix.clone();
        Box::pin(async move {
            if !has_auth {
                let mut response = HttpResponse::new(401);
                response.headers.insert("WWW-Authenticate".to_string(), format!("{} realm=\"gg-engine\"", prefix));
                response.body = b"Unauthorized".to_vec();
                return Ok(response);
            }
            next.call(request).await
        })
    }
}
