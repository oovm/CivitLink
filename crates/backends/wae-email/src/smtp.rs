//! SMTP 协议客户端实现
//!
//! 提供异步 SMTP 客户端，支持：
//! - EHLO/HELO 握手
//! - STARTTLS 加密升级
//! - AUTH 认证（PLAIN 和 LOGIN 机制）
//! - 完整的邮件发送流程

use std::{sync::Arc, time::Duration};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
};
use tokio_rustls::{TlsConnector, rustls::pki_types::ServerName};

/// SMTP 响应码分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpResponseCategory {
    /// 成功响应 (2xx)
    Success,
    /// 继续响应 (3xx)
    Continue,
    /// 暂时失败 (4xx)
    TransientFailure,
    /// 永久失败 (5xx)
    PermanentFailure,
}

/// SMTP 认证机制
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMechanism {
    /// PLAIN 认证机制
    Plain,
    /// LOGIN 认证机制
    Login,
}

/// SMTP 客户端配置
#[derive(Debug, Clone)]
pub struct SmtpClientConfig {
    /// 服务器主机名
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// 连接超时时间
    pub timeout: Duration,
    /// 本地主机名（用于 EHLO/HELO）
    pub local_hostname: String,
    /// 是否使用 STARTTLS
    pub use_starttls: bool,
    /// 认证机制
    pub auth_mechanism: AuthMechanism,
}

impl Default for SmtpClientConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 25,
            timeout: Duration::from_secs(30),
            local_hostname: "localhost".to_string(),
            use_starttls: true,
            auth_mechanism: AuthMechanism::Plain,
        }
    }
}

/// SMTP 响应
#[derive(Debug, Clone)]
pub struct SmtpResponse {
    /// 响应码
    pub code: u16,
    /// 响应消息
    pub message: String,
}

impl SmtpResponse {
    /// 获取响应码分类
    pub fn category(&self) -> SmtpResponseCategory {
        match self.code {
            200..=299 => SmtpResponseCategory::Success,
            300..=399 => SmtpResponseCategory::Continue,
            400..=499 => SmtpResponseCategory::TransientFailure,
            500..=599 => SmtpResponseCategory::PermanentFailure,
            _ => SmtpResponseCategory::PermanentFailure,
        }
    }

    /// 检查是否为成功响应
    pub fn is_success(&self) -> bool {
        self.category() == SmtpResponseCategory::Success
    }

    /// 检查是否为继续响应
    pub fn is_continue(&self) -> bool {
        self.category() == SmtpResponseCategory::Continue
    }
}

/// SMTP 错误类型
#[derive(Debug, Clone)]
pub enum SmtpError {
    /// 连接错误
    ConnectionError(String),
    /// TLS 错误
    TlsError(String),
    /// 认证错误
    AuthenticationError(String),
    /// 协议错误
    ProtocolError(String),
    /// 响应错误
    ResponseError {
        /// 响应码
        code: u16,
        /// 响应消息
        message: String,
    },
    /// IO 错误
    IoError(String),
    /// 超时错误
    TimeoutError,
}

impl std::fmt::Display for SmtpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmtpError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            SmtpError::TlsError(msg) => write!(f, "TLS error: {}", msg),
            SmtpError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            SmtpError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            SmtpError::ResponseError { code, message } => {
                write!(f, "SMTP response error: {} {}", code, message)
            }
            SmtpError::IoError(msg) => write!(f, "IO error: {}", msg),
            SmtpError::TimeoutError => write!(f, "Timeout error"),
        }
    }
}

impl std::error::Error for SmtpError {}

/// SMTP 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpConnectionState {
    /// 未连接
    Disconnected,
    /// 已连接，等待问候
    Connected,
    /// 已发送 EHLO/HELO
    Ready,
    /// 已升级到 TLS
    Secured,
    /// 已认证
    Authenticated,
    /// 正在发送邮件
    SendingMail,
}

/// TCP 流类型枚举
enum TcpStreamType {
    /// 普通流
    Plain(TcpStream),
    /// TLS 加密流
    Tls(Box<tokio_rustls::client::TlsStream<TcpStream>>),
}

