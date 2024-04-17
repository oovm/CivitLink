#![warn(missing_docs)]

use std::path::PathBuf;

use gg_core::{
    GResult,
    platform::{BuildConfig, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};

/// 桌面平台目标操作系统
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopTargetOs {
    /// Windows
    Windows,
    /// macOS
    Macos,
    /// Linux
    Linux,
}

/// 桌面平台构建时实现
///
/// 为 Windows、macOS、Linux 提供构建、生成、打包和运行的 Platform trait 实现。
pub struct DesktopPlatform {
    /// 目标操作系统
    target_os: DesktopTargetOs,
}

impl DesktopPlatform {
    /// 创建 Windows 桌面平台
    pub fn windows() -> Self {
        Self { target_os: DesktopTargetOs::Windows }
    }

    /// 创建 macOS 桌面平台
    pub fn macos() -> Self {
        Self { target_os: DesktopTargetOs::Macos }
    }

    /// 创建 Linux 桌面平台
    pub fn linux() -> Self {
        Self { target_os: DesktopTargetOs::Linux }
    }

    /// 获取目标操作系统
    pub fn target_os(&self) -> DesktopTargetOs {
        self.target_os
    }

    /// 获取目标三元组
    fn target_triple(&self) -> &'static str {
        match self.target_os {
            DesktopTargetOs::Windows => "x86_64-pc-windows-msvc",
            DesktopTargetOs::Macos => "x86_64-apple-darwin",
            DesktopTargetOs::Linux => "x86_64-unknown-linux-gnu",
        }
    }
}

impl Platform for DesktopPlatform {
    fn id(&self) -> PlatformId {
        "desktop".to_string()
    }

    fn display_name(&self) -> &str {
        match self.target_os {
            DesktopTargetOs::Windows => "Windows Desktop",
            DesktopTargetOs::Macos => "macOS Desktop",
            DesktopTargetOs::Linux => "Linux Desktop",
        }
    }

    fn configure_build(&self, config: &mut BuildConfig) {
        config.target_triple = self.target_triple().to_string();
    }

    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf> {
        Ok(ctx.output_dir.join("desktop_main.rs"))
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let exe_name = match self.target_os {
            DesktopTargetOs::Windows => "game.exe",
            DesktopTargetOs::Macos => "game",
            DesktopTargetOs::Linux => "game",
        };
        Ok(vec![ctx.build_output_dir.join(exe_name)])
    }

    fn run(&self, _ctx: &RunContext) -> GResult<()> {
        Ok(())
    }
}
