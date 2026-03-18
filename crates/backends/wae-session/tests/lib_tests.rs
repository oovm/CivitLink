use wae_session::SessionConfig;

#[test]
fn test_session_config() {
    let config = SessionConfig::default();
    assert!(!config.cookie_name.is_empty());
}
