//! Sendmail 传输模块
//!
//! 通过调用本地 sendmail 命令发送邮件，使用异步子进程实现。

use std::process::Stdio;

use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, process::Command};
use wae_types::{WaeError, WaeResult};

use crate::EmailProvider;

/// Sendmail 传输配置
#[derive(Debug, Clone)]
pub struct SendmailConfig {
    /// sendmail 命令路径
    pub command: String,
}

impl Default for SendmailConfig {
    fn default() -> Self {
        Self { command: "sendmail".to_string() }
    }
}

/// Sendmail 传输提供者
///
/// 通过调用本地 sendmail 命令发送邮件。
/// 使用 `-t` 参数从邮件头读取收件人信息。
pub struct SendmailTransport {
    /// 配置
    config: SendmailConfig,
}

impl SendmailTransport {
    /// 创建新的 Sendmail 传输
    ///
    /// 使用默认配置（sendmail 命令）
    pub fn new() -> Self {
        Self { config: SendmailConfig::default() }
    }

    /// 使用指定配置创建 Sendmail 传输
    pub fn with_config(config: SendmailConfig) -> Self {
        Self { config }
    }

    /// 发送原始邮件内容
    ///
    /// 通过标准输入将邮件内容传递给 sendmail 命令。
    ///
    /// # 参数
    /// - `raw_email`: 完整的原始邮件内容（包含邮件头和正文）
    pub async fn send_raw(&self, raw_email: &str) -> WaeResult<()> {
        let mut child = Command::new(&self.config.command)
            .arg("-t")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| WaeError::internal(format!("Failed to spawn sendmail process: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(raw_email.as_bytes())
                .await
                .map_err(|e| WaeError::internal(format!("Failed to write to sendmail stdin: {}", e)))?;
            stdin.flush().await.map_err(|e| WaeError::internal(format!("Failed to flush sendmail stdin: {}", e)))?;
        }
        else {
            return Err(WaeError::internal("Failed to open stdin for sendmail process"));
        }

        let status =
            child.wait().await.map_err(|e| WaeError::internal(format!("Failed to wait for sendmail process: {}", e)))?;

        if status.success() {
            Ok(())
        }
        else {
            let code = status.code().unwrap_or(-1);
            Err(WaeError::internal(format!("Sendmail exited with non-zero status: {}", code)))
        }
    }
}

impl Default for SendmailTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Sendmail 邮件提供者
///
/// 实现 `EmailProvider` trait，通过 sendmail 命令发送邮件。
pub struct SendmailEmailProvider {
    /// 传输实例
    transport: SendmailTransport,
    /// 发件人邮箱
    from_email: String,
}

impl SendmailEmailProvider {
    /// 创建新的 Sendmail 邮件提供者
    ///
    /// # 参数
    /// - `from_email`: 发件人邮箱地址
    pub fn new(from_email: String) -> Self {
        Self { transport: SendmailTransport::new(), from_email }
    }

    /// 使用指定配置创建 Sendmail 邮件提供者
    pub fn with_config(from_email: String, config: SendmailConfig) -> Self {
        Self { transport: SendmailTransport::with_config(config), from_email }
    }

    /// 构建原始邮件内容
    pub fn build_raw_email(&self, to: &str, subject: &str, body: &str) -> String {
        format!("From: {}\r\nTo: {}\r\nSubject: {}\r\n\r\n{}", self.from_email, to, subject, body)
    }
}

#[async_trait]
impl EmailProvider for SendmailEmailProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> WaeResult<()> {
        let raw_email = self.build_raw_email(to, subject, body);
        self.transport.send_raw(&raw_email).await
    }
}
