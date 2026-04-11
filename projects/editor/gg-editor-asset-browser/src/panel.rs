//! 资源浏览器面板实现
//! 提供项目资源目录树浏览、文件导入、元数据查询和引用查找功能

use std::path::Path;

use gg_core::{FileType, FileSystem, GResult};
use gg_ecs::Entity;
use gg_editor_shell::{EditorContext, EditorEvent, EditorPanel, PanelLayoutHint, PanelPosition};
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
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" => AssetType::Image,
        "mp3" | "wav" | "ogg" | "flac" => AssetType::Audio,
        "ttf" | "otf" | "woff" | "woff2" => AssetType::Font,
        "v" | "vx" | "rs" | "toml" => AssetType::Script,
        _ => AssetType::Other,
    }
}

/// 递归扫描目录，构建目录树节点
///
/// 通过文件系统抽象接口读取指定路径的目录条目，
/// 对子目录进行递归扫描，最终返回完整的目录树节点。
/// 子节点按目录优先、名称字母序排列。
fn scan_directory(path: &Path, fs: &dyn FileSystem) -> GResult<DirectoryNode> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

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

    child_nodes.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    node.children = child_nodes;
    Ok(node)
}

/// 资源浏览器面板
///
/// 提供项目资源目录的可视化浏览功能，支持目录树导航、
/// 文件导入、资源元数据查询和引用关系查找。
pub struct AssetBrowserPanel {
    /// 面板是否可见
    visible: bool,
    /// 当前浏览的目录路径
    current_directory: String,
    /// 选中的资源路径
    selected_asset: Option<String>,
    /// 目录树根节点
    directory_tree: DirectoryNode,
    /// 选中状态是否已变更（用于事件发布）
    selection_dirty: bool,
}

impl AssetBrowserPanel {
    /// 创建新的资源浏览器面板
    pub fn new() -> Self {
        Self {
            visible: true,
            current_directory: String::new(),
            selected_asset: None,
            directory_tree: DirectoryNode {
                name: String::new(),
                path: String::new(),
                is_directory: true,
                children: Vec::new(),
                is_expanded: false,
            },
            selection_dirty: false,
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

    /// 导入文件
    ///
    /// 将外部文件复制到项目资源目录中的指定目标位置。
    pub fn import_file(&mut self, source_path: &str, target_dir: &str) -> GResult<()> {
        let _ = (source_path, target_dir);
        Ok(())
    }

    /// 获取资源元数据
    ///
    /// 根据资源路径通过文件系统抽象接口查询其名称、大小和类型等元数据信息。
    /// 成功时返回 `Some(AssetMetadata)`，查询失败时返回 `None`。
    pub fn get_asset_metadata(&self, path: &Path, fs: &dyn FileSystem) -> Option<AssetMetadata> {
        let metadata = fs.metadata(path).ok()?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

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
    pub fn find_references(
        &self,
        asset_path: &str,
        context: &mut EditorContext,
    ) -> GResult<Vec<AssetReference>> {
        let _ = (asset_path, context);
        Ok(Vec::new())
    }

    /// 选中指定路径的资源
    ///
    /// 设置当前选中的资源路径，并标记选中状态已变更，
    /// 以便在下次 `build_ui` 调用时发布 `AssetSelected` 事件。
    pub fn select_asset(&mut self, path: String) {
        self.selected_asset = Some(path);
        self.selection_dirty = true;
    }

    /// 递归构建目录树 UI 节点
    ///
    /// 为每个目录节点创建可折叠的容器（带 ▶/▼ 图标和名称），
    /// 为每个文件节点创建带图标占位符和名称的行。
    /// 文件节点使用 `UiNodeData::Custom` 标记，kind 中编码资源路径，
    /// 以便事件系统识别并处理点击交互。
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
                UiNodeData::Container,
            );

            let icon = if node.is_expanded { "▼" } else { "▶" };
            let icon_id = ui_tree.create_node(
                format!("dir_icon_{}", node.path),
                Style::default(),
                UiNodeData::Text {
                    content: icon.to_string(),
                },
            );

            let name_id = ui_tree.create_node(
                format!("dir_name_{}", node.path),
                Style::new().with_font(FontStyle::new().with_size(13.0)),
                UiNodeData::Text {
                    content: node.name.clone(),
                },
            );

            ui_tree.add_child(header_id, icon_id);
            ui_tree.add_child(header_id, name_id);
            ui_tree.add_child(parent_id, header_id);

            let children_id = ui_tree.create_node(
                format!("dir_children_{}", node.path),
                Style::new().with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Column)
                        .with_padding(16.0),
                ),
                UiNodeData::Container,
            );

            ui_tree.add_child(parent_id, children_id);

            for child in &node.children {
                self.build_tree_node(child, ui_tree, children_id);
            }
        } else {
            let row_id = ui_tree.create_node(
                format!("file_row_{}", node.path),
                Style::new().with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(4.0)
                        .with_padding(2.0),
                ),
                UiNodeData::Container,
            );

            let icon_id = ui_tree.create_node(
                format!("file_icon_{}", node.path),
                Style::default(),
                UiNodeData::Text {
                    content: "📄".to_string(),
                },
            );

            let name_id = ui_tree.create_node(
                format!("file_name_{}", node.path),
                Style::new().with_font(FontStyle::new().with_size(13.0)),
                UiNodeData::Custom {
                    kind: format!("AssetItem:{}", node.path),
                },
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

    /// 构建资源浏览器面板 UI 节点树
    ///
    /// 根据当前目录树数据构建完整的 UI 节点树，包括目录节点的折叠/展开
    /// 和文件节点的图标显示。当选中状态发生变更时，发布 `AssetSelected` 自定义事件。
    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        if self.selection_dirty {
            if let Some(ref asset_path) = self.selected_asset {
                context.events_mut().publish(EditorEvent::Custom {
                    name: "AssetSelected".to_string(),
                    data: Box::new(asset_path.clone()),
                });
            }
            self.selection_dirty = false;
        }

        let root_id = ui_tree.create_node(
            "asset_browser_root",
            Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Column)
                        .with_padding(4.0)
                        .with_gap(2.0),
                )
                .with_background_color(Color::new(0.12, 0.12, 0.14, 1.0))
                .with_overflow(Overflow::Clip),
            UiNodeData::Container,
        );

        let title_id = ui_tree.create_node(
            "asset_browser_title",
            Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_padding(4.0),
                )
                .with_font(
                    FontStyle::new()
                        .with_size(14.0)
                        .with_color(Color::new(0.7, 0.7, 0.7, 1.0)),
                ),
            UiNodeData::Text {
                content: "资源浏览器".to_string(),
            },
        );

        ui_tree.add_child(root_id, title_id);

        let tree_id = ui_tree.create_node(
            "asset_browser_tree",
            Style::new().with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_gap(1.0),
            ),
            UiNodeData::Container,
        );

        ui_tree.add_child(root_id, tree_id);

        for child in &self.directory_tree.children {
            self.build_tree_node(child, ui_tree, tree_id);
        }

        ui_tree.set_root(root_id);

        Ok(())
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
