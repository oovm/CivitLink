use wae_crypto::{PasswordAlgorithm, PasswordHasher, PasswordHasherConfig};

#[test]
fn test_password_hasher_default() {
    let hasher = PasswordHasher::default();
    let password = "my_secure_password";

    let hash = hasher.hash_password(password).unwrap();
    assert!(!hash.is_empty());

    let result = hasher.verify_password(password, &hash).unwrap();
    assert!(result);
}

#[test]
fn test_password_hasher_bcrypt() {
    let config =
        PasswordHasherConfig { algorithm: PasswordAlgorithm::Bcrypt, bcrypt_cost: 4, ..PasswordHasherConfig::default() };
    let hasher = PasswordHasher::new(config);
    let password = "test_password_123";

    let hash = hasher.hash_password(password).unwrap();
    assert!(hash.starts_with("$2b$"));

    let result = hasher.verify_password(password, &hash).unwrap();
    assert!(result);
}

#[test]
fn test_password_hasher_argon2() {
    let config = PasswordHasherConfig {
        algorithm: PasswordAlgorithm::Argon2,
        argon2_memory_cost: 8192,
        argon2_time_cost: 1,
        argon2_parallelism: 1,
        ..PasswordHasherConfig::default()
    };
    let hasher = PasswordHasher::new(config);
    let password = "argon2_test_password";

    let hash = hasher.hash_password(password).unwrap();
    assert!(hash.starts_with("$argon2id$"));

    let result = hasher.verify_password(password, &hash).unwrap();
    assert!(result);
}

#[test]
fn test_password_wrong_password() {
    let hasher = PasswordHasher::default();
    let password = "correct_password";
    let wrong_password = "wrong_password";

    let hash = hasher.hash_password(password).unwrap();
    let result = hasher.verify_password(wrong_password, &hash).unwrap();
    assert!(!result);
}

#[test]
fn test_password_empty_password() {
    let hasher = PasswordHasher::default();
    let password = "";

    let hash = hasher.hash_password(password).unwrap();
    let result = hasher.verify_password(password, &hash).unwrap();
    assert!(result);
}

#[test]
fn test_password_long_password() {
    let hasher = PasswordHasher::default();
    let password = "a".repeat(1000);

    let hash = hasher.hash_password(&password).unwrap();
    let result = hasher.verify_password(&password, &hash).unwrap();
    assert!(result);
}

#[test]
fn test_password_config_default() {
    let config = PasswordHasherConfig::default();
    assert_eq!(config.algorithm, PasswordAlgorithm::Bcrypt);
    assert_eq!(config.bcrypt_cost, 12);
}

#[test]
fn test_password_hasher_new() {
    let config = PasswordHasherConfig::default();
    let hasher = PasswordHasher::new(config);
    let hash = hasher.hash_password("test").unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_password_algorithm_clone_copy() {
    let alg = PasswordAlgorithm::Argon2;
    let alg2 = alg;
    assert_eq!(alg, alg2);

    let alg3 = alg.clone();
    assert_eq!(alg, alg3);
}

#[test]
fn test_password_algorithm_debug() {
    let alg = PasswordAlgorithm::Bcrypt;
    let debug_str = format!("{:?}", alg);
    assert!(debug_str.contains("Bcrypt"));
}

#[test]
fn test_password_hasher_clone() {
    let hasher = PasswordHasher::default();
    let hasher2 = hasher.clone();

    let password = "test_clone";
    let hash = hasher.hash_password(password).unwrap();
    let result = hasher2.verify_password(password, &hash).unwrap();
    assert!(result);
}

#[test]
fn test_password_hasher_config_clone() {
    let config = PasswordHasherConfig::default();
    let config2 = config.clone();
    assert_eq!(config.algorithm, config2.algorithm);
}
