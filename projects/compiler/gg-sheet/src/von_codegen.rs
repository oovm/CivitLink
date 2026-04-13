//! VON 数据文件代码生成模块
//! 将结构化表格数据生成 VON 格式的数据文件

use crate::{
    error::SheetResult,
    schema::{SheetTable, TableKind},
    types::{self, SheetType},
};

/// 为表格生成 VON 格式数据
///
/// 根据表格类型生成对应的 VON 数据文件内容，
/// 支持 Dict/List、Enum、Class 和 Language 四种表格类型。
pub fn generate_von_data(table: &SheetTable) -> SheetResult<String> {
    match table.kind {
        TableKind::List | TableKind::Dict => generate_dict_list_von(table),
        TableKind::Enumerate => generate_enum_von(table),
        TableKind::Class => generate_class_von(table),
        TableKind::Language => generate_language_von(table),
    }
}

/// 格式化值为 VON 格式字面量
///
/// 根据字段类型将字符串值转换为 VON 格式的字面量表示：
/// - 布尔值：`true` 或 `false`
/// - 整数：直接数字，空值转换为 `0`
/// - 浮点数：带小数点的数字，空值转换为 `0.0`
/// - 字符串：`"escaped_value"`，双引号转义
/// - 颜色：`"#RRGGBB"` 字符串格式
/// - 向量：字符串格式
/// - 列表：`[item1, item2, ...]`
/// - 映射：字符串格式
/// - 可选：空值转换为 `null`，否则使用内部值格式
/// - 枚举：`EnumName.VariantName`
/// - 引用：直接值（键值）
pub fn format_von_value(value: &str, ty: &SheetType) -> String {
    let trimmed = value.trim();
    match ty {
        SheetType::Boolean => match trimmed.to_lowercase().as_str() {
            "true" | "1" => "true".to_string(),
            _ => "false".to_string(),
        },
        SheetType::Integer(_) | SheetType::Reference(_) => {
            if trimmed.is_empty() {
                "0".to_string()
            }
            else {
                trimmed.to_string()
            }
        }
        SheetType::Decimal(_) => {
            if trimmed.is_empty() {
                "0.0".to_string()
            }
            else {
                trimmed.to_string()
            }
        }
        SheetType::String => format!("\"{}\"", escape_von_string(trimmed)),
        SheetType::Color => {
            if trimmed.is_empty() {
                "Color { r: 0, g: 0, b: 0, a: 255 }".to_string()
            }
            else if let Ok((r, g, b, a)) = types::parse_color_str(trimmed) {
                format!("Color {{ r: {}, g: {}, b: {}, a: {} }}", r, g, b, a)
            }
            else {
                "Color { r: 0, g: 0, b: 0, a: 255 }".to_string()
            }
        }
        SheetType::Vector(_) => {
            if trimmed.is_empty() {
                "[0.0, 0.0]".to_string()
            }
            else if let Ok(components) = types::parse_vector_str(trimmed) {
                let formatted: Vec<String> = components.iter().map(|c| format_von_f32(*c)).collect();
                format!("[{}]", formatted.join(", "))
            }
            else {
                "[0.0, 0.0]".to_string()
            }
        }
        SheetType::List(inner) => {
            if trimmed.is_empty() {
                "[]".to_string()
            }
            else {
                let items: Vec<String> = trimmed.split(',').map(|s| format_von_value(s.trim(), inner)).collect();
                format!("[{}]", items.join(", "))
            }
        }
        SheetType::Map { value, .. } => {
            if trimmed.is_empty() {
                return "{}".to_string();
            }
            let entries = match types::parse_map_str(trimmed) {
                Ok(e) => e,
                Err(_) => return "{}".to_string(),
            };
            if entries.is_empty() {
                return "{}".to_string();
            }
            let formatted_entries: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let formatted_value = format_von_value(v, value);
                    format!("\"{}\": {}", escape_von_string(k), formatted_value)
                })
                .collect();
            format!("{{ {} }}", formatted_entries.join(", "))
        }
        SheetType::Optional(inner) => {
            if trimmed.is_empty() {
                "null".to_string()
            }
            else {
                format_von_value(trimmed, inner)
            }
        }
        SheetType::Enumerate(name) => {
            if trimmed.is_empty() {
                format!("{}.None", name)
            }
            else {
                format!("{}.{}", name, trimmed)
            }
        }
    }
}

