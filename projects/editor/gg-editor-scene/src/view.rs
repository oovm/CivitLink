//! 场景视图 trait 定义

use gg_core::GResult;
use gg_ecs::Entity;
use gg_editor_shell::{EditorContext, EditorPanel};

use crate::viewport::ViewportState;

/// 场景视图 trait
///
/// 扩展 `EditorPanel`，提供场景相关的回调方法，
/// 包括场景加载/卸载、实体选中/移动和叠加层渲染。
pub trait SceneView: EditorPanel {
    /// 场景加载完成时调用
    fn on_scene_load(&mut self, _context: &mut EditorContext) {}

    /// 场景卸载完成时调用
    fn on_scene_unload(&mut self, _context: &mut EditorContext) {}

    /// 实体被选中时调用
    fn on_entity_selected(&mut self, _entity: Entity, _context: &mut EditorContext) {}

    /// 实体移动时调用
    fn on_entity_moved(&mut self, _entity: Entity, _delta: (f32, f32), _context: &mut EditorContext) {}

    /// 渲染叠加层
    ///
    /// 在场景内容之上渲染叠加元素（如选中框、辅助线等）。
    /// 接收视口状态和选中实体列表，用于将世界坐标转换为屏幕坐标进行绘制。
    fn render_overlay(
        &mut self,
        _viewport: &ViewportState,
        _selected_entities: &[Entity],
        _context: &mut EditorContext,
    ) -> GResult<()> {
        Ok(())
    }
}
