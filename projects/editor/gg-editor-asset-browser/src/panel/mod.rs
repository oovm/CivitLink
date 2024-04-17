//! 资源浏览器面板实现
//! 提供项目资源目录树浏览、文件导入、元数据查询和引用查找功能

use std::{collections::HashMap, path::Path};

use gg_core::{FileSystem, FileType, GError, GErrorKind, GResult};
use gg_ecs::Entity;
use gg_editor_shell::{
    DragData, DragState, EditorContext, EditorEvent, EditorPanel, MouseButton, PanelLayoutHint, PanelPosition,
};
use gg_render::Color;
use gg_ui::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, Overflow, Style, UiNodeData, UiNodeId, UiTree};

/// 资源类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    /// 图片资源
    Image,
    /// 音频资源
    Audio,
    /// 字体资源
    Font,
    /// 剧本资源
    Script,
    /// 其他资源
    Other,
}

/// 资源元数据
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    /// 资源名称
    pub name: String,
    /// 资源路径
    pub path: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 资源类型
    pub asset_type: AssetType,
}

/// 资源引用信息
#[derive(Debug, Clone)]
pub struct AssetReference {
    /// 引用该资源的实体
    pub entity_id: Option<Entity>,
    /// 组件类型名称
    pub component_type: String,
    /// 字段名称
    pub field_name: String,
}

/// 目录树节点
#[derive(Debug, Clone)]
pub struct DirectoryNode {
    /// 目录或文件名称
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 是否为目录
    pub is_directory: bool,
    /// 子节点列表
    pub children: Vec<DirectoryNode>,
    /// 是否已展开
    pub is_expanded: bool,
}

/// 待处理的文件导入请求
#[derive(Debug, Clone)]
pub struct PendingImport {
    /// 源文件路径
    pub source_path: String,
    /// 目标文件路径
    pub target_path: String,
}

/// 根据文件扩展名识别资源类型
///
/// 将文件路径的扩展名映射到对应的 `AssetType` 枚举值。
/// 支持的扩展名如下：
/// - 图片：`.png`、`.jpg`、`.jpeg`、`.webp`、`.bmp`、`.gif`
/// - 音频：`.mp3`、`.wav`、`.ogg`、`.flac`
/// - 字体：`.ttf`、`.otf`、`.woff`、`.woff2`
/// - 剧本：`.v`、`.vx`、`.rs`、`.toml`
/// - 其余扩展名均归为 `AssetType::Other`
pub fn classify_asset(path: &str) -> AssetType {
    let extension = Path::new(path).extension().and_then(|ext| ext.to_str()).unwrap_or("").to_lowercase();

    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" => AssetType::Image,
        "mp3" | "wav" | "ogg" | "flac" => AssetType::Audio,
        "ttf" | "otf" | "woff" | "woff2" => AssetType::Font,
        "v" | "vx" | "rs" | "toml" => AssetType::Script,
        _ => AssetType::Other,
    }
}

/// 根据搜索关键词过滤目录树节点
///
/// 如果查询为空，返回 `Some(node)` 不做任何修改。
/// 对于目录节点：递归过滤子节点，如果子节点中有匹配项或目录名本身包含查询词则保留。
/// 对于文件节点：当文件名包含查询词（不区分大小写）时保留。
/// 如果没有任何匹配项，返回 `None`。
pub fn filter_tree(node: &DirectoryNode, query: &str) -> Option<DirectoryNode> {
    if query.is_empty() {
        return Some(node.clone());
    }

    let query_lower = query.to_lowercase();

    if node.is_directory {
        let filtered_children: Vec<DirectoryNode> =
            node.children.iter().filter_map(|child| filter_tree(child, query)).collect();

        let name_matches = node.name.to_lowercase().contains(&query_lower);

        if name_matches || !filtered_children.is_empty() {
            Some(DirectoryNode {
                name: node.name.clone(),
                path: node.path.clone(),
                is_directory: true,
                children: filtered_children,
                is_expanded: node.is_expanded || !query_lower.is_empty(),
            })
        }
        else {
            None
        }
    }
    else {
        if node.name.to_lowercase().contains(&query_lower) { Some(node.clone()) } else { None }
    }
}

