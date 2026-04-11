#![warn(missing_docs)]

use std::{path::PathBuf, process::Command};

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{BuildConfig, DeviceInfo, EnvironmentReport, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};
use gg_platform::utils;

/// Web 平台构建时实现
///
/// 为 WebAssembly/Web 环境提供构建、生成、打包和运行的 Platform trait 实现。
/// 支持 WASM 代码分割、PWA 增强、GG Shader 编译管线集成。
pub struct WebPlatform;

impl Platform for WebPlatform {
    fn id(&self) -> PlatformId {
        "web".to_string()
    }

    fn display_name(&self) -> &str {
        "Web (WASM)"
    }

    fn configure_build(&self, config: &mut BuildConfig) {
        config.target_triple = "wasm32-unknown-unknown".to_string();
    }

    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf> {
        let web_dir = ctx.output_dir.join("web");
        std::fs::create_dir_all(&web_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create web directory '{}': {}", web_dir.display(), e),
        })?;

        let web_main_content = r#"use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn wasm_bindgen_start() {
    // Initialize the GG engine for web
    // The actual game initialization will be handled by the engine runtime
}
"#;
        let web_main_path = web_dir.join("web_main.rs");
        std::fs::write(&web_main_path, web_main_content).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write web_main.rs '{}': {}", web_main_path.display(), e),
        })?;

        let pwa_enabled = ctx.get_platform_config("pwa") == Some("true");
        let cache_strategy = ctx.get_platform_config("pwa_cache_strategy").unwrap_or("cache_first");

        let index_html_content = build_index_html(pwa_enabled);

        let index_html_path = web_dir.join("index.html");
        std::fs::write(&index_html_path, index_html_content).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to write index.html '{}': {}", index_html_path.display(), e),
        })?;

        if pwa_enabled {
            generate_pwa_manifest(&web_dir)?;
            generate_service_worker(&web_dir, cache_strategy)?;
        }

        Ok(web_dir)
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let package_dir = ctx.package_output_dir.join("game-web");
        std::fs::create_dir_all(&package_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create package directory '{}': {}", package_dir.display(), e),
        })?;

        for entry in std::fs::read_dir(&ctx.build_output_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read build output directory '{}': {}", ctx.build_output_dir.display(), e),
        })? {
            let entry = entry
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str == "wasm" || ext_str == "js" {
                    let file_name = entry.file_name();
                    let dst = package_dir.join(&file_name);
                    std::fs::copy(&path, &dst).map_err(|e| GError {
                        kind: GErrorKind::Io,
                        message: format!("Failed to copy '{}' to '{}': {}", path.display(), dst.display(), e),
                    })?;
                }
            }
        }

        let html_from_build = ctx.build_output_dir.join("index.html");
        if html_from_build.exists() {
            let dst = package_dir.join("index.html");
            std::fs::copy(&html_from_build, &dst).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to copy index.html to '{}': {}", dst.display(), e),
            })?;
        }
        else {
            let web_dir = ctx.build_output_dir.join("web").join("index.html");
            if web_dir.exists() {
                let dst = package_dir.join("index.html");
                std::fs::copy(&web_dir, &dst).map_err(|e| GError {
                    kind: GErrorKind::Io,
                    message: format!("Failed to copy index.html to '{}': {}", dst.display(), e),
                })?;
            }
        }

        let web_pwa_manifest = ctx.build_output_dir.join("web").join("manifest.json");
        if web_pwa_manifest.exists() {
            let dst = package_dir.join("manifest.json");
            std::fs::copy(&web_pwa_manifest, &dst).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to copy manifest.json to '{}': {}", dst.display(), e),
            })?;
        }

        let web_sw = ctx.build_output_dir.join("web").join("sw.js");
        if web_sw.exists() {
            let dst = package_dir.join("sw.js");
            std::fs::copy(&web_sw, &dst).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to copy sw.js to '{}': {}", dst.display(), e),
            })?;
        }

        if ctx.assets_dir.exists() {
            let assets_dst = package_dir.join("assets");
            utils::copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
        }

        let opt_level = "O4";
        optimize_wasm_files(&package_dir, opt_level)?;

        let enable_compression = true;
        if enable_compression {
            precompress_files(&package_dir)?;
        }

        Ok(vec![package_dir])
    }

    fn run(&self, ctx: &RunContext) -> GResult<()> {
        if !ctx.executable_path.exists() {
            return Err(GError {
                kind: GErrorKind::Io,
                message: format!("Web package directory not found at '{}'", ctx.executable_path.display()),
            });
        }

        let result = Command::new("python").arg("-m").arg("http.server").arg("8080").current_dir(&ctx.executable_path).spawn();

        match result {
            Ok(_) => {
                println!("Web server started at http://localhost:8080");
                Ok(())
            }
            Err(_) => {
                let result =
                    Command::new("python3").arg("-m").arg("http.server").arg("8080").current_dir(&ctx.executable_path).spawn();

                match result {
                    Ok(_) => {
                        println!("Web server started at http://localhost:8080");
                        Ok(())
                    }
                    Err(_) => Err(GError {
                        kind: GErrorKind::Platform,
                        message: "Python is required for local web preview. Please install Python and try again.".to_string(),
                    }),
                }
            }
        }
    }

    fn list_devices(&self) -> GResult<Vec<DeviceInfo>> {
        Ok(vec![DeviceInfo {
            id: "browser".to_string(),
            name: "Web Browser".to_string(),
            platform: "web".to_string(),
            state: "available".to_string(),
        }])
    }

    fn validate_environment(&self) -> GResult<EnvironmentReport> {
        let mut available = Vec::new();
        let mut missing = Vec::new();
        let mut warnings = Vec::new();

        let tools = ["wasm-bindgen", "wasm-opt", "gzip", "python"];

        for tool in tools {
            if utils::is_command_available(tool) {
                available.push(tool.to_string());
            }
            else {
                missing.push(tool.to_string());
                if tool == "wasm-bindgen" {
                    warnings.push("wasm-bindgen is required for WebAssembly builds".to_string());
                }
                else if tool == "wasm-opt" {
                    warnings.push("wasm-opt is not available, WASM optimization will be skipped".to_string());
                }
            }
        }

        if !utils::is_command_available("python") && !utils::is_command_available("python3") {
            warnings.push("Python is not available, local web preview will not work".to_string());
        }

        Ok(EnvironmentReport { available_tools: available, missing_tools: missing, warnings })
    }
}

