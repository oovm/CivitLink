use std::path::PathBuf;

use gg_core::{
    GError, GResult,
    platform::{BuildConfig, GenerateContext, PackageContext, PlatformId, RunContext},
};

/// 平台构建时接口
///
/// 为各平台提供构建、生成、打包和运行的统一接口。
pub trait Platform {
    /// 获取平台 ID
    fn id(&self) -> PlatformId;

    /// 获取平台显示名称
    fn display_name(&self) -> &str;

    /// 配置构建参数
    fn configure_build(&self, config: &mut BuildConfig);

    /// 生成平台特定代码
    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf>;

    /// 打包平台特定产物
    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>>;

    /// 运行平台特定产物
    fn run(&self, ctx: &RunContext) -> GResult<()>;
}
