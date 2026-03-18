# wae-email

邮件服务抽象层，提供统一的邮件发送服务抽象，支持 SMTP、Sendmail 和直接交付。

## 概述

深度融合 tokio 运行时，所有邮件发送操作都是异步的。支持多种邮件传输方式，提供完整的 MIME 消息构建能力。

## 快速开始

### SMTP 发送

```rust
use wae::email::{SmtpEmailProvider, EmailProvider, SmtpConfig};

let config = SmtpConfig {
    host: "smtp.example.com".to_string(),
    port: 587,
    username: "user@example.com".to_string(),
    password: "your-password".to_string(),
    from_email: "noreply@example.com".to_string(),
};

let provider = SmtpEmailProvider::new(config);

provider.send_email(
    "recipient@example.com",
    "Test Subject",
    "Hello, World!"
).await?;
```

### Sendmail 发送

```rust
use wae::email::{SendmailEmailProvider, EmailProvider};

let provider = SendmailEmailProvider::new("noreply@example.com".to_string());

provider.send_email(
    "recipient@example.com",
    "Test Subject",
    "Hello, World!"
).await?;
```

## 核心用法

### EmailProvider Trait

所有邮件服务商实现此 trait：

```rust
pub trait EmailProvider: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> CloudResult<()>;
}
```

### SMTP 服务商

#### 基础配置

```rust
use wae::email::{SmtpEmailProvider, EmailProvider, SmtpConfig};

let config = SmtpConfig {
    host: "smtp.gmail.com".to_string(),
    port: 587,
    username: "your-email@gmail.com".to_string(),
    password: "your-app-password".to_string(),
    from_email: "your-email@gmail.com".to_string(),
};

let provider = SmtpEmailProvider::new(config);
```

#### 发送邮件

```rust
provider.send_email(
    "recipient@example.com",
    "Welcome!",
    "Thank you for registering."
).await?;
```

### Sendmail 服务商

使用服务器本地的 sendmail 命令：

```rust
use wae::email::{SendmailEmailProvider, EmailProvider};

let provider = SendmailEmailProvider::new("noreply@example.com".to_string());

provider.send_email(
    "recipient@example.com",
    "Test Subject",
    "Hello, World!"
).await?;
```

自定义 sendmail 路径：

```rust
use wae::email::{SendmailEmailProvider, SendmailConfig};

let config = SendmailConfig {
    command: "/usr/sbin/sendmail".to_string(),
};

let provider = SendmailEmailProvider::with_config(
    "noreply@example.com".to_string(),
    config
);
```

### 直接交付

让 Rust 程序直接充当 MTA：

```rust
use wae::email::{DirectEmailProvider, EmailProvider};

let provider = DirectEmailProvider::new("noreply@example.com".to_string());

provider.send_email(
    "recipient@example.com",
    "Test Subject",
    "Hello, World!"
).await?;
```

这种方式会：
1. 查询收件人域名的 MX 记录
2. 直接连接到对方的邮件服务器发送

> **注意**: 直接交付方式通常不需要身份验证，但需要服务器有正确的 DNS 配置和反向解析。

## MIME 消息构建

### EmailBuilder

链式构建复杂邮件：

```rust
use wae::email::mime::{EmailBuilder, Attachment};

let email = EmailBuilder::new()
    .from("sender@example.com")
    .to("recipient@example.com")
    .cc("cc@example.com")
    .bcc("bcc@example.com")
    .subject("Test Subject")
    .body("Plain text body")
    .html_body("<html><body>HTML body</body></html>")
    .attachment(Attachment::new("file.txt", "text/plain", b"content".to_vec()))
    .message_id("unique-id@example.com")
    .reply_to("reply@example.com")
    .header("X-Custom-Header", "value")
    .build();

let raw_email = email.to_string();
let raw_bytes = email.to_bytes();
```

### 附件

```rust
use wae::email::mime::Attachment;

let attachment = Attachment::new(
    "document.pdf",
    "application/pdf",
    std::fs::read("document.pdf")?
);

let text_attachment = Attachment::from_text("readme.txt", "Hello, World!");
```

### 辅助函数

```rust
use wae::email::mime::{encode_subject, generate_date};

let encoded = encode_subject("中文主题");
let date = generate_date();
```

## SMTP 客户端

### SmtpClientBuilder