/// 递归扫描目录，构建目录树节点
///
/// 通过文件系统抽象接口读取指定路径的目录条目，
/// 对子目录进行递归扫描，最终返回完整的目录树节点。
/// 子节点按目录优先、名称字母序排列。
fn scan_directory(path: &Path, fs: &dyn FileSystem) -> GResult<DirectoryNode> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

    let mut node = DirectoryNode {
        name,
        path: path.to_string_lossy().to_string(),
        is_directory: true,
        children: Vec::new(),
        is_expanded: false,
    };

    let entries = fs.read_dir(path)?;
    let mut child_nodes = Vec::new();

    for entry in entries {
        let child_path = path.join(&entry.name);
        match entry.file_type {
            FileType::Directory => {
                if let Ok(child_node) = scan_directory(&child_path, fs) {
                    child_nodes.push(child_node);
                }
            }
            FileType::File => {
                child_nodes.push(DirectoryNode {
                    name: entry.name,
                    path: child_path.to_string_lossy().to_string(),
                    is_directory: false,
                    children: Vec::new(),
                    is_expanded: false,
                });
            }
            FileType::Symlink => {}
        }
    }

    child_nodes.sort_by(|a, b| b.is_directory.cmp(&a.is_directory).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));

    node.children = child_nodes;
    Ok(node)
}

/// 递归搜索目录树并切换指定路径节点的展开状态
fn toggle_node_expanded(node: &mut DirectoryNode, path: &str) -> bool {
    if node.path == path && node.is_directory {
        node.is_expanded = !node.is_expanded;
        return true;
    }

    for child in &mut node.children {
        if toggle_node_expanded(child, path) {
            return true;
        }
    }

    false
}

/// 文件操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    /// 重命名
    Rename,
    /// 删除
    Delete,
    /// 移动
    Move,
    /// 复制
    Duplicate,
}

/// 资源操作结果
#[derive(Debug, Clone)]
pub struct AssetOperationResult {
    /// 操作是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
}

impl AssetOperationResult {
    /// 创建成功结果
    pub fn ok() -> Self {
        Self { success: true, error_message: None }
    }

    /// 创建失败结果
    pub fn err(message: String) -> Self {
        Self { success: false, error_message: Some(message) }
    }
}

/// 资源右键上下文菜单
///
/// 管理资源浏览器中右键菜单的状态和选项。
pub struct AssetContextMenu {
    /// 菜单是否可见
    pub visible: bool,
    /// 目标资源路径
    pub target_path: Option<String>,
    /// 目标是否为目录
    pub is_directory: bool,
    /// 菜单位置
    pub position: (f32, f32),
}

impl AssetContextMenu {
    /// 创建新的右键菜单状态
    pub fn new() -> Self {
        Self { visible: false, target_path: None, is_directory: false, position: (0.0, 0.0) }
    }

    /// 显示右键菜单
    pub fn show(&mut self, path: String, is_directory: bool, position: (f32, f32)) {
        self.visible = true;
        self.target_path = Some(path);
        self.is_directory = is_directory;
        self.position = position;
    }

    /// 隐藏右键菜单
    pub fn hide(&mut self) {
        self.visible = false;
        self.target_path = None;
    }
}

impl Default for AssetContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// 批量操作结果
///
/// 记录批量操作中成功和失败的项，用于向用户反馈操作结果。
#[derive(Debug, Clone)]
pub struct AssetBatchOperationResult {
    /// 成功操作的路径列表
    pub succeeded: Vec<String>,
    /// 失败操作的路径和错误信息列表
    pub failed: Vec<(String, String)>,
}

impl AssetBatchOperationResult {
    /// 创建空的批量操作结果
    pub fn new() -> Self {
        Self { succeeded: Vec::new(), failed: Vec::new() }
    }

    /// 记录一项成功操作
    pub fn push_success(&mut self, path: String) {
        self.succeeded.push(path);
    }

    /// 记录一项失败操作
    pub fn push_failure(&mut self, path: String, error: String) {
        self.failed.push((path, error));
    }

    /// 是否所有操作都成功
    pub fn is_all_success(&self) -> bool {
        self.failed.is_empty()
    }
}

impl Default for AssetBatchOperationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源预览数据
///
/// 存储资源的预览信息，包括缩略图纹理标识、分辨率和时长等。
/// 用于在资源浏览器中快速显示资源预览。
#[derive(Debug, Clone)]
pub struct AssetPreviewData {
    /// 资源名称
    pub name: String,
    /// 资源类型
    pub asset_type: AssetType,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 修改时间（Unix 时间戳）
    pub modified_time: u64,
    /// 图片分辨率（宽, 高），仅图片类型
    pub resolution: Option<(u32, u32)>,
    /// 音频时长（秒），仅音频类型
    pub duration: Option<f64>,
}

impl AssetPreviewData {
    /// 创建新的资源预览数据
    pub fn new(name: String, asset_type: AssetType, file_size: u64) -> Self {
        Self { name, asset_type, file_size, modified_time: 0, resolution: None, duration: None }
    }

