//! 测试生命周期钩子模块

use wae_types::WaeResult as TestingResult;

/// 测试生命周期钩子 trait
///
/// 定义测试环境在各个阶段执行的钩子函数，用于自定义测试环境的初始化和清理逻辑。
pub trait TestLifecycleHook: Send + Sync {
    /// 在测试环境初始化之前执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    fn before_setup(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }

    /// 在测试环境初始化之后执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    fn after_setup(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }

    /// 在测试环境清理之前执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    fn before_teardown(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }

    /// 在测试环境清理之后执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    fn after_teardown(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }
}

impl<F> TestLifecycleHook for F
where
    F: Fn(&crate::TestEnv) -> TestingResult<()> + Send + Sync,
{
    fn after_setup(&self, env: &crate::TestEnv) -> TestingResult<()> {
        self(env)
    }
}

/// 异步测试生命周期钩子 trait
///
/// 定义测试环境在各个阶段执行的异步钩子函数，支持异步操作的初始化和清理。
#[async_trait::async_trait]
pub trait AsyncTestLifecycleHook: Send + Sync {
    /// 在测试环境初始化之前异步执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    async fn before_setup_async(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }

    /// 在测试环境初始化之后异步执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    async fn after_setup_async(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }

    /// 在测试环境清理之前异步执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    async fn before_teardown_async(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }

    /// 在测试环境清理之后异步执行
    ///
    /// # Errors
    ///
    /// 如果钩子执行失败，将返回 [`WaeError`] 错误。
    async fn after_teardown_async(&self, _env: &crate::TestEnv) -> TestingResult<()> {
        Ok(())
    }
}
