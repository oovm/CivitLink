use std::path::PathBuf;

use gg_core::{
    GResult,
    platform::{BuildConfig, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};

/// 移动平台目标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileTarget {
    /// iOS
    Ios,
    /// Android
    Android,
}

/// 移动平台构建时实现
///
/// 为 iOS/Android 提供构建、生成、打包和运行的 Platform trait 占位实现。
pub struct MobilePlatform {
    /// 目标移动平台
    target: MobileTarget,
}

impl MobilePlatform {
    /// 创建 iOS 移动平台
    pub fn ios() -> Self {
        Self { target: MobileTarget::Ios }
    }

    /// 创建 Android 移动平台
    pub fn android() -> Self {
        Self { target: MobileTarget::Android }
    }

    /// 获取目标移动平台
    pub fn target(&self) -> MobileTarget {
        self.target
    }
}

impl Platform for MobilePlatform {
    fn id(&self) -> PlatformId {
        "mobile".to_string()
    }

    fn display_name(&self) -> &str {
        match self.target {
            MobileTarget::Ios => "iOS",
            MobileTarget::Android => "Android",
        }
    }

    fn configure_build(&self, config: &mut BuildConfig) {
        let triple = match self.target {
            MobileTarget::Ios => "aarch64-apple-ios",
            MobileTarget::Android => "aarch64-linux-android",
        };
        config.target_triple = triple.to_string();
    }

    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf> {
        Ok(ctx.output_dir.join("mobile_main.rs"))
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let package_file = match self.target {
            MobileTarget::Ios => ctx.build_output_dir.join("game.ipa"),
            MobileTarget::Android => ctx.build_output_dir.join("game.apk"),
        };
        Ok(vec![package_file])
    }
}
