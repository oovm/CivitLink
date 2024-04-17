//! LSP 传输层实现
//!
//! 提供与语言服务器通信的传输层抽象和 stdio 传输实现。

use std::io::{BufRead, BufReader, Write};

use gg_core::{GError, GErrorKind, GResult};

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
    fn send_request(&mut self, method: &str, params: serde_json::Value) -> GResult<serde_json::Value>;

    /// 发送通知，不等待响应
    ///
    /// # 参数
    /// - `method`: LSP 方法名
    /// - `params`: 通知参数
    fn send_notification(&mut self, method: &str, params: serde_json::Value) -> GResult<()>;
}

/// stdio 传输层实现，通过子进程标准输入/输出与语言服务器通信
pub struct StdioTransport {
    /// 子进程
    child: std::process::Child,
    /// 请求 ID 计数器，用于生成唯一的请求标识
    request_id: u64,
}

impl StdioTransport {
    /// 创建新的 stdio 传输层
    ///
    /// 启动子进程并将其标准输入/输出管道化，用于 JSON-RPC 通信
    ///
    /// # 参数
    /// - `server_command`: 语言服务器可执行文件路径
    /// - `args`: 传递给语言服务器的命令行参数
    pub fn new(server_command: &str, args: &[&str]) -> GResult<Self> {
        let child = std::process::Command::new(server_command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("启动语言服务器失败: {}", e) })?;
        Ok(Self { child, request_id: 0 })
    }
}

impl LspTransport for StdioTransport {
    fn send_request(&mut self, method: &str, params: serde_json::Value) -> GResult<serde_json::Value> {
        self.request_id += 1;
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params,
        });

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "无法获取子进程标准输入".to_string() })?;
        write_message(stdin, &message)?;

        let stdout = self
            .child
            .stdout
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "无法获取子进程标准输出".to_string() })?;
        let mut reader = BufReader::new(stdout);
        let response = read_message(&mut reader)?;

        let response_id = response.get("id").and_then(|v| v.as_u64());
        if response_id != Some(self.request_id) {
            return Err(GError {
                kind: GErrorKind::Other,
                message: format!("请求 ID 不匹配: 期望 {}, 收到 {:?}", self.request_id, response_id),
            });
        }

        if let Some(error) = response.get("error") {
            let msg = error.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
            return Err(GError { kind: GErrorKind::Other, message: format!("LSP 请求错误: {}", msg) });
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "响应中缺少 result 字段".to_string() })
    }

    fn send_notification(&mut self, method: &str, params: serde_json::Value) -> GResult<()> {
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "无法获取子进程标准输入".to_string() })?;
        write_message(stdin, &message)?;
        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 从子进程 stdout 读取 JSON-RPC 消息
///
/// 按照 JSON-RPC over stdio 协议，先读取 Content-Length 头部，
/// 再读取指定长度的 JSON 消息体
///
/// # 参数
/// - `stdout`: 子进程的标准输出读取器
///
/// # 返回
/// 解析后的 JSON-RPC 消息
pub fn read_message(stdout: &mut dyn BufRead) -> GResult<serde_json::Value> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut header_line = String::new();
        let bytes_read = stdout
            .read_line(&mut header_line)
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("读取头部失败: {}", e) })?;
        if bytes_read == 0 {
            return Err(GError { kind: GErrorKind::Other, message: "子进程已关闭".to_string() });
        }
        let header_line = header_line.trim();
        if header_line.is_empty() {
            break;
        }
        if let Some(length_str) = header_line.strip_prefix("Content-Length:") {
            content_length = Some(
                length_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析 Content-Length 失败: {}", e) })?,
            );
        }
    }

    let length = content_length
        .ok_or_else(|| GError { kind: GErrorKind::Other, message: "缺少 Content-Length 头部".to_string() })?;

    let mut body = vec![0u8; length];
    stdout
        .read_exact(&mut body)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("读取消息体失败: {}", e) })?;

    let body_str = String::from_utf8(body)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("消息体不是有效的 UTF-8: {}", e) })?;

    serde_json::from_str(&body_str)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析 JSON 消息失败: {}", e) })
}

/// 将 JSON-RPC 消息写入子进程 stdin
///
/// 按照 JSON-RPC over stdio 协议，写入 Content-Length 头部，
/// 然后写入 JSON 消息体
///
/// # 参数
/// - `stdin`: 子进程的标准输入写入器
/// - `message`: 要发送的 JSON-RPC 消息
pub fn write_message(stdin: &mut dyn Write, message: &serde_json::Value) -> GResult<()> {
    let body = serde_json::to_string(message)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("序列化 JSON 消息失败: {}", e) })?;

    write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("写入消息失败: {}", e) })?;

    stdin.flush().map_err(|e| GError { kind: GErrorKind::Other, message: format!("刷新标准输入失败: {}", e) })?;

    Ok(())
}
