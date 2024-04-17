#![warn(missing_docs)]

//! 构建平台抽象
//! 提供跨平台的构建、生成、打包和运行接口

use std::{collections::HashMap, path::PathBuf};

use crate::GResult;

use super::PlatformId;

/// 构建模式
///
/// 定义构建的目标模式，影响优化级别和调试信息生成。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    /// 调试模式，包含完整调试信息，无优化
    Debug,
    /// 发布模式，启用优化，不包含调试信息
    Release,
    /// 性能分析模式，启用优化同时保留性能分析所需的调试信息
    Profile,
}

impl BuildProfile {
    /// 是否为发布模式
    pub fn is_release(&self) -> bool {
        matches!(self, BuildProfile::Release | BuildProfile::Profile)
    }

    /// 是否为调试模式
    pub fn is_debug(&self) -> bool {
        matches!(self, BuildProfile::Debug)
    }
}

/// 构建配置
///
/// 包含平台构建所需的配置参数，包括目标平台、构建模式、特性开关和平台特定配置。
pub struct BuildConfig {
    /// 编译目标三元组（如 "x86_64-pc-windows-msvc"）
    pub target_triple: String,
    /// 构建模式
    pub profile: BuildProfile,
    /// Cargo features 列表
    pub features: Vec<String>,
    /// 输出目录路径
    pub output_dir: PathBuf,
    /// 平台特定配置键值对
    ///
    /// 各平台可通过 `configure_build()` 写入平台特定配置，
    /// 其他方法可通过此字段读取配置。
    pub platform_config: HashMap<String, String>,
}

impl BuildConfig {
    /// 创建新的构建配置
    ///
    /// 默认使用 Debug 模式，无特性开关，空的平台配置。
    pub fn new(target_triple: impl Into<String>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            target_triple: target_triple.into(),
            profile: BuildProfile::Debug,
            features: Vec::new(),
            output_dir: output_dir.into(),
            platform_config: HashMap::new(),
        }
    }

    /// 获取平台特定配置值
    pub fn get_platform_config(&self, key: &str) -> Option<&str> {
        self.platform_config.get(key).map(|s| s.as_str())
    }

    /// 设置平台特定配置值
    pub fn set_platform_config(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.platform_config.insert(key.into(), value.into());
    }
}

/// 生成上下文
///
/// 提供代码生成所需的上下文信息，包括平台特定配置。
pub struct GenerateContext {
    /// 引擎清单文件路径
    pub manifest_path: PathBuf,
    /// 生成代码输出目录
    pub output_dir: PathBuf,
    /// 模板文件目录（可选）
    pub template_dir: Option<PathBuf>,
    /// 平台特定配置键值对
    ///
    /// 从 `BuildConfig.platform_config` 传递而来，
    /// 允许 `generate_code()` 读取平台特定配置。
    pub platform_config: HashMap<String, String>,
}

impl GenerateContext {
    /// 创建新的生成上下文
    pub fn new(manifest_path: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            manifest_path: manifest_path.into(),
            output_dir: output_dir.into(),
            template_dir: None,
            platform_config: HashMap::new(),
        }
    }

    /// 获取平台特定配置值
    pub fn get_platform_config(&self, key: &str) -> Option<&str> {
        self.platform_config.get(key).map(|s| s.as_str())
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
    pub fn new(executable_path: impl Into<PathBuf>, project_dir: impl Into<PathBuf>) -> Self {
        Self { executable_path: executable_path.into(), project_dir: project_dir.into(), args: Vec::new() }
    }
}

/// 设备信息
///
/// 描述连接到当前平台的设备，包括设备标识、名称、所属平台和状态。
pub struct DeviceInfo {
    /// 设备唯一标识
    pub id: String,
    /// 设备显示名称
    pub name: String,
    /// 设备所属平台标识
    pub platform: String,
    /// 设备当前状态（如 "connected"、"disconnected"、"unauthorized"）
    pub state: String,
}

/// 环境检查报告
///
/// 描述当前构建环境的检查结果，包括可用工具、缺失工具和警告信息。
pub struct EnvironmentReport {
    /// 已安装且可用的工具列表
    pub available_tools: Vec<String>,
    /// 缺失的必需工具列表
    pub missing_tools: Vec<String>,
    /// 环境检查中产生的警告信息
    pub warnings: Vec<String>,
}

impl EnvironmentReport {
    /// 创建空的环境检查报告
    pub fn new() -> Self {
        Self { available_tools: Vec::new(), missing_tools: Vec::new(), warnings: Vec::new() }
    }

    /// 环境是否完整（无缺失工具）
    pub fn is_ok(&self) -> bool {
        self.missing_tools.is_empty()
    }
}

impl Default for EnvironmentReport {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具状态
///
/// 描述构建环境中某个工具的可用性及版本信息。
pub struct ToolStatus {
    /// 工具名称
    pub name: String,
    /// 工具是否可用
    pub available: bool,
    /// 工具版本号（如可用）
    pub version: Option<String>,
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

    /// 列出当前平台连接的设备
    ///
    /// 默认返回空列表，各平台可覆写此方法以提供实际的设备列表。
    fn list_devices(&self) -> GResult<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }

    /// 验证当前构建环境
    ///
    /// 检查构建所需的工具链和依赖是否齐全，默认返回空报告。
    /// 各平台可覆写此方法以执行实际的环境检查。
    fn validate_environment(&self) -> GResult<EnvironmentReport> {
        Ok(EnvironmentReport::new())
    }
}
