//! DSL 命令模块
//! 
//! 提供将 .wae 文件转换为 .yaml 文件的功能。

use clap::Parser;
use std::path::Path;

/// DSL 命令
#[derive(Parser, Debug)]
pub enum DslCommand {
    /// 将 .wae 文件转换为 .yaml 文件
    Convert {
        /// 输入的 .wae 文件路径
        input: String,
        /// 输出的 .yaml 文件路径
        output: String,
    },
    /// 从远程同步 WAE 文件
    Pull {
        /// 可选：远程仓库 URL
        #[clap(long, short)]
        remote: Option<String>,
        /// 可选：目标目录
        #[clap(long, short, default_value = "schemas")]
        target: String,
    },
    /// 推送到数据库
    Push {
        /// WAE 文件路径或目录
        input: String,
        /// 可选：数据库连接字符串
        #[clap(long, short)]
        database: Option<String>,
        /// 强制执行破坏性操作
        #[clap(long, short)]
        force: bool,
    },
}

impl DslCommand {
    /// 执行 DSL 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DslCommand::Convert { input, output } => {
                let input_path = Path::new(input);
                let output_path = Path::new(output);
                
                // 加载 .wae 文件并解析为 TableSchema
                let schemas = super::super::dsl::load_schemas_from_wae_file(input_path)?;
                
                // 将 TableSchema 导出为 YAML 文件
                let yaml = serde_yaml::to_string(&schemas)?;
                std::fs::write(output_path, yaml)?;
                
                println!("Successfully converted {} to {}", input, output);
                Ok(())
            }
            DslCommand::Pull { remote, target } => {
                use std::fs;
                use std::path::Path;
                
                println!("WAE DSL Pull");
                println!("{}", "=".repeat(60));
                if let Some(remote_url) = remote {
                    println!("Remote: {}", remote_url);
                }
                println!("Target: {}", target);
                println!();
                
                // 创建目标目录
                let target_path = Path::new(target);
                if !target_path.exists() {
                    fs::create_dir_all(target_path)?;
                    println!("Created directory: {}", target);
                }
                
                // 这里应该实现从远程同步 WAE 文件的逻辑
                // 目前只是模拟实现
                println!("Simulating pull from remote...");
                println!("Would sync WAE files to: {}", target);
                
                // 自动生成 Rust table schema
                println!("\nGenerating Rust table schemas...");
                
                // 查找所有 WAE 文件并生成 Rust schema
                let mut all_schemas = Vec::new();
                for entry in fs::read_dir(target_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "wae") {
                        let file_schemas = super::super::dsl::load_schemas_from_wae_file(&path)?;
                        all_schemas.extend(file_schemas);
                    }
                }
                
                if !all_schemas.is_empty() {
                    let rust_code = super::super::dsl::generate_rust_schema(&all_schemas);
                    
                    // 写入 Rust schema 文件
                    let rust_output_path = target_path.join("schema.rs");
                    fs::write(&rust_output_path, rust_code)?;
                    println!("Generated Rust table schemas at: {}", rust_output_path.display());
                } else {
                    println!("No WAE files found to generate Rust schemas");
                }
                
                println!("\nPull completed successfully!");
                Ok(())
            }
            DslCommand::Push { input, database, force } => {
                use std::fs;
                use std::path::Path;
                
                println!("WAE DSL Push");
                println!("{}", "=".repeat(60));
                println!("Input: {}", input);
                if let Some(db_url) = database {
                    println!("Database: {}", db_url);
                }
                println!("Force: {}", force);
                println!();
                
                let input_path = Path::new(input);
                let mut schemas = Vec::new();
                
                if input_path.is_dir() {
                    // 处理目录，遍历所有 .wae 文件
                    for entry in fs::read_dir(input_path)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "wae") {
                            println!("Processing file: {}", path.display());
                            let file_schemas = super::super::dsl::load_schemas_from_wae_file(&path)?;
                            schemas.extend(file_schemas);
                        }
                    }
                } else {
                    // 处理单个文件
                    println!("Processing file: {}", input);
                    schemas = super::super::dsl::load_schemas_from_wae_file(input)?;
                }
                
                println!("\nLoaded {} table schemas", schemas.len());
                
                // 自动生成 Rust table schema
                println!("\nGenerating Rust table schemas...");
                
                if !schemas.is_empty() {
                    let rust_code = super::super::dsl::generate_rust_schema(&schemas);
                    
                    // 写入 Rust schema 文件
                    let rust_output_path = input_path.parent().unwrap_or_else(|| Path::new("")).join("schema.rs");
                    fs::write(&rust_output_path, rust_code)?;
                    println!("Generated Rust table schemas at: {}", rust_output_path.display());
                } else {
                    println!("No table schemas found to generate Rust schemas");
                }
                
                // 这里应该实现推送到数据库的逻辑
                // 目前只是模拟实现
                println!("\nSimulating push to database...");
                println!("Would apply {} table schemas to database", schemas.len());
                if *force {
                    println!("Force mode enabled - will perform destructive operations");
                }
                
                println!("\nPush completed successfully!");
                Ok(())
            }
        }
    }
}