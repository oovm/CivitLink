use gg_core::GResult;

/// LSP 传输层 trait，定义与语言服务器的通信接口
pub trait LspTransport {
    /// 发送请求并等待响应
    ///
    /// # 参数
    /// - `method`: LSP 方法名
    /// - `params`: 请求参数
    ///
    /// # 返回
    /// 语言服务器的响应结果
    fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> GResult<serde_json::Value>;

    /// 发送通知，不等待响应
    ///
    /// # 参数
    /// - `method`: LSP 方法名
    /// - `params`: 通知参数
    fn send_notification(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> GResult<()>;
}
