#![warn(missing_docs)]

use std::path::PathBuf;
use std::process::Command;

use gg_core::{
    GError, GErrorKind, GResult,
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

    /// 获取可执行文件名称
    fn exe_name(&self) -> &'static str {
        match self.target_os {
            DesktopTargetOs::Windows => "game.exe",
            DesktopTargetOs::Macos => "game",
            DesktopTargetOs::Linux => "game",
        }
    }

    /// 获取打包目录名称
    fn package_dir_name(&self) -> &'static str {
        match self.target_os {
            DesktopTargetOs::Windows => "game-windows",
            DesktopTargetOs::Macos => "game-macos",
            DesktopTargetOs::Linux => "game-linux",
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
        let package_dir = ctx.package_output_dir.join(self.package_dir_name());
        std::fs::create_dir_all(&package_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create package directory '{}': {}", package_dir.display(), e),
        })?;

        let exe_src = ctx.build_output_dir.join(self.exe_name());
        if !exe_src.exists() {
            return Err(GError {
                kind: GErrorKind::Io,
                message: format!("Executable not found at '{}'", exe_src.display()),
            });
        }

        let exe_dst = package_dir.join(self.exe_name());
        std::fs::copy(&exe_src, &exe_dst).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to copy executable to '{}': {}", exe_dst.display(), e),
        })?;

        if ctx.assets_dir.exists() {
            let assets_dst = package_dir.join("assets");
            copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
        }

        Ok(vec![package_dir])
    }

    fn run(&self, ctx: &RunContext) -> GResult<()> {
        if !ctx.executable_path.exists() {
            return Err(GError {
                kind: GErrorKind::Io,
                message: format!("Executable not found at '{}'", ctx.executable_path.display()),
            });
        }

        let mut command = Command::new(&ctx.executable_path);
        command.args(&ctx.args).current_dir(&ctx.project_dir);

        let status = command.status().map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to run executable '{}': {}", ctx.executable_path.display(), e),
        })?;

        if !status.success() {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("Executable exited with status: {}", status),
            });
        }

        Ok(())
    }
}

/// 递归复制目录
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> GResult<()> {
    std::fs::create_dir_all(dst).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create directory '{}': {}", dst.display(), e),
    })?;

    for entry in std::fs::read_dir(src).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read directory '{}': {}", src.display(), e),
    })? {
        let entry = entry.map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read directory entry: {}", e),
        })?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to copy file '{}': {}", src_path.display(), e),
            })?;
        }
    }

    Ok(())
}
