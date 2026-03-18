use serde::{Deserialize, Serialize};
use wae_event::{BaseEvent, EventBusConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestPayload {
    message: String,
}

#[test]
fn test_event_bus_config_default() {
    let config = EventBusConfig::default();
    assert_eq!(config.queue_capacity, 1000);
}

#[test]
fn test_base_event() {
    let payload = TestPayload { message: "test".to_string() };
    let event = BaseEvent::new("test.event", payload);
    assert!(!event.id.is_empty());
}