/// 构建 index.html 内容
///
/// 根据 PWA 是否启用动态生成包含渲染后端检测的 index.html 内容。
/// 渲染检测优先级：WebGPU > WebGL2 > 显示错误提示。
///
/// ## Shader 编译管线说明
///
/// 本引擎的 shader 始终使用 GG Shader (.shader) 格式编写，通过以下管线编译：
/// GG Shader (.shader) → GG Shader Compiler → Naga IR → 目标后端格式
/// - WebGPU 后端：Naga IR → SPIR-V（内部中间表示）
/// - WebGL2 后端：Naga IR → GLSL
///
/// 绝不直接使用 WGSL，WGSL 仅在 Naga 内部作为中间表示使用。
fn build_index_html(pwa_enabled: bool) -> String {
    let mut head_extra = String::new();
    let mut body_extra = String::new();

    if pwa_enabled {
        head_extra.push_str(
            r#"    <link rel="manifest" href="manifest.json">
"#,
        );
        body_extra.push_str(
            r#"    <script>
        if ('serviceWorker' in navigator) {
            navigator.serviceWorker.register('/sw.js').then(reg => {
                console.log('Service Worker registered:', reg.scope);
            }).catch(err => {
                console.warn('Service Worker registration failed:', err);
            });
        }
    </script>
"#,
        );
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GG Game</title>{head_extra}    <style>
        body {{ margin: 0; overflow: hidden; }}
        canvas {{ display: block; width: 100vh; height: 100vh; }}
        #gg-error {{ display: none; color: white; background: red; padding: 20px; text-align: center; font-family: sans-serif; }}
    </style>
</head>
<body>
    <canvas id="gg-canvas" data-gg-render-backend=""></canvas>
    <div id="gg-error">Your browser does not support WebGPU or WebGL2. Please use a modern browser.</div>
    <!--
        GG Engine Shader Pipeline:
        All shaders are written in GG Shader (.shader) format and compiled through:
        GG Shader -> GG Shader Compiler -> Naga IR -> Target Backend Format
        - WebGPU: Naga IR -> SPIR-V (internal intermediate representation)
        - WebGL2: Naga IR -> GLSL
        WGSL is NEVER used directly by developers.
    -->
    <script>
        (async function() {{
            let backend = null;
            try {{
                const adapter = await navigator.gpu.requestAdapter();
                if (adapter) backend = 'webgpu';
            }} catch (e) {{}}
            if (!backend) {{
                const canvas = document.createElement('canvas');
                const gl = canvas.getContext('webgl2');
                if (gl) backend = 'webgl2';
            }}
            if (!backend) {{
                document.getElementById('gg-canvas').style.display = 'none';
                document.getElementById('gg-error').style.display = 'block';
            }} else {{
                document.getElementById('gg-canvas').setAttribute('data-gg-render-backend', backend);
                window.__GG_RENDER_BACKEND__ = backend;
            }}
        }})();
    </script>
    <script type="module">
        import("./game.js").then(module => module.default());
    </script>
{body_extra}</body>
</html>
"#,
        head_extra = head_extra,
        body_extra = body_extra,
    )
}

