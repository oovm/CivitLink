//! `gg run` 命令实现
//!
//! 若 `generated/` 不存在则先自动生成，若构建产物不存在则先自动构建，
//! 然后运行生成的引擎。桌面平台使用 `cargo run`，Web 平台启动 HTTP 服务器。

use crate::{
    GError, GErrorKind, GResult,
    platform::{Platform, resolve_platform},
};
use std::path::PathBuf;

use super::{build::cmd_build, generate::cmd_generate};

/// 执行 `run` 子命令
///
/// 自动检查 `generated/` 目录和构建产物是否存在，不存在则依次调用生成和构建。
/// 桌面平台调用 `cargo run`，Web 平台在 wasm 产物目录启动 HTTP 服务器。
pub fn cmd_run(manifest_path: &str, platform: Option<&str>, release: bool, editor: bool) -> GResult<()> {
    let project_dir = PathBuf::from(manifest_path);
    let generated_dir = project_dir.join("generated");

    if !generated_dir.exists() {
        cmd_generate(manifest_path)?;
    }

    let resolved = platform
        .map(|name| resolve_platform(name).map(|pt| (pt.name.clone(), pt.target.clone(), Platform::from_target(&pt.target))));

    let target_dir = match &resolved {
        Some(Some((_, target, _))) => generated_dir.join("target").join(target),
        _ => generated_dir.join("target"),
    };

    let profile_dir = if release { target_dir.join("release") } else { target_dir.join("debug") };

    let has_build_output = profile_dir.exists();

    if !has_build_output {
        cmd_build(manifest_path, platform, release)?;
    }

    match resolved {
        None => run_desktop(&generated_dir, None, release, editor),
        Some(None) => Err(GError {
            kind: GErrorKind::Runtime,
            message: format!(
                "Unknown platform '{}'. Available: windows, windows-gnu, macos, macos-x86, linux, web, android, android-x86, ios, ios-sim",
                platform.unwrap_or("")
            ),
        }),
        Some(Some((name, target, plat))) => match plat {
            Platform::Desktop => run_desktop(&generated_dir, Some(&target), release, editor),
            Platform::Web => run_web(&generated_dir, &target, release),
            Platform::Mobile => Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("Running on platform '{}' is not supported yet", name),
            }),
        },
    }
}

/// 在桌面平台运行引擎
///
/// 调用 `cargo run --manifest-path generated/Cargo.toml`，可选传递 `--target`、`--release` 和 `--editor` 参数。
fn run_desktop(generated_dir: &PathBuf, target: Option<&str>, release: bool, editor: bool) -> GResult<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run");
    cmd.arg("--manifest-path").arg(generated_dir.join("Cargo.toml"));

    if let Some(t) = target {
        cmd.arg("--target").arg(t);
    }

    if release {
        cmd.arg("--release");
    }

    if editor {
        cmd.arg("--").arg("--editor");
    }

    let status =
        cmd.status().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to run cargo run: {}", e) })?;

    if !status.success() {
        return Err(GError { kind: GErrorKind::Runtime, message: "Run failed".to_string() });
    }

    Ok(())
}

/// 在 Web 平台运行引擎
///
/// 检测 `simple-http-server` 或 `python -m http.server` 是否可用，
/// 在 wasm 产物目录启动 HTTP 服务器。
fn run_web(generated_dir: &PathBuf, target: &str, release: bool) -> GResult<()> {
    let wasm_dir = generated_dir.join("target").join(target).join(if release { "release" } else { "debug" });

    if !wasm_dir.exists() {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: format!("WASM output directory not found: {}", wasm_dir.display()),
        });
    }

    if is_command_available("simple-http-server") {
        println!("Starting HTTP server with simple-http-server on port 8000...");
        let status = std::process::Command::new("simple-http-server")
            .arg("--port")
            .arg("8000")
            .arg("--index")
            .current_dir(&wasm_dir)
            .status()
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to start simple-http-server: {}", e) })?;

        if !status.success() {
            return Err(GError { kind: GErrorKind::Runtime, message: "HTTP server exited with error".to_string() });
        }
    }
    else if is_command_available("python") {
        println!("Starting HTTP server with python on port 8000...");
        let status =
            std::process::Command::new("python").args(["-m", "http.server", "8000"]).current_dir(&wasm_dir).status().map_err(
                |e| GError { kind: GErrorKind::Runtime, message: format!("Failed to start python http.server: {}", e) },
            )?;

        if !status.success() {
            return Err(GError { kind: GErrorKind::Runtime, message: "HTTP server exited with error".to_string() });
        }
    }
    else {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: "No HTTP server available. Install simple-http-server or python to serve WASM content".to_string(),
        });
    }

    Ok(())
}

/// 检查指定命令是否可用
///
/// 在系统 PATH 中查找指定命令是否可执行。
fn is_command_available(command: &str) -> bool {
    std::process::Command::new(command)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