/// SMTP 客户端
pub struct SmtpClient {
    /// 客户端配置
    config: SmtpClientConfig,
    /// TCP 流
    stream: Option<TcpStreamType>,
    /// 连接状态
    state: SmtpConnectionState,
    /// 服务器支持的扩展
    server_extensions: Vec<String>,
}

impl SmtpClient {
    /// 创建新的 SMTP 客户端
    pub fn new(config: SmtpClientConfig) -> Self {
        Self { config, stream: None, state: SmtpConnectionState::Disconnected, server_extensions: Vec::new() }
    }

    /// 获取客户端配置
    pub fn config(&self) -> &SmtpClientConfig {
        &self.config
    }

    /// 连接到 SMTP 服务器
    pub async fn connect(&mut self) -> Result<(), SmtpError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);

        let stream = tokio::time::timeout(self.config.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| SmtpError::TimeoutError)?
            .map_err(|e| SmtpError::ConnectionError(e.to_string()))?;

        self.stream = Some(TcpStreamType::Plain(stream));
        self.state = SmtpConnectionState::Connected;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        Ok(())
    }

    /// 发送 EHLO 命令
    pub async fn ehlo(&mut self) -> Result<(), SmtpError> {
        self.send_command(&format!("EHLO {}", self.config.local_hostname)).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        self.parse_extensions(&response.message);
        self.state = SmtpConnectionState::Ready;

        Ok(())
    }

    /// 发送 HELO 命令（用于不支持 EHLO 的服务器）
    pub async fn helo(&mut self) -> Result<(), SmtpError> {
        self.send_command(&format!("HELO {}", self.config.local_hostname)).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        self.state = SmtpConnectionState::Ready;

        Ok(())
    }

    /// 升级到 TLS 连接（STARTTLS）
    pub async fn starttls(&mut self) -> Result<(), SmtpError> {
        if !self.server_extensions.iter().any(|e| e.eq_ignore_ascii_case("STARTTLS")) {
            return Err(SmtpError::ProtocolError("Server does not support STARTTLS".to_string()));
        }

        self.send_command("STARTTLS").await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        let plain_stream = match self.stream.take() {
            Some(TcpStreamType::Plain(stream)) => stream,
            Some(TcpStreamType::Tls(_)) => {
                return Err(SmtpError::ProtocolError("Connection already secured".to_string()));
            }
            None => {
                return Err(SmtpError::ConnectionError("No active connection".to_string()));
            }
        };

        let mut roots = tokio_rustls::rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = tokio_rustls::rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));

        let server_name = ServerName::try_from(self.config.host.clone())
            .map_err(|e| SmtpError::TlsError(format!("Invalid server name: {}", e)))?;

        let tls_stream = connector.connect(server_name, plain_stream).await.map_err(|e| SmtpError::TlsError(e.to_string()))?;

        self.stream = Some(TcpStreamType::Tls(Box::new(tls_stream)));
        self.state = SmtpConnectionState::Secured;

        self.ehlo().await?;

        Ok(())
    }

    /// 进行认证
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<(), SmtpError> {
        match self.config.auth_mechanism {
            AuthMechanism::Plain => self.auth_plain(username, password).await,
            AuthMechanism::Login => self.auth_login(username, password).await,
        }
    }

    /// PLAIN 认证
    async fn auth_plain(&mut self, username: &str, password: &str) -> Result<(), SmtpError> {
        let mut auth_string = Vec::new();
        auth_string.push(0);
        auth_string.extend(username.as_bytes());
        auth_string.push(0);
        auth_string.extend(password.as_bytes());

        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &auth_string);
        self.send_command(&format!("AUTH PLAIN {}", encoded)).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::AuthenticationError(format!("{} {}", response.code, response.message)));
        }

        self.state = SmtpConnectionState::Authenticated;

        Ok(())
    }

    /// LOGIN 认证
    async fn auth_login(&mut self, username: &str, password: &str) -> Result<(), SmtpError> {
        self.send_command("AUTH LOGIN").await?;

        let response = self.read_response().await?;
        if response.code != 334 {
            return Err(SmtpError::AuthenticationError(format!("Expected 334, got {} {}", response.code, response.message)));
        }

        let encoded_username = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, username);
        self.send_command(&encoded_username).await?;

        let response = self.read_response().await?;
        if response.code != 334 {
            return Err(SmtpError::AuthenticationError(format!("Expected 334, got {} {}", response.code, response.message)));
        }

        let encoded_password = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, password);
        self.send_command(&encoded_password).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::AuthenticationError(format!("{} {}", response.code, response.message)));
        }

        self.state = SmtpConnectionState::Authenticated;

        Ok(())
    }

    /// 发送 MAIL FROM 命令
    pub async fn mail_from(&mut self, from: &str) -> Result<(), SmtpError> {
        self.send_command(&format!("MAIL FROM:<{}>", from)).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        self.state = SmtpConnectionState::SendingMail;

        Ok(())
    }

    /// 发送 RCPT TO 命令
    pub async fn rcpt_to(&mut self, to: &str) -> Result<(), SmtpError> {
        self.send_command(&format!("RCPT TO:<{}>", to)).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        Ok(())
    }

    /// 发送 DATA 命令
    pub async fn data(&mut self) -> Result<(), SmtpError> {
        self.send_command("DATA").await?;

        let response = self.read_response().await?;
        if !response.is_continue() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        Ok(())
    }

    /// 发送邮件内容
    pub async fn send_data(&mut self, content: &str) -> Result<(), SmtpError> {
        let normalized_content = content.replace("\r\n.", "\r\n..");
        let data = format!("{}\r\n.\r\n", normalized_content);

        self.send_raw(&data).await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        self.state = SmtpConnectionState::Ready;

        Ok(())
    }

    /// 发送完整邮件
    pub async fn send_mail(&mut self, from: &str, to: &[&str], content: &str) -> Result<(), SmtpError> {
        self.mail_from(from).await?;

        for recipient in to {
            self.rcpt_to(recipient).await?;
        }

        self.data().await?;
        self.send_data(content).await?;

        Ok(())
    }

    /// 发送 QUIT 命令并关闭连接
    pub async fn quit(&mut self) -> Result<(), SmtpError> {
        self.send_command("QUIT").await?;

        let response = self.read_response().await;
        self.stream = None;
        self.state = SmtpConnectionState::Disconnected;

        if let Ok(resp) = response
            && !resp.is_success()
        {
            return Err(SmtpError::ResponseError { code: resp.code, message: resp.message });
        }

        Ok(())
    }

    /// 重置当前邮件会话
    pub async fn rset(&mut self) -> Result<(), SmtpError> {
        self.send_command("RSET").await?;

        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(SmtpError::ResponseError { code: response.code, message: response.message });
        }

        self.state = SmtpConnectionState::Ready;

        Ok(())
    }

    /// 验证邮箱地址
    pub async fn vrfy(&mut self, address: &str) -> Result<bool, SmtpError> {
        self.send_command(&format!("VRFY {}", address)).await?;

        let response = self.read_response().await?;
        Ok(response.is_success())
    }

    /// 获取当前连接状态
    pub fn state(&self) -> SmtpConnectionState {
        self.state
    }

    /// 获取服务器支持的扩展
    pub fn extensions(&self) -> &[String] {
        &self.server_extensions
    }

    /// 检查服务器是否支持指定扩展
    pub fn has_extension(&self, extension: &str) -> bool {
        self.server_extensions.iter().any(|e| e.eq_ignore_ascii_case(extension))
    }

    /// 发送 SMTP 命令
    async fn send_command(&mut self, command: &str) -> Result<(), SmtpError> {
        let data = format!("{}\r\n", command);
        self.send_raw(&data).await
    }

    /// 发送原始数据
    async fn send_raw(&mut self, data: &str) -> Result<(), SmtpError> {
        match &mut self.stream {
            Some(TcpStreamType::Plain(stream)) => {
                let mut writer = BufWriter::new(stream);
                writer.write_all(data.as_bytes()).await.map_err(|e| SmtpError::IoError(e.to_string()))?;
                writer.flush().await.map_err(|e| SmtpError::IoError(e.to_string()))?;
            }
            Some(TcpStreamType::Tls(stream)) => {
                let mut writer = BufWriter::new(stream);
                writer.write_all(data.as_bytes()).await.map_err(|e| SmtpError::IoError(e.to_string()))?;
                writer.flush().await.map_err(|e| SmtpError::IoError(e.to_string()))?;
            }
            None => {
                return Err(SmtpError::ConnectionError("No active connection".to_string()));
            }
        }

        Ok(())
    }

    /// 读取 SMTP 响应
    async fn read_response(&mut self) -> Result<SmtpResponse, SmtpError> {
        let mut lines = Vec::new();

        loop {
            let line = self.read_line().await?;
            let line_str = String::from_utf8_lossy(&line).to_string();

            if line_str.len() >= 4 {
                let code_str = &line_str[0..3];
                let separator = line_str.chars().nth(3);

                if let Ok(_code) = code_str.parse::<u16>() {
                    lines.push(line_str[4..].to_string());

                    if separator == Some(' ') {
                        break;
                    }
                }
                else {
                    lines.push(line_str);
                }
            }
            else {
                lines.push(line_str);
            }
        }

        if lines.is_empty() {
            return Err(SmtpError::ProtocolError("Empty response".to_string()));
        }

        let first_line = &lines[0];
        let code_str = first_line.get(0..3).unwrap_or("000");
        let code = code_str.parse::<u16>().unwrap_or(0);

        let message = lines.join("\n");

        Ok(SmtpResponse { code, message })
    }

    /// 读取一行数据
    async fn read_line(&mut self) -> Result<Vec<u8>, SmtpError> {
        match &mut self.stream {
            Some(TcpStreamType::Plain(stream)) => {
                let mut reader = BufReader::new(stream);
                let mut line = Vec::new();
                reader.read_until(b'\n', &mut line).await.map_err(|e| SmtpError::IoError(e.to_string()))?;
                Ok(line)
            }
            Some(TcpStreamType::Tls(stream)) => {
                let mut reader = BufReader::new(stream);
                let mut line = Vec::new();
                reader.read_until(b'\n', &mut line).await.map_err(|e| SmtpError::IoError(e.to_string()))?;
                Ok(line)
            }
            None => Err(SmtpError::ConnectionError("No active connection".to_string())),
        }
    }

    /// 解析服务器扩展
    fn parse_extensions(&mut self, message: &str) {
        self.server_extensions.clear();

        for line in message.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let extension = line.split_whitespace().next().unwrap_or("");
                if !extension.is_empty() {
                    self.server_extensions.push(extension.to_uppercase());
                }
            }
        }
    }
}

