//! Pull 命令模块
//!
//! 提供从远程同步 WAE 文件的功能。

use clap::Parser;
use oak_pretty_print::to_doc::AsDocument;
use oak_rbq::ast::*;
use std::{fs, path::Path};
use wae_types::WaeResult;

/// Pull 命令
#[derive(Parser, Debug)]
pub struct PullCommand {
    /// 数据库 URL
    #[clap(long, short)]
    remote: String,
    /// 可选：目标目录
    #[clap(long, short, default_value = "schemas")]
    target: String,
}

impl PullCommand {
    /// 执行 Pull 命令
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("WAE DSL Pull");
        println!("{}", "=".repeat(60));
        println!("Remote: {}", self.remote);
        println!("Target: {}", self.target);
        println!();

        // 创建目标目录
        let schemas_dir = Path::new(&self.target);
        if !schemas_dir.exists() {
            fs::create_dir_all(&schemas_dir)?;
            println!("Created directory: {}", schemas_dir.display());
        }

        // 从数据库 URL 中提取数据库名称
        let db_name = if let Some(name) = Self::extract_db_name(&self.remote) { name } else { "authentication".to_string() };

        // 生成 WAE 文件路径
        let wae_file_path = schemas_dir.join(format!("{}.wae", db_name));

        // 读取现有 WAE 文件，提取模型名称映射
        let existing_model_mappings = if wae_file_path.exists() {
            println!("Reading existing WAE file...");
            Self::extract_existing_model_mappings(&wae_file_path)?
        }
        else {
            std::collections::HashMap::new()
        };

        // 实现从数据库提取 schema 的逻辑
        println!("Extracting schema from database...");
        println!("Would extract schema to: {}", wae_file_path.display());

        #[cfg(feature = "database-mysql")]
        {
            // 连接数据库
            let conn = Self::connect_to_mysql(&self.remote).await?;

            // 创建 SchemaReflector
            let reflector = crate::SchemaReflector::new(&conn);

            // 获取所有表的 schema
            let schemas = reflector.get_all_schemas().await?;

            // 生成 WAE 文件内容
            let wae_content = Self::generate_wae_content(&db_name, &schemas, &existing_model_mappings)?;

            // 写入 WAE 文件
            fs::write(&wae_file_path, wae_content)?;
            println!("Generated WAE file: {}", wae_file_path.display());
        }

        #[cfg(not(feature = "database-mysql"))]
        {
            // 模拟生成 WAE 文件
            println!("Simulating schema extraction...");
            let wae_content = format!(
                "# RBQ Schema for {}\n\n@database(\"mysql.{}\")\nnamespace {};\n\n// Tables will be generated here\n",
                db_name, db_name, db_name
            );

            // 写入 WAE 文件
            fs::write(&wae_file_path, wae_content)?;
            println!("Generated WAE file: {}", wae_file_path.display());
        }

