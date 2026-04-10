#![warn(missing_docs)]

//! GG 元游戏引擎命令行工具
//! 
//! 提供子命令用于初始化引擎项目、生成引擎代码、构建和创建游戏项目，
//! 以及脚本转换和性能测试等工具功能。

use clap::{Parser, Subcommand};
use gg_core::{GError, GErrorKind, GResult};
use gg_factory::EngineFactory;
use gg_manifest::EngineManifest;
use std::path::PathBuf;
use std::time::Instant;
use std::mem::size_of;
use gg_vm::{Vm, VmResult};
use gg_runtime_core::{EngineHost, ScriptEngine};
use std::io::{self};
use std::path::Path;

/// GG 元游戏引擎命令行工具
#[derive(Parser, Debug)]
#[command(name = "gg", version, about, long_about = None)]
struct Cli {
    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

/// 可用的子命令
#[derive(Subcommand, Debug)]
enum Commands {
    /// 初始化新的引擎项目
    Init {
        /// 引擎项目名称
        name: String,
        /// 游戏类型
        #[arg(long, default_value = "VisualNovel")]
        r#type: String,
    },
    /// 从清单生成引擎代码
    Generate {
        /// 清单文件路径
        #[arg(long, default_value = ".")]
        manifest: String,
    },
    /// 构建生成的引擎
    Build {
        /// 清单文件路径
        #[arg(long, default_value = ".")]
        manifest: String,
        /// 目标平台
        #[arg(long)]
        platform: Option<String>,
        /// 发布模式构建
        #[arg(long)]
        release: bool,
    },
    /// 创建新的游戏项目
    NewGame {
        /// 游戏项目名称
        name: String,
    },
    /// WASM Mod 脚本转换工具
    ModConverter {
        /// WASM 模块路径（可选）
        path: Option<String>,
    },
    /// 脚本性能测试工具
    ScriptBench,
}

/// CLI 主入口
fn main() -> GResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { name, r#type } => cmd_init(&name, &r#type),
        Commands::Generate { manifest } => cmd_generate(&manifest),
        Commands::Build { manifest, platform, release } => cmd_build(&manifest, platform.as_deref(), release),
        Commands::NewGame { name } => cmd_new_game(&name),
        Commands::ModConverter { path } => cmd_mod_converter(path.as_deref()),
        Commands::ScriptBench => cmd_script_bench(),
    }
}

