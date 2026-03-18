use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use wae_distributed::*;

#[tokio::test]
async fn test_in_memory_lock() {
    let manager = Arc::new(InMemoryLockManager::new());
    let lock = manager.create_lock("test-lock");

    assert!(!lock.is_locked().await);
    assert!(lock.try_lock().await.unwrap());
    assert!(lock.is_locked().await);
    assert!(!lock.try_lock().await.unwrap());
    lock.unlock().await.unwrap();
    assert!(!lock.is_locked().await);
}

#[tokio::test]
async fn test_feature_flag() {
    let manager = FeatureFlagManager::new();

    let flag = FlagDefinition::new("test-feature")
        .with_description("Test feature flag")
        .with_strategy(Strategy::On)
        .with_enabled(true);

    manager.register(flag);

    assert!(manager.is_enabled("test-feature").await);

    let flag_off = FlagDefinition::new("test-off").with_strategy(Strategy::Off).with_enabled(true);

    manager.register(flag_off);
    assert!(!manager.is_enabled("test-off").await);
}

#[tokio::test]
async fn test_percentage_strategy() {
    let strategy = Strategy::Percentage(50);

    let mut enabled_count = 0;
    for i in 0..100 {
        if evaluate(&strategy, &format!("user-{}", i)) {
            enabled_count += 1;
        }
    }

    assert!(enabled_count > 30 && enabled_count < 70);
}

#[tokio::test]
async fn test_user_list_strategy() {
    let strategy = Strategy::UserList(vec!["user-1".to_string(), "user-2".to_string()]);

    assert!(evaluate(&strategy, "user-1"));
    assert!(evaluate(&strategy, "user-2"));
    assert!(!evaluate(&strategy, "user-3"));
}

#[tokio::test]
async fn test_snowflake_generator() {
    let generator = SnowflakeGenerator::new(1, 1).unwrap();

    let id1 = generator.generate().await;
    let id2 = generator.generate().await;

    assert_ne!(id1, id2);

    let ids = generator.generate_batch(10).await;
    assert_eq!(ids.len(), 10);

    let unique_ids: std::collections::HashSet<_> = ids.into_iter().collect();
    assert_eq!(unique_ids.len(), 10);
}

#[tokio::test]
async fn test_uuid_generator() {
    let gen_v4 = UuidGenerator::v4();
    let id1 = gen_v4.generate().await;
    let id2 = gen_v4.generate().await;

    assert_ne!(id1, id2);
    assert_eq!(id1.len(), 36);

    let gen_v7 = UuidGenerator::v7();
    let id3 = gen_v7.generate().await;
    let id4 = gen_v7.generate().await;

    assert_ne!(id3, id4);
    assert_eq!(id3.len(), 36);
    assert!(id3.starts_with(|c: char| c.is_ascii_hexdigit()));
}