        println!("\nPull completed successfully!");
        Ok(())
    }

    /// 提取现有 WAE 文件中的模型名称映射
    fn extract_existing_model_mappings(
        wae_file_path: &Path,
    ) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error>> {
        use oak_rbq::parse;

        let content = fs::read_to_string(wae_file_path)?;
        let root = parse(&content)?;

        let mut mappings = std::collections::HashMap::new();

        for item in &root.items {
            if let RbqItem::Struct(struct_def) = item {
                // 查找 @table 注解，获取表名
                for annotation in &struct_def.annotations {
                    if annotation.name == "table" && !annotation.args.is_empty() {
                        // 解析 table 注解的参数，格式为 "name = \"table_name\""
                        let arg = &annotation.args[0];
                        if let Some(table_name) = arg.split('"').nth(1) {
                            mappings.insert(table_name.to_string(), struct_def.name.clone());
                        }
                        break;
                    }
                }
            }
        }

        Ok(mappings)
    }

    /// 连接到 MySQL 数据库
    #[cfg(feature = "database-mysql")]
    async fn connect_to_mysql(url: &str) -> Result<wae_database::MySqlConnection, Box<dyn std::error::Error>> {
        use wae_database::{DatabaseConfig, MySqlDatabaseService};
        println!("Attempting to connect to MySQL database: {}", url);
        let config = DatabaseConfig::MySql { connection_string: url.to_string(), pool_config: Default::default() };
        println!("Creating database service...");
        let service = MySqlDatabaseService::new(&config).await?;
        println!("Service created successfully. Connecting to database...");
        let conn = service.connect().await?;
        println!("Connection established successfully!");
        Ok(conn)
    }

    /// 生成 WAE 文件内容
    #[cfg(feature = "database-mysql")]
    pub fn generate_wae_content(
        db_name: &str,
        schemas: &std::collections::HashMap<String, wae_database::TableSchema>,
        existing_model_mappings: &std::collections::HashMap<String, String>,
    ) -> WaeResult<String> {
        // 构造 RBQ AST
        let mut items = Vec::new();

        // 添加数据库注解和命名空间
        let namespace_annotations =
            vec![RbqAnnotation { name: "database".to_string(), args: vec![format!("mysql.{}", db_name)], span: (0..0).into() }];

        let namespace = RbqNamespace { annotations: namespace_annotations, path: db_name.to_string(), span: (0..0).into() };
        items.push(RbqItem::Namespace(namespace));

        // 生成每个表的定义
        for (table_name, table_schema) in schemas {
            // 构造表的注解
            let mut table_annotations = vec![RbqAnnotation {
                name: "table".to_string(),
                args: vec![format!("name = \"{}\"", table_name)],
                span: (0..0).into(),
            }];

            // 构造字段定义
            let mut fields = Vec::new();
            for column in &table_schema.columns {
                let mut field_annotations = Vec::new();

                if column.primary_key {
                    field_annotations.push(RbqAnnotation { name: "key".to_string(), args: Vec::new(), span: (0..0).into() });
                }

                if column.unique {
                    field_annotations.push(RbqAnnotation { name: "unique".to_string(), args: Vec::new(), span: (0..0).into() });
                }

                // 构造类型
                let rbq_type = Self::column_type_to_rbq(&column.col_type);
                let type_ref = if column.nullable {
                    RbqType::Optional(
                        Box::new(RbqType::Named {
                            path: rbq_type,
                            generic_args: Vec::new(),
                            is_physical_ptr: false,
                            is_optional: false,
                            span: (0..0).into(),
                        }),
                        (0..0).into(),
                    )
                }
                else {
                    RbqType::Named {
                        path: rbq_type,
                        generic_args: Vec::new(),
                        is_physical_ptr: false,
                        is_optional: false,
                        span: (0..0).into(),
                    }
                };

                // 构造默认值
                let default_value = if let Some(default) = &column.default_value {
                    Some(RbqExpr { kind: RbqExprKind::Identifier(default.to_string()), span: (0..0).into() })
                }
                else {
                    None
                };

                // 构造字段
                let field = RbqField {
                    annotations: field_annotations,
                    name: column.name.clone(),
                    type_ref,
                    default_value,
                    span: (0..0).into(),
                };
                fields.push(field);
            }

            // 生成索引定义
            for index in &table_schema.indexes {
                if !index.name.starts_with("PRIMARY") {
                    let annotation_name = if index.unique { "unique" } else { "index" };
                    let args = vec![format!("[\"{}\"]", index.columns.join(", \""))];

                    let index_annotation = RbqAnnotation { name: annotation_name.to_string(), args, span: (0..0).into() };
                    table_annotations.push(index_annotation);
                }
            }

            // 构造结构体
            // 优先使用现有的模型名称映射，否则生成新的
            let model_name = if let Some(existing_name) = existing_model_mappings.get(table_name) {
                existing_name.clone()
            }
            else {
                Self::table_name_to_model_name(table_name)
            };
            let struct_def = RbqStruct { annotations: table_annotations, name: model_name, fields, span: (0..0).into() };
            items.push(RbqItem::Struct(struct_def));
        }

        // 构造根节点
        let root = RbqRoot { items, span: (0..0).into() };

        // 生成文档
        let document = root.as_document(&());
        let wae_content = format!("# RBQ Schema for {}\n\n{}", db_name, document.render());

        Ok(wae_content)
    }

    /// 将 ColumnType 转换为 RBQ 类型
    #[cfg(feature = "database-mysql")]
    fn column_type_to_rbq(col_type: &wae_database::ColumnType) -> String {
        match col_type {
            wae_database::ColumnType::Integer => "i32".to_string(),
            wae_database::ColumnType::Real => "f64".to_string(),
            wae_database::ColumnType::Text => "string".to_string(),
            wae_database::ColumnType::Blob => "bytes".to_string(),
        }
    }

    /// 从数据库 URL 中提取数据库名称
    fn extract_db_name(url: &str) -> Option<String> {
        // 简单的 URL 解析，提取最后一个斜杠后的部分作为数据库名称
        if let Some(last_slash) = url.rfind('/') {
            let db_name = &url[last_slash + 1..];
            // 去除可能的查询参数
            if let Some(question_mark) = db_name.find('?') {
                Some(db_name[..question_mark].to_string())
            }
            else {
                Some(db_name.to_string())
            }
        }
        else {
            None
        }
    }

    /// 将表名转换为大写的类名格式
    fn table_name_to_model_name(table_name: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for c in table_name.chars() {
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
}
