# WAE Email - 邮件服务抽象层

提供统一的邮件发送服务抽象，支持 SMTP、Sendmail 和直接交付。

深度融合 tokio 运行时，所有邮件发送操作都是异步的。

## 模块结构

- [`mime`] - MIME 消息构建，支持纯文本、HTML、附件
- [`smtp`] - SMTP 协议客户端，支持 STARTTLS 和认证
- [`sendmail`] - Sendmail 传输，通过本地命令发送邮件

## 示例

```rust,no_run
use wae_email::{SmtpEmailProvider, SmtpConfig, EmailProvider};

#[tokio::main]
async fn main() {
    let config = SmtpConfig {
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        password: "password".to_string(),
        from_email: "sender@example.com".to_string(),
    };

    let provider = SmtpEmailProvider::new(config);
    provider.send_email("recipient@example.com", "Subject", "Body").await.unwrap();
}
```

---

邮件模块 - 提供邮件发送功能。

## 主要功能

- **SMTP 支持**: 标准 SMTP 协议邮件发送
- **模板渲染**: 支持邮件模板
- **附件支持**: 发送带附件的邮件
- **异步发送**: 非阻塞邮件发送

## 技术栈

- **邮件发送**: lettre
- **异步运行时**: Tokio

## 使用示例

```rust,no_run
use wae_email::{SmtpEmailProvider, SmtpConfig, EmailProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SmtpConfig {
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        password: "password".to_string(),
        from_email: "sender@example.com".to_string(),
    };

    let provider = SmtpEmailProvider::new(config);
    provider.send_email("recipient@example.com", "Subject", "Body").await?;
    Ok(())
}
```

## 模板支持

```rust,no_run
use wae_email::mime::EmailBuilder;

let email = EmailBuilder::new()
    .to("user@example.com")
    .subject("验证码")
    .body("验证码: 123456")
    .build();
```
