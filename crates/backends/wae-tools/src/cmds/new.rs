//! 新项目命令模块
//!
//! 提供创建新项目脚手架的功能。

use clap::Parser;
use std::{fs, path::PathBuf};

/// 新项目命令
#[derive(Parser, Debug)]
pub struct NewCommand {
    /// 项目名称
    name: String,
    /// 可选：项目目录
    #[clap(long, short)]
    path: Option<String>,
}

impl NewCommand {
    /// 执行新项目命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let project_path = match &self.path {
            Some(p) => PathBuf::from(p).join(&self.name),
            None => PathBuf::from(&self.name),
        };

        if project_path.exists() {
            return Err(format!("Project directory already exists: {}", project_path.display()).into());
        }

        fs::create_dir_all(&project_path)?;
        fs::create_dir_all(project_path.join("src"))?;

        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "0.0.0"
edition = "2024"

[dependencies]
wae-server = {{ workspace = true }}
tokio = {{ workspace = true }}
"#,
            self.name
        );
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content)?;

        let main_rs_content = r#"
use wae_server::prelude::*;

#[tokio::main]
async fn main() {
    println!("Hello, WAE!");
}
"#;
        fs::write(project_path.join("src/main.rs"), main_rs_content)?;

        println!("Project created successfully at: {}", project_path.display());
        Ok(())
    }
}
