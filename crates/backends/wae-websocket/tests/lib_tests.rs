use wae_websocket::{ClientConfig, Message, ServerConfig};

#[test]
fn test_message_text() {
    let msg = Message::text("hello world");
    assert!(msg.is_text());
}

#[test]
fn test_message_binary() {
    let data = vec![1, 2, 3, 4];
    let msg = Message::binary(data.clone());
    assert!(msg.is_binary());
}

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
}

#[test]
fn test_client_config_default() {
    let config = ClientConfig::default();
    assert_eq!(config.url, "ws://127.0.0.1:8080");
}