impl Drop for SmtpClient {
    fn drop(&mut self) {
        if self.stream.is_some() {
            futures::executor::block_on(async {
                let mut client = SmtpClient {
                    config: self.config.clone(),
                    stream: self.stream.take(),
                    state: self.state,
                    server_extensions: std::mem::take(&mut self.server_extensions),
                };
                let _ = client.quit().await;
            });
        }
    }
}

/// SMTP 客户端构建器
pub struct SmtpClientBuilder {
    config: SmtpClientConfig,
}

impl SmtpClientBuilder {
    /// 创建新的构建器
    pub fn new(host: &str, port: u16) -> Self {
        let config = SmtpClientConfig { host: host.to_string(), port, ..Default::default() };
        Self { config }
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// 设置本地主机名
    pub fn local_hostname(mut self, hostname: &str) -> Self {
        self.config.local_hostname = hostname.to_string();
        self
    }

    /// 设置是否使用 STARTTLS
    pub fn use_starttls(mut self, use_starttls: bool) -> Self {
        self.config.use_starttls = use_starttls;
        self
    }

    /// 设置认证机制
    pub fn auth_mechanism(mut self, mechanism: AuthMechanism) -> Self {
        self.config.auth_mechanism = mechanism;
        self
    }

    /// 构建 SMTP 客户端
    pub fn build(self) -> SmtpClient {
        SmtpClient::new(self.config)
    }
}
