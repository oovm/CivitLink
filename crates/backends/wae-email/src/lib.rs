#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod mime;
pub mod sendmail;
pub mod smtp;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wae_types::{WaeError, WaeResult};

use crate::{
    mime::{EmailBuilder, EmailMessage},
    sendmail::{SendmailConfig, SendmailTransport},
    smtp::{AuthMechanism, SmtpClient, SmtpClientBuilder, SmtpError},
};

/// SMTP 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    /// 服务器主机名
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// 用户名
    pub username: String,
    /// 密码或授权码
    pub password: String,
    /// 发件人邮箱
    pub from_email: String,
}

/// 邮件提供者 trait
///
/// 定义邮件发送的统一接口。
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// 发送邮件
    ///
    /// # 参数
    /// - `to`: 收件人邮箱地址
    /// - `subject`: 邮件主题
    /// - `body`: 邮件正文（纯文本）
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> WaeResult<()>;
}

/// SMTP 邮件提供者
///
/// 通过 SMTP 协议发送邮件，支持 STARTTLS 加密和认证。
pub struct SmtpEmailProvider {
    /// 配置
    config: SmtpConfig,
}

impl SmtpEmailProvider {
    /// 创建新的 SMTP 邮件提供者
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    /// 构建 SMTP 客户端
    fn build_client(&self) -> SmtpClient {
        SmtpClientBuilder::new(&self.config.host, self.config.port)
            .timeout(Duration::from_secs(30))
            .use_starttls(true)
            .auth_mechanism(AuthMechanism::Login)
            .build()
    }

    /// 将 SMTP 错误转换为 WaeError
    fn map_smtp_error(e: SmtpError) -> WaeError {
        WaeError::connection_failed(format!("SMTP error: {}", e))
    }

    /// 构建邮件消息
    pub fn build_email(&self, to: &str, subject: &str, body: &str) -> EmailMessage {
        EmailBuilder::new().from(&self.config.from_email).to(to).subject(subject).body(body).build()
    }
}

#[async_trait]
impl EmailProvider for SmtpEmailProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> WaeResult<()> {
        let mut client = self.build_client();

        client.connect().await.map_err(Self::map_smtp_error)?;
        client.ehlo().await.map_err(Self::map_smtp_error)?;
        client.starttls().await.map_err(Self::map_smtp_error)?;
        client.authenticate(&self.config.username, &self.config.password).await.map_err(Self::map_smtp_error)?;

        let email = self.build_email(to, subject, body);
        let content = email.to_string();

        client.send_mail(&self.config.from_email, &[to], &content).await.map_err(Self::map_smtp_error)?;

        let _ = client.quit().await;

        Ok(())
    }
}

/// Sendmail 邮件提供者
///
/// 通过本地 sendmail 命令发送邮件。
pub struct SendmailEmailProvider {
    /// 传输实例
    transport: SendmailTransport,
    /// 发件人邮箱
    from_email: String,
}

impl SendmailEmailProvider {
    /// 创建新的 Sendmail 邮件提供者
    pub fn new(from_email: String) -> Self {
        Self { transport: SendmailTransport::new(), from_email }
    }

    /// 使用指定配置创建 Sendmail 邮件提供者
    pub fn with_config(from_email: String, config: SendmailConfig) -> Self {
        Self { transport: SendmailTransport::with_config(config), from_email }
    }

    /// 构建邮件消息
    pub fn build_email(&self, to: &str, subject: &str, body: &str) -> EmailMessage {
        EmailBuilder::new().from(&self.from_email).to(to).subject(subject).body(body).build()
    }
}

#[async_trait]
impl EmailProvider for SendmailEmailProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> WaeResult<()> {
        let email = self.build_email(to, subject, body);
        let raw_email = email.to_string();
        self.transport.send_raw(&raw_email).await
    }
}

/// 直接交付邮件提供者
///
/// 直接查询收件人域名的 MX 记录，并连接目标邮件服务器发送邮件。
/// 这种方式让程序直接充当 MTA（邮件传输代理）。
pub struct DirectEmailProvider {
    /// 发件人邮箱
    from_email: String,
}

impl DirectEmailProvider {
    /// 创建新的直接交付邮件提供者
    pub fn new(from_email: String) -> Self {
        Self { from_email }
    }

    /// 解析域名的 MX 记录
    async fn resolve_mx(&self, domain: &str) -> WaeResult<String> {
        use hickory_resolver::{Resolver, name_server::TokioConnectionProvider};

        let resolver = Resolver::builder_with_config(
            hickory_resolver::config::ResolverConfig::default(),
            TokioConnectionProvider::default(),
        )
        .build();

        let response = resolver
            .mx_lookup(domain)
            .await
            .map_err(|_| WaeError::connection_failed(format!("DNS resolution failed for {}", domain)))?;

        let mx = response
            .iter()
            .min_by_key(|mx| mx.preference())
            .ok_or_else(|| WaeError::storage_file_not_found(format!("No MX records found for {}", domain)))?;

        Ok(mx.exchange().to_string().trim_end_matches('.').to_string())
    }

    /// 构建邮件消息
    pub fn build_email(&self, to: &str, subject: &str, body: &str) -> EmailMessage {
        EmailBuilder::new().from(&self.from_email).to(to).subject(subject).body(body).build()
    }
}

#[async_trait]
impl EmailProvider for DirectEmailProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> WaeResult<()> {
        let domain =
            to.split('@').nth(1).ok_or_else(|| WaeError::invalid_format("email", format!("Invalid email address: {}", to)))?;

        let mx_host = self.resolve_mx(domain).await?;

        let mut client = SmtpClientBuilder::new(&mx_host, 25).timeout(Duration::from_secs(30)).use_starttls(false).build();

        client.connect().await.map_err(|e| WaeError::connection_failed(format!("{}: {}", mx_host, e)))?;
        client.ehlo().await.map_err(|e| WaeError::connection_failed(format!("{}: EHLO failed: {}", mx_host, e)))?;

        let email = self.build_email(to, subject, body);
        let content = email.to_string();

        client
            .send_mail(&self.from_email, &[to], &content)
            .await
            .map_err(|e| WaeError::connection_failed(format!("{}: {}", mx_host, e)))?;

        let _ = client.quit().await;

        Ok(())
    }
}
