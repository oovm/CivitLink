//! Valkyrie 脚本代码生成模块
//! 将结构化表格数据生成 .script 脚本文件

use std::path::PathBuf;

use crate::{
    error::SheetResult,
    schema::{SheetTable, TableKind},
    types::{self, SheetType, VectorKind},
};

/// 代码生成配置
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 命名空间前缀（可选，默认为 package::table）
    pub namespace: Option<String>,
    /// 延迟加载阈值，行数超过此值的表使用延迟加载
    pub lazy_load_threshold: Option<usize>,
}

impl CodegenConfig {
    /// 创建新的代码生成配置
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir, namespace: None, lazy_load_threshold: None }
    }
}

/// 为表格生成 Valkyrie 脚本代码
pub fn generate_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let use_lazy = config.lazy_load_threshold.map(|threshold| table.rows.len() > threshold).unwrap_or(false);

    if use_lazy {
        generate_lazy_table(table, config)
    }
    else {
        match table.kind {
            TableKind::Dict => generate_dict_table(table, config),
            TableKind::List => generate_list_table(table, config),
            TableKind::Enumerate => generate_enum_table(table, config),
            TableKind::Class => generate_class_table(table, config),
            TableKind::Language => generate_language_table(table, config),
        }
    }
}

/// 生成文件头部注释和命名空间声明
fn generate_header(table_name: &str, namespace: Option<&str>) -> String {
    let ns = namespace.unwrap_or("package::table");
    format!("# {}Table.script, 自动生成，修改无效\nnamespace {}", table_name, ns)
}

