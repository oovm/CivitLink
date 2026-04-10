//! `gg build` 命令实现
//!
//! 若 `generated/` 不存在则先自动生成，然后调用 cargo build 构建生成的项目。
//! 支持多平台构建，可通过 `--platform` 参数指定目标平台。

use crate::{
    GError, GErrorKind, GResult,
    platform::{PlatformTarget, check_target_installed, resolve_from_manifest, resolve_platform},
};
use gg_manifest::EngineManifest;
use std::path::PathBuf;

use super::generate::cmd_generate;

/// 执行 `build` 子命令
///
/// 若 `generated/` 不存在则先自动生成，然后调用 cargo build 构建生成的项目。
/// 当 `--platform` 参数为 `Some("all")` 时，从清单中读取所有平台依次构建；
/// 当 `--platform` 参数为 `Some(name)` 时，使用 `resolve_platform(name)` 映射到 target 和 features；
/// 当 `--platform` 参数为 `None` 时，保持原有行为（直接 cargo build）。
pub fn cmd_build(manifest_path: &str, platform: Option<&str>, release: bool) -> GResult<()> {
    let project_dir = PathBuf::from(manifest_path);
    let generated_dir = project_dir.join("generated");

    if !generated_dir.exists() {
        cmd_generate(manifest_path)?;
    }

    match platform {
        None => build_default(&generated_dir, release),
        Some("all") => {
            let manifest = load_manifest(manifest_path)?;
            let platforms = resolve_from_manifest(&manifest);
            if platforms.is_empty() {
                return Err(GError { kind: GErrorKind::Runtime, message: "No platforms defined in manifest".to_string() });
            }
            build_all_platforms(&generated_dir, &platforms, release)
        }
        Some(name) => {
            let resolved = resolve_platform(name).cloned();
            match resolved {
                Some(pt) => {
                    if !check_target_installed(&pt.target) {
                        eprintln!(
                            "Warning: target '{}' is not installed. Install with: rustup target add {}",
                            pt.target, pt.target
                        );
                    }
                    build_for_platform(&generated_dir, &pt, release)
                }
                None => Err(GError {
                    kind: GErrorKind::Runtime,
                    message: format!(
                        "Unknown platform '{}'. Available: windows, windows-gnu, macos, macos-x86, linux, web, android, android-x86, ios, ios-sim",
                        name
                    ),
                }),
            }
        }
    }
}

/// 加载引擎清单
fn load_manifest(manifest_path: &str) -> GResult<EngineManifest> {
    let path = PathBuf::from(manifest_path).join("Engine.toml");
    EngineManifest::load_from_file(&path)
}

/// 执行默认构建（不指定平台）
fn build_default(generated_dir: &PathBuf, release: bool) -> GResult<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--manifest-path").arg(generated_dir.join("Cargo.toml"));

    if release {
        cmd.arg("--release");
    }

    let status =
        cmd.status().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to run cargo build: {}", e) })?;

    if !status.success() {
        return Err(GError { kind: GErrorKind::Runtime, message: "Build failed".to_string() });
    }

    println!("Build completed successfully");
    Ok(())
}

/// 依次构建所有平台
fn build_all_platforms(generated_dir: &PathBuf, platforms: &[PlatformTarget], release: bool) -> GResult<()> {
    for pt in platforms {
        if !check_target_installed(&pt.target) {
            eprintln!("Warning: target '{}' is not installed. Install with: rustup target add {}", pt.target, pt.target);
        }
        build_for_platform(generated_dir, pt, release)?;
    }
    Ok(())
}

/// 为指定平台执行构建
fn build_for_platform(generated_dir: &PathBuf, platform: &PlatformTarget, release: bool) -> GResult<()> {
    println!("Building for {} ({})...", platform.name, platform.target);

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--manifest-path").arg(generated_dir.join("Cargo.toml"));
    cmd.arg("--target").arg(&platform.target);

    if !platform.features.is_empty() {
        cmd.arg("--features").arg(platform.features.join(","));
    }

    if release {
        cmd.arg("--release");
    }

    let status =
        cmd.status().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to run cargo build: {}", e) })?;

    if !status.success() {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: format!("Build failed for platform '{}' ({})", platform.name, platform.target),
        });
    }

    println!("Build for {} completed successfully", platform.name);
    Ok(())
}
