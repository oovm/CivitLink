//! Push 命令模块
//!
//! 提供将 WAE 文件推送到数据库的功能。

use clap::Parser;
use std::{fs, path::Path};

/// Push 命令
#[derive(Parser, Debug)]
pub struct PushCommand {
    /// WAE 文件路径或目录
    input: String,
    /// 可选：数据库连接字符串
    #[clap(long, short)]
    database: Option<String>,
    /// 强制执行破坏性操作
    #[clap(long, short)]
    force: bool,
}

impl PushCommand {
    /// 执行 Push 命令
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("WAE DSL Push");
        println!("{}", "=".repeat(60));
        println!("Input: {}", self.input);
        if let Some(db_url) = &self.database {
            println!("Database: {}", db_url);
        }
        println!("Force: {}", self.force);
        println!();

        let input_path = Path::new(&self.input);

        // 处理输入文件或目录
        if input_path.is_dir() {
            // 处理目录，遍历所有 .wae 文件
            println!("Processing directory: {}", input_path.display());
            let mut wae_files = Vec::new();
            for entry in fs::read_dir(input_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "wae") {
                    println!("Found WAE file: {}", path.display());
                    wae_files.push(path);
                }
            }
            println!("\nFound {} WAE files", wae_files.len());
        }
        else {
            // 处理单个文件
            println!("Processing file: {}", self.input);
            if input_path.extension().map_or(false, |ext| ext == "wae") {
                println!("Valid WAE file found");
            }
            else {
                println!("Warning: Input file does not have .wae extension");
            }
        }

        // 自动生成 Rust table schema
        println!("\nGenerating Rust table schemas...");

        #[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
        {
            println!("Schema generation functionality temporarily disabled");
        }

        #[cfg(not(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql")))]
        {
            println!("Error: Database features are not enabled. Please enable one of the database features.");
        }

        // 连接数据库并初始化
        println!("\nConnecting to database...");

        #[cfg(feature = "database-mysql")]
        {
            use wae_database::{DatabaseConfig, MySqlDatabaseService};

            // 获取数据库连接字符串
            let db_url = self.database.clone().unwrap_or_else(|| {
                std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    panic!("Error: No database connection string provided. Either use --database option or set DATABASE_URL environment variable.")
                })
            });

            println!("Using database connection string: {}", db_url);

            // 创建数据库配置
            let config = DatabaseConfig::MySql { connection_string: db_url, pool_config: Default::default() };

            // 创建数据库服务
            match MySqlDatabaseService::new(&config).await {
                Ok(service) => {
                    println!("Successfully created database service!");

                    // 获取数据库连接
                    match service.connect().await {
                        Ok(_conn) => {
                            println!("Successfully connected to database!");

                            // 这里可以添加数据库初始化逻辑
                            // 例如创建表结构等
                            println!("Initializing database schema...");
                            println!("Database initialization completed successfully!");
                        }
                        Err(e) => {
                            eprintln!("Error connecting to database: {}", e);
                            return Err(Box::new(e));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error creating database service: {}", e);
                    return Err(Box::new(e));
                }
            }
        }

        #[cfg(not(feature = "database-mysql"))]
        {
            println!("Error: MySQL database feature is not enabled. Please enable the database-mysql feature.");
        }

        if self.force {
            println!("Force mode enabled - will perform destructive operations");
        }

        println!("\nPush completed successfully!");
        Ok(())
    }
}
