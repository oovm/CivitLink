use clap::Parser;
use wae_tools::cmds::Commands;

/// WAE Tools CLI - 项目脚手架、代码生成、数据库迁移命令行工具
#[derive(Parser)]
#[command(name = "wae-tools", about = "WAE Tools CLI", long_about = None, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.command.run(&wae_tools::cmds::Arguements {}).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
