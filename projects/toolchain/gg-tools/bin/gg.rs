#![warn(missing_docs)]

//! GG 元游戏引擎命令行工具
//!
//! 提供子命令用于初始化引擎项目、生成引擎代码、构建和创建游戏项目，
//! 以及脚本转换和性能测试等工具功能。

fn main() -> gg_tools::GResult<()> {
    let cli = gg_tools::Cli::parse();

    match cli.command {
        gg_tools::Commands::Init { name, r#type } => gg_tools::cmds::init::cmd_init(&name, &r#type),
        gg_tools::Commands::Generate { manifest } => gg_tools::cmds::generate::cmd_generate(&manifest),
        gg_tools::Commands::Build { manifest, platform, release } => {
            gg_tools::cmds::build::cmd_build(&manifest, platform.as_deref(), release)
        }
        gg_tools::Commands::Package { manifest, platform, release } => {
            gg_tools::cmds::package::cmd_package(&manifest, platform.as_deref(), release)
        }
        gg_tools::Commands::Sheet { command } => gg_tools::cmds::sheet::cmd_sheet(&command),
        gg_tools::Commands::NewGame { name } => gg_tools::cmds::new_game::cmd_new_game(&name),
        gg_tools::Commands::ModConverter { path } => gg_tools::cmds::mod_converter::cmd_mod_converter(path.as_deref()),
        gg_tools::Commands::ScriptBench => gg_tools::cmds::script_bench::cmd_script_bench(),
        gg_tools::Commands::Meta { target, recursive } => {
            let platform = gg_tools::platform::Platform::new()?;
            let args = gg_tools::cmds::meta::MetaArgs { target, recursive };
            gg_tools::cmds::meta::execute(&args, &platform)
        }
    }
}
