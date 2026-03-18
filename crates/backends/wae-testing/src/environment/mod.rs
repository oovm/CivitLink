//! 测试环境管理模块
//!
//! 提供完整的测试生命周期管理、多服务集成测试支持和容器化测试环境。

use parking_lot::RwLock;
use std::{
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

mod builder;
mod config;
mod hooks;
mod state;

pub use builder::TestEnvBuilder;
pub use config::{TestEnvConfig, TestServiceConfig};
pub use hooks::{AsyncTestLifecycleHook, TestLifecycleHook};
pub use state::TestEnvState;

/// 测试环境管理器
///
/// 提供完整的测试生命周期管理，支持同步和异步钩子函数，多服务集成测试，以及容器化测试环境。
pub struct TestEnv {
    /// 配置
    config: TestEnvConfig,
    /// 状态
    state: Arc<RwLock<TestEnvState>>,
    /// 创建时间
    created_at: Instant,
    /// 初始化完成时间
    initialized_at: Arc<RwLock<Option<Instant>>>,
    /// 同步生命周期钩子
    lifecycle_hooks: Arc<RwLock<Vec<Box<dyn TestLifecycleHook>>>>,
    /// 异步生命周期钩子
    async_lifecycle_hooks: Arc<RwLock<Vec<Box<dyn AsyncTestLifecycleHook>>>>,
    /// 清理函数列表
    #[allow(clippy::type_complexity)]
    cleanup_handlers: Arc<RwLock<Vec<Box<dyn Fn() + Send + Sync>>>>,
    /// 异步清理函数列表
    #[allow(clippy::type_complexity)]
    async_cleanup_handlers: Arc<RwLock<Vec<Box<dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>>>,
    /// 存储的数据
    storage: Arc<RwLock<std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    /// 测试服务配置
    services: Arc<RwLock<std::collections::HashMap<String, TestServiceConfig>>>,
}

impl std::fmt::Debug for TestEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestEnv")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("created_at", &self.created_at)
            .field("initialized_at", &self.initialized_at)
            .field("services", &self.services)
            .finish()
    }
}

impl TestEnv {
    /// 创建新的测试环境
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::{TestEnv, TestEnvConfig};
    ///
    /// let config = TestEnvConfig::default();
    /// let env = TestEnv::new(config);
    /// ```
    pub fn new(config: TestEnvConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(TestEnvState::Uninitialized)),
            created_at: Instant::now(),
            initialized_at: Arc::new(RwLock::new(None)),
            lifecycle_hooks: Arc::new(RwLock::new(Vec::new())),
            async_lifecycle_hooks: Arc::new(RwLock::new(Vec::new())),
            cleanup_handlers: Arc::new(RwLock::new(Vec::new())),
            async_cleanup_handlers: Arc::new(RwLock::new(Vec::new())),
            storage: Arc::new(RwLock::new(std::collections::HashMap::new())),
            services: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 创建默认测试环境
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// ```
    pub fn default_env() -> Self {
        Self::new(TestEnvConfig::default())
    }

    /// 初始化测试环境
    ///
    /// 按顺序执行所有 `before_setup` 钩子、初始化环境、然后执行所有 `after_setup` 钩子。
    ///
    /// # Errors
    ///
    /// 如果环境已初始化或任何钩子执行失败，将返回 [`WaeError`] 错误。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// env.setup().unwrap();
    /// ```
    pub fn setup(&self) -> TestingResult<()> {
        {
            let mut state = self.state.write();
            if *state != TestEnvState::Uninitialized {
                return Err(WaeError::new(WaeErrorKind::EnvironmentError {
                    reason: "Environment already initialized".to_string(),
                }));
            }
            *state = TestEnvState::Initializing;
        }

        let result = (|| {
            for hook in self.lifecycle_hooks.read().iter() {
                hook.before_setup(self)?;
            }

            for hook in self.lifecycle_hooks.read().iter() {
                hook.after_setup(self)?;
            }

            Ok(())
        })();

        let mut state = self.state.write();
        match result {
            Ok(_) => {
                *state = TestEnvState::Initialized;
                *self.initialized_at.write() = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                *state = TestEnvState::Uninitialized;
                Err(e)
            }
        }
    }

    /// 异步初始化测试环境
    ///
    /// 异步执行所有生命周期钩子函数，适合需要异步初始化的场景。
    ///
    /// # Errors
    ///
    /// 如果环境已初始化或任何钩子执行失败，将返回 [`WaeError`] 错误。
    pub async fn setup_async(&self) -> TestingResult<()> {
        {
            let mut state = self.state.write();
            if *state != TestEnvState::Uninitialized {
                return Err(WaeError::new(WaeErrorKind::EnvironmentError {
                    reason: "Environment already initialized".to_string(),
                }));
            }
            *state = TestEnvState::Initializing;
        }

        let result = (async {
            for hook in self.lifecycle_hooks.read().iter() {
                hook.before_setup(self)?;
            }

            #[allow(clippy::await_holding_lock)]
            for hook in self.async_lifecycle_hooks.read().iter() {
                hook.before_setup_async(self).await?;
            }

            #[allow(clippy::await_holding_lock)]
            for hook in self.async_lifecycle_hooks.read().iter() {
                hook.after_setup_async(self).await?;
            }

            for hook in self.lifecycle_hooks.read().iter() {
                hook.after_setup(self)?;
            }

            Ok(())
        })
        .await;

        let mut state = self.state.write();
        match result {
            Ok(_) => {
                *state = TestEnvState::Initialized;
                *self.initialized_at.write() = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                *state = TestEnvState::Uninitialized;
                Err(e)
            }
        }
    }