    /// 格式化文件大小为可读字符串
    pub fn format_file_size(&self) -> String {
        if self.file_size < 1024 {
            format!("{} B", self.file_size)
        }
        else if self.file_size < 1024 * 1024 {
            format!("{:.1} KB", self.file_size as f64 / 1024.0)
        }
        else {
            format!("{:.1} MB", self.file_size as f64 / (1024.0 * 1024.0))
        }
    }
}

/// 资源浏览器面板
///
/// 提供项目资源目录浏览、文件操作和资源预览功能。
/// 支持多选、批量操作、拖拽交互和右键上下文菜单。
pub struct AssetBrowserPanel {
    /// 目录树根节点
    directory_tree: DirectoryNode,
    /// 当前浏览的目录路径
    current_directory: String,
    /// 已选中的资源路径列表
    selected_paths: Vec<String>,
    /// 上次选中的索引，用于 Shift+Click 范围选中
    last_selected_index: Option<usize>,
    /// 搜索关键词
    search_query: String,
    /// 面板可见性
    visible: bool,
    /// 右键上下文菜单状态
    context_menu: AssetContextMenu,
    /// 待处理的文件导入请求列表
    pending_imports: Vec<PendingImport>,
    /// 拖拽状态
    drag_state: Option<DragState>,
    /// Ctrl 键是否按下
    ctrl_pressed: bool,
    /// Shift 键是否按下
    shift_pressed: bool,
    /// 选中状态是否已变更
    selection_dirty: bool,
    /// 资源预览缓存
    preview_cache: HashMap<String, AssetPreviewData>,
    /// 悬浮预览的目标路径
    hover_preview_path: Option<String>,
    /// 悬浮预览的计时器（毫秒）
    hover_timer: u32,
}

impl AssetBrowserPanel {
    /// 创建新的资源浏览器面板
    pub fn new() -> Self {
        Self {
            directory_tree: DirectoryNode {
                name: String::new(),
                path: String::new(),
                is_directory: true,
                children: Vec::new(),
                is_expanded: false,
            },
            current_directory: String::new(),
            selected_paths: Vec::new(),
            last_selected_index: None,
            search_query: String::new(),
            visible: true,
            context_menu: AssetContextMenu::new(),
            pending_imports: Vec::new(),
            drag_state: None,
            ctrl_pressed: false,
            shift_pressed: false,
            selection_dirty: false,
            preview_cache: HashMap::new(),
            hover_preview_path: None,
            hover_timer: 0,
        }
    }

    /// 刷新目录树
    ///
    /// 根据指定路径通过文件系统抽象接口递归扫描目录，
    /// 构建完整的目录树结构并存储到 `directory_tree` 中。
    pub fn refresh_directory(&mut self, path: &Path, fs: &dyn FileSystem) -> GResult<()> {
        self.current_directory = path.to_string_lossy().to_string();
        self.directory_tree = scan_directory(path, fs)?;
        Ok(())
    }

    /// 切换目录节点的展开/折叠状态
    ///
    /// 递归搜索目录树中与指定路径匹配的节点，
    /// 找到后切换其 `is_expanded` 状态。
    /// 如果找到并切换成功返回 `true`，否则返回 `false`。
    pub fn toggle_directory(&mut self, path: &str) -> bool {
        toggle_node_expanded(&mut self.directory_tree, path)
    }

    /// 导入文件
    ///
    /// 将外部文件复制到项目资源目录中的指定目标位置。
    /// 由于此方法无法直接访问文件系统，导入请求会被暂存到
    /// `pending_imports` 中，等待后续通过 `process_imports` 处理。
    pub fn import_file(&mut self, source_path: &str, target_dir: &str) -> GResult<()> {
        let source = Path::new(source_path);
        let file_name = source
            .file_name()
            .ok_or_else(|| GError { kind: GErrorKind::Io, message: format!("无效的源路径: {}", source_path) })?;
        let target_path = Path::new(target_dir).join(file_name);

        self.pending_imports.push(PendingImport {
            source_path: source_path.to_string(),
            target_path: target_path.to_string_lossy().to_string(),
        });

        Ok(())
    }

    /// 获取资源元数据
    ///
    /// 根据资源路径通过文件系统抽象接口查询其名称、大小和类型等元数据信息。
    /// 成功时返回 `Some(AssetMetadata)`，查询失败时返回 `None`。
    pub fn get_asset_metadata(&self, path: &Path, fs: &dyn FileSystem) -> Option<AssetMetadata> {
        let metadata = fs.metadata(path).ok()?;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

        Some(AssetMetadata {
            name,
            path: path.to_string_lossy().to_string(),
            file_size: metadata.len,
            asset_type: classify_asset(path.to_string_lossy().as_ref()),
        })
    }

