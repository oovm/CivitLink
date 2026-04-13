//! 脚本编辑器面板
//!
//! 提供脚本文件浏览、搜索过滤和外部 IDE 打开功能。

use std::{fs, path::Path};

use gg_core::{GError, GErrorKind, GResult};
use gg_editor_shell::{EditorContext, EditorEvent, EditorPanel, PanelLayoutHint, PanelPosition};
use gg_ui::{UiNodeId, UiTree};

use crate::{ide::IdeLauncher, script_file::ScriptFile};

/// 脚本编辑器面板
///
/// 列出项目中的 Valkyrie 脚本文件（.vk），支持搜索过滤和外部 IDE 打开操作。
pub struct ScriptEditorPanel {
    /// 可见性
    visible: bool,
    /// 滚动偏移
    scroll_offset: (f32, f32),
    /// 脚本文件列表
    script_files: Vec<ScriptFile>,
    /// 搜索关键词
    search_query: String,
    /// 选中的文件索引
    selected_file: Option<usize>,
    /// IDE 启动器
    ide_launcher: IdeLauncher,
    /// 项目根路径
    project_path: String,
}

impl ScriptEditorPanel {
    /// 创建面板
    pub fn new(project_path: String) -> Self {
        let ide_launcher = IdeLauncher::new();
        Self {
            visible: true,
            scroll_offset: (0.0, 0.0),
            script_files: Vec::new(),
            search_query: String::new(),
            selected_file: None,
            ide_launcher,
            project_path,
        }
    }

    /// 扫描项目目录中的 .vk 文件
    pub fn scan_scripts(&mut self) {
        self.script_files.clear();
        self.scan_dir(&self.project_path.clone());
        self.script_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    }

    /// 递归扫描目录
    fn scan_dir(&mut self, dir: &str) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_str = path.to_string_lossy().to_string();
                    self.scan_dir(&dir_str);
                }
                else if path.extension().is_some_and(|ext| ext == "vk") {
                    if let Some(file_name) = path.file_name().map(|n| n.to_string_lossy().to_string()) {
                        let metadata = fs::metadata(&path).ok();
                        let modified_time = metadata
                            .as_ref()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let size = metadata.map(|m| m.len()).unwrap_or(0);
                        self.script_files.push(ScriptFile::new(path, file_name, modified_time, size));
                    }
                }
            }
        }
    }

    /// 返回过滤后的脚本文件列表
    pub fn filtered_scripts(&self) -> Vec<&ScriptFile> {
        if self.search_query.is_empty() {
            return self.script_files.iter().collect();
        }
        self.script_files
            .iter()
            .filter(|f| f.file_name.contains(&self.search_query) || f.path.to_string_lossy().contains(&self.search_query))
            .collect()
    }

    /// 在外部 IDE 中打开选中文件
    pub fn open_in_ide(&mut self, file_index: usize) -> GResult<()> {
        let filtered = self.filtered_scripts();
        let file = filtered
            .get(file_index)
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "无效的文件索引".to_string() })?;
        let path_str = file.path.to_string_lossy().to_string();
        self.ide_launcher.open_file(&path_str)
    }

    /// 在外部 IDE 中打开项目
    pub fn open_project_in_ide(&mut self) -> GResult<()> {
        self.ide_launcher.open_project(&self.project_path)
    }

    /// 更新脚本文件诊断摘要
    pub fn update_diagnostics(&mut self, uri: &str, errors: usize, warnings: usize) {
        for file in &mut self.script_files {
            let file_uri = format!("file:///{}", file.path.to_string_lossy());
            if file_uri == uri || file.path.to_string_lossy() == uri {
                file.update_diagnostics(errors, warnings);
                return;
            }
        }
    }

    /// 获取搜索关键词
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// 设置搜索关键词
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// 获取滚动偏移
    pub fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset
    }

    /// 设置滚动偏移
    pub fn set_scroll_offset(&mut self, offset: (f32, f32)) {
        self.scroll_offset = offset;
    }

    /// 获取脚本文件列表
    pub fn script_files(&self) -> &[ScriptFile] {
        &self.script_files
    }

    /// 获取选中的文件索引
    pub fn selected_file(&self) -> Option<usize> {
        self.selected_file
    }

    /// 设置选中的文件索引
    pub fn set_selected_file(&mut self, index: Option<usize>) {
        self.selected_file = index;
    }

    /// 获取项目根路径
    pub fn project_path(&self) -> &str {
        &self.project_path
    }

    /// 获取 IDE 启动器引用
    pub fn ide_launcher(&self) -> &IdeLauncher {
        &self.ide_launcher
    }

    /// 获取 IDE 启动器可变引用
    pub fn ide_launcher_mut(&mut self) -> &mut IdeLauncher {
        &mut self.ide_launcher
    }
}

impl Default for ScriptEditorPanel {
    fn default() -> Self {
        Self::new(".".to_string())
    }
}

impl EditorPanel for ScriptEditorPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Script Browser"
    }

    /// 获取面板可见性
    fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置面板可见性
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 处理编辑器事件
    ///
    /// 监听 LSP 诊断事件，更新脚本文件的诊断摘要。
    /// 诊断事件数据格式为 "uri:error_count:warning_count" 字符串。
    fn on_event(&mut self, event: &EditorEvent, _context: &mut EditorContext) {
        if let EditorEvent::Custom { name, data } = event {
            if name == "LspDiagnostics" {
                if let Some(diag_str) = data.downcast_ref::<String>() {
                    let parts: Vec<&str> = diag_str.splitn(3, ':').collect();
                    if parts.len() == 3 {
                        if let (Ok(errors), Ok(warnings)) = (parts[1].parse::<usize>(), parts[2].parse::<usize>()) {
                            self.update_diagnostics(parts[0], errors, warnings);
                        }
                    }
                }
            }
        }
    }

    /// 构建脚本编辑器面板 UI 节点树
    fn build_ui(&mut self, _context: &mut EditorContext, _ui_tree: &mut UiTree) -> Option<UiNodeId> {
        None
    }

    /// 获取面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint { position: PanelPosition::Left, preferred_size: Some((300.0, 600.0)), min_size: Some((200.0, 400.0)) }
    }
}
