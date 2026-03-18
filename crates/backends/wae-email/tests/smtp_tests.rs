use std::time::Duration;
use wae_email::smtp::*;

#[test]
fn test_smtp_response_category() {
    let response = SmtpResponse { code: 220, message: "OK".to_string() };
    assert_eq!(response.category(), SmtpResponseCategory::Success);
    assert!(response.is_success());

    let response = SmtpResponse { code: 354, message: "Start mail input".to_string() };
    assert_eq!(response.category(), SmtpResponseCategory::Continue);
    assert!(response.is_continue());

    let response = SmtpResponse { code: 450, message: "Mailbox busy".to_string() };
    assert_eq!(response.category(), SmtpResponseCategory::TransientFailure);

    let response = SmtpResponse { code: 550, message: "Mailbox unavailable".to_string() };
    assert_eq!(response.category(), SmtpResponseCategory::PermanentFailure);
}

#[test]
fn test_smtp_client_builder() {
    let client = SmtpClientBuilder::new("smtp.example.com", 587)
        .timeout(Duration::from_secs(60))
        .local_hostname("myhost.local")
        .use_starttls(true)
        .auth_mechanism(AuthMechanism::Login)
        .build();

    let config = client.config();
    assert_eq!(config.host, "smtp.example.com");
    assert_eq!(config.port, 587);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.local_hostname, "myhost.local");
    assert!(config.use_starttls);
    assert_eq!(config.auth_mechanism, AuthMechanism::Login);
}
