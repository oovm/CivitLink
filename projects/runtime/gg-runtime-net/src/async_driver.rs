#![warn(missing_docs)]

//! 异步网络驱动模块
//! 提供基于 tokio 异步运行时的网络驱动和连接 trait

use std::{future::Future, pin::Pin};

use gg_core::GResult;

use crate::http::{HttpRequest, HttpResponse};

/// 异步网络驱动 trait
///
/// 定义基于 tokio 异步运行时的网络监听、接受连接和关闭接口。
/// 不同的网络协议（TCP、WebSocket 等）实现此 trait 以提供协议特定的异步驱动。
pub trait AsyncNetDriver: Send + Sync {
    /// 在指定地址上开始异步监听
    ///
    /// # 参数
    /// - `addr`: 监听地址，格式为 "host:port"
    fn listen<'a>(&'a mut self, addr: &'a str) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>>;

    /// 异步接受一个新的连接
    ///
    /// 非阻塞等待直到有新的客户端连接到达，返回封装后的异步连接对象。
    fn accept(&mut self) -> Pin<Box<dyn Future<Output = GResult<Box<dyn AsyncConnection>>> + Send + '_>>;

    /// 异步关闭网络驱动，释放资源
    fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>>;
}

/// 异步网络连接 trait
///
/// 定义基于 tokio 异步运行时的数据收发和连接管理接口。
/// 所有协议的异步连接对象都需要实现此 trait。
pub trait AsyncConnection: Send + Sync {
    /// 异步发送数据到对端
    ///
    /// # 参数
    /// - `data`: 要发送的字节数据
    fn send<'a>(&'a mut self, data: &'a [u8]) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>>;

    /// 异步从对端接收数据
    ///
    /// 非阻塞等待直到接收到数据，返回接收到的字节数据。
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = GResult<Vec<u8>>> + Send + '_>>;

    /// 异步关闭连接，释放资源
    fn close(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>>;

    /// 获取对端地址
    ///
    /// 返回对端的地址字符串，如果无法获取则返回 None。
    fn peer_addr(&self) -> Option<String>;
}

/// 中间件链中的下一个处理器
///
/// 表示中间件链中当前中间件之后的下一个处理器。
/// 中间件可以选择调用 `next` 将请求传递给下一个处理器，
/// 也可以直接返回响应以短路中间件链。
pub struct NextMiddleware<'a> {
    /// 剩余的中间件列表
    handlers: &'a [Box<dyn Middleware>],
    /// 最终的路由处理器
    final_handler: &'a (dyn Fn(&HttpRequest) -> GResult<HttpResponse> + Send + Sync),
}

impl<'a> NextMiddleware<'a> {
    /// 创建新的 NextMiddleware
    ///
    /// # 参数
    /// - `handlers`: 剩余的中间件列表
    /// - `final_handler`: 最终的路由处理器
    pub fn new(
        handlers: &'a [Box<dyn Middleware>],
        final_handler: &'a (dyn Fn(&HttpRequest) -> GResult<HttpResponse> + Send + Sync),
    ) -> Self {
        Self { handlers, final_handler }
    }

    /// 创建新的 NextMiddleware（接受 Box<dyn Fn + Send + Sync>）
    ///
    /// # 参数
    /// - `handlers`: 剩余的中间件列表
    /// - `final_handler`: 最终的路由处理器（Box 形式）
    pub fn new_sync(
        handlers: &'a [Box<dyn Middleware>],
        final_handler: &'a (dyn Fn(&HttpRequest) -> GResult<HttpResponse> + Send + Sync),
    ) -> Self {
        Self { handlers, final_handler }
    }

    /// 调用下一个中间件或最终处理器
    ///
    /// 如果还有中间件，则调用第一个中间件并将剩余的传递给下一个 NextMiddleware。
    /// 如果没有更多中间件，则调用最终的路由处理器。
    ///
    /// # 参数
    /// - `request`: 要传递的 HTTP 请求
    pub async fn call(&self, request: &HttpRequest) -> GResult<HttpResponse> {
        if let Some((first, rest)) = self.handlers.split_first() {
            let next = NextMiddleware::new(rest, self.final_handler);
            first.handle(request, next).await
        }
        else {
            (self.final_handler)(request)
        }
    }
}

/// HTTP 中间件 trait
///
/// 实现此 trait 以在 HTTP 请求处理前后执行自定义逻辑，
/// 例如日志记录、认证、CORS 处理等。
pub trait Middleware: Send + Sync {
    /// 处理 HTTP 请求
    ///
    /// 中间件可以在请求处理前后执行逻辑，通过调用 `next` 将请求传递给下一个处理器。
    ///
    /// # 参数
    /// - `request`: 收到的 HTTP 请求
    /// - `next`: 中间件链中的下一个处理器
    fn handle<'a>(
        &'a self,
        request: &'a HttpRequest,
        next: NextMiddleware<'a>,
    ) -> Pin<Box<dyn Future<Output = GResult<HttpResponse>> + Send + 'a>>;
}
