//! Rust 代码生成模块
//! 将结构化表格数据生成类型安全的 Rust 数据结构和查找函数

use crate::{
    codegen::CodegenConfig,
    error::SheetResult,
    schema::{SheetHeader, SheetTable, TableKind},
    types::SheetType,
};

/// 为表格生成 Rust 代码
pub fn generate_table_rust(table: &SheetTable, _config: &CodegenConfig) -> SheetResult<String> {
    match table.kind {
        TableKind::List => generate_list_table_rust(table),
        TableKind::Dict => generate_dict_table_rust(table),
        TableKind::Enumerate => generate_enum_table_rust(table),
        TableKind::Class => generate_class_table_rust(table),
        TableKind::Language => generate_language_table_rust(table),
    }
}

/// 将 SheetType 转换为 Rust 类型字符串
fn to_rust_type(ty: &SheetType) -> String {
    match ty {
        SheetType::Boolean => "bool".to_string(),
        SheetType::Integer(_) => "i32".to_string(),
        SheetType::Decimal(_) => "f64".to_string(),
        SheetType::String => "String".to_string(),
        SheetType::Color => "String".to_string(),
        SheetType::Vector(_) => "String".to_string(),
        SheetType::List(inner) => format!("Vec<{}>", to_rust_type(inner)),
        SheetType::Map { value, .. } => format!("HashMap<String, {}>", to_rust_type(value)),
        SheetType::Optional(inner) => format!("Option<{}>", to_rust_type(inner)),
        SheetType::Enumerate(name) => name.clone(),
        SheetType::Reference(_) => "String".to_string(),
    }
}

/// 将字段值格式化为 Rust 字面量
fn format_rust_value(value: &str, ty: &SheetType) -> String {
    let trimmed = value.trim();
    match ty {
        SheetType::Boolean => match trimmed.to_lowercase().as_str() {
            "true" | "1" => "true".to_string(),
            _ => "false".to_string(),
        },
        SheetType::Integer(_) | SheetType::Reference(_) => {
            if trimmed.is_empty() { "0".to_string() } else { trimmed.to_string() }
        }
        SheetType::Decimal(_) => {
            if trimmed.is_empty() {
                "0.0".to_string()
            } else {
                format!("{}{}", trimmed, if trimmed.contains('.') { "" } else { ".0" })
            }
        }
        SheetType::String | SheetType::Color | SheetType::Vector(_) => {
            format!("\"{}\".to_string()", escape_rust_string(trimmed))
        }
        SheetType::List(inner) => {
            if trimmed.is_empty() {
                "Vec::new()".to_string()
            } else {
                let items: Vec<String> =
                    trimmed.split(',').map(|s| format_rust_value(s.trim(), inner)).collect();
                format!("vec![{}]", items.join(", "))
            }
        }
        SheetType::Map { .. } => "HashMap::new()".to_string(),
        SheetType::Optional(inner) => {
            if trimmed.is_empty() {
                "None".to_string()
            } else {
                format!("Some({})", format_rust_value(trimmed, inner))
            }
        }
        SheetType::Enumerate(name) => {
            if trimmed.is_empty() {
                format!("{}::None", name)
            } else {
                format!("{}::{}", name, trimmed)
            }
        }
    }
}

/// 转义 Rust 字符串字面量中的特殊字符
fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}

/// 将 PascalCase 转换为 snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c);
        }
    }
    result
}