```rust
use wae::email::smtp::{SmtpClient, SmtpClientBuilder, AuthMechanism};
use std::time::Duration;

let mut client = SmtpClientBuilder::new("smtp.example.com", 587)
    .timeout(Duration::from_secs(30))
    .local_hostname("myhost.example.com")
    .use_starttls(true)
    .auth_mechanism(AuthMechanism::Login)
    .build();
```

### 完整 SMTP 会话

```rust
client.connect().await?;
client.ehlo().await?;
client.starttls().await?;
client.authenticate("username", "password").await?;
client.send_mail("from@example.com", &["to@example.com"], "email content").await?;
client.quit().await?;
```

### SmtpClient 方法

| 方法 | 说明 |
|------|------|
| `connect()` | 连接到 SMTP 服务器 |
| `ehlo()` | 发送 EHLO 命令 |
| `helo()` | 发送 HELO 命令 |
| `starttls()` | 升级到 TLS 连接 |
| `authenticate()` | 进行认证 |
| `mail_from()` | 发送 MAIL FROM 命令 |
| `rcpt_to()` | 发送 RCPT TO 命令 |
| `data()` | 发送 DATA 命令 |
| `send_data()` | 发送邮件内容 |
| `send_mail()` | 发送完整邮件 |
| `quit()` | 发送 QUIT 命令 |
| `rset()` | 重置当前邮件会话 |
| `vrfy()` | 验证邮箱地址 |
| `state()` | 获取当前连接状态 |
| `extensions()` | 获取服务器支持的扩展 |

### 认证机制

```rust
use wae::email::smtp::AuthMechanism;

let client = SmtpClientBuilder::new("smtp.example.com", 587)
    .auth_mechanism(AuthMechanism::Plain)
    .build();

let client = SmtpClientBuilder::new("smtp.example.com", 587)
    .auth_mechanism(AuthMechanism::Login)
    .build();
```

## 最佳实践

### HTML + 纯文本邮件

```rust
let email = EmailBuilder::new()
    .from("noreply@example.com")
    .to("user@example.com")
    .subject("Newsletter")
    .body("This is the plain text version.")
    .html_body("<html><body><h1>Newsletter</h1><p>This is the HTML version.</p></body></html>")
    .build();
```

### 带附件的邮件

```rust
let email = EmailBuilder::new()
    .from("sender@example.com")
    .to("recipient@example.com")
    .subject("Document Attached")
    .body("Please find the attached document.")
    .attachment(Attachment::new(
        "report.pdf",
        "application/pdf",
        pdf_bytes
    ))
    .build();
```

### 批量发送

```rust
async fn send_newsletter(
    provider: &SmtpEmailProvider,
    recipients: &[String],
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for recipient in recipients {
        provider.send_email(recipient, subject, body).await?;
    }
    Ok(())
}
```

### 错误处理

```rust
use wae::email::smtp::SmtpError;

match provider.send_email(to, subject, body).await {
    Ok(()) => println!("Email sent successfully"),
    Err(SmtpError::AuthenticationError(msg)) => {
        eprintln!("Authentication failed: {}", msg);
    }
    Err(SmtpError::ConnectionError(msg)) => {
        eprintln!("Connection failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Failed to send email: {:?}", e);
    }
}
```

## 常见问题

### Q: 如何配置 Gmail SMTP？

```rust
let config = SmtpConfig {
    host: "smtp.gmail.com".to_string(),
    port: 587,
    username: "your-email@gmail.com".to_string(),
    password: "your-app-password".to_string(),
    from_email: "your-email@gmail.com".to_string(),
};
```

> **注意**: Gmail 需要使用应用专用密码，而不是账户密码。

### Q: 如何发送中文主题？

使用 `encode_subject` 函数：

```rust
use wae::email::mime::encode_subject;

let subject = encode_subject("中文邮件主题");
```

### Q: 如何检测邮件发送状态？

检查 SMTP 响应：

```rust
let response = client.mail_from("from@example.com").await?;

if response.is_success() {
    println!("MAIL FROM accepted");
}
```

### Q: 直接交付有什么限制？

直接交付需要：
- 服务器有正确的 DNS 反向解析 (PTR 记录)
- 服务器 IP 不在黑名单中
- 正确配置 SPF 和 DKIM 记录

建议在生产环境使用 SMTP 服务商。

### Q: 如何处理发送超时？

```rust
let client = SmtpClientBuilder::new("smtp.example.com", 587)
    .timeout(Duration::from_secs(60))
    .build();
```

### Q: 如何发送多收件人邮件？

```rust
client.send_mail(
    "from@example.com",
    &["to1@example.com", "to2@example.com"],
    "email content"
).await?;
```