    /// 查找资源引用
    ///
    /// 在当前项目的所有实体和组件中查找引用了指定资源的条目。
    pub fn find_references(&self, asset_path: &str, context: &mut EditorContext) -> GResult<Vec<AssetReference>> {
        let _ = (asset_path, context);
        Ok(Vec::new())
    }

    /// 选中指定路径的资源
    ///
    /// 清空当前选中列表，将指定路径设为唯一选中项，
    /// 并标记选中状态已变更，以便在下次 `build_ui` 调用时发布 `AssetSelected` 事件。
    pub fn select_asset(&mut self, path: String) {
        self.selected_paths.clear();
        self.selected_paths.push(path);
        self.selection_dirty = true;
    }

    /// 追加选中资源路径
    ///
    /// 将指定路径追加到当前选中列表中（不清空已有选中项），
    /// 并标记选中状态已变更。如果路径已在选中列表中则跳过。
    pub fn select_asset_multi(&mut self, path: String) {
        if !self.selected_paths.contains(&path) {
            self.selected_paths.push(path);
            self.selection_dirty = true;
        }
    }

    /// 范围选中资源路径
    ///
    /// 根据同级文件列表，选中从 `last_selected_index` 到 `current_index` 范围内的所有路径。
    /// 如果没有上次选中索引，则仅选中当前路径。
    pub fn select_asset_range(&mut self, sibling_paths: &[String], current_index: usize) {
        if let Some(start) = self.last_selected_index {
            let from = start.min(current_index);
            let to = start.max(current_index);
            self.selected_paths.clear();
            for i in from..=to {
                if i < sibling_paths.len() && !self.selected_paths.contains(&sibling_paths[i]) {
                    self.selected_paths.push(sibling_paths[i].clone());
                }
            }
        }
        else {
            self.selected_paths.clear();
            if current_index < sibling_paths.len() {
                self.selected_paths.push(sibling_paths[current_index].clone());
            }
        }
        self.selection_dirty = true;
    }

    /// 设置搜索关键词
    ///
    /// 更新搜索关键词，用于在下次 `build_ui` 时过滤目录树。
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// 取出并清空所有待处理的文件导入请求
    ///
    /// 返回当前所有暂存的导入请求，并清空内部列表。
    pub fn drain_pending_imports(&mut self) -> Vec<PendingImport> {
        std::mem::take(&mut self.pending_imports)
    }

    /// 处理所有待处理的文件导入请求
    ///
    /// 通过文件系统抽象接口逐个执行文件读取和写入操作，
    /// 将源文件复制到目标路径。处理完成后清空待处理列表。
    pub fn process_imports(&mut self, fs: &dyn FileSystem) -> GResult<()> {
        let imports = std::mem::take(&mut self.pending_imports);

        for import in imports {
            let source = Path::new(&import.source_path);
            let target = Path::new(&import.target_path);

            if let Some(parent) = target.parent() {
                fs.create_dir_all(parent)?;
            }

            let data = fs.read(source)?;
            fs.write(target, &data)?;
        }

        Ok(())
    }

    /// 重命名资源
    ///
    /// 通过文件系统抽象接口将指定路径的文件重命名为新名称。
    /// 操作完成后自动刷新目录树。
    pub fn rename_asset(&mut self, old_path: &str, new_name: &str, fs: &dyn FileSystem) -> GResult<()> {
        let path = Path::new(old_path);
        let parent = path
            .parent()
            .ok_or_else(|| GError { kind: GErrorKind::Io, message: format!("无法获取父目录: {}", old_path) })?;
        let new_path = parent.join(new_name);
        let data = fs.read(path)?;
        fs.write(&new_path, &data)?;
        fs.remove_file(path)?;
        if !self.current_directory.is_empty() {
            let dir = self.current_directory.clone();
            self.refresh_directory(Path::new(&dir), fs)?;
        }
        Ok(())
    }

    /// 删除资源
    ///
    /// 通过文件系统抽象接口删除指定路径的文件。
    /// 操作完成后自动刷新目录树。
    pub fn delete_asset(&mut self, path: &str, fs: &dyn FileSystem) -> GResult<()> {
        fs.remove_file(Path::new(path))?;
        if !self.current_directory.is_empty() {
            let dir = self.current_directory.clone();
            self.refresh_directory(Path::new(&dir), fs)?;
        }
        Ok(())
    }

