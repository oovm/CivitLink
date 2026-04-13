#![warn(missing_docs)]

//! 跨平台构建协调器
//!
//! 提供统一的多平台构建管理，支持同时构建多个平台，
//! 并汇总构建进度和错误报告。

use gg_core::{
    GError, GResult,
    platform::{BuildConfig, GenerateContext, PackageContext, Platform, RunContext},
};

/// 构建结果
///
/// 记录单个平台的构建结果，包括平台 ID、是否成功和错误信息。
#[derive(Debug)]
pub struct BuildResult {
    /// 平台标识
    pub platform_id: String,
    /// 是否构建成功
    pub success: bool,
    /// 错误信息（仅在失败时有值）
    pub error: Option<String>,
}

/// 构建协调器
///
/// 统一管理多平台构建流程，支持依次构建所有已配置的平台，
/// 并汇总构建进度和错误。
pub struct BuildCoordinator {
    /// 已注册的平台列表
    platforms: Vec<Box<dyn Platform>>,
}

impl BuildCoordinator {
    /// 创建新的构建协调器
    pub fn new() -> Self {
        Self { platforms: Vec::new() }
    }

    /// 注册平台
    pub fn register_platform(&mut self, platform: Box<dyn Platform>) {
        self.platforms.push(platform);
    }

    /// 获取已注册的平台数量
    pub fn platform_count(&self) -> usize {
        self.platforms.len()
    }

    /// 获取所有已注册平台的 ID 列表
    pub fn platform_ids(&self) -> Vec<String> {
        self.platforms.iter().map(|p| p.id()).collect()
    }

    /// 构建指定平台
    ///
    /// 执行指定平台的完整构建流程：配置 → 代码生成 → 打包。
    pub fn build_platform(
        &self,
        platform_id: &str,
        config: &mut BuildConfig,
        gen_ctx: &GenerateContext,
        pkg_ctx: &PackageContext,
    ) -> BuildResult {
        let platform = match self.find_platform(platform_id) {
            Some(p) => p,
            None => {
                return BuildResult {
                    platform_id: platform_id.to_string(),
                    success: false,
                    error: Some(format!("Platform '{}' not found", platform_id)),
                };
            }
        };

        println!("Building for platform: {} ({})", platform.display_name(), platform.id());

        platform.configure_build(config);

        match platform.generate_code(gen_ctx) {
            Ok(path) => println!("  Generated code at: {}", path.display()),
            Err(e) => {
                return BuildResult {
                    platform_id: platform_id.to_string(),
                    success: false,
                    error: Some(format!("Code generation failed: {}", e.message)),
                };
            }
        }

        match platform.package(pkg_ctx) {
            Ok(outputs) => {
                for output in &outputs {
                    println!("  Package output: {}", output.display());
                }
                BuildResult { platform_id: platform_id.to_string(), success: true, error: None }
            }
            Err(e) => BuildResult {
                platform_id: platform_id.to_string(),
                success: false,
                error: Some(format!("Packaging failed: {}", e.message)),
            },
        }
    }

    /// 构建所有已注册平台
    ///
    /// 依次构建所有已注册的平台，返回每个平台的构建结果。
    pub fn build_all(&self, config: &mut BuildConfig, gen_ctx: &GenerateContext, pkg_ctx: &PackageContext) -> Vec<BuildResult> {
        let mut results = Vec::new();

        for platform in &self.platforms {
            let platform_id = platform.id();
            let result = self.build_platform(&platform_id, config, gen_ctx, pkg_ctx);
            results.push(result);
        }

        results
    }

    /// 运行指定平台
    pub fn run_platform(&self, platform_id: &str, run_ctx: &RunContext) -> GResult<()> {
        let platform = self.find_platform(platform_id).ok_or_else(|| GError {
            kind: gg_core::GErrorKind::Platform,
            message: format!("Platform '{}' not found", platform_id),
        })?;

        platform.run(run_ctx)
    }

    /// 汇总构建结果
    ///
    /// 返回成功数量、失败数量和失败的平台列表。
    pub fn summarize_results(results: &[BuildResult]) -> (usize, usize, Vec<&str>) {
        let success_count = results.iter().filter(|r| r.success).count();
        let fail_count = results.len() - success_count;
        let failed_platforms: Vec<&str> = results.iter().filter(|r| !r.success).map(|r| r.platform_id.as_str()).collect();
        (success_count, fail_count, failed_platforms)
    }

    /// 查找指定 ID 的平台
    fn find_platform(&self, platform_id: &str) -> Option<&dyn Platform> {
        self.platforms.iter().find(|p| p.id() == platform_id).map(|p| p.as_ref())
    }
}

impl Default for BuildCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
