#![warn(missing_docs)]

//! `gg package` 命令实现
//!
//! 将构建产物打包为平台分发格式

use crate::{GError, GErrorKind, GResult, platform::resolve_platform};
use gg_manifest::EngineManifest;
use std::path::PathBuf;

use super::build::cmd_build;

/// 执行 `package` 子命令
///
/// 将构建产物打包为平台分发格式，若构建产物不存在则先执行构建
pub fn cmd_package(manifest_path: &str, platform: Option<&str>, release: bool) -> GResult<()> {
    let platform_name = platform.unwrap_or("windows");
    let platform_target = resolve_platform(platform_name)
        .cloned()
        .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: format!("Unknown platform '{}'", platform_name) })?;

    let project_dir = PathBuf::from(manifest_path);
    let generated_dir = project_dir.join("generated");

    let manifest_file = project_dir.join("Engine.toml");
    let manifest = EngineManifest::load_from_file(&manifest_file)?;
    let package_name = manifest.engine.name.replace('-', "_");

    let profile = if release { "release" } else { "debug" };

    let build_artifact_dir = if platform_target.name == "web" {
        generated_dir.join("target").join(&platform_target.target).join(profile)
    }
    else {
        generated_dir.join("target").join(profile)
    };

    let needs_build = if platform_target.name == "windows" {
        !build_artifact_dir.join(format!("{}.exe", package_name)).exists()
    }
    else if platform_target.name == "web" {
        !build_artifact_dir.join(format!("{}.wasm", package_name)).exists()
    }
    else {
        !build_artifact_dir.join(&package_name).exists()
    };

    if needs_build {
        println!("Build artifacts not found, building...");
        cmd_build(manifest_path, Some(&platform_target.target), release)?;
    }

    let dist_dir = generated_dir.join("dist").join(&platform_target.name);
    std::fs::create_dir_all(&dist_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create dist directory '{}': {}", dist_dir.display(), e),
    })?;

    match platform_target.name.as_str() {
        "windows" => package_windows(&build_artifact_dir, &dist_dir, &package_name)?,
        "web" => package_web(&build_artifact_dir, &dist_dir, &package_name)?,
        other => {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("Packaging for platform '{}' is not yet supported", other),
            });
        }
    }

    println!("Package completed: {}", dist_dir.display());
    Ok(())
}

/// 打包 Windows 平台产物
///
/// 收集 exe 和同目录下的 DLL 文件，复制到 dist 目录并压缩为 zip
fn package_windows(build_dir: &PathBuf, dist_dir: &PathBuf, package_name: &str) -> GResult<()> {
    let exe_path = build_dir.join(format!("{}.exe", package_name));
    copy_file(&exe_path, dist_dir)?;

    for entry in std::fs::read_dir(build_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read build directory '{}': {}", build_dir.display(), e),
    })? {
        let entry =
            entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("dll") {
                copy_file(&path, dist_dir)?;
            }
        }
    }

    let zip_path = dist_dir.parent().unwrap_or(dist_dir).join(format!("{}-windows.zip", package_name));
    create_zip_from_dir(dist_dir, &zip_path)?;
    println!("  Zipped: {}", zip_path.display());

    Ok(())
}

/// 打包 Web 平台产物
///
/// 收集 wasm 和 js 文件，复制到 dist 目录，生成 index.html 壳文件
fn package_web(build_dir: &PathBuf, dist_dir: &PathBuf, package_name: &str) -> GResult<()> {
    let wasm_path = build_dir.join(format!("{}.wasm", package_name));
    copy_file(&wasm_path, dist_dir)?;

    for entry in std::fs::read_dir(build_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read build directory '{}': {}", build_dir.display(), e),
    })? {
        let entry =
            entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("js") {
                copy_file(&path, dist_dir)?;
            }
        }
    }

    let html_content = generate_web_html(package_name);
    let html_path = dist_dir.join("index.html");
    std::fs::write(&html_path, html_content)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write index.html: {}", e) })?;

    Ok(())
}

/// 复制单个文件到目标目录
fn copy_file(src: &PathBuf, dest_dir: &PathBuf) -> GResult<()> {
    let file_name = src
        .file_name()
        .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: format!("Invalid file path: {}", src.display()) })?;
    let dest = dest_dir.join(file_name);
    std::fs::copy(src, &dest).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to copy '{}' to '{}': {}", src.display(), dest.display(), e),
    })?;
    println!("  Copied: {}", file_name.to_string_lossy());
    Ok(())
}

/// 生成 Web 平台的 index.html 壳文件
fn generate_web_html(package_name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{package_name}</title>
    <style>
        body {{ margin: 0; overflow: hidden; background: #000; }}
        canvas {{ display: block; width: 100vw; height: 100vh; }}
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import init from './{package_name}.js';
        async function run() {{
            await init();
        }}
        run();
    </script>
</body>
</html>
"#
    )
}

/// 将目录中的所有文件压缩为 zip
///
/// 遍历指定目录下的文件，逐个添加到 zip 归档中
fn create_zip_from_dir(dir: &std::path::Path, zip_path: &std::path::Path) -> GResult<()> {
    let file = std::fs::File::create(zip_path).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create zip file '{}': {}", zip_path.display(), e),
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in std::fs::read_dir(dir)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory '{}': {}", dir.display(), e) })?
    {
        let entry =
            entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        zip.start_file(name, options)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to add '{}' to zip: {}", name, e) })?;
        let mut f = std::fs::File::open(&path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to open '{}': {}", path.display(), e) })?;
        std::io::copy(&mut f, &mut zip)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write '{}' to zip: {}", name, e) })?;
    }

    zip.finish().map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to finalize zip: {}", e) })?;
    Ok(())
}
