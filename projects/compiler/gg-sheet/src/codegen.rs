//! Valkyrie 脚本代码生成模块
//! 将结构化表格数据生成 .v 脚本文件

use std::path::PathBuf;

use crate::error::SheetResult;
use crate::schema::{SheetHeader, SheetTable, TableKind};
use crate::types::SheetType;

/// 代码生成配置
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 命名空间前缀（可选）
    pub namespace: Option<String>,
}

impl CodegenConfig {
    /// 创建新的代码生成配置
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir, namespace: None }
    }
}

/// 为表格生成 Valkyrie 脚本代码
pub fn generate_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    match table.kind {
        TableKind::List => generate_list_table(table, config),
        TableKind::Dict => generate_dict_table(table, config),
        TableKind::Enumerate => generate_enum_table(table, config),
    }
}

/// 生成列表表的 Valkyrie 脚本代码
pub fn generate_list_table(table: &SheetTable, _config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = format!("{}Row", table_name);
    let class_name = format!("{}Table", table_name);

    let mut output = String::new();

    output.push_str(&format!("// {} - 自动生成的配置表\n\n", class_name));

    output.push_str(&format!("struct {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        if !header.comment.is_empty() {
            output.push_str(&format!("    /// {}\n", header.comment));
        }
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    output.push_str(&format!("    rows: Map<i32, {}>\n\n", row_name));
    output.push_str(&format!("    micro get(id: i32): {} {{\n", row_name));
    output.push_str("        return rows[id]\n");
    output.push_str("    }\n");

    output.push_str("\n    micro load() {\n");
    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or("0");
        let fields: Vec<String> = table
            .headers
            .iter()
            .zip(row.iter())
            .map(|(header, value)| format_field_value(value, &header.typing))
            .collect();
        output.push_str(&format!("        rows[{}] = {}({})\n", key, row_name, fields.join(", ")));
    }
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成字典表的 Valkyrie 脚本代码
pub fn generate_dict_table(table: &SheetTable, _config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = format!("{}Row", table_name);
    let class_name = format!("{}Table", table_name);

    let mut output = String::new();

    output.push_str(&format!("// {} - 自动生成的配置表\n\n", class_name));

    output.push_str(&format!("struct {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        if !header.comment.is_empty() {
            output.push_str(&format!("    /// {}\n", header.comment));
        }
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    output.push_str(&format!("    rows: Map<string, {}>\n\n", row_name));
    output.push_str(&format!("    micro get(key: string): {} {{\n", row_name));
    output.push_str("        return rows[key]\n");
    output.push_str("    }\n");

    output.push_str("\n    micro load() {\n");
    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or("");
        let fields: Vec<String> = table
            .headers
            .iter()
            .zip(row.iter())
            .map(|(header, value)| format_field_value(value, &header.typing))
            .collect();
        output.push_str(&format!("        rows[\"{}\"] = {}({})\n", escape_string(key), row_name, fields.join(", ")));
    }
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成枚举表的 Valkyrie 脚本代码
pub fn generate_enum_table(table: &SheetTable, _config: &CodegenConfig) -> SheetResult<String> {
    let enum_name = &table.name;
    let class_name = format!("{}Table", table.name);

    let enum_col = table.enum_name_column().unwrap_or(0);
    let id_col = table.enum_id_column().unwrap_or(1);

    let mut output = String::new();

    output.push_str(&format!("// {} - 自动生成的枚举表\n\n", class_name));

    output.push_str(&format!("enum {} {{\n", enum_name));
    for row in &table.rows {
        let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
        let variant_id = row.get(id_col).map(|s| s.as_str()).unwrap_or("0").trim();
        if variant_name.is_empty() {
            continue;
        }
        output.push_str(&format!("    {} = {}\n", variant_name, variant_id));
    }
    output.push_str("}\n");

    let extra_headers: Vec<&SheetHeader> = table
        .headers
        .iter()
        .filter(|h| h.field_name != "enum" && h.field_name != "id")
        .collect();

    if !extra_headers.is_empty() {
        output.push_str(&format!("\nclass {} {{\n", class_name));
        for header in &extra_headers {
            let field_type = header.typing.to_valkyrie_type();
            output.push_str(&format!("    {}: Map<{}, {}>\n", header.field_name, enum_name, field_type));
        }
        output.push_str("\n    micro load() {\n");
        for row in &table.rows {
            let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
            if variant_name.is_empty() {
                continue;
            }
            for header in &extra_headers {
                let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");
                output.push_str(&format!(
                    "        {}[{}.{}] = {}\n",
                    header.field_name,
                    enum_name,
                    variant_name,
                    format_field_value(value, &header.typing)
                ));
            }
        }
        output.push_str("    }\n");
        output.push_str("}\n");
    }

    Ok(output)
}

/// 格式化字段值为 Valkyrie 脚本字面量
fn format_field_value(value: &str, ty: &SheetType) -> String {
    let trimmed = value.trim();
    match ty {
        SheetType::Boolean => {
            match trimmed.to_lowercase().as_str() {
                "true" | "1" => "true".to_string(),
                _ => "false".to_string(),
            }
        }
        SheetType::Integer(_) | SheetType::Reference(_) => {
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        }
        SheetType::Decimal(_) => {
            if trimmed.is_empty() {
                "0.0".to_string()
            } else {
                trimmed.to_string()
            }
        }
        SheetType::String => format!("\"{}\"", escape_string(trimmed)),
        SheetType::Color => format!("\"{}\"", escape_string(trimmed)),
        SheetType::Vector(_) => format!("\"{}\"", escape_string(trimmed)),
        SheetType::List(inner) => {
            if trimmed.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = trimmed
                    .split(',')
                    .map(|s| format_field_value(s.trim(), inner))
                    .collect();
                format!("[{}]", items.join(", "))
            }
        }
        SheetType::Map { .. } => format!("\"{}\"", escape_string(trimmed)),
        SheetType::Optional(inner) => {
            if trimmed.is_empty() {
                "null".to_string()
            } else {
                format_field_value(trimmed, inner)
            }
        }
        SheetType::Enumerate(name) => {
            if trimmed.is_empty() {
                format!("{}.None", name)
            } else {
                format!("{}.{}", name, trimmed)
            }
        }
    }
}

/// 转义字符串中的特殊字符
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
