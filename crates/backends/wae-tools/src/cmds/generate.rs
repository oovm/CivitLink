//! 代码生成命令模块
//!
//! 提供从 OpenAPI/Swagger 生成代码的功能。

use clap::Parser;

/// 代码生成命令
#[derive(Parser, Debug)]
pub struct GenerateCommand {
    /// Schemas 目录路径
    #[clap(long, short = 's', default_value = "schemas")]
    schemas: String,
    /// 输出目录路径
    #[clap(long, short = 'o', default_value = "src/entities")]
    out: String,
}

impl GenerateCommand {
    /// 执行代码生成命令
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::{fs, path::Path};

        println!("WAE Generate ORM");
        println!("{}", "=".repeat(60));
        println!("Schemas directory: {}", self.schemas);
        println!("Output directory: {}", self.out);
        println!();

        // 检查 schemas 目录是否存在
        let schemas_path = Path::new(&self.schemas);
        if !schemas_path.exists() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Schemas directory not found: {}", self.schemas),
            )));
        }

        // 检查输出目录是否存在，不存在则创建
        let out_path = Path::new(&self.out);
        if !out_path.exists() {
            fs::create_dir_all(out_path)?;
            println!("Created output directory: {}", self.out);
        }

        // 读取 schemas 目录下的 WAE 文件
        let wae_files = fs::read_dir(schemas_path)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("wae")) { Some(path) } else { None }
            })
            .collect::<Vec<_>>();

        if wae_files.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No WAE files found in directory: {}", self.schemas),
            )));
        }

        println!("Found {} WAE files:", wae_files.len());
        for file in &wae_files {
            println!("  - {}", file.display());
        }

        // 解析 WAE 文件并生成 ORM 代码
        let mut entities = Vec::new();

        for file in wae_files {
            println!();
            println!("Processing file: {}", file.display());

            // 读取 WAE 文件内容
            let content = fs::read_to_string(&file)?;

            // 解析 WAE 文件
            use oak_rbq::parse;

            println!("Parsing WAE content...");
            let root = parse(&content)?;
            println!("Parsed successfully, found {} items", root.items.len());

            // 打印根节点的所有项目
            for (index, item) in root.items.iter().enumerate() {
                println!("Root item {}: {:?}", index, item);
                if let oak_rbq::ast::RbqItem::Namespace(ns) = item {
                    println!("Namespace path: {}", ns.path);
                    println!("Namespace items: {}", ns.items.len());
                    for (ns_index, ns_item) in ns.items.iter().enumerate() {
                        println!("Namespace item {}: {:?}", ns_index, ns_item);
                    }
                }
            }

            // 提取模型定义
            for (index, item) in root.items.iter().enumerate() {
                println!("Item {}: {:?}", index, item);
                match item {
                    oak_rbq::ast::RbqItem::Struct(struct_def) => {
                        println!("Found model: {}", struct_def.name);

                        // 提取表名
                        let mut table_name = Self::to_snake_case(&struct_def.name);
                        let mut has_table_annotation = false;
                        for annotation in &struct_def.annotations {
                            if annotation.name == "table" && !annotation.args.is_empty() {
                                // 解析 table 注解的参数，格式为 "name = \"table_name\""
                                let arg = &annotation.args[0];
                                if let Some(name) = arg.split('"').nth(1) {
                                    table_name = name.to_string();
                                    has_table_annotation = true;
                                    break;
                                }
                            }
                        }
                        println!("  Table name: {}", table_name);

                        // 提取字段定义
                        println!("  Fields:");
                        for field in &struct_def.fields {
                            println!("    - {}: {:?}", field.name, field.type_ref);
                        }

                        // 生成 ORM 代码
                        let entity_code = Self::generate_entity_code(struct_def, &table_name);
                        entities.push((table_name, struct_def.name.clone(), entity_code));
                    }
                    oak_rbq::ast::RbqItem::Namespace(ns) => {
                        println!("Found namespace: {}", ns.path);
                        // 遍历命名空间内部的项目
                        for (ns_index, ns_item) in ns.items.iter().enumerate() {
                            println!("Namespace item {}: {:?}", ns_index, ns_item);
                            if let oak_rbq::ast::RbqItem::Struct(struct_def) = ns_item {
                                println!("Found model in namespace: {}", struct_def.name);

                                // 提取表名
                                let mut table_name = Self::to_snake_case(&struct_def.name);
                                let mut has_table_annotation = false;
                                for annotation in &struct_def.annotations {
                                    if annotation.name == "table" && !annotation.args.is_empty() {
                                        // 解析 table 注解的参数，格式为 "name = \"table_name\""
                                        let arg = &annotation.args[0];
                                        if let Some(name) = arg.split('"').nth(1) {
                                            table_name = name.to_string();
                                            has_table_annotation = true;
                                            break;
                                        }
                                    }
                                }
                                println!("  Table name: {}", table_name);

                                // 提取字段定义
                                println!("  Fields:");
                                for field in &struct_def.fields {
                                    println!("    - {}: {:?}", field.name, field.type_ref);
                                }

                                // 生成 ORM 代码
                                let entity_code = Self::generate_entity_code(struct_def, &table_name);
                                entities.push((table_name, struct_def.name.clone(), entity_code));
                            }
                        }
                    }
                    _ => {}
                }
            }

            if entities.is_empty() {
                println!("No models found in WAE file");
            }
            else {
                println!("Found {} models", entities.len());
            }
        }

        // 输出 ORM 代码到文件
        for (table_name, _, code) in &entities {
            // 转换表名为有效的 Rust 模块名
            let file_name = table_name.replace('_', "_").to_lowercase();
            let file_path = out_path.join(format!("{}.rs", file_name));
            fs::write(&file_path, code)?;
            println!("Generated file: {}", file_path.display());
        }

        // 生成 mod.rs 文件
        let mod_content = Self::generate_mod_file(entities);
        let mod_path = out_path.join("mod.rs");
        fs::write(&mod_path, mod_content)?;
        println!("Generated file: {}", mod_path.display());

        println!();
        println!("ORM code generation completed successfully!");
        Ok(())
    }

    /// 生成实体代码
    fn generate_entity_code(struct_def: &oak_rbq::ast::RbqStruct, table_name: &str) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        let struct_name = &struct_def.name;

        // 生成文档注释
        writeln!(output, "/// 数据库表 `{}` 的实体", table_name).unwrap();
        writeln!(output, "#[derive(Debug, Clone)]").unwrap();
        writeln!(output, "pub struct {} {{", struct_name).unwrap();

        // 生成字段定义
        for field in &struct_def.fields {
            let field_name = &field.name;
            let rust_type = Self::type_ref_to_rust_type(&field.type_ref);
            writeln!(output, "    /// 列 `{}`", field_name).unwrap();
            writeln!(output, "    pub {}: {},", field_name, rust_type).unwrap();
        }

        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // 生成 Entity trait 实现
        writeln!(output, "impl Entity for {} {{", struct_name).unwrap();

        // 生成 Id 类型
        let id_type = Self::find_id_type(&struct_def.fields);
        writeln!(output, "    type Id = {};", id_type).unwrap();
        writeln!(output).unwrap();

        // 生成 table_name 方法
        writeln!(output, "    fn table_name() -> &'static str {{",).unwrap();
        writeln!(output, "        \"{}\"", table_name).unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output).unwrap();

        // 生成 id 方法
        writeln!(output, "    fn id(&self) -> Self::Id {{",).unwrap();
        let id_field = Self::find_id_field(&struct_def.fields);
        writeln!(output, "        self.{}.clone()", id_field).unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output).unwrap();

        // 生成 id_column 方法
        writeln!(output, "    fn id_column() -> &'static str {{",).unwrap();
        writeln!(output, "        \"{}\"", id_field).unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // 生成 FromRow trait 实现
        writeln!(output, "impl FromRow for {} {{", struct_name).unwrap();
        writeln!(output, "    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {{",).unwrap();
        writeln!(output, "        Ok(Self {{").unwrap();

        for field in &struct_def.fields {
            let field_name = &field.name;
            let get_method = Self::type_ref_to_get_method(&field.type_ref);
            writeln!(output, "            {}: row.{}({})?,", field_name, get_method, field_name).unwrap();
        }

        writeln!(output, "        }})").unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // 生成 ToRow trait 实现
        writeln!(output, "impl ToRow for {} {{", struct_name).unwrap();
        writeln!(output, "    fn to_row(&self) -> Vec<(&'static str, Value)> {{",).unwrap();
        writeln!(output, "        vec![").unwrap();

        for field in &struct_def.fields {
            let field_name = &field.name;
            let into_method = Self::type_ref_to_into_method(&field.type_ref);
            writeln!(output, "            (\"{}\", self.{}.{}()),", field_name, field_name, into_method).unwrap();
        }

        writeln!(output, "        ]").unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        output
    }

    /// 生成 mod.rs 文件内容
    fn generate_mod_file(entities: Vec<(String, String, String)>) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        // 生成文档注释
        writeln!(output, "//! 自动生成的实体代码").unwrap();
        writeln!(output, "//!").unwrap();
        writeln!(output, "//! 此文件由 wae-tools 自动生成，请勿手动修改。").unwrap();
        writeln!(output).unwrap();

        // 生成导入
        writeln!(output, "use wae_database::{{Entity, FromRow, ToRow, DatabaseRow, DatabaseResult}};").unwrap();
        writeln!(output, "use wae_types::Value;").unwrap();
        writeln!(output).unwrap();

        // 生成模块声明
        for (table_name, _, _) in &entities {
            let module_name = table_name.replace('_', "_").to_lowercase();
            writeln!(output, "pub mod {};", module_name).unwrap();
        }
        writeln!(output).unwrap();

        // 生成导出声明
        for (table_name, struct_name, _) in &entities {
            let module_name = table_name.replace('_', "_").to_lowercase();
            writeln!(output, "pub use {}::Entity as {};", module_name, struct_name).unwrap();
        }
        writeln!(output).unwrap();

        output
    }

    /// 将类型引用转换为 Rust 类型
    fn type_ref_to_rust_type(type_ref: &oak_rbq::ast::RbqType) -> String {
        match type_ref {
            oak_rbq::ast::RbqType::Named { path, .. } => {
                match path.as_str() {
                    "string" => "String".to_string(),
                    "i32" => "i32".to_string(),
                    "i64" => "i64".to_string(),
                    "f32" => "f32".to_string(),
                    "f64" => "f64".to_string(),
                    "bool" => "bool".to_string(),
                    "uuid" => "String".to_string(),      // 暂时使用 String 表示 UUID
                    "date_time" => "String".to_string(), // 暂时使用 String 表示日期时间
                    "utf8" => "String".to_string(),      // 暂时使用 String 表示 UTF8
                    _ => "String".to_string(),           // 默认为 String
                }
            }
            oak_rbq::ast::RbqType::Optional(inner, _) => {
                format!("Option<{}>", Self::type_ref_to_rust_type(inner))
            }
            _ => "String".to_string(), // 默认为 String
        }
    }

    /// 将类型引用转换为 get 方法
    fn type_ref_to_get_method(type_ref: &oak_rbq::ast::RbqType) -> &'static str {
        "get"
    }

    /// 将类型引用转换为 into 方法
    fn type_ref_to_into_method(type_ref: &oak_rbq::ast::RbqType) -> &'static str {
        "into"
    }

    /// 查找 ID 类型
    fn find_id_type(fields: &Vec<oak_rbq::ast::RbqField>) -> String {
        for field in fields {
            for annotation in &field.annotations {
                if annotation.name == "key" {
                    return Self::type_ref_to_rust_type(&field.type_ref);
                }
            }
        }
        "String".to_string() // 默认为 String
    }

    /// 查找 ID 字段
    fn find_id_field(fields: &Vec<oak_rbq::ast::RbqField>) -> &str {
        for field in fields {
            for annotation in &field.annotations {
                if annotation.name == "key" {
                    return &field.name;
                }
            }
        }
        "id" // 默认为 id
    }

    /// 将驼峰命名转换为蛇形命名
    fn to_snake_case(name: &str) -> String {
        let mut result = String::new();
        let mut chars = name.chars();

        if let Some(first_char) = chars.next() {
            result.push(first_char.to_lowercase().next().unwrap());
        }

        for c in chars {
            if c.is_uppercase() {
                result.push('_');
                result.push(c.to_lowercase().next().unwrap());
            }
            else {
                result.push(c);
            }
        }

        result
    }
}
