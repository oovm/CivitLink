//! 外部 IDE 启动服务

use std::{path::PathBuf, process::Command};

use gg_core::{GError, GErrorKind, GResult};

/// 外部 IDE 描述符
#[derive(Debug, Clone)]
pub struct IdeDescriptor {
    /// IDE 名称
    pub name: String,
    /// 可执行文件路径
    pub executable_path: PathBuf,
    /// 是否可用（可执行文件是否存在）
    pub available: bool,
}

/// 外部 IDE 启动服务
///
/// 检测系统已安装的 IDE 并提供文件/项目打开功能。
/// 至少支持 VS Code（code）和 Cursor。
pub struct IdeLauncher {
    /// 已检测到的 IDE 列表
    pub detected_ides: Vec<IdeDescriptor>,
    /// 默认 IDE 名称
    pub default_ide: Option<String>,
}

impl IdeLauncher {
    /// 创建新的 IDE 启动器并检测已安装的 IDE
    pub fn new() -> Self {
        let detected_ides = Self::detect();
        let default_ide = detected_ides.first().map(|ide| ide.name.clone());
        Self { detected_ides, default_ide }
    }

    /// 检测系统已安装的 IDE
    ///
    /// 在 Windows 上检测以下 IDE：
    /// - VS Code（code.cmd）
    /// - Cursor（cursor.cmd）
    pub fn detect() -> Vec<IdeDescriptor> {
        let candidates: Vec<(&str, &str)> = vec![("VS Code", "code.cmd"), ("Cursor", "cursor.cmd")];

        candidates
            .into_iter()
            .map(|(name, cmd)| {
                let path = which_ide(cmd);
                let available = path.is_some();
                IdeDescriptor { name: name.to_string(), executable_path: path.unwrap_or_default(), available }
            })
            .filter(|ide| ide.available)
            .collect()
    }

    /// 使用默认 IDE 打开文件
    ///
    /// # 参数
    /// - `path`: 要打开的文件路径
    pub fn open_file(&self, path: &str) -> GResult<()> {
        let ide = self.get_default_ide()?;
        self.launch(&ide.executable_path, &[path])
    }

    /// 使用指定 IDE 打开文件
    ///
    /// # 参数
    /// - `path`: 要打开的文件路径
    /// - `ide_name`: IDE 名称
    pub fn open_file_with(&self, path: &str, ide_name: &str) -> GResult<()> {
        let ide = self.get_ide_by_name(ide_name)?;
        self.launch(&ide.executable_path, &[path])
    }

    /// 使用默认 IDE 打开文件并跳转到指定位置
    ///
    /// # 参数
    /// - `path`: 要打开的文件路径
    /// - `line`: 行号（从 1 开始）
    /// - `column`: 列号（从 1 开始）
    pub fn open_file_at(&self, path: &str, line: u32, column: u32) -> GResult<()> {
        let ide = self.get_default_ide()?;
        let goto_arg = format!("{}:{}:{}", path, line, column);
        self.launch(&ide.executable_path, &["--goto", &goto_arg])
    }

    /// 使用默认 IDE 打开项目文件夹
    ///
    /// # 参数
    /// - `project_path`: 项目根目录路径
    pub fn open_project(&self, project_path: &str) -> GResult<()> {
        let ide = self.get_default_ide()?;
        self.launch(&ide.executable_path, &[project_path])
    }

    /// 获取默认 IDE
    fn get_default_ide(&self) -> GResult<&IdeDescriptor> {
        if let Some(ref name) = self.default_ide {
            self.get_ide_by_name(name)
        }
        else {
            self.detected_ides
                .first()
                .ok_or_else(|| GError { kind: GErrorKind::Other, message: "未检测到可用的 IDE".to_string() })
        }
    }

    /// 根据名称获取 IDE
    fn get_ide_by_name(&self, name: &str) -> GResult<&IdeDescriptor> {
        self.detected_ides
            .iter()
            .find(|ide| ide.name == name)
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: format!("未找到 IDE: {}", name) })
    }

    /// 启动 IDE 进程
    fn launch(&self, executable: &PathBuf, args: &[&str]) -> GResult<()> {
        Command::new(executable)
            .args(args)
            .spawn()
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("启动 IDE 失败: {}", e) })?;
        Ok(())
    }
}

impl Default for IdeLauncher {
    fn default() -> Self {
        Self::new()
    }
}

/// 在 PATH 中查找 IDE 可执行文件
fn which_ide(cmd: &str) -> Option<PathBuf> {
    let candidates = [
        format!("C:\\Program Files\\Microsoft VS Code\\bin\\{}", cmd),
        format!("C:\\Program Files\\Cursor\\bin\\{}", cmd),
        format!("C:\\Users\\{}\\AppData\\Local\\Programs\\Microsoft VS Code\\bin\\{}", whoami(), cmd),
        format!("C:\\Users\\{}\\AppData\\Local\\Programs\\Cursor\\bin\\{}", whoami(), cmd),
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(output) = Command::new("where").arg(cmd).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = stdout.lines().next() {
                let path = PathBuf::from(first_line.trim());
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// 获取当前用户名
fn whoami() -> String {
    std::env::var("USERNAME").unwrap_or_default()
}