    /// 清理测试环境
    ///
    /// 按顺序执行所有 `before_teardown` 钩子、清理资源、然后执行所有 `after_teardown` 钩子。
    ///
    /// # Errors
    ///
    /// 如果环境未初始化或任何钩子执行失败，将返回 [`WaeError`] 错误。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// env.setup().unwrap();
    /// env.teardown().unwrap();
    /// ```
    pub fn teardown(&self) -> TestingResult<()> {
        {
            let mut state = self.state.write();
            if *state != TestEnvState::Initialized {
                return Err(WaeError::new(WaeErrorKind::EnvironmentError {
                    reason: "Environment not initialized".to_string(),
                }));
            }
            *state = TestEnvState::Destroying;
        }

        let result = (|| {
            for hook in self.lifecycle_hooks.read().iter() {
                hook.before_teardown(self)?;
            }

            let handlers = self.cleanup_handlers.write();
            for handler in handlers.iter().rev() {
                handler();
            }

            self.storage.write().clear();

            for hook in self.lifecycle_hooks.read().iter() {
                hook.after_teardown(self)?;
            }

            Ok(())
        })();

        let mut state = self.state.write();
        *state = TestEnvState::Destroyed;
        result
    }

    /// 异步清理测试环境
    ///
    /// 异步执行所有清理操作，适合需要异步清理的场景。
    ///
    /// # Errors
    ///
    /// 如果环境未初始化或任何钩子执行失败，将返回 [`WaeError`] 错误。
    pub async fn teardown_async(&self) -> TestingResult<()> {
        {
            let mut state = self.state.write();
            if *state != TestEnvState::Initialized {
                return Err(WaeError::new(WaeErrorKind::EnvironmentError {
                    reason: "Environment not initialized".to_string(),
                }));
            }
            *state = TestEnvState::Destroying;
        }

        let result = (async {
            for hook in self.lifecycle_hooks.read().iter() {
                hook.before_teardown(self)?;
            }

            #[allow(clippy::await_holding_lock)]
            for hook in self.async_lifecycle_hooks.read().iter() {
                hook.before_teardown_async(self).await?;
            }

            #[allow(clippy::await_holding_lock)]
            {
                let handlers = self.async_cleanup_handlers.write();
                for handler in handlers.iter().rev() {
                    handler().await;
                }
            }

            {
                let handlers = self.cleanup_handlers.write();
                for handler in handlers.iter().rev() {
                    handler();
                }
            }

            self.storage.write().clear();

            #[allow(clippy::await_holding_lock)]
            for hook in self.async_lifecycle_hooks.read().iter() {
                hook.after_teardown_async(self).await?;
            }

            for hook in self.lifecycle_hooks.read().iter() {
                hook.after_teardown(self)?;
            }

            Ok(())
        })
        .await;

        let mut state = self.state.write();
        *state = TestEnvState::Destroyed;
        result
    }

    /// 获取环境状态
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::{TestEnv, TestEnvState};
    ///
    /// let env = TestEnv::default_env();
    /// assert_eq!(env.state(), TestEnvState::Uninitialized);
    /// ```
    pub fn state(&self) -> TestEnvState {
        self.state.read().clone()
    }

    /// 获取环境运行时间
    ///
    /// 返回从环境创建到现在经过的时间。
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// 获取环境初始化后运行的时间
    ///
    /// 如果环境尚未初始化，返回 [`None`]。
    pub fn initialized_elapsed(&self) -> Option<Duration> {
        self.initialized_at.read().map(|t| t.elapsed())
    }

    /// 注册同步生命周期钩子
    ///
    /// 添加一个同步钩子函数，在测试环境的各个生命周期阶段执行。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// struct MyHook;
    ///
    /// impl wae_testing::TestLifecycleHook for MyHook {
    ///     fn after_setup(&self, _env: &TestEnv) -> wae_testing::TestingResult<()> {
    ///         println!("Environment setup complete!");
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let env = TestEnv::default_env();
    /// env.add_lifecycle_hook(MyHook);
    /// ```
    pub fn add_lifecycle_hook<H>(&self, hook: H)
    where
        H: TestLifecycleHook + 'static,
    {
        self.lifecycle_hooks.write().push(Box::new(hook));
    }

    /// 注册异步生命周期钩子
    ///
    /// 添加一个异步钩子函数，在测试环境的各个生命周期阶段异步执行。
    pub fn add_async_lifecycle_hook<H>(&self, hook: H)
    where
        H: AsyncTestLifecycleHook + 'static,
    {
        self.async_lifecycle_hooks.write().push(Box::new(hook));
    }

