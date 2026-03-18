use base64::{Engine, engine::general_purpose::STANDARD};
use wae_email::mime::*;

#[test]
fn test_encode_subject_ascii() {
    let subject = "Hello World";
    assert_eq!(encode_subject(subject), "Hello World");
}

#[test]
fn test_encode_subject_utf8() {
    let subject = "你好世界";
    let encoded = encode_subject(subject);
    assert!(encoded.starts_with("=?UTF-8?B?"));
    assert!(encoded.ends_with("?="));
}

#[test]
fn test_simple_email() {
    let email = EmailBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test Subject")
        .body("Hello, this is a test email.")
        .build();

    let content = email.to_string();
    assert!(content.contains("From: sender@example.com"));
    assert!(content.contains("To: recipient@example.com"));
    assert!(content.contains("Subject: Test Subject"));
    assert!(content.contains("Hello, this is a test email."));
}

#[test]
fn test_html_email() {
    let email = EmailBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("HTML Email")
        .body("Plain text content")
        .html_body("<html><body><h1>HTML content</h1></body></html>")
        .build();

    let content = email.to_string();
    assert!(content.contains("multipart/alternative"));
    assert!(content.contains("Plain text content"));
    assert!(content.contains("<html>"));
}

#[test]
fn test_attachment() {
    let attachment = Attachment::new("test.txt", "text/plain", b"Hello attachment".to_vec());

    let email = EmailBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Email with attachment")
        .body("Please see attachment.")
        .attachment(attachment)
        .build();

    let content = email.to_string();
    assert!(content.contains("multipart/mixed"));
    assert!(content.contains("Content-Disposition: attachment"));
}

#[test]
fn test_utf8_subject() {
    let email = EmailBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("测试邮件主题")
        .body("测试内容")
        .build();

    let content = email.to_string();
    assert!(content.contains("=?UTF-8?B?"));
}

#[test]
fn test_base64_encoding() {
    let input = "Hello, World!";
    let encoded = STANDARD.encode(input.as_bytes());
    assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
}