/// 生成字典表的 Valkyrie 脚本代码
///
/// 字典表使用字符串键（UTF8Text），首列字段名为 key
fn generate_dict_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = table_name.clone();
    let class_name = format!("{}Table", table_name);
    let namespace = config.namespace.as_deref();

    let mut output = String::new();
    output.push_str(&generate_header(table_name, namespace));
    output.push_str("\n\n");

    output.push_str(&format!("class {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    output.push_str(&format!("    private _data: HashMap<UTF8Text, {}>\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("imply {} {{\n", class_name));
    output.push_str(&format!("    micro load() -> {} {{\n", class_name));
    output.push_str(&format!("        {} {{\n", class_name));
    output.push_str(&format!("            _data: gg.load_sheet(\"{}\")\n", table_name));
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro find(self, key: UTF8Text) -> Option<{}> {{\n", row_name));
    output.push_str("        return self._data[key]\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro all(self) -> Generator<Item={}> {{\n", row_name));
    output.push_str("        return self._data.values()\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成列表表的 Valkyrie 脚本代码
///
/// 列表表使用整数键（i32），首列字段名为 id
fn generate_list_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = table_name.clone();
    let class_name = format!("{}Table", table_name);
    let namespace = config.namespace.as_deref();

    let mut output = String::new();
    output.push_str(&generate_header(table_name, namespace));
    output.push_str("\n\n");

    output.push_str(&format!("class {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    output.push_str(&format!("    private _data: HashMap<i32, {}>\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("imply {} {{\n", class_name));
    output.push_str(&format!("    micro load() -> {} {{\n", class_name));
    output.push_str(&format!("        {} {{\n", class_name));
    output.push_str(&format!("            _data: gg.load_sheet(\"{}\")\n", table_name));
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro find(self, id: i32) -> Option<{}> {{\n", row_name));
    output.push_str("        return self._data[id]\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro all(self) -> Generator<Item={}> {{\n", row_name));
    output.push_str("        return self._data.values()\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成枚举表的 Valkyrie 脚本代码
///
/// 枚举表同时维护 HashMap 和 List，并生成枚举常量对象
fn generate_enum_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = table_name.clone();
    let class_name = format!("{}Table", table_name);
    let enum_const_name = format!("{}Enum", table_name);
    let namespace = config.namespace.as_deref();

    let enum_col = table.enum_name_column().unwrap_or(0);

    let mut output = String::new();
    output.push_str(&generate_header(table_name, namespace));
    output.push_str("\n\n");

    output.push_str(&format!("class {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    output.push_str(&format!("    private _data: HashMap<UTF8Text, {}>\n", row_name));
    output.push_str(&format!("    private _values: List<{}>\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("imply {} {{\n", class_name));
    output.push_str(&format!("    micro load() -> {} {{\n", class_name));
    output.push_str(&format!("        {} {{\n", class_name));
    output.push_str(&format!("            _data: gg.load_sheet(\"{}\"),\n", table_name));
    output.push_str(&format!("            _values: gg.load_sheet_list(\"{}\")\n", table_name));
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro find(self, name: UTF8Text) -> Option<{}> {{\n", row_name));
    output.push_str("        return self._data[name]\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro all(self) -> Generator<Item={}> {{\n", row_name));
    output.push_str("        return self._values.values()\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    let mut variants: Vec<String> = Vec::new();
    for row in &table.rows {
        let variant_name = row.get(enum_col).map(|s| s.as_str()).unwrap_or("").trim();
        if variant_name.is_empty() {
            continue;
        }
        variants.push(variant_name.to_string());
    }

    output.push_str(&format!("const {} = {{\n", enum_const_name));
    for (i, variant) in variants.iter().enumerate() {
        if i < variants.len() - 1 {
            output.push_str(&format!("    {}: \"{}\",\n", variant, variant));
        }
        else {
            output.push_str(&format!("    {}: \"{}\"\n", variant, variant));
        }
    }
    output.push_str("}\n");

    Ok(output)
}

/// 生成单例配置表的 Valkyrie 脚本代码
///
/// 单例配置表仅包含一条数据，生成全局获取函数
fn generate_class_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let class_name = table_name.clone();
    let table_class_name = format!("{}Table", table_name);
    let getter_name = format!("get_{}", to_snake_case(table_name));
    let namespace = config.namespace.as_deref();

    let mut output = String::new();
    output.push_str(&generate_header(table_name, namespace));
    output.push_str("\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", table_class_name));
    output.push_str(&format!("    private _data: {}\n", class_name));
    output.push_str("}\n\n");

    output.push_str(&format!("imply {} {{\n", table_class_name));
    output.push_str(&format!("    micro load() -> {} {{\n", table_class_name));
    output.push_str(&format!("        {} {{\n", table_class_name));
    output.push_str(&format!("            _data: gg.load_sheet(\"{}\")\n", table_name));
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro get(self) -> {} {{\n", class_name));
    output.push_str("        return self._data\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    output.push_str(&format!("micro {}() -> {} {{\n", getter_name, class_name));
    output.push_str(&format!("    static mut config: Option<{}> = None\n", table_class_name));
    output.push_str("    if config == None {\n");
    output.push_str(&format!("        config = Some({}.load())\n", table_class_name));
    output.push_str("    }\n");
    output.push_str("    return config.unwrap().get()\n");
    output.push_str("}\n");

    Ok(output)
}

/// 生成多语言表的 Valkyrie 脚本代码
///
/// 多语言表生成 LanguageManager 和 get_text 辅助函数
fn generate_language_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = table_name.clone();
    let table_class_name = format!("{}Table", table_name);
    let namespace = config.namespace.as_deref();

    let lang_fields: Vec<&crate::schema::SheetHeader> = table.headers.iter().filter(|h| h.field_name != "key").collect();

    let mut output = String::new();
    output.push_str(&generate_header(table_name, namespace));
    output.push_str("\n\n");

    output.push_str(&format!("class {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", table_class_name));
    output.push_str(&format!("    private _data: HashMap<UTF8Text, {}>\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("imply {} {{\n", table_class_name));
    output.push_str(&format!("    micro load() -> {} {{\n", table_class_name));
    output.push_str(&format!("        {} {{\n", table_class_name));
    output.push_str(&format!("            _data: gg.load_sheet(\"{}\")\n", table_name));
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro find(self, key: UTF8Text) -> Option<{}> {{\n", row_name));
    output.push_str("        return self._data[key]\n");
    output.push_str("    }\n\n");
    output.push_str("    micro get_text(self, key: UTF8Text, lang: UTF8Text) -> UTF8Text {\n");
    output.push_str("        if let Some(lang_data) = self._data[key] {\n");
    output.push_str("            match lang {\n");
    for field in &lang_fields {
        output.push_str(&format!("                \"{}\" => return lang_data.{}\n", field.field_name, field.field_name));
    }
    if let Some(first) = lang_fields.first() {
        output.push_str(&format!("                else => return lang_data.{}\n", first.field_name));
    }
    output.push_str("            }\n");
    output.push_str("        }\n");
    output.push_str("        return \"\"\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    output.push_str("class LanguageManager {\n");
    output.push_str("    current_lang: UTF8Text\n");
    output.push_str(&format!("    table: {}\n", table_class_name));
    output.push_str("}\n\n");

    output.push_str("static mut lang_manager: Option<LanguageManager> = None\n\n");

    output.push_str("micro init_language(lang: UTF8Text) {\n");
    output.push_str("    lang_manager = Some(LanguageManager {\n");
    output.push_str("        current_lang: lang,\n");
    output.push_str(&format!("        table: {}.load()\n", table_class_name));
    output.push_str("    })\n");
    output.push_str("}\n\n");

    output.push_str("micro get_text(key: UTF8Text) -> UTF8Text {\n");
    output.push_str("    if let Some(manager) = lang_manager {\n");
    output.push_str("        return manager.table.get_text(key, manager.current_lang)\n");
    output.push_str("    }\n");
    output.push_str("    return \"\"\n");
    output.push_str("}\n");

    Ok(output)
}

fn generate_lazy_table(table: &SheetTable, config: &CodegenConfig) -> SheetResult<String> {
    let table_name = &table.name;
    let row_name = table_name.clone();
    let class_name = format!("{}Table", table_name);
    let namespace = config.namespace.as_deref();

    let mut output = String::new();
    output.push_str(&generate_header(table_name, namespace));
    output.push_str("\n\n");

    output.push_str(&format!("class {} {{\n", row_name));
    for header in &table.headers {
        let field_type = header.typing.to_valkyrie_type();
        output.push_str(&format!("    {}: {}\n", header.field_name, field_type));
    }
    output.push_str("}\n\n");

    output.push_str(&format!("class {} {{\n", class_name));
    output.push_str(&format!("    private _loader: SheetLazyLoader<{}>\n", row_name));
    output.push_str("}\n\n");

    output.push_str(&format!("imply {} {{\n", class_name));
    output.push_str(&format!("    micro load() -> {} {{\n", class_name));
    output.push_str(&format!("        {} {{\n", class_name));
    output.push_str(&format!("            _loader: gg.load_sheet_lazy(\"{}\")\n", table_name));
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro find(self, key: i32) -> Option<{}> {{\n", row_name));
    output.push_str("        return self._loader.find(key)\n");
    output.push_str("    }\n\n");
    output.push_str(&format!("    micro all(self) -> Generator<Item={}> {{\n", row_name));
    output.push_str("        return self._loader.all()\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    Ok(output)
}

/// 格式化字段值为 Valkyrie 脚本字面量
pub fn format_field_value(value: &str, ty: &SheetType) -> String {
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
        SheetType::String => format!("\"{}\"", escape_string(trimmed)),
        SheetType::Color => {
            if trimmed.is_empty() {
                "Color(\"#000000FF\")".to_string()
            }
            else if let Ok((r, g, b, a)) = types::parse_color_str(trimmed) {
                format!("Color(\"#{:02X}{:02X}{:02X}{:02X}\")", r, g, b, a)
            }
            else {
                format!("Color(\"#000000FF\")")
            }
        }
        SheetType::Vector(kind) => {
            let default = match kind {
                VectorKind::Vec2 => "Vec2(0.0, 0.0)".to_string(),
                VectorKind::Vec3 => "Vec3(0.0, 0.0, 0.0)".to_string(),
                VectorKind::Vec4 => "Vec4(0.0, 0.0, 0.0, 0.0)".to_string(),
            };
            if trimmed.is_empty() {
                return default;
            }
            let components = match types::parse_vector_str(trimmed) {
                Ok(c) => c,
                Err(_) => return default,
            };
            match kind {
                VectorKind::Vec2 if components.len() == 2 => {
                    format!("Vec2({}, {})", format_f32(components[0]), format_f32(components[1]))
                }
                VectorKind::Vec3 if components.len() == 3 => {
                    format!("Vec3({}, {}, {})", format_f32(components[0]), format_f32(components[1]), format_f32(components[2]))
                }
                VectorKind::Vec4 if components.len() == 4 => {
                    format!(
                        "Vec4({}, {}, {}, {})",
                        format_f32(components[0]),
                        format_f32(components[1]),
                        format_f32(components[2]),
                        format_f32(components[3])
                    )
                }
                _ => default,
            }
        }
        SheetType::List(inner) => {
            if trimmed.is_empty() {
                "[]".to_string()
            }
            else {
                let items: Vec<String> = trimmed.split(',').map(|s| format_field_value(s.trim(), inner)).collect();
                format!("[{}]", items.join(", "))
            }
        }
        SheetType::Map { key, value } => {
            if trimmed.is_empty() {
                return format!("Map<{}, {}> {{}}", key.to_valkyrie_type(), value.to_valkyrie_type());
            }
            let entries = match types::parse_map_str(trimmed) {
                Ok(e) => e,
                Err(_) => return format!("Map<{}, {}> {{}}", key.to_valkyrie_type(), value.to_valkyrie_type()),
            };
            if entries.is_empty() {
                return format!("Map<{}, {}> {{}}", key.to_valkyrie_type(), value.to_valkyrie_type());
            }
            let formatted_entries: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let formatted_value = format_field_value(v, value);
                    format!("\"{}\": {}", escape_string(k), formatted_value)
                })
                .collect();
            format!("Map<{}, {}> {{ {} }}", key.to_valkyrie_type(), value.to_valkyrie_type(), formatted_entries.join(", "))
        }
        SheetType::Optional(inner) => {
            if trimmed.is_empty() {
                "null".to_string()
            }
            else {
                format_field_value(trimmed, inner)
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

/// 转义字符串中的特殊字符
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 格式化 f32 为字符串，确保整数显示为带小数点形式
fn format_f32(v: f32) -> String {
    if v.fract() == 0.0 { format!("{:.1}", v) } else { format!("{}", v) }
}

/// 将 CamelCase 或 PascalCase 转换为 snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
        else {
            result.push(c);
        }
    }
    result
}
