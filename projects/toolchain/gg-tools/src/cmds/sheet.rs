#![warn(missing_docs)]

//! `gg sheet` 命令实现
//!
//! 配置表管理工具，直接调用 gg-sheet 库 API

use std::path::PathBuf;

use crate::{GError, GErrorKind, GResult, commands::SheetCommands};

/// 执行 `sheet` 子命令
pub fn cmd_sheet(command: &SheetCommands) -> GResult<()> {
    match command {
        SheetCommands::Check { workspace } => cmd_check(workspace),
        SheetCommands::Generate { workspace } => cmd_generate(workspace),
    }
}

/// 执行 check 子命令
fn cmd_check(workspace: &str) -> GResult<()> {
    let config = gg_sheet::config::SheetConfig::load_from_workspace(&PathBuf::from(workspace))
        .map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })?;

    let sheet_dir = PathBuf::from(workspace).join(&config.sheet_dir);
    let output_dir = PathBuf::from(workspace).join(&config.output_dir);

    let compiler = gg_sheet::compiler::SheetCompiler::new(&sheet_dir, &output_dir);
    let files = compiler.scan().map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })?;

    let mut tables: Vec<gg_sheet::schema::SheetTable> = Vec::new();
    for path in &files {
        let raw_table =
            gg_sheet::reader::load_table(path).map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })?;
        let table = gg_sheet::schema::SheetTable::from_raw(raw_table)
            .map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })?;
        tables.push(table);
    }

    let report = gg_sheet::validate::validate_tables(&tables);
    println!("{}", report.format_report());

    if !report.is_valid() {
        std::process::exit(1);
    }

    Ok(())
}

/// 执行 generate 子命令
fn cmd_generate(workspace: &str) -> GResult<()> {
    let config = gg_sheet::config::SheetConfig::load_from_workspace(&PathBuf::from(workspace))
        .map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })?;

    let sheet_dir = PathBuf::from(workspace).join(&config.sheet_dir);
    let output_dir = PathBuf::from(workspace).join(&config.output_dir);

    let mut compiler = gg_sheet::compiler::SheetCompiler::new(&sheet_dir, &output_dir);
    compiler.compile().map_err(|e| GError { kind: GErrorKind::Other, message: e.to_string() })?;

    println!("代码生成完成");
    Ok(())
}
