//! 配置表命令行接口模块
//! 提供 init、check、generate、watch 四个命令

use std::path::PathBuf;

use notify::Watcher;

use crate::{compiler::SheetCompiler, config::SheetConfig, validate::validate_tables_with_references};

/// CLI 命令
#[derive(Debug, Clone)]
pub enum Command {
    /// 初始化工作区
    Init {
        /// 工作区路径
        workspace: PathBuf,
    },
    /// 检查配置表
    Check {
        /// 工作区路径
        workspace: PathBuf,
    },
    /// 生成代码
    Generate {
        /// 工作区路径
        workspace: PathBuf,
    },
    /// 监听模式
    Watch {
        /// 工作区路径
        workspace: PathBuf,
    },
}

/// 执行 CLI 命令
pub fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Init { workspace } => run_init(&workspace),
        Command::Check { workspace } => run_check(&workspace),
        Command::Generate { workspace } => run_generate(&workspace),
        Command::Watch { workspace } => run_watch(&workspace),
    }
}

/// 执行 init 命令
///
/// 创建默认目录结构和配置文件
fn run_init(workspace: &std::path::Path) -> Result<(), String> {
    let sheet_dir = workspace.join("asset/sheet");
    let output_dir = workspace.join("asset/script/table");

    std::fs::create_dir_all(&sheet_dir).map_err(|e| format!("无法创建配置表目录: {}", e))?;
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("无法创建输出目录: {}", e))?;

    let config_path = workspace.join("sheet.config");
    if !config_path.exists() {
        let default_config =
            "sheet: SheetConfig {\n    sheet_dir: \"asset/sheet\"\n    output_dir: \"asset/script/table\"\n}\n";
        std::fs::write(&config_path, default_config).map_err(|e| format!("无法创建配置文件: {}", e))?;
    }

    println!("工作区初始化完成: {}", workspace.display());
    println!("  配置表目录: {}", sheet_dir.display());
    println!("  输出目录: {}", output_dir.display());
    Ok(())
}

/// 执行 check 命令
///
/// 扫描并验证所有配置表
fn run_check(workspace: &std::path::Path) -> Result<(), String> {
    let config = SheetConfig::load_from_workspace(workspace).map_err(|e| format!("无法加载配置: {}", e))?;

    let sheet_dir = workspace.join(&config.sheet_dir);
    let output_dir = workspace.join(&config.output_dir);

    let compiler = SheetCompiler::new(&sheet_dir, &output_dir);
    let files = compiler.scan().map_err(|e| format!("扫描失败: {}", e))?;

    if files.is_empty() {
        return Ok(());
    }

    let mut tables = Vec::new();
    for path in &files {
        match crate::reader::load_table(path) {
            Ok(raw) => match crate::schema::SheetTable::from_raw(raw) {
                Ok(table) => tables.push(table),
                Err(e) => println!("解析错误 {}: {}", path.display(), e),
            },
            Err(e) => println!("读取错误 {}: {}", path.display(), e),
        }
    }

    let report = validate_tables_with_references(&tables);

    if report.is_valid() {
        println!("验证通过，共检查 {} 张表", tables.len());
    }
    else {
        println!("{}", report.format_report());
        return Err(format!("发现 {} 个错误", report.error_count()));
    }

    Ok(())
}

/// 执行 generate 命令
///
/// 编译所有配置表并生成代码
fn run_generate(workspace: &std::path::Path) -> Result<(), String> {
    let config = SheetConfig::load_from_workspace(workspace).map_err(|e| format!("无法加载配置: {}", e))?;

    let sheet_dir = workspace.join(&config.sheet_dir);
    let output_dir = workspace.join(&config.output_dir);

    let mut compiler = SheetCompiler::new(&sheet_dir, &output_dir);
    compiler.compile().map_err(|e| format!("编译失败: {}", e))?;

    let files = compiler.scan().map_err(|e| format!("扫描失败: {}", e))?;

    println!("代码生成完成，共处理 {} 个配置表文件", files.len());
    Ok(())
}

/// 执行 watch 命令
///
/// 监听配置表目录变化，自动重新生成
fn run_watch(workspace: &std::path::Path) -> Result<(), String> {
    let config = SheetConfig::load_from_workspace(workspace).map_err(|e| format!("无法加载配置: {}", e))?;

    let sheet_dir = workspace.join(&config.sheet_dir);
    let output_dir = workspace.join(&config.output_dir);

    println!("开始监听配置表目录: {}", sheet_dir.display());
    println!("按 Ctrl+C 停止监听");

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )
    .map_err(|e| format!("无法创建文件监听器: {}", e))?;

    watcher.watch(&sheet_dir, notify::RecursiveMode::Recursive).map_err(|e| format!("无法监听目录: {}", e))?;

    let supported_extensions = ["xlsx", "xls", "csv", "tsv", "json"];

    for res in rx.iter() {
        let relevant = res
            .paths
            .iter()
            .any(|p| p.extension().and_then(|e| e.to_str()).map(|e| supported_extensions.contains(&e)).unwrap_or(false));

        if relevant {
            println!("检测到配置表变更，重新生成...");
            let mut compiler = SheetCompiler::new(&sheet_dir, &output_dir);
            match compiler.compile() {
                Ok(()) => println!("重新生成完成"),
                Err(e) => println!("重新生成失败: {}", e),
            }
        }
    }

    Ok(())
}
