//! `gg build` 命令实现
//!
//! 若 `generated/` 不存在则先自动生成，然后调用 cargo build 构建生成的项目。

use crate::GError;
use crate::GErrorKind;
use crate::GResult;
use std::path::PathBuf;

use super::generate::cmd_generate;

/// 执行 `build` 子命令
///
/// 若 `generated/` 不存在则先自动生成，然后调用 cargo build 构建生成的项目。
pub fn cmd_build(manifest_path: &str, platform: Option<&str>, release: bool) -> GResult<()> {
    let project_dir = PathBuf::from(manifest_path);
    let generated_dir = project_dir.join("generated");

    if !generated_dir.exists() {
        cmd_generate(manifest_path)?;
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--manifest-path").arg(generated_dir.join("Cargo.toml"));

    if let Some(target) = platform {
        cmd.arg("--target").arg(target);
    }
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
