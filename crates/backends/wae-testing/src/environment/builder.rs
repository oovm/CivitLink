//! 测试环境构建器模块

use super::{AsyncTestLifecycleHook, TestEnv, TestEnvConfig, TestLifecycleHook, TestServiceConfig};

/// 测试环境构建器
///
/// 提供流式 API 构建测试环境，支持配置、生命周期钩子和多服务设置。
pub struct TestEnvBuilder {
    config: TestEnvConfig,
    lifecycle_hooks: Vec<Box<dyn TestLifecycleHook>>,
    async_lifecycle_hooks: Vec<Box<dyn AsyncTestLifecycleHook>>,
    services: Vec<TestServiceConfig>,
}

impl TestEnvBuilder {
    /// 创建新的构建器
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let builder = TestEnvBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: TestEnvConfig::default(),
            lifecycle_hooks: Vec::new(),
            async_lifecycle_hooks: Vec::new(),
            services: Vec::new(),
        }
    }

    /// 设置环境名称
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let env = TestEnvBuilder::new().name("my_test_env").build();
    /// ```
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// 启用日志
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let env = TestEnvBuilder::new().with_logging(false).build();
    /// ```
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.config.enable_logging = enable;
        self
    }

    /// 启用追踪
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let env = TestEnvBuilder::new().with_tracing(true).build();
    /// ```
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.config.enable_tracing = enable;
        self
    }

    /// 设置超时时间
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let env = TestEnvBuilder::new().timeout(Duration::from_secs(60)).build();
    /// ```
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    /// 添加自定义配置
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let env = TestEnvBuilder::new().custom("key", "value").build();
    /// ```
    pub fn custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.custom.insert(key.into(), value.into());
        self
    }

    /// 添加同步生命周期钩子
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::{TestEnv, TestEnvBuilder, TestLifecycleHook};
    ///
    /// struct MyHook;
    ///
    /// impl TestLifecycleHook for MyHook {}
    ///
    /// let env = TestEnvBuilder::new().with_lifecycle_hook(MyHook).build();
    /// ```
    pub fn with_lifecycle_hook<H>(mut self, hook: H) -> Self
    where
        H: TestLifecycleHook + 'static,
    {
        self.lifecycle_hooks.push(Box::new(hook));
        self
    }

    /// 添加异步生命周期钩子
    pub fn with_async_lifecycle_hook<H>(mut self, hook: H) -> Self
    where
        H: AsyncTestLifecycleHook + 'static,
    {
        self.async_lifecycle_hooks.push(Box::new(hook));
        self
    }

    /// 添加测试服务配置
    ///
    /// 向构建器中添加一个服务配置，构建时会自动注册到测试环境。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::{TestEnvBuilder, TestServiceConfig};
    ///
    /// let env = TestEnvBuilder::new().with_service(TestServiceConfig::new("database")).build();
    /// ```
    pub fn with_service(mut self, service: TestServiceConfig) -> Self {
        self.services.push(service);
        self
    }

    /// 批量添加测试服务配置
    ///
    /// 向构建器中添加多个服务配置。
    pub fn with_services<I>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = TestServiceConfig>,
    {
        self.services.extend(services);
        self
    }

    /// 构建测试环境
    ///
    /// 使用当前配置构建并返回测试环境实例。
    ///
    /// # Examples
    ///
    /// ```
    /// use wae_testing::TestEnvBuilder;
    ///
    /// let env = TestEnvBuilder::new().build();
    /// ```
    pub fn build(self) -> TestEnv {
        let env = TestEnv::new(self.config);

        for hook in self.lifecycle_hooks {
            env.lifecycle_hooks.write().push(hook);
        }

        for hook in self.async_lifecycle_hooks {
            env.async_lifecycle_hooks.write().push(hook);
        }

        for service in self.services {
            env.add_service(service);
        }

        env
    }
}

impl Default for TestEnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}