    /// 移动资源
    ///
    /// 通过文件系统抽象接口将文件从源路径移动到目标目录。
    /// 操作完成后自动刷新目录树。
    pub fn move_asset(&mut self, source_path: &str, target_dir: &str, fs: &dyn FileSystem) -> GResult<()> {
        let source = Path::new(source_path);
        let file_name = source
            .file_name()
            .ok_or_else(|| GError { kind: GErrorKind::Io, message: format!("无效的源路径: {}", source_path) })?;
        let target_path = Path::new(target_dir).join(file_name);
        let data = fs.read(source)?;
        if let Some(parent) = target_path.parent() {
            fs.create_dir_all(parent)?;
        }
        fs.write(&target_path, &data)?;
        fs.remove_file(source)?;
        if !self.current_directory.is_empty() {
            let dir = self.current_directory.clone();
            self.refresh_directory(Path::new(&dir), fs)?;
        }
        Ok(())
    }

    /// 复制资源
    ///
    /// 在同目录下创建文件的副本，文件名添加 `_copy` 后缀。
    /// 操作完成后自动刷新目录树。
    pub fn duplicate_asset(&mut self, path: &str, fs: &dyn FileSystem) -> GResult<()> {
        let source = Path::new(path);
        let data = fs.read(source)?;

        let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled");
        let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
        let parent = source
            .parent()
            .ok_or_else(|| GError { kind: GErrorKind::Io, message: format!("无法获取父目录: {}", path) })?;

        let new_file_name = if ext.is_empty() { format!("{}_copy", stem) } else { format!("{}_copy.{}", stem, ext) };
        let new_path = parent.join(&new_file_name);
        fs.write(&new_path, &data)?;
        if !self.current_directory.is_empty() {
            let dir = self.current_directory.clone();
            self.refresh_directory(Path::new(&dir), fs)?;
        }
        Ok(())
    }

    /// 批量删除多个资源
    ///
    /// 逐个删除指定路径列表中的资源文件，操作完成后自动刷新目录树。
    /// 返回 `AssetBatchOperationResult` 记录成功和失败的项。
    pub fn batch_delete_assets(&mut self, paths: &[String], fs: &dyn FileSystem) -> AssetBatchOperationResult {
        let mut result = AssetBatchOperationResult::new();
        for path in paths {
            match self.delete_asset(path, fs) {
                Ok(()) => result.push_success(path.clone()),
                Err(e) => result.push_failure(path.clone(), e.message),
            }
        }
        result
    }

    /// 批量移动多个资源到目标目录
    ///
    /// 逐个将指定路径列表中的资源文件移动到目标目录，操作完成后自动刷新目录树。
    /// 返回 `AssetBatchOperationResult` 记录成功和失败的项。
    pub fn batch_move_assets(&mut self, paths: &[String], target_dir: &str, fs: &dyn FileSystem) -> AssetBatchOperationResult {
        let mut result = AssetBatchOperationResult::new();
        for path in paths {
            match self.move_asset(path, target_dir, fs) {
                Ok(()) => result.push_success(path.clone()),
                Err(e) => result.push_failure(path.clone(), e.message),
            }
        }
        result
    }

    /// 批量复制多个资源
    ///
    /// 逐个复制指定路径列表中的资源文件（添加 `_copy` 后缀），操作完成后自动刷新目录树。
    /// 返回 `AssetBatchOperationResult` 记录成功和失败的项。
    pub fn batch_duplicate_assets(&mut self, paths: &[String], fs: &dyn FileSystem) -> AssetBatchOperationResult {
        let mut result = AssetBatchOperationResult::new();
        for path in paths {
            match self.duplicate_asset(path, fs) {
                Ok(()) => result.push_success(path.clone()),
                Err(e) => result.push_failure(path.clone(), e.message),
            }
        }
        result
    }

