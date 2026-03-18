//! 代码生成模块
//!
//! 从数据库 Schema 生成 Rust 结构体和 Entity trait 实现。

use std::fmt::Write;
use wae_database::{ColumnDef, ColumnType, TableSchema};

/// 代码生成器
pub struct CodeGenerator;

impl CodeGenerator {
    /// 创建新的代码生成器
    pub fn new() -> Self {
        Self
    }

    /// 生成单个表的 Rust 代码
    pub fn generate_for_table(&self, schema: &TableSchema) -> String {
        let mut output = String::new();

        let struct_name = self.to_pascal_case(&schema.name);
        let pk_col = schema.primary_key();

        writeln!(output, "/// 数据库表 `{}` 的实体", schema.name).unwrap();
        writeln!(output, "#[derive(Debug, Clone)]").unwrap();
        writeln!(output, "pub struct {} {{", struct_name).unwrap();

        for col in &schema.columns {
            let field_name = self.to_snake_case(&col.name);
            let rust_type = self.column_to_rust_type(col);
            writeln!(output, "    /// 列 `{}`", col.name).unwrap();
            writeln!(output, "    pub {}: {},", field_name, rust_type).unwrap();
        }

        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "impl Entity for {} {{", struct_name).unwrap();

        if let Some(pk) = pk_col {
            let pk_rust_type = self.column_to_rust_type_non_null(pk);
            writeln!(output, "    type Id = {};", pk_rust_type).unwrap();
        }
        else {
            writeln!(output, "    type Id = ();").unwrap();
        }

        writeln!(output).unwrap();

        writeln!(output, "    fn table_name() -> &'static str {{").unwrap();
        writeln!(output, "        \"{}\"", schema.name).unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "    fn id(&self) -> Self::Id {{").unwrap();
        if let Some(pk) = pk_col {
            let pk_field = self.to_snake_case(&pk.name);
            writeln!(output, "        self.{}.clone()", pk_field).unwrap();
        }
        else {
            writeln!(output, "        ()").unwrap();
        }
        writeln!(output, "    }}").unwrap();

        if let Some(pk) = pk_col {
            writeln!(output).unwrap();
            writeln!(output, "    fn id_column() -> &'static str {{").unwrap();
            writeln!(output, "        \"{}\"", pk.name).unwrap();
            writeln!(output, "    }}").unwrap();
        }

        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "impl FromRow for {} {{", struct_name).unwrap();
        writeln!(output, "    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {{").unwrap();
        writeln!(output, "        Ok(Self {{").unwrap();

        for col in &schema.columns {
            let field_name = self.to_snake_case(&col.name);
            let get_method = self.column_to_get_method(col);
            writeln!(output, "            {}: row.{}({})?,", field_name, get_method, col.name).unwrap();
        }

        writeln!(output, "        }})").unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "impl ToRow for {} {{", struct_name).unwrap();
        writeln!(output, "    fn to_row(&self) -> Vec<(&'static str, Value)> {{").unwrap();
        writeln!(output, "        vec![").unwrap();

        for col in &schema.columns {
            let field_name = self.to_snake_case(&col.name);
            let into_method = self.column_to_into_method(col);
            writeln!(output, "            (\"{}\", self.{}.{}()),", col.name, field_name, into_method).unwrap();
        }

        writeln!(output, "        ]").unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}").unwrap();

        output
    }

    /// 生成所有表的 Rust 代码
    pub fn generate_for_schemas(&self, schemas: &std::collections::HashMap<String, TableSchema>) -> String {
        let mut output = String::new();

        writeln!(output, "//! 自动生成的实体代码").unwrap();
        writeln!(output, "//!").unwrap();
        writeln!(output, "//! 此文件由 wae-tools 自动生成，请勿手动修改。").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "use wae_database::{{Entity, FromRow, ToRow, DatabaseRow, DatabaseResult}};").unwrap();
        writeln!(output, "use wae_types::Value;").unwrap();
        writeln!(output).unwrap();

        let mut sorted_schemas: Vec<_> = schemas.values().collect();
        sorted_schemas.sort_by_key(|s| &s.name);

        for schema in sorted_schemas {
            writeln!(output, "{}", self.generate_for_table(schema)).unwrap();
            writeln!(output).unwrap();
        }

        output
    }

    fn to_pascal_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for c in s.chars() {
            if c == '_' {
                capitalize_next = true;
            }
            else if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            }
            else {
                result.push(c);
            }
        }

        result
    }

    fn to_snake_case(&self, s: &str) -> String {
        s.to_string()
    }

    fn column_to_rust_type(&self, col: &ColumnDef) -> String {
        let base = self.column_to_rust_type_non_null(col);
        if col.nullable { format!("Option<{}>", base) } else { base }
    }

    fn column_to_rust_type_non_null(&self, col: &ColumnDef) -> String {
        match col.col_type {
            ColumnType::Integer => "i64".to_string(),
            ColumnType::Real => "f64".to_string(),
            ColumnType::Text => "String".to_string(),
            ColumnType::Blob => "Vec<u8>".to_string(),
        }
    }

    fn column_to_get_method(&self, col: &ColumnDef) -> &'static str {
        match col.col_type {
            ColumnType::Integer => "get",
            ColumnType::Real => "get",
            ColumnType::Text => "get",
            ColumnType::Blob => "get",
        }
    }

    fn column_to_into_method(&self, col: &ColumnDef) -> &'static str {
        match col.col_type {
            ColumnType::Integer => "into",
            ColumnType::Real => "into",
            ColumnType::Text => "into",
            ColumnType::Blob => "into",
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
