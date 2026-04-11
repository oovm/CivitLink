//! 场景视图渲染集成测试
//!
//! 验证 ViewportState 坐标转换、BaseSceneView 实体选中、
//! 缩放操作和渲染命令收集功能。

use gg_ecs::Entity;
use gg_editor_scene::{BaseSceneView, RectRenderer, Transform2D, ViewportState};
use gg_render::Color;
use gg_world::GameWorld;

/// 测试 ViewportState 的世界坐标与屏幕坐标转换
///
/// 验证默认状态下 world_to_screen 和 screen_to_world 互为逆运算，
/// 以及带偏移和缩放的转换正确性。
#[test]
fn test_viewport_state_coordinates() {
    let viewport = ViewportState::new();

    let (sx, sy) = viewport.world_to_screen((100.0, 200.0));
    assert!((sx - 100.0).abs() < f32::EPSILON);
    assert!((sy - 200.0).abs() < f32::EPSILON);

    let (wx, wy) = viewport.screen_to_world((100.0, 200.0));
    assert!((wx - 100.0).abs() < f32::EPSILON);
    assert!((wy - 200.0).abs() < f32::EPSILON);

    let mut viewport_zoomed = ViewportState::new();
    viewport_zoomed.zoom = 2.0;
    let (sx, sy) = viewport_zoomed.world_to_screen((50.0, 100.0));
    assert!((sx - 100.0).abs() < f32::EPSILON);
    assert!((sy - 200.0).abs() < f32::EPSILON);

    let (wx, wy) = viewport_zoomed.screen_to_world((100.0, 200.0));
    assert!((wx - 50.0).abs() < f32::EPSILON);
    assert!((wy - 100.0).abs() < f32::EPSILON);

    let mut viewport_offset = ViewportState::new();
    viewport_offset.offset = (10.0, 20.0);
    let (sx, sy) = viewport_offset.world_to_screen((10.0, 20.0));
    assert!((sx - 0.0).abs() < f32::EPSILON);
    assert!((sy - 0.0).abs() < f32::EPSILON);

    let (wx, wy) = viewport_offset.screen_to_world((0.0, 0.0));
    assert!((wx - 10.0).abs() < f32::EPSILON);
    assert!((wy - 20.0).abs() < f32::EPSILON);
}

/// 测试 BaseSceneView 的实体选中功能
///
/// 验证选中实体后 selected_entities 包含该实体，
/// 重复选中不会重复添加。
#[test]
fn test_base_scene_view_select_entity() {
    let mut view = BaseSceneView::new();
    let entity = Entity::new(0, 0);

    assert!(view.selected_entities().is_empty());

    view.select_entity(entity);
    assert_eq!(view.selected_entities().len(), 1);
    assert_eq!(view.selected_entities()[0], entity);

    view.select_entity(entity);
    assert_eq!(view.selected_entities().len(), 1);

    let entity2 = Entity::new(1, 0);
    view.select_entity(entity2);
    assert_eq!(view.selected_entities().len(), 2);
}

/// 测试 BaseSceneView 的缩放方法
///
/// 验证 zoom_in 增大缩放因子、zoom_out 减小缩放因子、
/// reset_viewport 重置为默认值。
#[test]
fn test_base_scene_view_zoom_methods() {
    let mut view = BaseSceneView::new();
    let initial_zoom = view.viewport().zoom;
    assert!((initial_zoom - 1.0).abs() < f32::EPSILON);

    view.zoom_in();
    assert!(view.viewport().zoom > initial_zoom);

    let after_zoom_in = view.viewport().zoom;
    view.zoom_out();
    assert!(view.viewport().zoom < after_zoom_in);

    view.zoom_in();
    view.zoom_in();
    view.reset_viewport();
    assert!((view.viewport().zoom - 1.0).abs() < f32::EPSILON);
    assert_eq!(view.viewport().offset, (0.0, 0.0));
}

/// 测试 BaseSceneView 的渲染命令收集
///
/// 验证包含 Transform2D+RectRenderer 实体的 GameWorld
/// 在 collect_render_commands 后生成绘制命令。
#[test]
fn test_base_scene_view_render_commands() {
    let mut world = GameWorld::new("test_scene".to_string());

    let entity = world.spawn().id();
    world
        .add_component(
            entity,
            Transform2D {
                x: 100.0,
                y: 200.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            },
        )
        .unwrap();
    world
        .add_component(
            entity,
            RectRenderer {
                width: 50.0,
                height: 30.0,
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();

    let entity2 = world.spawn().id();
    world
        .add_component(
            entity2,
            Transform2D {
                x: 300.0,
                y: 400.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            },
        )
        .unwrap();
    world
        .add_component(
            entity2,
            RectRenderer {
                width: 80.0,
                height: 60.0,
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        )
        .unwrap();

    let mut view = BaseSceneView::new();
    view.collect_render_commands(&world);

    let commands = view.drain_render_commands();
    assert!(!commands.is_empty());
}
