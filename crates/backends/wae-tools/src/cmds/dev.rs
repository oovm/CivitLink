//! 开发服务器命令模块
//!
//! 提供热重载开发服务器的功能。

use clap::Parser;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::process::Command;

/// 开发服务器命令
#[derive(Parser, Debug)]
pub struct DevCommand {
    /// 可选：监听目录
    #[clap(long, short)]
    watch: Option<String>,
    /// 可选：命令
    #[clap(long, short)]
    cmd: Option<String>,
}

impl DevCommand {
    /// 执行开发服务器命令
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let watch_path = self.watch.as_deref().unwrap_or(".");
        let command = self.cmd.as_deref().unwrap_or("cargo run");

        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;
        debouncer.watcher().watch(Path::new(watch_path), RecursiveMode::Recursive)?;

        println!("Watching for changes in: {}", watch_path);
        println!("Running command: {}", command);

        let mut child = Self::run_command(command).await?;

        println!("\nPress Ctrl+C to stop.");

        for res in rx {
            match res {
                Ok(_) => {
                    println!("Changes detected, restarting...");
                    if let Some(ref mut c) = child {
                        let _ = c.kill().await;
                    }
                    child = Self::run_command(command).await?;
                }
                Err(e) => {
                    eprintln!("Watch error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 运行命令
    async fn run_command(cmd: &str) -> Result<Option<tokio::process::Child>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        let child = Command::new(parts[0]).args(&parts[1..]).stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn()?;

        Ok(Some(child))
    }
}