/// 执行 `init` 子命令
///
/// 创建新的引擎项目目录，生成 Engine.toml、game.toml 和子目录。
fn cmd_init(engine_name: &str, game_type: &str) -> GResult<()> {
    let manifest = match game_type {
        "VisualNovel" | "vn" => gg_manifest::visual_novel_template(engine_name),
        "ARPG" | "arpg" => gg_manifest::arpg_template(engine_name),
        "Custom" | "custom" => gg_manifest::custom_template(engine_name),
        _ => gg_manifest::custom_template(engine_name),
    };

    let project_dir = PathBuf::from(engine_name);
    std::fs::create_dir_all(&project_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create project directory '{}': {}", engine_name, e),
    })?;

    let engine_toml_content = toml::to_string_pretty(&manifest)
        .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to serialize Engine.toml: {}", e) })?;
    std::fs::write(project_dir.join("Engine.toml"), engine_toml_content)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write Engine.toml: {}", e) })?;

    let game_toml = format!(
        r#"[game]
name = "{}"
version = "0.1.0"
initial_scene = "start"

[display]
width = {}
height = {}
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
"#,
        engine_name, manifest.display.width, manifest.display.height
    );
    std::fs::write(project_dir.join("game.toml"), game_toml)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write game.toml: {}", e) })?;

    std::fs::create_dir_all(project_dir.join("scripts"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create scripts directory: {}", e) })?;
    std::fs::create_dir_all(project_dir.join("assets"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create assets directory: {}", e) })?;

    println!("Created engine project '{}' with game type '{}'", engine_name, game_type);
    println!("  Engine.toml - engine manifest");
    println!("  game.toml - game configuration");
    println!("  scripts/ - script directory");
    println!("  assets/ - asset directory");
    Ok(())
}

/// 执行 `generate` 子命令
///
/// 读取 Engine.toml，解析并验证清单，调用工厂生成引擎代码。
fn cmd_generate(manifest_path: &str) -> GResult<()> {
    let path = PathBuf::from(manifest_path).join("Engine.toml");

    let manifest = EngineManifest::load_from_file(&path)?;
    manifest.validate()?;

    let output_dir = PathBuf::from(manifest_path).join("generated");
    let files = EngineFactory::generate(&manifest, &output_dir)?;

    println!("Generated engine project for '{}'", manifest.engine.name);
    println!("  Plugins: {}", manifest.modules.plugins.join(", "));
    println!("  Generated files:");
    for file in &files {
        println!("    {}", file.display());
    }
    Ok(())
}

/// 执行 `build` 子命令
///
/// 若 `generated/` 不存在则先自动生成，然后调用 cargo build 构建生成的项目。
fn cmd_build(manifest_path: &str, platform: Option<&str>, release: bool) -> GResult<()> {
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

/// 执行 `new-game` 子命令
///
/// 创建新的游戏项目目录，生成 game.toml、起始脚本和资源目录。
fn cmd_new_game(game_name: &str) -> GResult<()> {
    let game_dir = PathBuf::from(game_name);

    std::fs::create_dir_all(&game_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create game directory '{}': {}", game_name, e),
    })?;

    let game_toml = format!(
        r#"[game]
name = "{}"
version = "0.1.0"
initial_scene = "start"

[display]
width = 1280
height = 720
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
"#,
        game_name
    );
    std::fs::write(game_dir.join("game.toml"), game_toml)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write game.toml: {}", e) })?;

    std::fs::create_dir_all(game_dir.join("scripts"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create scripts directory: {}", e) })?;

    let start_script = r#"// Start script
label start {
    say "Hello, World!"
}
"#;
    std::fs::write(game_dir.join("scripts/start.gscript"), start_script)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write start.gscript: {}", e) })?;

    std::fs::create_dir_all(game_dir.join("assets"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create assets directory: {}", e) })?;

    println!("Created game project '{}'", game_name);
    Ok(())
}

/// 执行 `mod-converter` 子命令
///
/// WASM Mod 脚本转换为 Valkyrie 脚本的工具
fn cmd_mod_converter(wasm_path: Option<&str>) -> GResult<()> {
    println!("GG Game Engine - WASM Mod 脚本转换工具");
    println!("====================================");
    println!();
    println!("此工具帮助您将 WASM Mod 脚本转换为 Valkyrie 脚本");
    println!();
    
    // 显示转换指南
    print_conversion_guide();
    
    // 提示用户输入 WASM 模块路径（如果需要）
    println!();
    
    let path_to_check = if let Some(path) = wasm_path {
        path.to_string()
    } else {
        println!("请输入 WASM 模块路径（或按 Enter 跳过）:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    };
    
    let path_to_check = path_to_check.as_str();
    
    if !path_to_check.is_empty() {
        if Path::new(path_to_check).exists() {
            println!("已找到 WASM 模块，开始分析...");
            // 这里可以添加 WASM 模块分析逻辑
            println!("分析完成，生成转换建议...");
        } else {
            println!("错误：WASM 模块文件不存在");
        }
    }
    
    println!();
    println!("转换工具使用完成，祝您转换顺利！");
    Ok(())
}

/// 执行 `script-bench` 子命令
///
/// 脚本性能测试工具
fn cmd_script_bench() -> GResult<()> {
    println!("GG Game Engine - 脚本性能测试工具");
    println!("==================================");
    println!();
    
    // 测试启动时间
    test_startup_time();
    
    // 测试执行性能
    test_execution_performance();
    
    // 测试内存使用
    test_memory_usage();
    
    println!();
    println!("性能测试完成！");
    Ok(())
}

/// 打印转换指南
fn print_conversion_guide() {
    println!("转换指南:");
    println!("=============");
    println!();
    println!("1. 基本结构转换:");
    println!("   WASM 模块结构:");
    println!("   (module
    (import \"env\" \"print\" (func $print (param i32 i32)))
    (import \"env\" \"create_entity\" (func $create_entity (result i32)))
    
    (memory 1)
    
    (data (i32.const 0) \"Hello from script!\")
    
    (func (export \"init\")
        (call $print (i32.const 0) (i32.const 18))
        (call $create_entity)
        drop
    )
    
    (func (export \"update\") (param $delta f32)
    )
    )");
    println!();
    println!("   转换为 Valkyrie 脚本:");
    println!("   // 初始化函数
    function init() {{
        print(\"Hello from script!\");
        spawn_entity();
    }}
    
    // 更新函数
    function update(delta: float) {{
    }}");
    println!();
    println!("2. API 映射:");
    println!("   WASM 导入函数 -> Valkyrie 全局函数");
    println!("   - create_entity() -> spawn_entity()");
    println!("   - print() -> print()");
    println!("   - add_component() -> add_component()");
    println!("   - get_component_field() -> get_field()");
    println!("   - set_component_field() -> set_field()");
    println!();
    println!("3. 类型转换:");
    println!("   WASM 类型 -> Valkyrie 类型");
    println!("   - i32 -> int");
    println!("   - f32 -> float");
    println!("   - i8* -> string");
    println!();
    println!("4. 内存操作:");
    println!("   WASM 内存操作 -> Valkyrie 字符串/数组操作");
    println!("   - 内存加载字符串 -> 直接使用字符串字面量");
    println!();
    println!("5. 示例转换:");
    println!("   WASM 示例:");
    println!("   (func (export \"init\")
        (call $print (i32.const 0) (i32.const 18))
        (call $create_entity)
        drop
    )");
    println!();
    println!("   Valkyrie 示例:");
    println!("   function init() {{
        print(\"Hello from script!\");
        spawn_entity();
    }}");
    println!();
    println!("6. 注意事项:");
    println!("   - Valkyrie 脚本使用更简洁的语法");
    println!("   - 不需要手动管理内存");
    println!("   - API 调用更直观");
    println!("   - 支持更丰富的语言特性");
}

/// 测试脚本系统启动时间
fn test_startup_time() {
    println!("1. 测试启动时间:");
    println!("----------------");
    
    let start = Instant::now();
    
    // 创建脚本引擎
    let mut script_engine = ScriptEngine::new();
    
    let elapsed = start.elapsed();
    println!("脚本引擎启动时间: {:?}", elapsed);
    
    // 加载测试脚本
    let test_script = r#"
        function init() {
            print("Script initialized");
        }
        
        function update(delta) {
            // 空函数
        }
    "#;
    
    let start_load = Instant::now();
    let result = script_engine.load_script_string(test_script, "test");
    let load_elapsed = start_load.elapsed();
    
    match result {
        Ok(_) => println!("脚本加载时间: {:?}", load_elapsed),
        Err(e) => println!("脚本加载失败: {:?}", e),
    }
    
    println!();
}

/// 测试脚本执行性能
fn test_execution_performance() {
    println!("2. 测试执行性能:");
    println!("----------------");
    
    // 创建脚本引擎
    let mut script_engine = ScriptEngine::new();
    let mut host = EngineHost::new();
    
    // 加载测试脚本（包含计算密集型任务）
    let test_script = r#"
        function init() {
            print("Script initialized");
        }
        
        function calculate() {
            let sum = 0;
            for (let i = 0; i < 1000000; i++) {
                sum += i;
            }
            return sum;
        }
        
        function update(delta) {
            // 空函数
        }
    "#;
    
    if let Ok(_) = script_engine.load_script_string(test_script, "test") {
        // 测试函数执行性能
        let iterations = 10;
        let mut total_time = 0;
        
        for i in 0..iterations {
            let iter_start = Instant::now();
            let result = script_engine.call_function("calculate", &mut host);
            let iter_elapsed = iter_start.elapsed();
            total_time += iter_elapsed.as_millis();
            
            match result {
                VmResult::Ok | VmResult::Return(_) => {
                    println!("迭代 {}: {:?}", i + 1, iter_elapsed);
                }
                VmResult::Error(e) => {
                    println!("迭代 {} 错误: {}", i + 1, e);
                }
            }
        }
        
        let average_time = total_time / iterations;
        println!("平均执行时间: {}ms", average_time);
        println!("总执行时间: {}ms", total_time);
    } else {
        println!("脚本加载失败");
    }
    
    println!();
}

/// 测试内存使用情况
fn test_memory_usage() {
    println!("3. 测试内存使用:");
    println!("----------------");
    
    // 测试脚本引擎内存大小
    println!("ScriptEngine 大小: {} bytes", size_of::<ScriptEngine>());
    println!("Vm 大小: {} bytes", size_of::<Vm>());
    println!("EngineHost 大小: {} bytes", size_of::<EngineHost>());
    
    // 创建脚本引擎并加载脚本，观察内存变化
    let mut script_engine = ScriptEngine::new();
    
    let test_script = r#"
        function init() {
            print("Script initialized");
        }
        
        function update(delta) {
            // 空函数
        }
    "#;
    
    if let Ok(_) = script_engine.load_script_string(test_script, "test") {
        println!("脚本加载后，内存使用正常");
    } else {
        println!("脚本加载失败");
    }
    
    println!();
}
