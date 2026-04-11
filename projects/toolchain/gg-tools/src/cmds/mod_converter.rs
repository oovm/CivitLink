#![warn(missing_docs)]

//! `gg mod-converter` 命令实现
//!
//! WASM 模块检查与转换工具

use crate::{GError, GErrorKind, GResult};
use std::path::Path;

/// WASM 文件魔数（\0asm）
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// WASM 模块版本号（1.0）
const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// WASM section ID 到名称的映射
fn section_name(id: u8) -> &'static str {
    match id {
        0 => "custom",
        1 => "type",
        2 => "import",
        3 => "function",
        4 => "table",
        5 => "memory",
        6 => "global",
        7 => "export",
        8 => "start",
        9 => "element",
        10 => "code",
        11 => "data",
        12 => "data count",
        _ => "unknown",
    }
}

/// WASM 模块检查结果
pub struct WasmInspection {
    /// 文件大小（字节）
    pub file_size: u64,
    /// WASM 版本号
    pub version: u32,
    /// section 数量
    pub section_count: usize,
    /// 各 section 的简要信息
    pub sections: Vec<WasmSectionInfo>,
}

/// WASM section 简要信息
pub struct WasmSectionInfo {
    /// section ID
    pub id: u8,
    /// section 名称
    pub name: &'static str,
    /// section 大小（字节）
    pub size: usize,
}

/// 执行 `mod-converter` 子命令
///
/// 检查 WASM 模块文件的基本信息，包括文件大小、WASM 版本和 section 列表。
/// 如果未提供路径参数，则打印使用说明。
pub fn cmd_mod_converter(wasm_path: Option<&str>) -> GResult<()> {
    let path_str = match wasm_path {
        Some(p) => p,
        None => {
            print_usage();
            return Ok(());
        }
    };

    let path = Path::new(path_str);
    if !path.exists() {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: format!("WASM module file not found: {}", path_str),
        });
    }

    let inspection = inspect_wasm(path)?;

    println!("WASM Module Inspection");
    println!("======================");
    println!();
    println!("  File:       {}", path.display());
    println!("  Size:       {} bytes", inspection.file_size);
    println!("  Version:    {}.{}", inspection.version, 0);
    println!("  Sections:   {}", inspection.section_count);
    println!();

    if !inspection.sections.is_empty() {
        println!("  {:<4} {:<12} {:<10}", "ID", "Name", "Size");
        println!("  {:<4} {:<12} {:<10}", "--", "----", "----");
        for sec in &inspection.sections {
            println!(
                "  {:<4} {:<12} {:<10}",
                sec.id, sec.name, sec.size
            );
        }
    }

    Ok(())
}

/// 打印使用说明
fn print_usage() {
    println!("GG Game Engine - WASM Module Inspector");
    println!("=======================================");
    println!();
    println!("Usage: gg mod-converter <wasm-path>");
    println!();
    println!("Inspects a WASM module file and displays basic information:");
    println!("  - File size");
    println!("  - WASM version");
    println!("  - Number of sections and their details");
    println!();
    println!("Example:");
    println!("  gg mod-converter ./my_module.wasm");
}

/// 检查 WASM 模块文件
///
/// 读取文件内容，验证 WASM 魔数和版本号，解析 section 列表。
fn inspect_wasm(path: &Path) -> GResult<WasmInspection> {
    let data = std::fs::read(path).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read WASM file '{}': {}", path.display(), e),
    })?;

    let file_size = data.len() as u64;

    if data.len() < 8 {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: "Invalid WASM file: too short to contain header".to_string(),
        });
    }

    if data[0..4] != WASM_MAGIC {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: format!(
                "Invalid WASM file: bad magic number (expected \\0asm, got {:02x?})",
                &data[0..4]
            ),
        });
    }

    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if data[4..8] != WASM_VERSION {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: format!(
                "Unsupported WASM version: {} (only version 1 is supported)",
                version
            ),
        });
    }

    let mut sections = Vec::new();
    let mut offset = 8usize;

    while offset < data.len() {
        let section_id = data[offset];
        offset += 1;

        let (section_size, bytes_consumed) = read_leb128_u32(&data[offset..]);
        offset += bytes_consumed;

        sections.push(WasmSectionInfo {
            id: section_id,
            name: section_name(section_id),
            size: section_size as usize,
        });

        offset += section_size as usize;
    }

    Ok(WasmInspection {
        file_size,
        version,
        section_count: sections.len(),
        sections,
    })
}

/// 读取 LEB128 编码的无符号 32 位整数
///
/// 返回解码后的值和消耗的字节数。
fn read_leb128_u32(data: &[u8]) -> (u32, usize) {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    let mut bytes_consumed = 0;

    for &byte in data {
        bytes_consumed += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    (result, bytes_consumed)
}
