//! 命令模块
//!
//! 包含所有 WAE Tools 的命令定义。

pub mod dev;
pub mod generate;
pub mod migrate;
pub mod new;
pub mod pull;
pub mod push;

use crate::{
    dev::DevCommand, generate::GenerateCommand, migrate::MigrateCommand, new::NewCommand, pull::PullCommand, push::PushCommand,
};
use wae_types::WaeResult;

#[derive(clap::Subcommand)]
pub enum Commands {
    /// 数据库迁移相关命令
    #[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
    Migrate {
        #[command(subcommand)]
        subcommand: MigrateCommand,
    },
    /// 创建新项目脚手架
    New(NewCommand),
    /// 热重载开发服务器
    Dev(DevCommand),
    /// 从 OpenAPI/Swagger 生成代码
    Generate(GenerateCommand),
    /// 从远程同步 WAE 文件
    Pull(PullCommand),
    /// 推送到数据库
    Push(PushCommand),
}

// 共享参数
pub struct Arguements {}

impl Commands {
    pub async fn run(&self, _args: &Arguements) -> WaeResult<()> {
        match self {
            #[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
            Commands::Migrate { subcommand } => {
                if let Err(e) = subcommand.run().await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Commands::New(cmd) => {
                if let Err(e) = cmd.run() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Commands::Dev(cmd) => {
                if let Err(e) = cmd.run().await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Commands::Generate(cmd) => {
                if let Err(e) = cmd.run().await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Commands::Pull(cmd) => {
                if let Err(e) = cmd.run().await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Commands::Push(cmd) => {
                if let Err(e) = cmd.run().await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Ok(())
    }
}
