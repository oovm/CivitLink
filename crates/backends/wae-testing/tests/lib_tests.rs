use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use wae_testing::*;

#[test]
fn test_random_string() {
    let generator = RandomString::new().length(16).prefix("test_");
    let s = generator.generate().unwrap();
    assert!(s.starts_with("test_"));
    assert_eq!(s.len(), 21);
}

#[test]
fn test_random_email() {
    let generator = RandomEmail::new().domain("example.org");
    let email = generator.generate().unwrap();
    assert!(email.ends_with("@example.org"));
}

#[test]
fn test_random_uuid() {
    let generator = RandomUuid::new();
    let uuid = generator.generate().unwrap();
    assert_eq!(uuid.get_version_num(), 4);
}

#[test]
fn test_random_number() {
    let generator = RandomNumber::new(1i32, 100i32);
    let n = generator.generate().unwrap();
    assert!(n >= 1 && n <= 100);
}

#[tokio::test]
async fn test_mock_builder() {
    let mock = MockBuilder::<i32>::new().return_value(42).expect(MockExpectation::new().times(1)).build();

    let result = mock.call(vec![]).unwrap();
    assert_eq!(result, 42);
    mock.verify().unwrap();
}

#[tokio::test]
async fn test_mock_sequence() {
    let mock = MockBuilder::<i32>::new().sequence(vec![1, 2, 3]).build();

    assert_eq!(mock.call(vec![]).unwrap(), 1);
    assert_eq!(mock.call(vec![]).unwrap(), 2);
    assert_eq!(mock.call(vec![]).unwrap(), 3);
}

#[tokio::test]
async fn test_test_env() {
    let env = TestEnv::default_env();
    assert_eq!(env.state(), TestEnvState::Uninitialized);

    env.setup().unwrap();
    assert_eq!(env.state(), TestEnvState::Initialized);

    env.teardown().unwrap();
    assert_eq!(env.state(), TestEnvState::Destroyed);
}

#[tokio::test]
async fn test_assert_eventually() {
    let counter = Arc::new(RwLock::new(0));
    let counter_clone = counter.clone();

    let condition = move || {
        let counter = counter_clone.clone();
        async move {
            let mut c = counter.write().await;
            *c += 1;
            *c >= 3
        }
    };

    let result = assert_eventually(condition, Duration::from_secs(2), Duration::from_millis(50)).await;

    assert!(result.is_ok());
}
