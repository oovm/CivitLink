#![warn(missing_docs)]

//! GG-Sheet 命令行接口模块
//! 提供初始化、检查、生成和监听等子命令

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::compiler::SheetCompiler;
use crate::config::SheetConfig;
use crate::error::SheetResult;
use crate::reader::load_table;
use crate::schema::SheetTable;
use crate::validate::validate_tables;
use crate::watch::SheetWatcher;

/// GG-Sheet 命令行工具
#[derive(Parser, Debug)]
#[command(name = "gg-sheet", version, about = "GG 引擎配置表编译工具", long_about = None)]
pub struct SheetCli {
    /// 子命令
    #[command(subcommand)]
    pub command: SheetCommands,

    /// 工作区目录路径
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,

    /// 启用详细日志输出
    #[arg(long)]
    pub verbose: bool,
}

/// GG-Sheet 可用的子命令
#[derive(Subcommand, Debug)]
pub enum SheetCommands {
    /// 初始化工作区，创建目录结构和默认配置文件
    Init,
    /// 检查配置表数据有效性
    Check,
    /// 生成 Valkyrie 脚本代码
    Generate,
    /// 启用文件监听模式，自动增量编译
    Watch,
}

impl SheetCli {
    /// 执行命令行指令
    pub fn run(&self) -> SheetResult<()> {
        if self.verbose {
            println!("工作区: {}", self.workspace.display());
        }

        match &self.command {
            SheetCommands::Init => self.run_init(),
            SheetCommands::Check => self.run_check(),
            SheetCommands::Generate => self.run_generate(),
            SheetCommands::Watch => self.run_watch(),
        }
    }

    /// 执行 init 子命令
    fn run_init(&self) -> SheetResult<()> {
        let workspace = &self.workspace;

        let sheet_dir = workspace.join("asset/sheet");
        let output_dir = workspace.join("asset/script/table");

        std::fs::create_dir_all(&sheet_dir).map_err(|e| crate::error::SheetError::Io {
            path: sheet_dir.clone(),
            message: format!("无法创建配置表目录: {}", e),
        })?;

        std::fs::create_dir_all(&output_dir).map_err(|e| crate::error::SheetError::Io {
            path: output_dir.clone(),
            message: format!("无法创建输出目录: {}", e),
        })?;

        let config_path = workspace.join("GGSheet.toml");
        let config_obj = SheetConfig::default();
        config_obj.save(&config_path)?;

        println!("已初始化工作区: {}", workspace.display());
        println!("  配置表目录: {}", sheet_dir.display());
        println!("  输出目录: {}", output_dir.display());
        println!("  配置文件: {}", config_path.display());

        Ok(())
    }

    /// 执行 check 子命令
    fn run_check(&self) -> SheetResult<()> {
        let config = self.load_config()?;
        let sheet_dir = self.workspace.join(&config.sheet_dir);

        let compiler = SheetCompiler::new(&sheet_dir, &self.workspace.join(&config.output_dir));
        let files = compiler.scan()?;

        if self.verbose {
            println!("发现 {} 个配置表文件", files.len());
        }

        let mut tables: Vec<SheetTable> = Vec::new();
        for path in &files {
            let raw_table = load_table(path)?;
            let table = SheetTable::from_raw(raw_table)?;
            tables.push(table);
        }

        let report = validate_tables(&tables);
        println!("{}", report.format_report());

        if !report.is_valid() {
            std::process::exit(1);
        }

        Ok(())
    }

    /// 执行 generate 子命令
    fn run_generate(&self) -> SheetResult<()> {
        let config = self.load_config()?;
        let sheet_dir = self.workspace.join(&config.sheet_dir);
        let output_dir = self.workspace.join(&config.output_dir);

        let mut compiler = SheetCompiler::new(&sheet_dir, &output_dir);
        compiler.compile()?;

        println!("代码生成完成");
        Ok(())
    }

    /// 执行 watch 子命令
    fn run_watch(&self) -> SheetResult<()> {
        let config = self.load_config()?;
        let sheet_dir = self.workspace.join(&config.sheet_dir);
        let output_dir = self.workspace.join(&config.output_dir);

        let compiler = SheetCompiler::new(&sheet_dir, &output_dir);
        let mut watcher = SheetWatcher::new(compiler);
        watcher.watch()
    }

    /// 从工作区加载配置文件
    fn load_config(&self) -> SheetResult<SheetConfig> {
        let config_path = self.workspace.join("GGSheet.toml");
        SheetConfig::load(&config_path)
    }
}
