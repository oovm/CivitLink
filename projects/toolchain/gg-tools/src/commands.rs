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
    /// 检查配置表
    Check {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
    /// 生成代码
    Generate {
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
    /// 运行生成的引擎
    Run {
        /// 清单文件路径
        #[arg(long, default_value = ".")]
        manifest: String,
        /// 目标平台
        #[arg(long)]
        platform: Option<String>,
        /// 发布模式运行
        #[arg(long)]
        release: bool,
        /// 编辑器模式
        #[arg(long)]
        editor: bool,
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
        /// 子命令（check/generate）
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
    /// 生成和管理资源的 meta 文件
    Meta {
        /// 目标目录或文件路径
        target: String,
        /// 递归处理目录
        #[arg(short, long, default_value_t = true)]
        recursive: bool,
        /// 先读取再输出（重新生成 meta 文件）
        #[arg(long, default_value_t = false)]
        regenerate: bool,
    },
    /// 启动 LSP 服务
    Lsp {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
    /// 启动 MCP 服务
    Mcp {
        /// 工作目录
        #[arg(long, default_value = ".")]
        workspace: String,
    },
    /// 打开已有的项目
    Open {
        /// 项目目录路径
        path: Option<String>,
    },
}
