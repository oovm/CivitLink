#![warn(missing_docs)]

//! GG-Sheet 命令行工具入口
//! 配置表编译工具的 binary 入口点

fn main() {
    let cli = gg_sheet::cli::SheetCli::parse();
    if let Err(e) = cli.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
