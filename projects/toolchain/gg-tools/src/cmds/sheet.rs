#![warn(missing_docs)]

//! `gg sheet` 命令实现
//!
//! 配置表管理工具，透传调用 gg-sheet 的 CLI 逻辑

use crate::{GError, GErrorKind, GResult, commands::SheetCommands};

/// 执行 `sheet` 子命令
///
/// 将 gg-tools 的 SheetCommands 转换为 gg-sheet 的 CLI 命令并执行
pub fn cmd_sheet(command: &SheetCommands) -> GResult<()> {
    let (workspace, sheet_command) = match command {
        SheetCommands::Init { workspace } => (workspace.as_str(), gg_sheet::cli::SheetCommands::Init),
        SheetCommands::Check { workspace } => (workspace.as_str(), gg_sheet::cli::SheetCommands::Check),
        SheetCommands::Generate { workspace, format } => {
            (workspace.as_str(), gg_sheet::cli::SheetCommands::Generate { format: format.clone() })
        }
        SheetCommands::Watch { workspace } => (workspace.as_str(), gg_sheet::cli::SheetCommands::Watch),
    };

    let cli =
        gg_sheet::cli::SheetCli { command: sheet_command, workspace: std::path::PathBuf::from(workspace), verbose: false };

    cli.run().map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })
}