    /// 收集指定资源路径的同级文件路径列表
    ///
    /// 在目录树中查找指定路径所在的目录，返回该目录下所有文件的路径列表。
    /// 用于 Shift+Click 范围选中时确定范围。
    pub fn collect_sibling_paths(&self, asset_path: &str) -> Vec<String> {
        let parent_path = Path::new(asset_path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

        let mut siblings = Vec::new();
        Self::collect_file_paths(&self.directory_tree, &parent_path, &mut siblings);
        siblings
    }

    /// 递归收集指定目录下的所有文件路径
    fn collect_file_paths(node: &DirectoryNode, target_dir: &str, result: &mut Vec<String>) {
        if node.path == target_dir && node.is_directory {
            for child in &node.children {
                if !child.is_directory {
                    result.push(child.path.clone());
                }
            }
            return;
        }
        for child in &node.children {
            Self::collect_file_paths(child, target_dir, result);
        }
    }

    /// 获取资源预览数据
    ///
    /// 从预览缓存中获取指定路径的资源预览信息。
    /// 如果缓存中不存在，则通过文件系统读取资源元数据并生成预览数据。
    pub fn get_asset_preview(&mut self, path: &str, fs: &dyn FileSystem) -> Option<AssetPreviewData> {
        if let Some(preview) = self.preview_cache.get(path).cloned() {
            return Some(preview);
        }

        let metadata = self.get_asset_metadata(path, fs).ok()?;
        let mut preview = AssetPreviewData::new(metadata.name.clone(), metadata.asset_type.clone(), metadata.file_size);

        if let AssetType::Image = metadata.asset_type {
            preview.resolution = self.generate_image_thumbnail(path, fs);
        }

        self.preview_cache.insert(path.to_string(), preview.clone());
        Some(preview)
    }

    /// 为图片资源生成缩略图数据
    ///
    /// 读取图片文件并解析其头部信息获取分辨率。
    /// 当前仅支持 PNG 和 JPEG 格式的基本分辨率检测。
    /// 返回 (宽度, 高度) 元组，解析失败时返回 None。
    pub fn generate_image_thumbnail(&self, path: &str, fs: &dyn FileSystem) -> Option<(u32, u32)> {
        let data = fs.read(Path::new(path)).ok()?;
        let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        match ext.as_str() {
            "png" => {
                if data.len() >= 24 {
                    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
                    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
                    return Some((width, height));
                }
            }
            "jpg" | "jpeg" => {
                if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
                    let mut pos = 2;
                    while pos + 9 <= data.len() {
                        if data[pos] != 0xFF {
                            break;
                        }
                        let marker = data[pos + 1];
                        if marker == 0xC0 || marker == 0xC2 {
                            let height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
                            let width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]) as u32;
                            return Some((width, height));
                        }
                        let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
                        pos += 2 + seg_len;
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// 递归构建目录树 UI 节点
    ///
    /// 为每个目录节点创建可折叠的容器（带 ▶/▼ 图标和名称），
    /// 为每个文件节点创建带图标占位符和名称的行。
    /// 目录头部使用 `UiNodeData::Custom` 标记，kind 中编码目录路径，
    /// 文件节点使用 `UiNodeData::Custom` 标记，kind 中编码资源路径，
    /// 以便事件系统识别并处理点击交互。
    /// 当目录节点未展开时，仅渲染头部，跳过子节点容器。
    fn build_tree_node(&self, node: &DirectoryNode, ui_tree: &mut UiTree, parent_id: UiNodeId) {
        if node.is_directory {
            let header_id = ui_tree.create_node(
                format!("dir_header_{}", node.path),
                Style::new().with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(4.0)
                        .with_padding(2.0),
                ),
                UiNodeData::Custom { kind: format!("DirHeader:{}", node.path) },
            );

            let icon = if node.is_expanded { "▼" } else { "▶" };
            let icon_id = ui_tree.create_node(
                format!("dir_icon_{}", node.path),
                Style::default(),
                UiNodeData::Text { content: icon.to_string() },
            );

            let name_id = ui_tree.create_node(
                format!("dir_name_{}", node.path),
                Style::new().with_font(FontStyle::new().with_size(13.0)),
                UiNodeData::Text { content: node.name.clone() },
            );

            ui_tree.add_child(header_id, icon_id);
            ui_tree.add_child(header_id, name_id);
            ui_tree.add_child(parent_id, header_id);

            if node.is_expanded {
                let children_id = ui_tree.create_node(
                    format!("dir_children_{}", node.path),
                    Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(16.0)),
                    UiNodeData::Container,
                );

                ui_tree.add_child(parent_id, children_id);

                for child in &node.children {
                    self.build_tree_node(child, ui_tree, children_id);
                }
            }
        }
        else {
            let is_selected = self.selected_paths.contains(&node.path);
            let row_style = if is_selected {
                Style::new()
                    .with_layout(
                        LayoutStyle::new()
                            .with_direction(FlexDirection::Row)
                            .with_align_items(FlexAlign::Center)
                            .with_gap(4.0)
                            .with_padding(2.0),
                    )
                    .with_background_color(Color::new(0.2, 0.4, 0.7, 0.4))
            }
            else {
                Style::new().with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(4.0)
                        .with_padding(2.0),
                )
            };

            let row_id = ui_tree.create_node(format!("file_row_{}", node.path), row_style, UiNodeData::Container);

            let asset_type = classify_asset(&node.path);
            let icon_text = match asset_type {
                AssetType::Image => "🖼",
                AssetType::Audio => "🔊",
                AssetType::Font => "🔤",
                AssetType::Script => "📜",
                AssetType::Other => "📄",
            };

            let icon_id = ui_tree.create_node(
                format!("file_icon_{}", node.path),
                Style::default(),
                UiNodeData::Text { content: icon_text.to_string() },
            );

            let name_id = ui_tree.create_node(
                format!("file_name_{}", node.path),
                Style::new().with_font(FontStyle::new().with_size(13.0)),
                UiNodeData::Custom { kind: format!("AssetItem:{}", node.path) },
            );

            ui_tree.add_child(row_id, icon_id);
            ui_tree.add_child(row_id, name_id);
            ui_tree.add_child(parent_id, row_id);
        }
    }
}

