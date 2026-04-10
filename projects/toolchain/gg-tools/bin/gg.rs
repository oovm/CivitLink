#![warn(missing_docs)]

//! GG 元游戏引擎命令行工具
//!
//! 提供子命令用于初始化引擎项目、生成引擎代码、构建和创建游戏项目，
//! 以及脚本转换和性能测试等工具功能。

mod init;
mod generate;
mod build;
mod new_game;
mod mod_converter;
mod script_bench;

use gg_tools::Cli;

fn main() -> gg_core::GResult<()> {
    let cli = Cli::parse();

    match cli.command {
        gg_tools::Commands::Init { name, r#type } => init::cmd_init(&name, &r#type),
        gg_tools::Commands::Generate { manifest } => generate::cmd_generate(&manifest),
        gg_tools::Commands::Build { manifest, platform, release } => build::cmd_build(&manifest, platform.as_deref(), release),
        gg_tools::Commands::NewGame { name } => new_game::cmd_new_game(&name),
        gg_tools::Commands::ModConverter { path } => mod_converter::cmd_mod_converter(path.as_deref()),
        gg_tools::Commands::ScriptBench => script_bench::cmd_script_bench(),
    }
}
