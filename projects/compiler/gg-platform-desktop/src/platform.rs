#![warn(missing_docs)]

use std::{path::PathBuf, process::Command};

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{BuildConfig, DeviceInfo, EnvironmentReport, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};
use gg_platform::utils;

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

    /// 生成 winit 事件循环模板代码
    fn winit_main_rs() -> &'static str {
        r#"use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("GG Game")
        .build(&event_loop)
        .expect("Failed to create window");

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Resized(_size) => {
                    // Handle window resize
                }
                _ => {}
            },
            Event::AboutToWait => {
                // Main game loop tick
            }
            _ => {}
        }
    }).expect("Event loop error");
}
"#
    }

    /// 生成 macOS Info.plist 内容
    fn macos_plist_content(bundle_id: &str, bundle_name: &str, version: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>{bundle_id}</string>
    <key>CFBundleName</key>
    <string>{bundle_name}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleExecutable</key>
    <string>game</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"#
        )
    }

    /// 生成 Windows NSIS 安装脚本内容
    fn windows_nsi_content(product_name: &str, version: &str, publisher: &str) -> String {
        format!(
            r#"!define PRODUCT_NAME "{product_name}"
!define PRODUCT_VERSION "{version}"
!define PRODUCT_PUBLISHER "{publisher}"

Name "${{PRODUCT_NAME}}"
OutFile "gg-game-setup.exe"
InstallDir "$PROGRAMFILES\${{PRODUCT_NAME}}"

Section "MainSection"
    SetOutPath $INSTDIR
    File /r "game-windows\*.*"

    CreateShortCut "$DESKTOP\${{PRODUCT_NAME}}.lnk" "$INSTDIR\game.exe"
    CreateDirectory "$SMPROGRAMS\${{PRODUCT_NAME}}"
    CreateShortCut "$SMPROGRAMS\${{PRODUCT_NAME}}\${{PRODUCT_NAME}}.lnk" "$INSTDIR\game.exe"
    CreateShortCut "$SMPROGRAMS\${{PRODUCT_NAME}}\Uninstall.lnk" "$INSTDIR\uninstall.exe"

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR"
    Delete "$DESKTOP\${{PRODUCT_NAME}}.lnk"
    Delete "$SMPROGRAMS\${{PRODUCT_NAME}}\*.*"
    RMDir "$SMPROGRAMS\${{PRODUCT_NAME}}"
SectionEnd
"#
        )
    }

    /// macOS .app 打包
    fn package_macos(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let app_dir = ctx.package_output_dir.join("game.app");
        let contents_dir = app_dir.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");

        std::fs::create_dir_all(&macos_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create macOS app directory '{}': {}", macos_dir.display(), e),
        })?;

        std::fs::create_dir_all(&resources_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create Resources directory '{}': {}", resources_dir.display(), e),
        })?;

        let exe_src = ctx.build_output_dir.join(self.exe_name());
        if !exe_src.exists() {
            return Err(GError { kind: GErrorKind::Io, message: format!("Executable not found at '{}'", exe_src.display()) });
        }

        let exe_dst = macos_dir.join("game");
        std::fs::copy(&exe_src, &exe_dst).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to copy executable to '{}': {}", exe_dst.display(), e),
        })?;

        let plist_path = contents_dir.join("Info.plist");
        let plist_content = Self::macos_plist_content("com.gg.game", "GG Game", "1.0");
        std::fs::write(&plist_path, plist_content).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write Info.plist to '{}': {}", plist_path.display(), e),
        })?;

        if ctx.assets_dir.exists() {
            let assets_dst = resources_dir.join("assets");
            utils::copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
        }

        if utils::is_command_available("codesign") {
            let identity = "-";
            let app_str = app_dir.to_string_lossy().to_string();

            let args = vec!["--deep", "--force", "--sign", identity, &app_str];

            if let Ok(output) = Command::new("codesign").args(&args).status() {
                if !output.success() {
                    eprintln!("Warning: codesign failed, app bundle may not be signed correctly");
                }
            }
        }
        else {
            eprintln!("Warning: codesign not found, skipping macOS code signing");
        }

        Ok(vec![app_dir])
    }

    /// Windows 目录打包
    fn package_dir(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let package_dir = ctx.package_output_dir.join(self.package_dir_name());
        std::fs::create_dir_all(&package_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create package directory '{}': {}", package_dir.display(), e),
        })?;

        let exe_src = ctx.build_output_dir.join(self.exe_name());
        if !exe_src.exists() {
            return Err(GError { kind: GErrorKind::Io, message: format!("Executable not found at '{}'", exe_src.display()) });
        }

        let exe_dst = package_dir.join(self.exe_name());
        std::fs::copy(&exe_src, &exe_dst).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to copy executable to '{}': {}", exe_dst.display(), e),
        })?;

        if ctx.assets_dir.exists() {
            let assets_dst = package_dir.join("assets");
            utils::copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
        }

        if self.target_os == DesktopTargetOs::Windows {
            let nsi_content = Self::windows_nsi_content("GG Game", "1.0", "GG Game Engine");
            let nsi_path = package_dir.join("installer.nsi");
            std::fs::write(&nsi_path, nsi_content).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to write installer.nsi to '{}': {}", nsi_path.display(), e),
            })?;

            if utils::is_command_available("makensis") {
                let nsi_str = nsi_path.to_string_lossy().to_string();
                if let Err(e) = Command::new("makensis").arg(&nsi_str).current_dir(&ctx.package_output_dir).status() {
                    eprintln!("Warning: Failed to run makensis: {}", e);
                }
            }
            else {
                eprintln!("Warning: makensis not found, skipping NSIS installer compilation");
            }
        }

        Ok(vec![package_dir])
    }

    /// Linux AppImage 打包
    fn package_linux_appimage(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let appdir = ctx.package_output_dir.join("game.AppDir");
        let usr_dir = appdir.join("usr");
        let bin_dir = usr_dir.join("bin");
        let share_dir = usr_dir.join("share");

        std::fs::create_dir_all(&bin_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create AppDir bin directory '{}': {}", bin_dir.display(), e),
        })?;

        std::fs::create_dir_all(&share_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create AppDir share directory '{}': {}", share_dir.display(), e),
        })?;

        let exe_src = ctx.build_output_dir.join(self.exe_name());
        if !exe_src.exists() {
            return Err(GError { kind: GErrorKind::Io, message: format!("Executable not found at '{}'", exe_src.display()) });
        }

        let exe_dst = bin_dir.join("game");
        std::fs::copy(&exe_src, &exe_dst).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to copy executable to '{}': {}", exe_dst.display(), e),
        })?;

        if ctx.assets_dir.exists() {
            let assets_dst = share_dir.join("assets");
            utils::copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
        }

        let apprun_path = appdir.join("AppRun");
        std::fs::write(&apprun_path, "#!/bin/sh\nexec ./usr/bin/game\n").map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write AppRun to '{}': {}", apprun_path.display(), e),
        })?;

        let desktop_content = "[Desktop Entry]\nType=Application\nName=GG Game\nExec=game\n";
        let desktop_path = appdir.join("game.desktop");
        std::fs::write(&desktop_path, desktop_content).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write game.desktop to '{}': {}", desktop_path.display(), e),
        })?;

        let diricon_path = appdir.join(".DirIcon");
        std::fs::write(&diricon_path, "").map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write .DirIcon to '{}': {}", diricon_path.display(), e),
        })?;

        if utils::is_command_available("appimagetool") {
            let appdir_str = appdir.to_string_lossy().to_string();
            let appimage_path = ctx.package_output_dir.join("game-x86_64.AppImage");
            let appimage_str = appimage_path.to_string_lossy().to_string();

            match Command::new("appimagetool").args([&appdir_str, &appimage_str]).status() {
                Ok(status) if status.success() => {
                    let _ = std::fs::remove_dir_all(&appdir);
                    return Ok(vec![appimage_path]);
                }
                Ok(_) => {
                    eprintln!("Warning: appimagetool failed, falling back to AppDir output");
                }
                Err(e) => {
                    eprintln!("Warning: appimagetool execution failed: {}, falling back to AppDir output", e);
                }
            }
        }
        else {
            eprintln!(
                "Warning: appimagetool not found, outputting AppDir directory. Install appimagetool to generate .AppImage files."
            );
        }

        Ok(vec![appdir])
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
        let file_path = ctx.output_dir.join("desktop_main.rs");

        std::fs::write(&file_path, Self::winit_main_rs()).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write desktop_main.rs to '{}': {}", file_path.display(), e),
        })?;

        Ok(file_path)
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        match self.target_os {
            DesktopTargetOs::Macos => self.package_macos(ctx),
            DesktopTargetOs::Windows => self.package_dir(ctx),
            DesktopTargetOs::Linux => self.package_linux_appimage(ctx),
        }
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
            return Err(GError { kind: GErrorKind::Runtime, message: format!("Executable exited with status: {}", status) });
        }

        Ok(())
    }

    fn list_devices(&self) -> GResult<Vec<DeviceInfo>> {
        let os_name = match self.target_os {
            DesktopTargetOs::Windows => "windows",
            DesktopTargetOs::Macos => "macos",
            DesktopTargetOs::Linux => "linux",
        };
        Ok(vec![DeviceInfo {
            id: "local".to_string(),
            name: "Local Machine".to_string(),
            platform: os_name.to_string(),
            state: "available".to_string(),
        }])
    }

    fn validate_environment(&self) -> GResult<EnvironmentReport> {
        let mut available = Vec::new();
        let mut missing = Vec::new();
        let mut warnings = Vec::new();

        let tools = match self.target_os {
            DesktopTargetOs::Windows => vec!["makensis"],
            DesktopTargetOs::Macos => vec!["codesign", "xcodebuild"],
            DesktopTargetOs::Linux => vec!["appimagetool"],
        };

        for tool in tools {
            if utils::is_command_available(tool) {
                available.push(tool.to_string());
            }
            else {
                missing.push(tool.to_string());
                warnings.push(format!("{} is not available, some packaging features may be limited", tool));
            }
        }

        Ok(EnvironmentReport { available_tools: available, missing_tools: missing, warnings })
    }
}