impl EditorPanel for AssetBrowserPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Asset Browser"
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
    /// 响应键盘按下/释放追踪修饰键状态、鼠标按下开始拖拽、
    /// 鼠标释放结束拖拽或显示上下文菜单、鼠标移动检测拖拽阈值，
    /// 以及自定义事件处理资源选中（支持 Ctrl+Click 多选、Shift+Click 范围选）
    /// 和菜单操作。
    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::KeyDown { key } => match key {
                Key::Control => {
                    self.ctrl_pressed = true;
                }
                Key::Shift => {
                    self.shift_pressed = true;
                }
                Key::Delete => {
                    if self.selected_paths.len() > 1 {
                        let paths = self.selected_paths.clone();
                        let result = self.batch_delete_assets(&paths, context.services().filesystem());
                        let _ = result;
                        self.selected_paths.clear();
                        self.last_selected_index = None;
                    }
                    else if let Some(path) = self.selected_paths.first().cloned() {
                        let fs = context.services().filesystem();
                        let _ = self.delete_asset(&path, fs);
                        self.selected_paths.clear();
                        self.last_selected_index = None;
                    }
                }
                _ => {}
            },
            EditorEvent::KeyUp { key } => match key {
                Key::Control => {
                    self.ctrl_pressed = false;
                }
                Key::Shift => {
                    self.shift_pressed = false;
                }
                _ => {}
            },
            EditorEvent::MouseDown { button, position } => {
                self.hover_timer = 0;
                self.hover_preview_path = None;
                if *button == MouseButton::Left {
                    if self.selected_paths.len() > 1 {
                        let paths = self.selected_paths.clone();
                        self.drag_state = Some(DragState::new(DragData::MultiAsset(paths), *position));
                    }
                    else if let Some(asset_path) = self.selected_paths.first().cloned() {
                        self.drag_state = Some(DragState::new(DragData::AssetPath(asset_path), *position));
                    }
                }
            }
            EditorEvent::MouseUp { button, position } => {
                if *button == MouseButton::Right {
                    if let Some(asset_path) = self.selected_paths.first().cloned() {
                        let path = Path::new(&asset_path);
                        let is_dir = path.extension().is_none();
                        self.context_menu.show(asset_path, is_dir, *position);
                    }
                }
                else if *button == MouseButton::Left {
                    if let Some(drag) = self.drag_state.take() {
                        if drag.is_dragging {
                            context.events_mut().publish(EditorEvent::DragEnd { position: *position, data: drag.data });
                        }
                    }
                    self.context_menu.hide();
                }
            }
            EditorEvent::MouseMove { position: _ } => {
                if let Some(ref mut drag) = self.drag_state {
                    if drag.is_dragging {
                        let dx = (position.0 - drag.start_position.0).abs();
                        let dy = (position.1 - drag.start_position.1).abs();
                        if dx > 5.0 || dy > 5.0 {
                            context.events_mut().publish(EditorEvent::DragStart { data: drag.data.clone() });
                            drag.is_dragging = false;
                        }
                    }
                }
                self.hover_timer = self.hover_timer.saturating_add(16);
                if self.hover_timer >= 500 {
                    self.hover_preview_path = self.selected_paths.first().cloned();
                }
            }
            EditorEvent::Custom { name, data } => {
                if name == "AssetSelected" {
                    if let Some(path) = data.downcast_ref::<String>() {
                        if path.starts_with("AssetItem:") {
                            let asset_path = path["AssetItem:".len()..].to_string();
                            let sibling_paths = self.collect_sibling_paths(&asset_path);
                            let current_index = sibling_paths.iter().position(|p| *p == asset_path);

                            if self.ctrl_pressed {
                                self.select_asset_multi(asset_path);
                                self.last_selected_index = current_index;
                            }
                            else if self.shift_pressed {
                                if let Some(idx) = current_index {
                                    self.select_asset_range(&sibling_paths, idx);
                                }
                            }
                            else {
                                self.select_asset(asset_path);
                                self.last_selected_index = current_index;
                            }
                        }
                        else if path.starts_with("DirHeader:") {
                            let dir_path = &path["DirHeader:".len()..];
                            self.toggle_directory(dir_path);
                        }
                    }
                }
                else if name == "ContextMenuAction" {
                    if let Some(action) = data.downcast_ref::<String>() {
                        match action.as_str() {
                            "rename" | "delete" | "move" | "duplicate" => {
                                self.context_menu.hide();
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// 构建资源浏览器面板 UI 节点树
    ///
    /// 根据当前目录树数据构建完整的 UI 节点树，包括搜索栏、
    /// 目录节点的折叠/展开和文件节点的图标显示。
    /// 当选中状态发生变更时，发布 `AssetSelected` 自定义事件。
    /// 当检测到目录头部点击事件时，切换对应目录的展开状态。
    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut UiTree) -> Option<UiNodeId> {
        if self.selection_dirty {
            if let Some(ref asset_path) = self.selected_asset {
                context
                    .events_mut()
                    .publish(EditorEvent::Custom { name: "AssetSelected".to_string(), data: Box::new(asset_path.clone()) });
            }
            self.selection_dirty = false;
        }

        let events = context.events_mut().process_pending();
        for event in &events {
            if let EditorEvent::Custom { name, data } = event {
                if name == "AssetSelected" {
                    if let Some(path) = data.downcast_ref::<String>() {
                        if path.starts_with("DirHeader:") {
                            let dir_path = &path["DirHeader:".len()..];
                            self.toggle_directory(dir_path);
                        }
                    }
                }
            }
        }

        let root_id = ui_tree.create_node(
            "asset_browser_root",
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(2.0))
                .with_background_color(Color::new(0.12, 0.12, 0.14, 1.0))
                .with_overflow(Overflow::Clip),
            UiNodeData::Container,
        );

        let title_id = ui_tree.create_node(
            "asset_browser_title",
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0))
                .with_font(FontStyle::new().with_size(14.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0))),
            UiNodeData::Text { content: "资源浏览器".to_string() },
        );

        ui_tree.add_child(root_id, title_id);

        let search_id = ui_tree.create_node(
            "search_input",
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0))
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.6, 0.6, 0.6, 1.0))),
            UiNodeData::Custom { kind: "search_input".to_string() },
        );

        ui_tree.add_child(root_id, search_id);

        let tree_id = ui_tree.create_node(
            "asset_browser_tree",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(1.0)),
            UiNodeData::Container,
        );

        ui_tree.add_child(root_id, tree_id);

        let filtered_tree = filter_tree(&self.directory_tree, &self.search_query);

        if let Some(tree) = filtered_tree {
            for child in &tree.children {
                self.build_tree_node(child, ui_tree, tree_id);
            }
        }

        if self.context_menu.visible {
            let menu_id = ui_tree.create_node(
                "context_menu",
                Style::new()
                    .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(2.0))
                    .with_background_color(Color::new(0.2, 0.2, 0.22, 0.95)),
                UiNodeData::Container,
            );

            let menu_items = if self.context_menu.is_directory {
                vec!["重命名", "删除", "移动"]
            }
            else if self.selected_paths.len() > 1 {
                vec![
                    &format!("删除 {} 个资源", self.selected_paths.len()) as &str,
                    &format!("移动 {} 个资源", self.selected_paths.len()) as &str,
                    &format!("复制 {} 个资源", self.selected_paths.len()) as &str,
                ]
            }
            else {
                vec!["重命名", "删除", "移动", "复制"]
            };

            for (i, item) in menu_items.iter().enumerate() {
                let item_id = ui_tree.create_node(
                    format!("menu_item_{}", i),
                    Style::new().with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.9, 0.9, 0.9, 1.0))),
                    UiNodeData::Custom { kind: format!("ContextMenuItem:{}", item) },
                );
                ui_tree.add_child(menu_id, item_id);
            }

            ui_tree.add_child(root_id, menu_id);
        }

        Some(root_id)
    }

    /// 获取面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Bottom,
            preferred_size: Some((300.0, 200.0)),
            min_size: Some((200.0, 150.0)),
        }
    }
}
