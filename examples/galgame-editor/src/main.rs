//! GG Galgame 编辑器示例
//!
//! 直接启动编辑器 Shell，注册核心面板并进入 winit 事件循环。

use gg_core::GResult;
use gg_editor_shell::EditorShell;
use gg_editor_scene::BaseSceneView;
use gg_editor_inspector::InspectorPanel;
use gg_editor_asset_browser::AssetBrowserPanel;

fn main() -> GResult<()> {
    let mut shell = EditorShell::new();

    shell.register_panel(Box::new(BaseSceneView::new()));
    shell.register_panel(Box::new(InspectorPanel::new()));
    shell.register_panel(Box::new(AssetBrowserPanel::new()));

    shell.run_with_renderer()
}
