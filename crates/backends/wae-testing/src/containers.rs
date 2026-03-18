//! 测试容器管理模块
//!
//! 提供基于 Docker CLI 的轻量级测试容器支持，包括 PostgreSQL、MySQL 和 Redis。
//! 不依赖任何第三方 Docker 客户端库，直接调用 docker 命令。

use std::{
    collections::HashMap,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// 测试容器错误类型
#[allow(dead_code)]
#[derive(Debug)]
pub enum ContainerError {
    /// Docker 命令执行失败
    CommandFailed(String),

    /// 容器启动超时
    Timeout,

    /// 容器未找到
    NotFound(String),

    /// IO 错误
    Io(std::io::Error),
}

impl std::error::Error for ContainerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ContainerError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::CommandFailed(msg) => write!(f, "Docker command failed: {}", msg),
            ContainerError::Timeout => write!(f, "Container startup timeout"),
            ContainerError::NotFound(name) => write!(f, "Container not found: {}", name),
            ContainerError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<std::io::Error> for ContainerError {
    fn from(err: std::io::Error) -> Self {
        ContainerError::Io(err)
    }
}

/// 测试容器结果类型
#[allow(dead_code)]
pub type ContainerResult<T> = Result<T, ContainerError>;

/// 测试容器 trait，定义容器生命周期管理方法
#[allow(dead_code)]
pub trait TestContainer {
    /// 获取容器的连接 URL
    fn connection_url(&self) -> String;

    /// 启动容器
    async fn start() -> ContainerResult<Self>
    where
        Self: Sized;

    /// 停止容器
    async fn stop(&mut self) -> ContainerResult<()>;

    /// 清理容器资源
    async fn cleanup(self) -> ContainerResult<()>;
}

/// 检查 Docker 是否可用
#[allow(dead_code)]
pub fn is_docker_available() -> bool {
    let output = Command::new("docker").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// 执行 Docker 命令
#[allow(dead_code)]
fn docker_command(args: &[&str]) -> ContainerResult<String> {
    let output = Command::new("docker").args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(stdout)
}

/// 等待容器就绪
#[allow(dead_code)]
async fn wait_for_ready(_container_id: &str, check_fn: impl Fn() -> bool, timeout: Duration) -> ContainerResult<()> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if check_fn() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(ContainerError::Timeout)
}

/// 检查容器日志是否包含特定消息
#[allow(dead_code)]
fn check_container_log(container_id: &str, message: &str) -> bool {
    let output = Command::new("docker").args(["logs", container_id]).stdout(Stdio::piped()).stderr(Stdio::piped()).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            stdout.contains(message) || stderr.contains(message)
        }
        Err(_) => false,
    }
}

/// PostgreSQL 容器
#[allow(dead_code)]
pub struct PostgresContainer {
    container_id: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

/// PostgreSQL 镜像配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostgresImage {
    tag: String,
    username: String,
    password: String,
    database: String,
}

#[allow(dead_code)]
impl Default for PostgresImage {
    fn default() -> Self {
        Self {
            tag: "15-alpine".to_string(),
            username: "postgres".to_string(),
            password: "password".to_string(),
            database: "test_db".to_string(),
        }
    }
}

#[allow(dead_code)]
impl PostgresContainer {
    /// 创建默认 PostgreSQL 容器
    pub async fn default() -> ContainerResult<Self> {
        Self::new(PostgresImage::default()).await
    }

    /// 使用自定义配置创建 PostgreSQL 容器
    pub async fn new(image: PostgresImage) -> ContainerResult<Self> {
        let mut env_vars = HashMap::new();
        env_vars.insert("POSTGRES_USER", image.username.clone());
        env_vars.insert("POSTGRES_PASSWORD", image.password.clone());
        env_vars.insert("POSTGRES_DB", image.database.clone());

        let container_id = run_container(&format!("postgres:{}", image.tag), &[5432], &env_vars)?;

        let port = get_host_port(&container_id, 5432)?;
        let host = "127.0.0.1".to_string();

        wait_for_ready(
            &container_id,
            || check_container_log(&container_id, "database system is ready to accept connections"),
            Duration::from_secs(30),
        )
        .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(Self { container_id, host, port, username: image.username, password: image.password, database: image.database })
    }

    /// 获取容器主机
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 获取容器端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取用户名
    pub fn username(&self) -> &str {
        &self.username
    }

    /// 获取密码
    pub fn password(&self) -> &str {
        &self.password
    }

    /// 获取数据库名
    pub fn database(&self) -> &str {
        &self.database
    }
}

#[allow(dead_code)]
impl TestContainer for PostgresContainer {
    fn connection_url(&self) -> String {
        format!("postgres://{}:{}@{}:{}/{}", self.username, self.password, self.host, self.port, self.database)
    }

    async fn start() -> ContainerResult<Self> {
        Self::default().await
    }

    async fn stop(&mut self) -> ContainerResult<()> {
        docker_command(&["stop", &self.container_id])?;
        Ok(())
    }

    async fn cleanup(self) -> ContainerResult<()> {
        docker_command(&["rm", "-f", &self.container_id])?;
        Ok(())
    }
}

/// MySQL 容器
#[allow(dead_code)]
pub struct MySqlContainer {
    container_id: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

/// MySQL 镜像配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MySqlImage {
    tag: String,
    username: String,
    password: String,
    database: String,
    root_password: String,
}

#[allow(dead_code)]
impl Default for MySqlImage {
    fn default() -> Self {
        Self {
            tag: "8.0".to_string(),
            username: "mysql".to_string(),
            password: "password".to_string(),
            database: "test_db".to_string(),
            root_password: "root_password".to_string(),
        }
    }
}

