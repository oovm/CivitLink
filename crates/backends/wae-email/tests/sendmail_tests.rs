use wae_email::sendmail::*;

#[test]
fn test_build_raw_email() {
    let provider = SendmailEmailProvider::new("sender@example.com".to_string());
    let raw = provider.build_raw_email("recipient@example.com", "Test Subject", "Test Body");

    assert!(raw.contains("From: sender@example.com"));
    assert!(raw.contains("To: recipient@example.com"));
    assert!(raw.contains("Subject: Test Subject"));
    assert!(raw.contains("Test Body"));
}

#[test]
fn test_default_config() {
    let config = SendmailConfig::default();
    assert_eq!(config.command, "sendmail");
}
