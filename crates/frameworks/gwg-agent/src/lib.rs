//! 子代理框架
//! 
//! 提供基于子代理的并行开发机制

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 处理中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
}

/// 任务类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    /// 编译任务
    Compile,
    /// 测试任务
    Test,
    /// 构建任务
    Build,
    /// 验证任务
    Verify,
}

/// 任务
#[derive(Debug, Clone)]
pub struct Task {
    /// 任务ID
    pub id: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务描述
    pub description: String,
    /// 依赖任务ID
    pub dependencies: Vec<String>,
    /// 任务状态
    pub status: TaskStatus,
    /// 执行结果
    pub result: Option<String>,
}

impl Task {
    /// 创建新任务
    pub fn new(id: &str, task_type: TaskType, description: &str) -> Self {
        Self {
            id: id.to_string(),
            task_type,
            description: description.to_string(),
            dependencies: vec![],
            status: TaskStatus::Pending,
            result: None,
        }
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, dependency_id: &str) {
        self.dependencies.push(dependency_id.to_string());
    }

    /// 更新状态
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// 设置结果
    pub fn set_result(&mut self, result: &str) {
        self.result = Some(result.to_string());
    }
}

/// 子代理
pub struct SubAgent {
    /// 代理ID
    id: String,
    /// 任务队列
    tasks: Arc<Mutex<Vec<Task>>>,
    /// 状态回调
    status_callback: Option<Box<dyn Fn(&Task) + Send + Sync>>,
}

impl SubAgent {
    /// 创建新子代理
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            tasks: Arc::new(Mutex::new(vec![])),
            status_callback: None,
        }
    }

    /// 设置状态回调
    pub fn set_status_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Task) + Send + Sync + 'static,
    {
        self.status_callback = Some(Box::new(callback));
    }

    /// 添加任务
    pub fn add_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }

    /// 开始执行
    pub fn start(&self) {
        let tasks = self.tasks.clone();
        let id = self.id.clone();
        let callback = self.status_callback.clone();

        thread::spawn(move || {
            loop {
                let mut tasks = tasks.lock().unwrap();
                
                // 查找待处理的任务
                if let Some(index) = tasks.iter().position(|t| t.status == TaskStatus::Pending) {
                    let mut task = &mut tasks[index];
                    
                    // 更新状态为处理中
                    task.update_status(TaskStatus::InProgress);
                    if let Some(ref callback) = callback {
                        callback(task);
                    }
                
                    // 模拟任务执行
                    thread::sleep(Duration::from_secs(2));
                
                    // 更新状态为完成
                    task.update_status(TaskStatus::Completed);
                    task.set_result(&format!("Task {} completed by agent {}", task.id, id));
                    if let Some(ref callback) = callback {
                        callback(task);
                    }
                } else {
                    // 无任务可执行，休眠一段时间
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });
    }

    /// 获取任务状态
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let tasks = self.tasks.lock().unwrap();
        tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.status.clone())
    }

    /// 获取所有任务
    pub fn get_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.clone()
    }
}

/// 代理管理器
pub struct AgentManager {
    /// 子代理列表
    agents: Arc<Mutex<Vec<SubAgent>>>,
    /// 任务列表
    tasks: Arc<Mutex<Vec<Task>>>,
}

impl AgentManager {
    /// 创建新的代理管理器
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(vec![])),
            tasks: Arc::new(Mutex::new(vec![])),
        }
    }

    /// 添加子代理
    pub fn add_agent(&self, agent: SubAgent) {
        let mut agents = self.agents.lock().unwrap();
        agents.push(agent);
    }

    /// 创建并添加子代理
    pub fn create_agent(&self, id: &str) -> SubAgent {
        let agent = SubAgent::new(id);
        self.add_agent(agent.clone());
        agent
    }

    /// 添加任务
    pub fn add_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }

    /// 分配任务
    pub fn assign_tasks(&self) {
        let tasks = self.tasks.lock().unwrap();
        let mut agents = self.agents.lock().unwrap();
        
        // 简单的轮询分配策略
        let mut agent_index = 0;
        for task in tasks.iter() {
            if task.status == TaskStatus::Pending {
                if agent_index < agents.len() {
                    agents[agent_index].add_task(task.clone());
                    agent_index = (agent_index + 1) % agents.len();
                }
            }
        }
    }

    /// 启动所有代理
    pub fn start_all(&self) {
        let agents = self.agents.lock().unwrap();
        for agent in agents.iter() {
            agent.start();
        }
    }

    /// 获取所有任务状态
    pub fn get_all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.clone()
    }

    /// 获取任务状态
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let tasks = self.tasks.lock().unwrap();
        tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.status.clone())
    }
}

/// 预导入模块
pub mod prelude {
    pub use super::{AgentManager, SubAgent, Task, TaskStatus, TaskType};
}