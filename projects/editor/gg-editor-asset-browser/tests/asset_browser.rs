//! 资源浏览器交互集成测试
//!
//! 验证 DirectoryNode 展开/折叠、filter_tree 过滤
//! 和 AssetBrowserPanel 文件导入功能。

use gg_editor_asset_browser::{AssetBrowserPanel, DirectoryNode, filter_tree};

/// 构建测试用目录树
///
/// 结构如下：
/// ```text
/// assets/
///   images/
///     hero.png
///     bg.jpg
///   audio/
///     bgm.ogg
///   config.toml
/// ```
fn build_test_tree() -> DirectoryNode {
    DirectoryNode {
        name: "assets".to_string(),
        path: "assets".to_string(),
        is_directory: true,
        is_expanded: false,
        children: vec![
            DirectoryNode {
                name: "images".to_string(),
                path: "assets/images".to_string(),
                is_directory: true,
                is_expanded: false,
                children: vec![
                    DirectoryNode {
                        name: "hero.png".to_string(),
                        path: "assets/images/hero.png".to_string(),
                        is_directory: false,
                        is_expanded: false,
                        children: vec![],
                    },
                    DirectoryNode {
                        name: "bg.jpg".to_string(),
                        path: "assets/images/bg.jpg".to_string(),
                        is_directory: false,
                        is_expanded: false,
                        children: vec![],
                    },
                ],
            },
            DirectoryNode {
                name: "audio".to_string(),
                path: "assets/audio".to_string(),
                is_directory: true,
                is_expanded: false,
                children: vec![DirectoryNode {
                    name: "bgm.ogg".to_string(),
                    path: "assets/audio/bgm.ogg".to_string(),
                    is_directory: false,
                    is_expanded: false,
                    children: vec![],
                }],
            },
            DirectoryNode {
                name: "config.toml".to_string(),
                path: "assets/config.toml".to_string(),
                is_directory: false,
                is_expanded: false,
                children: vec![],
            },
        ],
    }
}

/// 递归切换指定路径节点的展开状态
///
/// 与 AssetBrowserPanel::toggle_directory 内部使用的 toggle_node_expanded 逻辑一致。
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

/// 测试 DirectoryNode 的展开/折叠切换
///
/// 验证切换指定路径节点的 is_expanded 状态，
/// 以及不存在的路径返回 false。
#[test]
fn test_directory_node_toggle_expand() {
    let mut tree = build_test_tree();

    assert!(!tree.is_expanded);

    let toggled = toggle_node_expanded(&mut tree, "assets");
    assert!(toggled);
    assert!(tree.is_expanded);

    let toggled = toggle_node_expanded(&mut tree, "assets");
    assert!(toggled);
    assert!(!tree.is_expanded);

    let toggled = toggle_node_expanded(&mut tree, "assets/images");
    assert!(toggled);
    assert!(tree.children[0].is_expanded);

    let toggled = toggle_node_expanded(&mut tree, "nonexistent");
    assert!(!toggled);
}

/// 测试 filter_tree 空查询返回完整树
///
/// 验证空查询时返回 Some 且包含所有节点。
#[test]
fn test_filter_tree_empty_query() {
    let tree = build_test_tree();

    let result = filter_tree(&tree, "");
    assert!(result.is_some());

    let filtered = result.unwrap();
    assert_eq!(filtered.children.len(), 3);
    assert_eq!(filtered.children[0].children.len(), 2);
    assert_eq!(filtered.children[1].children.len(), 1);
}

/// 测试 filter_tree 匹配查询返回过滤后的树
///
/// 验证查询 "hero" 时仅返回包含匹配文件的子树。
#[test]
fn test_filter_tree_matching_query() {
    let tree = build_test_tree();

    let result = filter_tree(&tree, "hero");
    assert!(result.is_some());

    let filtered = result.unwrap();
    assert!(filtered.children.len() >= 1);

    let images_dir = filtered.children.iter().find(|c| c.name == "images").expect("images 目录应存在");
    assert!(images_dir.is_directory);
    assert_eq!(images_dir.children.len(), 1);
    assert_eq!(images_dir.children[0].name, "hero.png");
}

/// 测试 filter_tree 无匹配查询返回 None
///
/// 验证查询不存在的文件名时返回 None。
#[test]
fn test_filter_tree_no_match() {
    let tree = build_test_tree();

    let result = filter_tree(&tree, "nonexistent_file_xyz");
    assert!(result.is_none());
}

/// 测试 AssetBrowserPanel 的文件导入功能
///
/// 验证 import_file 后 pending_imports 包含对应条目，
/// drain_pending_imports 后列表为空。
#[test]
fn test_asset_browser_import_file() {
    let mut panel = AssetBrowserPanel::new();

    panel.import_file("external/hero.png", "assets/images/").unwrap();

    let imports = panel.drain_pending_imports();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source_path, "external/hero.png");
    assert_eq!(imports[0].target_path, "assets/images/hero.png");

    let imports_again = panel.drain_pending_imports();
    assert!(imports_again.is_empty());
}
