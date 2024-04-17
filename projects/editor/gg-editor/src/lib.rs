//! GG Editor 核心模块
//!
//! # 废弃通知
//!
//! **此 crate 已废弃，请迁移至 `gg_editor_shell`。**
//!
//! 迁移映射：
//! - `EditorConfig` → `gg_editor_shell::EditorConfig`
//! - `EditorPanel` → `gg_editor_shell::EditorPanel`
//! - `EditorEvent` → `gg_editor_shell::EditorEvent`

#![warn(missing_docs)]
#![deprecated(
    since = "0.1.0",
    note = "gg-editor 已废弃，请迁移至 gg-editor-shell。EditorConfig → gg_editor_shell::EditorConfig, EditorPanel → gg_editor_shell::EditorPanel, EditorEvent → gg_editor_shell::EditorEvent"
)]

use std::{path::Path, sync::Arc};

/// 编辑器接口
pub trait Editor: Send + Sync {
    /// 打开文件
    fn open_file(&mut self, path: &Path) -> Result<(), EditorError>;

    /// 保存文件
    fn save_file(&mut self) -> Result<(), EditorError>;

    /// 保存文件为
    fn save_file_as(&mut self, path: &Path) -> Result<(), EditorError>;

    /// 关闭文件
    fn close_file(&mut self) -> Result<(), EditorError>;

    /// 运行编辑器
    fn run(&mut self);
}

/// 编辑器错误
#[derive(Debug)]
pub enum EditorError {
    /// 文件操作错误
    FileError(std::io::Error),
    /// 解析错误
    ParseError(String),
    /// 编译错误
    CompileError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorError::FileError(e) => write!(f, "文件操作错误: {}", e),
            EditorError::ParseError(s) => write!(f, "解析错误: {}", s),
            EditorError::CompileError(s) => write!(f, "编译错误: {}", s),
            EditorError::Other(s) => write!(f, "其他错误: {}", s),
        }
    }
}

impl std::error::Error for EditorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EditorError::FileError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for EditorError {
    fn from(e: std::io::Error) -> Self {
        EditorError::FileError(e)
    }
}

/// 编辑器配置
pub struct EditorConfig {
    /// 主题名称
    pub theme: String,
    /// 字体大小
    pub font_size: u32,
    /// 是否自动保存
    pub auto_save: bool,
    /// 是否显示行号
    pub show_line_numbers: bool,
    /// 制表符宽度
    pub tab_size: u32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { theme: "dark".to_string(), font_size: 14, auto_save: false, show_line_numbers: true, tab_size: 4 }
    }
}

/// 编辑器工厂
pub trait EditorFactory {
    /// 创建编辑器实例
    fn create_editor(&self, config: EditorConfig) -> Arc<dyn Editor>;
}

/// 编辑器核心功能
pub struct EditorCore {
    config: EditorConfig,
    current_file: Option<String>,
    file_content: String,
}

impl EditorCore {
    /// 创建新的编辑器核心
    pub fn new(config: EditorConfig) -> Self {
        Self { config, current_file: None, file_content: String::new() }
    }

    /// 加载文件内容
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), EditorError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        self.current_file = Some(path.to_string_lossy().to_string());
        self.file_content = content;
        Ok(())
    }

    /// 保存文件内容
    pub fn save_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), EditorError> {
        let path = path.as_ref();
        std::fs::write(path, &self.file_content)?;
        self.current_file = Some(path.to_string_lossy().to_string());
        Ok(())
    }

    /// 获取文件内容
    pub fn get_file_content(&self) -> &str {
        &self.file_content
    }

    /// 设置文件内容
    pub fn set_file_content(&mut self, content: &str) {
        self.file_content = content.to_string();
    }

    /// 获取当前文件路径
    pub fn get_current_file(&self) -> Option<&str> {
        self.current_file.as_deref()
    }
}

/// 编辑器面板
pub trait EditorPanel: Send + Sync {
    /// 面板名称
    fn name(&self) -> &str;

    /// 渲染面板
    fn render(&mut self, editor: &mut EditorCore);

    /// 处理事件
    fn handle_event(&mut self, event: &EditorEvent);
}

/// 编辑器事件
pub enum EditorEvent {
    /// 文件打开事件
    FileOpened(String),
    /// 文件保存事件
    FileSaved(String),
    /// 文件关闭事件
    FileClosed,
    /// 内容变更事件
    ContentChanged(String),
    /// 编译事件
    Compile,
    /// 预览事件
    Preview,
    /// 其他事件
    Other(String),
}

/// 代码编辑器面板
pub struct CodeEditorPanel {
    name: String,
}

impl CodeEditorPanel {
    /// 创建新的代码编辑器面板
    pub fn new() -> Self {
        Self { name: "Code Editor".to_string() }
    }
}

impl EditorPanel for CodeEditorPanel {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&mut self, editor: &mut EditorCore) {
        println!("Rendering code editor with content: {}", editor.get_file_content());
    }

    fn handle_event(&mut self, event: &EditorEvent) {
        match event {
            EditorEvent::ContentChanged(content) => {
                println!("Content changed: {}", content);
            }
            _ => {}
        }
    }
}

/// 预览面板
pub struct PreviewPanel {
    name: String,
}

impl PreviewPanel {
    /// 创建新的预览面板
    pub fn new() -> Self {
        Self { name: "Preview".to_string() }
    }
}

impl EditorPanel for PreviewPanel {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&mut self, _editor: &mut EditorCore) {
        println!("Rendering preview");
    }

    fn handle_event(&mut self, event: &EditorEvent) {
        match event {
            EditorEvent::Preview => {
                println!("Preview requested");
            }
            _ => {}
        }
    }
}

/// 组件面板
pub struct ComponentsPanel {
    name: String,
}

impl ComponentsPanel {
    /// 创建新的组件面板
    pub fn new() -> Self {
        Self { name: "Components".to_string() }
    }
}

impl EditorPanel for ComponentsPanel {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&mut self, _editor: &mut EditorCore) {
        println!("Rendering components panel");
    }

    fn handle_event(&mut self, _event: &EditorEvent) {}
}