/// 生成字典/列表表的 VON 数据
///
/// 字典表使用字符串键加引号，列表表使用整数键不加引号。
/// 每行数据生成一个以表名为类名的 VON 对象。
fn generate_dict_list_von(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let class_name = format!("{}Table", table_name);
    let is_list = table.kind == TableKind::List;

    let mut output = String::new();

    output.push_str(&format!("# {} data\n", table_name));
    output.push_str(&format!("{} {{\n", class_name));
    output.push_str("    _data: {\n");

    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or(if is_list { "0" } else { "" });
        let key_formatted = if is_list { key.to_string() } else { format!("\"{}\"", escape_von_string(key)) };

        output.push_str(&format!("        {}: {} {{\n", key_formatted, table_name));

        for header in &table.headers {
            let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");
            let formatted = format_von_value(value, &header.typing);
            output.push_str(&format!("            {}: {},\n", header.field_name, formatted));
        }

        output.push_str("        },\n");
    }

    output.push_str("    },\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成枚举表的 VON 数据
///
/// 枚举表生成 `_data` 映射和 `_values` 数组两部分。
/// `_data` 以枚举变体名为键，`_values` 按顺序包含所有枚举变体。
fn generate_enum_von(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let class_name = format!("{}Table", table_name);
    let enum_col = table.enum_name_column().unwrap_or(0);

    let mut output = String::new();

    output.push_str(&format!("# {} data\n", table_name));
    output.push_str(&format!("{} {{\n", class_name));
    output.push_str("    _data: {\n");

    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
        if variant_name.is_empty() {
            continue;
        }

        output.push_str(&format!("        \"{}\": {} {{\n", escape_von_string(variant_name), table_name));

        for header in &table.headers {
            let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");
            let formatted = format_von_value(value, &header.typing);
            output.push_str(&format!("            {}: {},\n", header.field_name, formatted));
        }

        output.push_str("        },\n");
    }

    output.push_str("    },\n");
    output.push_str("    _values: [\n");

    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
        if variant_name.is_empty() {
            continue;
        }

        output.push_str(&format!("        {} {{\n", table_name));

        for header in &table.headers {
            let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");
            let formatted = format_von_value(value, &header.typing);
            output.push_str(&format!("            {}: {},\n", header.field_name, formatted));
        }

        output.push_str("        },\n");
    }

    output.push_str("    ],\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成单例配置表的 VON 数据
///
/// 单例配置表只有一行数据，`_data` 直接是该行的对象而非映射。
fn generate_class_von(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let class_name = format!("{}Table", table_name);

    let mut output = String::new();

    output.push_str(&format!("# {} data\n", table_name));
    output.push_str(&format!("{} {{\n", class_name));
    output.push_str(&format!("    _data: {} {{\n", table_name));

    if let Some(row) = table.rows.first() {
        for header in &table.headers {
            let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");
            let formatted = format_von_value(value, &header.typing);
            output.push_str(&format!("        {}: {},\n", header.field_name, formatted));
        }
    }

    output.push_str("    },\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成多语言表的 VON 数据
///
/// 多语言表以 key 字段值为映射键，每行包含各语言翻译字段。
fn generate_language_von(table: &SheetTable) -> SheetResult<String> {
    let table_name = &table.name;
    let class_name = format!("{}Table", table_name);

    let mut output = String::new();

    output.push_str(&format!("# {} data\n", table_name));
    output.push_str(&format!("{} {{\n", class_name));
    output.push_str("    _data: {\n");

    for row in &table.rows {
        if row.is_empty() {
            continue;
        }
        let key = row.first().map(|s| s.as_str()).unwrap_or("");

        output.push_str(&format!("        \"{}\": {} {{\n", escape_von_string(key), table_name));

        for header in &table.headers {
            let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");
            let formatted = format_von_value(value, &header.typing);
            output.push_str(&format!("            {}: {},\n", header.field_name, formatted));
        }

        output.push_str("        },\n");
    }

    output.push_str("    },\n");
    output.push_str("}\n");

    Ok(output)
}

/// 转义 VON 字符串中的特殊字符
///
/// 将反斜杠转义为双反斜杠，将双引号转义为反斜杠加双引号。
fn escape_von_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 格式化 f32 为 VON 格式字符串，确保整数显示为带小数点形式
fn format_von_f32(v: f32) -> String {
    if v.fract() == 0.0 { format!("{:.1}", v) } else { format!("{}", v) }
}
