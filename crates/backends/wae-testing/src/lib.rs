//! WAE Testing - 测试支持模块
//!
//! 提供统一的测试工具集，包含 Mock 工具、断言扩展、数据生成器和测试环境管理。
//!
//! 深度融合 tokio 运行时，支持异步测试场景。
//! 微服务架构友好，提供完整的测试基础设施。

#![warn(missing_docs)]

mod assertions;
mod containers;
mod db_mock;
mod environment;
mod fixture;
mod http_client;
mod mock;
mod service_mock;

#[cfg(any(feature = "bench", feature = "load-test"))]
mod benchmark;

#[cfg(any(feature = "bench", feature = "load-test"))]
mod load_test;

#[cfg(feature = "fuzz")]
mod fuzz;

#[cfg(feature = "pact")]
mod contract;

pub use assertions::{AsyncAssert, assert_eventually, assert_json_contains, assert_matches_regex};

pub use environment::{
    AsyncTestLifecycleHook, TestEnv, TestEnvBuilder, TestEnvConfig, TestEnvState, TestLifecycleHook, TestServiceConfig,
    create_test_env, create_test_env_with_config,
};

pub use fixture::{
    Fixture, FixtureBuilder, FixtureGenerator, RandomBool, RandomChoice, RandomDateTime, RandomEmail, RandomNumber,
    RandomString, RandomUuid,
};

pub use mock::{AsyncMock, Mock, MockBuilder, MockCall, MockExpectation, MockFn, MockResult, verify, verify_async};

#[cfg(feature = "mockall")]
pub use mock::{automock, mock, mockall, predicate, sequence};

pub use http_client::{RequestBuilder, TestClient, TestResponse};

pub use db_mock::{DatabaseExpectation, DatabaseQuery, DatabaseResult, MockDatabase, MockDatabaseBuilder};

pub use service_mock::{
    MockExternalService, MockExternalServiceBuilder, ServiceExpectation, ServiceMatchRule, ServiceRequest, ServiceResponse,
    ServiceResponseConfig,
};

#[cfg(feature = "containers")]
pub use containers::{
    MySqlContainer, MySqlImage, PostgresContainer, PostgresImage, RedisContainer, RedisImage, TestContainer,
    is_docker_available,
};

pub use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

#[cfg(any(feature = "bench", feature = "load-test"))]
pub use benchmark::{BenchmarkConfig, PerformanceStats, SimpleTimer};

#[cfg(any(feature = "bench", feature = "load-test"))]
pub use load_test::{LatencyStats, LoadTestConfig, LoadTestResult, RequestResult, SyncLoadTester};

#[cfg(feature = "fuzz")]
pub use fuzz::{Arbitrary, SafeArbitrary, SafeEmail, SafeHttpStatus, SafeUuid, Unstructured};

#[cfg(feature = "pact")]
pub use contract::*;
