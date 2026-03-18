use serde::{Deserialize, Serialize};
use std::{env, fs};
use wae_config::*;
use wae_types::WaeResult;

fn set_env_var(key: &str, value: &str) {
    unsafe {
        env::set_var(key, value);
    }
}

fn remove_env_var(key: &str) {
    unsafe {
        env::remove_var(key);
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    port: u16,
    debug: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NestedConfig {
    database: DbConfig,
    server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DbConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ServerConfig {
    address: String,
    port: u16,
}

#[test]
fn test_config_loader_new() {
    let loader = ConfigLoader::new();
    let config: Result<TestConfig, _> = loader.extract();
    assert!(config.is_err());
}

#[test]
fn test_config_loader_default() {
    let loader = ConfigLoader::default();
    let config: Result<TestConfig, _> = loader.extract();
    assert!(config.is_err());
}

#[test]
fn test_with_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let toml_path = temp_dir.path().join("config.toml");

    let toml_content = r#"
name = "test-app"
port = 9090
debug = true
"#;

    fs::write(&toml_path, toml_content).unwrap();

    let config: TestConfig = ConfigLoader::new().with_toml(toml_path.to_str().unwrap()).extract().unwrap();

    assert_eq!(config.name, "test-app");
    assert_eq!(config.port, 9090);
    assert!(config.debug);
}

#[test]
fn test_with_yaml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let yaml_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
name: "test-yaml"
port: 8888
debug: false
"#;

    fs::write(&yaml_path, yaml_content).unwrap();

    let config: TestConfig = ConfigLoader::new().with_yaml(yaml_path.to_str().unwrap()).extract().unwrap();

    assert_eq!(config.name, "test-yaml");
    assert_eq!(config.port, 8888);
    assert!(!config.debug);
}

#[test]
fn test_with_env() {
    set_env_var("TEST_NAME", "env-app");
    set_env_var("TEST_PORT", "1234");
    set_env_var("TEST_DEBUG", "true");

    let config: TestConfig = ConfigLoader::new().with_env("TEST_").extract().unwrap();

    assert_eq!(config.name, "env-app");
    assert_eq!(config.port, 1234);
    assert!(config.debug);

    remove_env_var("TEST_NAME");
    remove_env_var("TEST_PORT");
    remove_env_var("TEST_DEBUG");
}

#[test]
fn test_with_env_separator() {
    set_env_var("APP_DATABASE_HOST", "localhost");
    set_env_var("APP_DATABASE_PORT", "5432");
    set_env_var("APP_SERVER_ADDRESS", "0.0.0.0");
    set_env_var("APP_SERVER_PORT", "8080");

    let config: NestedConfig = ConfigLoader::new().with_env_separator("APP_", "_").extract().unwrap();

    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.server.address, "0.0.0.0");
    assert_eq!(config.server.port, 8080);

    remove_env_var("APP_DATABASE_HOST");
    remove_env_var("APP_DATABASE_PORT");
    remove_env_var("APP_SERVER_ADDRESS");
    remove_env_var("APP_SERVER_PORT");
}

#[test]
fn test_with_defaults() {
    let defaults = TestConfig { name: "default".to_string(), port: 8080, debug: false };

    let config: TestConfig = ConfigLoader::new().with_defaults(&defaults).extract().unwrap();

    assert_eq!(config, defaults);
}

#[test]
fn test_extract_missing_field() {
    let loader = ConfigLoader::new();
    let result: WaeResult<TestConfig> = loader.extract();
    assert!(result.is_err());
}

#[test]
fn test_extract_with_context() {
    let defaults = TestConfig { name: "context-test".to_string(), port: 9999, debug: true };

    let config: TestConfig = ConfigLoader::new().with_defaults(&defaults).extract_with_context("Loading test config").unwrap();

    assert_eq!(config, defaults);
}

#[test]
fn test_extract_with_context_error() {
    let loader = ConfigLoader::new();
    let result: WaeResult<TestConfig> = loader.extract_with_context("Failed to load config");
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(err_msg.contains("Failed to load config"));
}

#[test]
fn test_load_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let toml_path = temp_dir.path().join("app.toml");

    let toml_content = r#"
name = "load-config-test"
port = 7777
debug = false
"#;

    fs::write(&toml_path, toml_content).unwrap();

    let config: TestConfig = load_config(toml_path.to_str().unwrap(), "LOAD_TEST_").unwrap();

    assert_eq!(config.name, "load-config-test");
    assert_eq!(config.port, 7777);
    assert!(!config.debug);
}

#[test]
fn test_from_env() {
    set_env_var("FROMENV_NAME", "from-env");
    set_env_var("FROMENV_PORT", "3000");
    set_env_var("FROMENV_DEBUG", "false");

    let config: TestConfig = from_env("FROMENV_").unwrap();

    assert_eq!(config.name, "from-env");
    assert_eq!(config.port, 3000);
    assert!(!config.debug);

    remove_env_var("FROMENV_NAME");
    remove_env_var("FROMENV_PORT");
    remove_env_var("FROMENV_DEBUG");
}

#[test]
fn test_config_priority() {
    let temp_dir = tempfile::tempdir().unwrap();
    let toml_path = temp_dir.path().join("priority.toml");

    let toml_content = r#"
name = "toml-name"
port = 1000
debug = false
"#;

    fs::write(&toml_path, toml_content).unwrap();

    set_env_var("PRIORITY_NAME", "env-name");
    set_env_var("PRIORITY_PORT", "2000");

    let defaults = TestConfig { name: "default-name".to_string(), port: 3000, debug: true };

    let config: TestConfig = ConfigLoader::new()
        .with_defaults(&defaults)
        .with_toml(toml_path.to_str().unwrap())
        .with_env("PRIORITY_")
        .extract()
        .unwrap();

    assert_eq!(config.name, "env-name");
    assert_eq!(config.port, 2000);
    assert!(!config.debug);

    remove_env_var("PRIORITY_NAME");
    remove_env_var("PRIORITY_PORT");
}
