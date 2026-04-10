//! GG Tools 命令定义
//!
//! 提供 gg-tools 命令行工具的所有子命令定义

use clap::{Parser, Subcommand};

/// GG 元游戏引擎命令行工具
#[derive(Parser, Debug)]
#[command(name = "gg", version, about, long_about = None)]
pub struct Cli {
    /// 子命令
    #[command(subcommand)]
    pub command: Commands,
}

/// GG-Sheet 子命令
#[derive(Subcommand, Debug)]
pub enum SheetCommands {
    /// 初始化配置表工作区
    Init {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
    /// 检查配置表
    Check {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
    /// 生成 Valkyrie 脚本
    Generate {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
    /// 监听配置表变化
    Watch {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
}

/// 可用的子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
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
    /// 将构建产物打包为平台分发格式
    Package {
        /// 清单文件路径
        #[arg(long, default_value = ".")]
        manifest: String,
        /// 目标平台
        #[arg(long)]
        platform: Option<String>,
        /// 发布模式
        #[arg(long)]
        release: bool,
    },
    /// 配置表管理工具
    Sheet {
        /// 子命令（init/check/generate/watch）
        #[command(subcommand)]
        command: SheetCommands,
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
