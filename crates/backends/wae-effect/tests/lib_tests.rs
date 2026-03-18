use wae_effect::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct TestConfig {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestAuthService;

#[test]
fn test_dependencies() {
    let mut deps = Dependencies::new();
    deps.register("test", 42i32);
    assert_eq!(deps.get::<i32>("test").unwrap(), 42);
}

#[test]
fn test_algebraic_effect() {
    let deps = AlgebraicEffect::new().with("config", TestConfig { name: "test".to_string() }).build();

    let config: TestConfig = deps.get("config").unwrap();
    assert_eq!(config.name, "test");
}

#[test]
fn test_algebraic_effect_simple() {
    let deps = AlgebraicEffect::new().with("answer", 42i32).build();
    assert_eq!(deps.get::<i32>("answer").unwrap(), 42);
}

#[test]
fn test_algebraic_effect_with_type() {
    let deps = AlgebraicEffect::new().with_type(TestConfig { name: "test".to_string() }).build();

    let config: TestConfig = deps.get_type().unwrap();
    assert_eq!(config.name, "test");
}

#[test]
fn test_algebraic_effect_with_config() {
    let deps = AlgebraicEffect::new().with_config(TestConfig { name: "test_config".to_string() }).build();

    let config: TestConfig = deps.get_type().unwrap();
    assert_eq!(config.name, "test_config");
}

#[test]
fn test_algebraic_effect_with_auth() {
    let deps = AlgebraicEffect::new().with_auth(TestAuthService).build();

    let auth: TestAuthService = deps.get_type().unwrap();
    assert_eq!(auth, TestAuthService);
}

#[test]
fn test_effectful_use_type() {
    let deps = AlgebraicEffect::new().with_type(TestConfig { name: "test".to_string() }).build();

    let (parts, _) = http::Request::new(()).into_parts();
    let effectful = Effectful::new(deps, parts);

    let config: TestConfig = effectful.use_type().unwrap();
    assert_eq!(config.name, "test");
}

#[test]
fn test_effectful_use_config() {
    let deps = AlgebraicEffect::new().with_config(TestConfig { name: "use_config_test".to_string() }).build();

    let (parts, _) = http::Request::new(()).into_parts();
    let effectful = Effectful::new(deps, parts);

    let config: TestConfig = effectful.use_config().unwrap();
    assert_eq!(config.name, "use_config_test");
}

#[test]
fn test_effectful_use_auth() {
    let deps = AlgebraicEffect::new().with_auth(TestAuthService).build();

    let (parts, _) = http::Request::new(()).into_parts();
    let effectful = Effectful::new(deps, parts);

    let auth: TestAuthService = effectful.use_auth().unwrap();
    assert_eq!(auth, TestAuthService);
}

#[test]
fn test_scope_singleton() {
    let mut deps = Dependencies::new();
    deps.register_type_with_scope(42i32, Scope::Singleton);
    assert_eq!(deps.get_type_scope::<i32>(), Some(Scope::Singleton));
}

#[test]
fn test_scope_request_scoped() {
    let deps = AlgebraicEffect::new().with_type_scope(42i32, Scope::RequestScoped).build();

    assert_eq!(deps.get_type_scope::<i32>(), Some(Scope::RequestScoped));
}

#[test]
fn test_effectful_set_request_scoped() {
    let deps = AlgebraicEffect::new().with_type_scope(42i32, Scope::RequestScoped).build();

    let (parts, _) = http::Request::new(()).into_parts();
    let mut effectful = Effectful::new(deps, parts);

    effectful.set_type(100i32).unwrap();
    let value: i32 = effectful.get_type().unwrap();
    assert_eq!(value, 100);
}