/// 生成 PWA manifest.json 文件
///
/// 在指定目录下创建标准 PWA manifest.json，包含应用名称、图标和显示配置。
fn generate_pwa_manifest(web_dir: &PathBuf) -> GResult<()> {
    let manifest_content = r##"{
    "name": "GG Game",
    "short_name": "GG Game",
    "icons": [{"src": "icon-192.png", "sizes": "192x192", "type": "image/png"}, {"src": "icon-512.png", "sizes": "512x512", "type": "image/png"}],
    "start_url": "/",
    "display": "standalone",
    "background_color": "#000000",
    "theme_color": "#000000"
}
"##;
    let manifest_path = web_dir.join("manifest.json");
    std::fs::write(&manifest_path, manifest_content).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to write manifest.json '{}': {}", manifest_path.display(), e),
    })?;
    Ok(())
}

/// 生成 Service Worker 模板 sw.js 文件
///
/// 在指定目录下创建包含缓存策略的 Service Worker 脚本，
/// 支持 install、fetch 和 activate 事件处理。
/// 缓存策略支持：cache_first、network_first、stale_while_revalidate。
fn generate_service_worker(web_dir: &PathBuf, cache_strategy: &str) -> GResult<()> {
    let fetch_handler = match cache_strategy {
        "network_first" => {
            r#"event.respondWith(
            fetch(event.request).then(response => {
                return caches.open(CACHE_NAME).then(cache => {
                    cache.put(event.request, response.clone());
                    return response;
                });
            }).catch(() => caches.match(event.request))
        );"#
        }
        "stale_while_revalidate" => {
            r#"event.respondWith(
            caches.match(event.request).then(cached => {
                const fetchPromise = fetch(event.request).then(response => {
                    return caches.open(CACHE_NAME).then(cache => {
                        cache.put(event.request, response.clone());
                        return response;
                    });
                });
                return cached || fetchPromise;
            })
        );"#
        }
        _ => {
            r#"event.respondWith(
            caches.match(event.request).then(response => {
                return response || fetch(event.request).then(fetchResponse => {
                    return caches.open(CACHE_NAME).then(cache => {
                        cache.put(event.request, fetchResponse.clone());
                        return fetchResponse;
                    });
                });
            })
        );"#
        }
    };

    let sw_content = format!(
        r#"const CACHE_NAME = 'gg-game-v1';
const ASSETS = [
    '/',
    '/index.html',
    '/game.js',
    '/game.wasm',
];

self.addEventListener('install', (event) => {{
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS))
    );
}});

self.addEventListener('fetch', (event) => {{
    {fetch_handler}
}});

self.addEventListener('activate', (event) => {{
    event.waitUntil(
        caches.keys().then((cacheNames) => {{
            return Promise.all(
                cacheNames.filter((name) => name !== CACHE_NAME).map((name) => caches.delete(name))
            );
        }})
    );
}});

self.addEventListener('message', (event) => {{
    if (event.data && event.data.type === 'SKIP_WAITING') {{
        self.skipWaiting();
    }}
}});
"#
    );
    let sw_path = web_dir.join("sw.js");
    std::fs::write(&sw_path, sw_content).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to write sw.js '{}': {}", sw_path.display(), e),
    })?;
    Ok(())
}

/// 使用 wasm-opt 优化 .wasm 文件
///
/// 检测系统中是否安装了 wasm-opt 工具，若可用则对打包目录中
/// 所有 .wasm 文件执行指定级别的优化（原地覆盖）。
/// 若 wasm-opt 不可用则输出警告并跳过。
fn optimize_wasm_files(package_dir: &PathBuf, opt_level: &str) -> GResult<()> {
    if !utils::is_command_available("wasm-opt") {
        eprintln!("warning: wasm-opt is not available, skipping WASM optimization");
        return Ok(());
    }

    let entries = match std::fs::read_dir(package_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "wasm" {
                let path_str = path.to_string_lossy().to_string();
                let opt_arg = format!("-{}", opt_level);
                let result = Command::new("wasm-opt").args([&opt_arg, &path_str, "-o", &path_str]).output();

                if let Err(e) = result {
                    eprintln!("warning: wasm-opt failed for '{}': {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

/// 对打包目录中的文件进行 gzip 预压缩
///
/// 对 .wasm、.js、.html、.json 扩展名的文件生成 .gz 预压缩副本，
/// 便于 Web 服务器启用静态 gzip 传输。
/// 若 gzip 命令不可用则输出警告并跳过。
fn precompress_files(package_dir: &PathBuf) -> GResult<()> {
    let compressible_extensions = ["wasm", "js", "html", "json"];

    if !utils::is_command_available("gzip") {
        eprintln!("warning: gzip is not available, skipping pre-compression");
        return Ok(());
    }

    let entries = match std::fs::read_dir(package_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if compressible_extensions.contains(&ext_str.as_ref()) {
                let result = Command::new("gzip").arg("-k").arg(&path).output();

                if let Err(e) = result {
                    eprintln!("warning: gzip failed for '{}': {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}
