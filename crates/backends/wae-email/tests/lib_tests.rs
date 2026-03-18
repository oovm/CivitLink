use wae_email::*;

#[test]
fn test_smtp_config() {
    let config = SmtpConfig {
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        password: "password".to_string(),
        from_email: "sender@example.com".to_string(),
    };

    assert_eq!(config.host, "smtp.example.com");
    assert_eq!(config.port, 587);
}

#[test]
fn test_build_email() {
    let provider = SmtpEmailProvider::new(SmtpConfig {
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_email: "sender@example.com".to_string(),
    });

    let email = provider.build_email("recipient@example.com", "Test Subject", "Test Body");
    let content = email.to_string();

    assert!(content.contains("From: sender@example.com"));
    assert!(content.contains("To: recipient@example.com"));
    assert!(content.contains("Subject: Test Subject"));
    assert!(content.contains("Test Body"));
}

#[test]
fn test_sendmail_build_email() {
    let provider = SendmailEmailProvider::new("sender@example.com".to_string());
    let email = provider.build_email("recipient@example.com", "Test", "Body");
    let content = email.to_string();

    assert!(content.contains("From: sender@example.com"));
    assert!(content.contains("To: recipient@example.com"));
}

#[test]
fn test_direct_build_email() {
    let provider = DirectEmailProvider::new("sender@example.com".to_string());
    let email = provider.build_email("recipient@example.com", "Test", "Body");
    let content = email.to_string();

    assert!(content.contains("From: sender@example.com"));
    assert!(content.contains("To: recipient@example.com"));
}