/// 为列表表生成 Rust 代码
fn generate_list_table_rust(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = format!("{}Row", table_name);
    let table_struct_name = format!("{}Table", table_name);

    let mut output = String::new();

    output.push_str(&format!("//! Auto-generated from {} sheet\n\n", table_name));
    output.push_str("use std::collections::HashMap;\n\n");

    output.push_str(&format!("/// Row data for {} table\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", row_name));
    for header in &table.headers {
        let rust_type = to_rust_type(&header.typing);
        if !header.comment.is_empty() {
            output.push_str(&format!("    /// {}\n", header.comment));
        }
        output.push_str(&format!("    pub {}: {},\n", header.field_name, rust_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("/// Table data for {} table\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", table_struct_name));
    output.push_str(&format!("    rows: HashMap<i32, {}>,\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("impl {} {{\n", table_struct_name));
    output.push_str("    /// Load table data\n");
    output.push_str("    pub fn load() -> Self {\n");
    output.push_str(&format!(
        "        let mut table = {} {{ rows: HashMap::new() }};\n",
        table_struct_name
    ));
    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or("0");
        let fields: Vec<String> = table
            .headers
            .iter()
            .zip(row.iter())
            .map(|(header, value)| format!("{}: {}", header.field_name, format_rust_value(value, &header.typing)))
            .collect();
        output.push_str(&format!(
            "        table.rows.insert({}, {} {{ {} }});\n",
            key,
            row_name,
            fields.join(", ")
        ));
    }
    output.push_str("        table\n");
    output.push_str("    }\n\n");
    output.push_str("    /// Get a row by id\n");
    output.push_str(&format!("    pub fn get(&self, id: i32) -> Option<&{}> {{\n", row_name));
    output.push_str("        self.rows.get(&id)\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 为字典表生成 Rust 代码
fn generate_dict_table_rust(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = format!("{}Row", table_name);
    let table_struct_name = format!("{}Table", table_name);

    let mut output = String::new();

    output.push_str(&format!("//! Auto-generated from {} sheet\n\n", table_name));
    output.push_str("use std::collections::HashMap;\n\n");

    output.push_str(&format!("/// Row data for {} table\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", row_name));
    for header in &table.headers {
        let rust_type = to_rust_type(&header.typing);
        if !header.comment.is_empty() {
            output.push_str(&format!("    /// {}\n", header.comment));
        }
        output.push_str(&format!("    pub {}: {},\n", header.field_name, rust_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("/// Table data for {} table\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", table_struct_name));
    output.push_str(&format!("    rows: HashMap<String, {}>,\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("impl {} {{\n", table_struct_name));
    output.push_str("    /// Load table data\n");
    output.push_str("    pub fn load() -> Self {\n");
    output.push_str(&format!(
        "        let mut table = {} {{ rows: HashMap::new() }};\n",
        table_struct_name
    ));
    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or("");
        let fields: Vec<String> = table
            .headers
            .iter()
            .zip(row.iter())
            .map(|(header, value)| format!("{}: {}", header.field_name, format_rust_value(value, &header.typing)))
            .collect();
        output.push_str(&format!(
            "        table.rows.insert(\"{}\".to_string(), {} {{ {} }});\n",
            escape_rust_string(key),
            row_name,
            fields.join(", ")
        ));
    }
    output.push_str("        table\n");
    output.push_str("    }\n\n");
    output.push_str("    /// Get a row by key\n");
    output.push_str(&format!("    pub fn get(&self, key: &str) -> Option<&{}> {{\n", row_name));
    output.push_str("        self.rows.get(key)\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 为枚举表生成 Rust 代码
fn generate_enum_table_rust(table: &SheetTable) -> SheetResult<String> {
    let enum_name = &table.name;
    let enum_col = table.enum_name_column().unwrap_or(0);
    let id_col = table.enum_id_column().unwrap_or(1);

    let mut output = String::new();

    output.push_str(&format!("//! Auto-generated from {} sheet\n\n", enum_name));

    output.push_str(&format!("/// {} enumeration\n", enum_name));
    output.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    output.push_str(&format!("pub enum {} {{\n", enum_name));
    for row in &table.rows {
        let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
        if variant_name.is_empty() {
            continue;
        }
        output.push_str(&format!("    {},\n", variant_name));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("impl {} {{\n", enum_name));
    output.push_str("    /// Get the numeric id for this variant\n");
    output.push_str("    pub fn id(&self) -> i32 {\n");
    output.push_str("        match self {\n");
    for row in &table.rows {
        let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
        let variant_id = row.get(id_col).map(|s| s.as_str()).unwrap_or("0").trim();
        if variant_name.is_empty() {
            continue;
        }
        output.push_str(&format!("            {}::{} => {},\n", enum_name, variant_name, variant_id));
    }
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 为单例配置表生成 Rust 代码
fn generate_class_table_rust(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let class_name = table_name.clone();
    let getter_name = format!("get_{}", to_snake_case(table_name));

    let mut output = String::new();

    output.push_str(&format!("//! Auto-generated from {} sheet\n\n", table_name));

    output.push_str(&format!("/// {} configuration\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", class_name));
    for header in &table.headers {
        let rust_type = to_rust_type(&header.typing);
        if !header.comment.is_empty() {
            output.push_str(&format!("    /// {}\n", header.comment));
        }
        output.push_str(&format!("    pub {}: {},\n", header.field_name, rust_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("impl {} {{\n", class_name));
    output.push_str("    /// Load configuration from sheet data\n");
    output.push_str("    pub fn load() -> Self {\n");
    if let Some(first_row) = table.rows.first() {
        let fields: Vec<String> = table
            .headers
            .iter()
            .zip(first_row.iter())
            .map(|(header, value)| format!("{}: {}", header.field_name, format_rust_value(value, &header.typing)))
            .collect();
        output.push_str(&format!("        {} {{ {} }}\n", class_name, fields.join(", ")));
    } else {
        output.push_str(&format!("        {} {{ /* no data */ }}\n", class_name));
    }
    output.push_str("    }\n");
    output.push_str("}\n\n");

    output.push_str(&format!("/// Get the global {} configuration\n", table_name));
    output.push_str(&format!("pub fn {}() -> {} {{\n", getter_name, class_name));
    output.push_str(&format!("    {}::load()\n", class_name));
    output.push_str("}\n");

    Ok(output)
}

/// 为多语言表生成 Rust 代码
fn generate_language_table_rust(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = table_name.clone();
    let table_struct_name = format!("{}Table", table_name);
    let lang_fields: Vec<&SheetHeader> = table.headers.iter().filter(|h| h.field_name != "key").collect();

    let mut output = String::new();

    output.push_str(&format!("//! Auto-generated from {} sheet\n\n", table_name));
    output.push_str("use std::collections::HashMap;\n\n");

    output.push_str(&format!("/// Language entry for {} table\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", row_name));
    for header in &table.headers {
        let rust_type = to_rust_type(&header.typing);
        if !header.comment.is_empty() {
            output.push_str(&format!("    /// {}\n", header.comment));
        }
        output.push_str(&format!("    pub {}: {},\n", header.field_name, rust_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("/// Language table for {}\n", table_name));
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str(&format!("pub struct {} {{\n", table_struct_name));
    output.push_str(&format!("    entries: HashMap<String, {}>,\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("impl {} {{\n", table_struct_name));
    output.push_str("    /// Load language table\n");
    output.push_str("    pub fn load() -> Self {\n");
    output.push_str(&format!(
        "        let mut table = {} {{ entries: HashMap::new() }};\n",
        table_struct_name
    ));
    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or("");
        let fields: Vec<String> = table
            .headers
            .iter()
            .zip(row.iter())
            .map(|(header, value)| format!("{}: {}", header.field_name, format_rust_value(value, &header.typing)))
            .collect();
        output.push_str(&format!(
            "        table.entries.insert(\"{}\".to_string(), {} {{ {} }});\n",
            escape_rust_string(key),
            row_name,
            fields.join(", ")
        ));
    }
    output.push_str("        table\n");
    output.push_str("    }\n\n");
    output.push_str("    /// Get a language entry by key\n");
    output.push_str(&format!("    pub fn get(&self, key: &str) -> Option<&{}> {{\n", row_name));
    output.push_str("        self.entries.get(key)\n");
    output.push_str("    }\n\n");
    output.push_str("    /// Get translated text by key and language code\n");
    output.push_str("    pub fn get_text(&self, key: &str, lang: &str) -> &str {\n");
    output.push_str("        if let Some(entry) = self.entries.get(key) {\n");
    output.push_str("            match lang {\n");
    for field in &lang_fields {
        output.push_str(&format!(
            "                \"{}\" => &entry.{},\n",
            field.field_name, field.field_name
        ));
    }
    if let Some(first) = lang_fields.first() {
        output.push_str(&format!("                _ => &entry.{},\n", first.field_name));
    }
    output.push_str("            }\n");
    output.push_str("        } else {\n");
    output.push_str("            \"\"\n");
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}
