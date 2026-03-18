//! 同步开发命令行工具
//! 
//! 提供基于子代理的并行开发命令行界面

use clap::{App, Arg, ArgMatches, SubCommand};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gwg_agent::prelude::*;
use gwg_dependency::prelude::*;
use gwg_verification::prelude::*;

/// 同步开发工具
pub struct SyncDevTool {
    /// 代理管理器
    agent_manager: AgentManager,
    /// 依赖管理器
    dependency_manager: DependencyManager,
    /// 验证系统
    verification_system: VerificationSystem,
    /// 任务列表
    tasks: Vec<Task>,
}

impl SyncDevTool {
    /// 创建新的同步开发工具
    pub fn new() -> Self {
        Self {
            agent_manager: AgentManager::new(),
            dependency_manager: DependencyManager::new(),
            verification_system: VerificationSystem::new(),
            tasks: vec![],
        }
    }

    /// 运行命令
    pub fn run(&mut self, matches: &ArgMatches) {
        match matches.subcommand() {
            ("init", Some(_)) => self.init(),
            ("add", Some(add_matches)) => self.add_task(add_matches),
            ("run", Some(_)) => self.run_tasks(),
            ("status", Some(_)) => self.show_status(),
            ("verify", Some(verify_matches)) => self.verify_task(verify_matches),
            _ => self.show_help(),
        }
    }

    /// 初始化
    fn init(&mut self) {
        println!("Initializing sync development environment...");
        
        // 创建默认代理
        for i in 1..=5 {
            let agent = self.agent_manager.create_agent(&format!("agent-{}", i));
            agent.start();
        }
        
        println!("Created 5 sub-agents");
        println!("Sync development environment initialized successfully!");
    }

    /// 添加任务
    fn add_task(&mut self, matches: &ArgMatches) {
        let id = matches.value_of("id").unwrap();
        let task_type = matches.value_of("type").unwrap();
        let description = matches.value_of("description").unwrap();
        let dependencies: Vec<&str> = matches.values_of("dependencies").unwrap_or_default().collect();
        
        let task_type = match task_type {
            "compile" => TaskType::Compile,
            "test" => TaskType::Test,
            "build" => TaskType::Build,
            "verify" => TaskType::Verify,
            _ => {
                println!("Invalid task type. Must be one of: compile, test, build, verify");
                return;
            }
        };
        
        let mut task = Task::new(id, task_type, description);
        for dep in dependencies {
            task.add_dependency(dep);
            self.dependency_manager.add_dependency(id, dep);
        }
        
        self.tasks.push(task);
        self.agent_manager.add_task(task.clone());
        
        println!("Task '{}' added successfully!");
    }

    /// 运行任务
    fn run_tasks(&mut self) {
        println!("Running tasks...");
        
        // 验证依赖关系
        match self.dependency_manager.validate_dependencies(&self.tasks) {
            Ok(_) => println!("Dependency validation passed"),
            Err(e) => {
                println!("Dependency validation failed: {}", e);
                return;
            }
        }
        
        // 拓扑排序
        match self.dependency_manager.topological_sort(&self.tasks) {
            Ok(ordered_tasks) => {
                println!("Task execution order:");
                for task_id in &ordered_tasks {
                    println!("- {}", task_id);
                }
                
                // 分配任务
                self.agent_manager.assign_tasks();
                
                // 启动所有代理
                self.agent_manager.start_all();
                
                // 等待任务完成
                self.wait_for_completion();
            }
            Err(e) => {
                println!("Task ordering failed: {}", e);
                return;
            }
        }
    }

    /// 等待任务完成
    fn wait_for_completion(&self) {
        println!("Waiting for tasks to complete...");
        
        let mut all_completed = false;
        while !all_completed {
            thread::sleep(Duration::from_secs(1));
            
            let tasks = self.agent_manager.get_all_tasks();
            all_completed = tasks.iter().all(|t| {
                t.status == TaskStatus::Completed || t.status == TaskStatus::Failed
            });
            
            if all_completed {
                break;
            }
        }
        
        println!("All tasks completed!");
        self.show_status();
    }

    /// 显示状态
    fn show_status(&self) {
        println!("Task status:");
        println!("{:<10} {:<10} {:<20} {:<15}", "ID", "Type", "Description", "Status");
        println!("{:-<10} {:-<10} {:-<20} {:-<15}", "", "", "", "");
        
        let tasks = self.agent_manager.get_all_tasks();
        for task in tasks {
            let status_str = match task.status {
                TaskStatus::Pending => "Pending",
                TaskStatus::InProgress => "In Progress",
                TaskStatus::Completed => "Completed",
                TaskStatus::Failed => "Failed",
            };
            
            let type_str = match task.task_type {
                TaskType::Compile => "Compile",
                TaskType::Test => "Test",
                TaskType::Build => "Build",
                TaskType::Verify => "Verify",
            };
            
            println!("{:<10} {:<10} {:<20} {:<15}", task.id, type_str, task.description, status_str);
        }
    }

    /// 验证任务
    fn verify_task(&mut self, matches: &ArgMatches) {
        let task_id = matches.value_of("id").unwrap();
        
        let task = self.tasks.iter().find(|t| t.id == task_id);
        match task {
            Some(task) => {
                println!("Verifying task '{}'...", task_id);
                let report = self.verification_system.verify(task);
                println!("{}", report.to_string());
            }
            None => {
                println!("Task '{}' not found", task_id);
            }
        }
    }

    /// 显示帮助
    fn show_help(&self) {
        println!("Sync Development Tool");
        println!("Usage: sync-dev <command> [options]");
        println!();
        println!("Commands:");
        println!("  init             Initialize sync development environment");
        println!("  add              Add a new task");
        println!("  run              Run all tasks");
        println!("  status           Show task status");
        println!("  verify           Verify a task");
        println!();
        println!("Use 'sync-dev <command> --help' for more information about a command.");
    }
}

/// 主函数
pub fn main() {
    let matches = App::new("sync-dev")
        .version("0.1.0")
        .about("Sync Development Tool for GWG Engine")
        .subcommand(SubCommand::with_name("init").about("Initialize sync development environment"))
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a new task")
                .arg(Arg::with_name("id").required(true).help("Task ID"))
                .arg(Arg::with_name("type").required(true).help("Task type (compile, test, build, verify)"))
                .arg(Arg::with_name("description").required(true).help("Task description"))
                .arg(Arg::with_name("dependencies").multiple(true).help("Task dependencies"))
        )
        .subcommand(SubCommand::with_name("run").about("Run all tasks"))
        .subcommand(SubCommand::with_name("status").about("Show task status"))
        .subcommand(
            SubCommand::with_name("verify")
                .about("Verify a task")
                .arg(Arg::with_name("id").required(true).help("Task ID"))
        )
        .get_matches();
    
    let mut tool = SyncDevTool::new();
    tool.run(&matches);
}