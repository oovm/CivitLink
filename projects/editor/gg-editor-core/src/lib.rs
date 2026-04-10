//! GG Editor 核心模块

use std::path::Path;
use std::sync::Arc;

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
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("文件操作错误: {0}")]
    FileError(#[from] std::io::Error),
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("编译错误: {0}")]
    CompileError(String),
    #[error("其他错误: {0}")]
    Other(String),
}

/// 编辑器配置
pub struct EditorConfig {
    pub theme: String,
    pub font_size: u32,
    pub auto_save: bool,
    pub show_line_numbers: bool,
    pub tab_size: u32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14,
            auto_save: false,
            show_line_numbers: true,
            tab_size: 4,
        }
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
        Self {
            config,
            current_file: None,
            file_content: String::new(),
        }
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
    pub fn new() -> Self {
        Self {
            name: "Code Editor".to_string(),
        }
    }
}

impl EditorPanel for CodeEditorPanel {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn render(&mut self, editor: &mut EditorCore) {
        // 渲染代码编辑器
        println!("Rendering code editor with content: {}", editor.get_file_content());
    }
    
    fn handle_event(&mut self, event: &EditorEvent) {
        // 处理事件
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
    pub fn new() -> Self {
        Self {
            name: "Preview".to_string(),
        }
    }
}

impl EditorPanel for PreviewPanel {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn render(&mut self, editor: &mut EditorCore) {
        // 渲染预览
        println!("Rendering preview");
    }
    
    fn handle_event(&mut self, event: &EditorEvent) {
        // 处理事件
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
    pub fn new() -> Self {
        Self {
            name: "Components".to_string(),
        }
    }
}

impl EditorPanel for ComponentsPanel {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn render(&mut self, editor: &mut EditorCore) {
        // 渲染组件面板
        println!("Rendering components panel");
    }
    
    fn handle_event(&mut self, event: &EditorEvent) {
        // 处理事件
    }
}