    /// 注册清理函数
    ///
    /// 添加一个同步清理函数，在测试环境清理时执行。清理函数按注册的逆序执行。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// env.on_cleanup(|| println!("Cleaning up!"));
    /// ```
    pub fn on_cleanup<F>(&self, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.cleanup_handlers.write().push(Box::new(handler));
    }

    /// 注册异步清理函数
    ///
    /// 添加一个异步清理函数，在测试环境清理时异步执行。
    pub fn on_cleanup_async<F, Fut>(&self, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.async_cleanup_handlers.write().push(Box::new(move || Box::pin(handler())));
    }

    /// 存储数据
    ///
    /// 在测试环境中存储任意类型的数据。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// env.set("test_key", "test_value");
    /// ```
    pub fn set<T: 'static + Send + Sync>(&self, key: &str, value: T) {
        self.storage.write().insert(key.to_string(), Box::new(value));
    }

    /// 获取数据
    ///
    /// 从测试环境中获取之前存储的数据。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// env.set("test_key", "test_value".to_string());
    /// let value: Option<String> = env.get("test_key");
    /// assert_eq!(value, Some("test_value".to_string()));
    /// ```
    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        let storage = self.storage.read();
        storage.get(key).and_then(|v| v.downcast_ref::<T>().cloned())
    }

    /// 移除数据
    ///
    /// 从测试环境中移除并返回之前存储的数据。
    pub fn remove<T: 'static>(&self, key: &str) -> Option<T> {
        let mut storage = self.storage.write();
        storage.remove(key).and_then(|v| v.downcast::<T>().ok()).map(|v| *v)
    }

    /// 检查是否存在指定键的数据
    pub fn has(&self, key: &str) -> bool {
        self.storage.read().contains_key(key)
    }

    /// 获取配置
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// let config = env.config();
    /// assert_eq!(config.name, "test");
    /// ```
    pub fn config(&self) -> &TestEnvConfig {
        &self.config
    }

    /// 添加测试服务配置
    ///
    /// 向测试环境中添加一个服务配置，用于多服务集成测试。
    pub fn add_service(&self, service_config: TestServiceConfig) {
        self.services.write().insert(service_config.name.clone(), service_config);
    }

    /// 获取测试服务配置
    ///
    /// 根据服务名称获取服务配置。
    pub fn get_service(&self, name: &str) -> Option<TestServiceConfig> {
        self.services.read().get(name).cloned()
    }

    /// 获取所有启用的服务
    ///
    /// 返回所有已启用的服务配置列表。
    pub fn enabled_services(&self) -> Vec<TestServiceConfig> {
        self.services.read().values().filter(|s| s.enabled).cloned().collect()
    }

    /// 使用 fixture 运行测试
    ///
    /// 自动管理测试环境的初始化和清理，执行测试函数。
    ///
    /// # Errors
    ///
    /// 如果环境初始化、测试执行或清理失败，将返回 [`WaeError`] 错误。
    pub async fn with_fixture<F, R>(&self, fixture: F) -> TestingResult<R>
    where
        F: FnOnce() -> TestingResult<R>,
    {
        self.setup()?;

        let result = fixture();

        self.teardown()?;

        result
    }

    /// 运行异步测试
    ///
    /// 自动管理测试环境的初始化和清理，执行异步测试函数。
    ///
    /// # Errors
    ///
    /// 如果环境初始化、测试执行或清理失败，将返回 [`WaeError`] 错误。
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use wae_testing::TestEnv;
    ///
    /// let env = TestEnv::default_env();
    /// env.run_test(|| async { Ok(()) }).await.unwrap();
    /// ```
    pub async fn run_test<F, Fut>(&self, test: F) -> TestingResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = TestingResult<()>>,
    {
        self.setup()?;

        let result = test().await;

        self.teardown()?;

        result
    }

    /// 运行带异步生命周期的测试
    ///
    /// 使用异步初始化和清理运行测试，适合需要异步操作的测试场景。
    ///
    /// # Errors
    ///
    /// 如果环境初始化、测试执行或清理失败，将返回 [`WaeError`] 错误。
    pub async fn run_test_async<F, Fut>(&self, test: F) -> TestingResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = TestingResult<()>>,
    {
        self.setup_async().await?;

        let result = test().await;

        self.teardown_async().await?;

        result
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let state = self.state.read().clone();
        if state == TestEnvState::Initialized {
            let _ = self.teardown();
        }
    }
}

/// 创建测试环境
///
/// 便捷函数，创建一个默认配置的测试环境。
///
/// # Examples
///
/// ```
/// use wae_testing::create_test_env;
///
/// let env = create_test_env();
/// ```
pub fn create_test_env() -> TestEnv {
    TestEnv::default_env()
}

/// 使用配置创建测试环境
///
/// 便捷函数，使用指定配置创建测试环境。
///
/// # Examples
///
/// ```
/// use wae_testing::{TestEnvConfig, create_test_env_with_config};
///
/// let config = TestEnvConfig::default();
/// let env = create_test_env_with_config(config);
/// ```
pub fn create_test_env_with_config(config: TestEnvConfig) -> TestEnv {
    TestEnv::new(config)
}