#[allow(dead_code)]
impl MySqlContainer {
    /// 创建默认 MySQL 容器
    pub async fn default() -> ContainerResult<Self> {
        Self::new(MySqlImage::default()).await
    }

    /// 使用自定义配置创建 MySQL 容器
    pub async fn new(image: MySqlImage) -> ContainerResult<Self> {
        let mut env_vars = HashMap::new();
        env_vars.insert("MYSQL_ROOT_PASSWORD", image.root_password.clone());
        env_vars.insert("MYSQL_USER", image.username.clone());
        env_vars.insert("MYSQL_PASSWORD", image.password.clone());
        env_vars.insert("MYSQL_DATABASE", image.database.clone());

        let container_id = run_container(&format!("mysql:{}", image.tag), &[3306], &env_vars)?;

        let port = get_host_port(&container_id, 3306)?;
        let host = "127.0.0.1".to_string();

        wait_for_ready(&container_id, || check_container_log(&container_id, "ready for connections"), Duration::from_secs(60))
            .await?;

        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(Self { container_id, host, port, username: image.username, password: image.password, database: image.database })
    }

    /// 获取容器主机
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 获取容器端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取用户名
    pub fn username(&self) -> &str {
        &self.username
    }

    /// 获取密码
    pub fn password(&self) -> &str {
        &self.password
    }

    /// 获取数据库名
    pub fn database(&self) -> &str {
        &self.database
    }
}

#[allow(dead_code)]
impl TestContainer for MySqlContainer {
    fn connection_url(&self) -> String {
        format!("mysql://{}:{}@{}:{}/{}", self.username, self.password, self.host, self.port, self.database)
    }

    async fn start() -> ContainerResult<Self> {
        Self::default().await
    }

    async fn stop(&mut self) -> ContainerResult<()> {
        docker_command(&["stop", &self.container_id])?;
        Ok(())
    }

    async fn cleanup(self) -> ContainerResult<()> {
        docker_command(&["rm", "-f", &self.container_id])?;
        Ok(())
    }
}

/// Redis 容器
#[allow(dead_code)]
pub struct RedisContainer {
    container_id: String,
    host: String,
    port: u16,
    password: Option<String>,
}

/// Redis 镜像配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RedisImage {
    tag: String,
    password: Option<String>,
}

#[allow(dead_code)]
impl Default for RedisImage {
    fn default() -> Self {
        Self { tag: "7-alpine".to_string(), password: None }
    }
}

#[allow(dead_code)]
impl RedisContainer {
    /// 创建默认 Redis 容器
    pub async fn default() -> ContainerResult<Self> {
        Self::new(RedisImage::default()).await
    }

    /// 使用自定义配置创建 Redis 容器
    pub async fn new(image: RedisImage) -> ContainerResult<Self> {
        let env_vars = HashMap::new();
        let mut cmd_args = Vec::new();

        if let Some(pass) = &image.password {
            cmd_args.push("--requirepass".to_string());
            cmd_args.push(pass.clone());
        }

        let container_id = run_container_with_cmd(
            &format!("redis:{}", image.tag),
            &[6379],
            &env_vars,
            if cmd_args.is_empty() { None } else { Some(&cmd_args) },
        )?;

        let port = get_host_port(&container_id, 6379)?;
        let host = "127.0.0.1".to_string();

        wait_for_ready(
            &container_id,
            || check_container_log(&container_id, "Ready to accept connections"),
            Duration::from_secs(30),
        )
        .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(Self { container_id, host, port, password: image.password })
    }

    /// 获取容器主机
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 获取容器端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取密码（如果设置）
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }
}

#[allow(dead_code)]
impl TestContainer for RedisContainer {
    fn connection_url(&self) -> String {
        match &self.password {
            Some(pass) => format!("redis://:{}@{}:{}", pass, self.host, self.port),
            None => format!("redis://{}:{}", self.host, self.port),
        }
    }

    async fn start() -> ContainerResult<Self> {
        Self::default().await
    }

    async fn stop(&mut self) -> ContainerResult<()> {
        docker_command(&["stop", &self.container_id])?;
        Ok(())
    }

    async fn cleanup(self) -> ContainerResult<()> {
        docker_command(&["rm", "-f", &self.container_id])?;
        Ok(())
    }
}

/// 运行容器
#[allow(dead_code)]
fn run_container(image: &str, ports: &[u16], env_vars: &HashMap<&str, String>) -> ContainerResult<String> {
    run_container_with_cmd(image, ports, env_vars, None)
}

/// 运行容器（带自定义命令）
#[allow(dead_code)]
fn run_container_with_cmd(
    image: &str,
    ports: &[u16],
    env_vars: &HashMap<&str, String>,
    cmd: Option<&[String]>,
) -> ContainerResult<String> {
    let mut args: Vec<String> = vec!["run".to_string(), "-d".to_string(), "--rm".to_string()];

    for port in ports {
        args.push("-p".to_string());
        args.push(format!("{}:{}", 0, port));
    }

    for (key, value) in env_vars {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }

    args.push(image.to_string());

    if let Some(cmd_args) = cmd {
        for arg in cmd_args {
            args.push(arg.clone());
        }
    }

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let container_id = docker_command(&args_ref)?;
    Ok(container_id)
}

/// 获取主机端口映射
#[allow(dead_code)]
fn get_host_port(container_id: &str, container_port: u16) -> ContainerResult<u16> {
    let output = docker_command(&["port", container_id, &container_port.to_string()])?;

    let port_str = output
        .split(':')
        .next_back()
        .ok_or_else(|| ContainerError::CommandFailed("Failed to parse port mapping".to_string()))?;

    let port = port_str.parse().map_err(|_| ContainerError::CommandFailed("Failed to parse port number".to_string()))?;

    Ok(port)
}
