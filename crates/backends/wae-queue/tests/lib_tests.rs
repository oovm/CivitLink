use wae_queue::{ConsumerConfig, ProducerConfig, QueueConfig};

#[test]
fn test_queue_config() {
    let config = QueueConfig::new("test-queue");
    assert_eq!(config.name, "test-queue");
}

#[test]
fn test_producer_config_default() {
    let config = ProducerConfig::default();
    assert!(config.default_queue.is_none());
}

#[test]
fn test_consumer_config() {
    let config = ConsumerConfig::new("test-queue");
    assert_eq!(config.queue, "test-queue");
}
