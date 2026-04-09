#![warn(missing_docs)]

//! 构建平台抽象
//! 提供跨平台的构建、生成、打包和运行接口

use std::path::PathBuf;

use crate::GResult;

use super::PlatformId;

/// 构建配置
///
/// 包含平台构建所需的配置参数。
pub struct BuildConfig {
    /// 编译目标三元组（如 "x86_64-pc-windows-msvc"）
    pub target_triple: String,
    /// 是否启用 release 模式
    pub release: bool,
    /// Cargo features 列表
    pub features: Vec<String>,
    /// 输出目录路径
    pub output_dir: PathBuf,
}

impl BuildConfig {
    /// 创建新的构建配置
    pub fn new(target_triple: impl Into<String>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            target_triple: target_triple.into(),
            release: false,
            features: Vec::new(),
            output_dir: output_dir.into(),
        }
    }
}

/// 生成上下文
///
/// 提供代码生成所需的上下文信息。
pub struct GenerateContext {
    /// 引擎清单文件路径
    pub manifest_path: PathBuf,
    /// 生成代码输出目录
    pub output_dir: PathBuf,
    /// 模板文件目录（可选）
    pub template_dir: Option<PathBuf>,
}

impl GenerateContext {
    /// 创建新的生成上下文
    pub fn new(
        manifest_path: impl Into<PathBuf>,
        output_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            manifest_path: manifest_path.into(),
            output_dir: output_dir.into(),
            template_dir: None,
        }
    }
}

/// 打包上下文
///
/// 提供打包最终产物所需的上下文信息。
pub struct PackageContext {
    /// 构建输出目录
    pub build_output_dir: PathBuf,
    /// 打包输出目录
    pub package_output_dir: PathBuf,
    /// 引擎清单文件路径
    pub manifest_path: PathBuf,
    /// 游戏资源目录
    pub assets_dir: PathBuf,
}

impl PackageContext {
    /// 创建新的打包上下文
    pub fn new(
        build_output_dir: impl Into<PathBuf>,
        package_output_dir: impl Into<PathBuf>,
        manifest_path: impl Into<PathBuf>,
        assets_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            build_output_dir: build_output_dir.into(),
            package_output_dir: package_output_dir.into(),
            manifest_path: manifest_path.into(),
            assets_dir: assets_dir.into(),
        }
    }
}

/// 运行上下文
///
/// 提供本地运行/部署所需的上下文信息。
pub struct RunContext {
    /// 可执行文件路径
    pub executable_path: PathBuf,
    /// 游戏项目目录
    pub project_dir: PathBuf,
    /// 运行时参数
    pub args: Vec<String>,
}

impl RunContext {
    /// 创建新的运行上下文
    pub fn new(
        executable_path: impl Into<PathBuf>,
        project_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            executable_path: executable_path.into(),
            project_dir: project_dir.into(),
            args: Vec::new(),
        }
    }
}

/// 平台 trait
///
/// 定义平台适配的统一接口，各平台实现此 trait 以支持构建、生成、打包和运行。
pub trait Platform {
    /// 获取平台标识
    fn id(&self) -> PlatformId;

    /// 获取平台显示名称
    fn display_name(&self) -> &str;

    /// 配置构建
    fn configure_build(&self, config: &mut BuildConfig);

    /// 生成平台特定代码
    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf>;

    /// 打包最终产物
    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>>;

    /// 本地运行/部署
    fn run(&self, _ctx: &RunContext) -> GResult<()> {
        Ok(())
    }
}
