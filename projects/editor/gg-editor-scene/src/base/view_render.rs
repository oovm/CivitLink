//! 基础场景视图渲染方法

use gg_ecs::Entity;
use gg_render::{Color, DrawCommand, Rect, RenderContext};

use crate::{
    components::{RectRenderer, SpriteRenderer, Transform2D},
    world::GameWorld,
};

use super::{
    clipboard::{ClipboardEntity, EntitySnapshot},
    types::{BASE_GRID_SPACING, SceneEntity, SceneEntityKind, TransformGizmo, TransformValue, entity_to_u64},
    view::BaseSceneView,
};

impl BaseSceneView {
    /// 渲染变换工具
    pub fn render_gizmo(&self, context: &mut RenderContext) {
        if self.selected_entities.is_empty() {
            return;
        }

        let entity = self.selected_entities[0];
        if let Some(se) = self.scene_entities.iter().find(|e| e.entity == entity) {
            let (screen_x, screen_y) = self.viewport.world_to_screen((se.world_x, se.world_y));
            let screen_w = se.width * self.viewport.zoom;
            let screen_h = se.height * self.viewport.zoom;
            let center_x = screen_x + screen_w / 2.0;
            let center_y = screen_y + screen_h / 2.0;

            let gizmo_size = 60.0;

            match self.transform_gizmo {
                TransformGizmo::Translate => {
                    let x_color = Color::new(1.0, 0.3, 0.3, 0.9);
                    let y_color = Color::new(0.3, 1.0, 0.3, 0.9);
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x + gizmo_size, center_y],
                        color: x_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x, center_y + gizmo_size],
                        color: y_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(center_x + gizmo_size - 4.0, center_y - 4.0, 8.0, 8.0),
                        color: x_color,
                        corner_radius: 0.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(center_x - 4.0, center_y + gizmo_size - 4.0, 8.0, 8.0),
                        color: y_color,
                        corner_radius: 0.0,
                    });
                }
                TransformGizmo::Rotate => {
                    let rotate_color = Color::new(0.3, 0.6, 1.0, 0.9);
                    let segments = 32;
                    for i in 0..segments {
                        let angle1 = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                        let angle2 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
                        let x1 = center_x + gizmo_size * angle1.cos();
                        let y1 = center_y + gizmo_size * angle1.sin();
                        let x2 = center_x + gizmo_size * angle2.cos();
                        let y2 = center_y + gizmo_size * angle2.sin();
                        context.draw(DrawCommand::Line { start: [x1, y1], end: [x2, y2], color: rotate_color, width: 2.0 });
                    }
                }
                TransformGizmo::Scale => {
                    let x_color = Color::new(1.0, 0.3, 0.3, 0.9);
                    let y_color = Color::new(0.3, 1.0, 0.3, 0.9);
                    let handle_size = 8.0;
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x + gizmo_size, center_y],
                        color: x_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x, center_y + gizmo_size],
                        color: y_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(
                            center_x + gizmo_size - handle_size / 2.0,
                            center_y - handle_size / 2.0,
                            handle_size,
                            handle_size,
                        ),
                        color: x_color,
                        corner_radius: 2.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(
                            center_x - handle_size / 2.0,
                            center_y + gizmo_size - handle_size / 2.0,
                            handle_size,
                            handle_size,
                        ),
                        color: y_color,
                        corner_radius: 2.0,
                    });
                }
            }

            let center_color = Color::new(1.0, 1.0, 1.0, 0.9);
            context.draw(DrawCommand::Rect {
                rect: Rect::new(center_x - 3.0, center_y - 3.0, 6.0, 6.0),
                color: center_color,
                corner_radius: 3.0,
            });
        }
    }

    /// 渲染框选矩形
    pub fn render_selection_box(&self, context: &mut RenderContext) {
        if !self.selection_box.is_active {
            return;
        }

        let x = self.selection_box.start_pos.0.min(self.selection_box.current_pos.0);
        let y = self.selection_box.start_pos.1.min(self.selection_box.current_pos.1);
        let w = (self.selection_box.current_pos.0 - self.selection_box.start_pos.0).abs();
        let h = (self.selection_box.current_pos.1 - self.selection_box.start_pos.1).abs();

        let fill_color = Color::new(0.2, 0.5, 1.0, 0.15);
        let border_color = Color::new(0.2, 0.5, 1.0, 0.6);

        context.draw(DrawCommand::Rect { rect: Rect::new(x, y, w, h), color: fill_color, corner_radius: 0.0 });
        context.draw(DrawCommand::Line { start: [x, y], end: [x + w, y], color: border_color, width: 1.0 });
        context.draw(DrawCommand::Line { start: [x + w, y], end: [x + w, y + h], color: border_color, width: 1.0 });
        context.draw(DrawCommand::Line { start: [x + w, y + h], end: [x, y + h], color: border_color, width: 1.0 });
        context.draw(DrawCommand::Line { start: [x, y + h], end: [x, y], color: border_color, width: 1.0 });
    }

    /// 处理变换工具交互
    pub(crate) fn handle_gizmo_interaction(
        &mut self,
        screen_pos: (f32, f32),
        context: &mut gg_editor_shell::EditorContext,
    ) -> bool {
        if self.selected_entities.is_empty() {
            return false;
        }

        let entity = self.selected_entities[0];
        let entity_id = entity_to_u64(entity);
        if let Some(se) = self.scene_entities.iter().find(|e| e.entity == entity) {
            let (screen_x, screen_y) = self.viewport.world_to_screen((se.world_x, se.world_y));
            let screen_w = se.width * self.viewport.zoom;
            let screen_h = se.height * self.viewport.zoom;
            let center_x = screen_x + screen_w / 2.0;
            let center_y = screen_y + screen_h / 2.0;

            let gizmo_size = 60.0;
            let hit_threshold = 10.0;

            match self.transform_gizmo {
                TransformGizmo::Translate | TransformGizmo::Scale => {
                    let x_axis_start = center_x;
                    let x_axis_end = center_x + gizmo_size;
                    let y_axis_start = center_y;
                    let y_axis_end = center_y + gizmo_size;

                    let on_x_axis = screen_pos.1 >= center_y - hit_threshold
                        && screen_pos.1 <= center_y + hit_threshold
                        && screen_pos.0 >= x_axis_start - hit_threshold
                        && screen_pos.0 <= x_axis_end + hit_threshold;

                    let on_y_axis = screen_pos.0 >= center_x - hit_threshold
                        && screen_pos.0 <= center_x + hit_threshold
                        && screen_pos.1 >= y_axis_start - hit_threshold
                        && screen_pos.1 <= y_axis_end + hit_threshold;

                    if on_x_axis || on_y_axis {
                        self.is_gizmo_dragging = true;
                        self.gizmo_state.active_axis = if on_x_axis && on_y_axis {
                            Some("xy".to_string())
                        }
                        else if on_x_axis {
                            Some("x".to_string())
                        }
                        else {
                            Some("y".to_string())
                        };
                        self.gizmo_state.drag_start_world = Some(self.viewport.screen_to_world(screen_pos));
                        self.gizmo_state.drag_entity_start_pos = Some((se.world_x, se.world_y));
                        self.gizmo_state.drag_entity_start_scale = Some((se.scale_x, se.scale_y));

                        let world = context.world();
                        if let Some(transform) = world.get_component::<Transform2D>(entity) {
                            self.gizmo_drag_old_value = match self.transform_gizmo {
                                TransformGizmo::Translate => Some(TransformValue::Position((transform.x, transform.y))),
                                TransformGizmo::Scale => Some(TransformValue::Scale((transform.scale_x, transform.scale_y))),
                                TransformGizmo::Rotate => None,
                            };
                        }

                        return true;
                    }
                }
                TransformGizmo::Rotate => {
                    let dx = screen_pos.0 - center_x;
                    let dy = screen_pos.1 - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist >= gizmo_size - hit_threshold && dist <= gizmo_size + hit_threshold {
                        self.is_gizmo_dragging = true;
                        self.gizmo_state.active_axis = Some("rotation".to_string());
                        self.gizmo_state.drag_start_world = Some(self.viewport.screen_to_world(screen_pos));
                        self.gizmo_state.drag_entity_start_rotation = Some(se.rotation);

                        let world = context.world();
                        if let Some(transform) = world.get_component::<Transform2D>(entity) {
                            self.gizmo_drag_old_value = Some(TransformValue::Rotation(transform.rotation));
                        }

                        return true;
                    }
                }
            }
        }
        false
    }

    /// 从 GameWorld 查询实体并构建场景实体列表
    pub(crate) fn build_scene_entities(world: &GameWorld) -> Vec<SceneEntity> {
        let mut entities = Vec::new();
        let query = world.query::<Transform2D>();
        for (entity, transform) in query {
            if let Some(sprite) = world.get_component::<SpriteRenderer>(entity) {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: sprite.width,
                    height: sprite.height,
                    rotation: transform.rotation,
                    scale_x: transform.scale_x,
                    scale_y: transform.scale_y,
                    color: sprite.color,
                    kind: SceneEntityKind::Sprite,
                });
            }
            else if let Some(rect) = world.get_component::<RectRenderer>(entity) {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: rect.width,
                    height: rect.height,
                    rotation: transform.rotation,
                    scale_x: transform.scale_x,
                    scale_y: transform.scale_y,
                    color: rect.color,
                    kind: SceneEntityKind::Rect,
                });
            }
            else {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: 50.0,
                    height: 50.0,
                    rotation: transform.rotation,
                    scale_x: transform.scale_x,
                    scale_y: transform.scale_y,
                    color: Color::new(0.5, 0.5, 0.5, 1.0),
                    kind: SceneEntityKind::Rect,
                });
            }
        }
        entities
    }

    /// 渲染场景实体
    pub fn render_scene(&mut self, context: &mut RenderContext, world: &GameWorld) {
        self.scene_entities = Self::build_scene_entities(world);
        let zoom = self.viewport.zoom;

        for entity in &self.scene_entities {
            let (screen_x, screen_y) = self.viewport.world_to_screen((entity.world_x, entity.world_y));
            let screen_w = entity.width * zoom;
            let screen_h = entity.height * zoom;

            let has_transform = entity.rotation != 0.0 || entity.scale_x != 1.0 || entity.scale_y != 1.0;

            if has_transform {
                let corners = Self::compute_transformed_corners(
                    screen_x,
                    screen_y,
                    screen_w,
                    screen_h,
                    entity.rotation,
                    entity.scale_x,
                    entity.scale_y,
                );

                context.draw(DrawCommand::Line {
                    start: [corners[0].0, corners[0].1],
                    end: [corners[1].0, corners[1].1],
                    color: entity.color,
                    width: 2.0,
                });
                context.draw(DrawCommand::Line {
                    start: [corners[1].0, corners[1].1],
                    end: [corners[2].0, corners[2].1],
                    color: entity.color,
                    width: 2.0,
                });
                context.draw(DrawCommand::Line {
                    start: [corners[2].0, corners[2].1],
                    end: [corners[3].0, corners[3].1],
                    color: entity.color,
                    width: 2.0,
                });
                context.draw(DrawCommand::Line {
                    start: [corners[3].0, corners[3].1],
                    end: [corners[0].0, corners[0].1],
                    color: entity.color,
                    width: 2.0,
                });
            }
            else {
                context.draw(DrawCommand::Rect {
                    rect: Rect::new(screen_x, screen_y, screen_w, screen_h),
                    color: entity.color,
                    corner_radius: 0.0,
                });
            }
        }

        self.render_grid(context);
        self.render_selection_overlay(context);
        self.render_gizmo(context);
        self.render_selection_box(context);
    }

    /// 收集场景绘制命令
    pub fn collect_render_commands(&mut self, world: &GameWorld) {
        let mut context = RenderContext::new(self.viewport.size.0 as u32, self.viewport.size.1 as u32);
        self.render_scene(&mut context, world);
        self.render_commands = context.commands().to_vec();
    }

    /// 取出待提交的场景绘制命令
    pub fn drain_render_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.render_commands)
    }

    /// 渲染背景网格
    pub(crate) fn render_grid(&self, context: &mut RenderContext) {
        if !self.grid_visible {
            return;
        }

        let zoom = self.viewport.zoom;
        let offset = self.viewport.offset;
        let (vp_width, vp_height) = self.viewport.size;

        let screen_spacing = BASE_GRID_SPACING * zoom;
        let world_step = if screen_spacing < 20.0 {
            BASE_GRID_SPACING * 2.0
        }
        else if screen_spacing > 200.0 {
            BASE_GRID_SPACING * 0.5
        }
        else {
            BASE_GRID_SPACING
        };

        let grid_color = Color::new(0.3, 0.3, 0.3, 0.5);

        let start_x = offset.0;
        let start_y = offset.1;
        let end_x = offset.0 + vp_width / zoom;
        let end_y = offset.1 + vp_height / zoom;

        let grid_start_x = (start_x / world_step).floor() * world_step;
        let grid_start_y = (start_y / world_step).floor() * world_step;

        let mut x = grid_start_x;
        while x <= end_x {
            let (screen_x, _) = self.viewport.world_to_screen((x, 0.0));
            context.draw(DrawCommand::Line {
                start: [screen_x, 0.0],
                end: [screen_x, vp_height],
                color: grid_color,
                width: 1.0,
            });
            x += world_step;
        }

        let mut y = grid_start_y;
        while y <= end_y {
            let (_, screen_y) = self.viewport.world_to_screen((0.0, y));
            context.draw(DrawCommand::Line {
                start: [0.0, screen_y],
                end: [vp_width, screen_y],
                color: grid_color,
                width: 1.0,
            });
            y += world_step;
        }
    }

    /// 渲染选中实体高亮叠加层
    pub fn render_selection_overlay(&self, context: &mut RenderContext) {
        let highlight_color = Color::new(0.2, 0.5, 1.0, 0.8);

        for &entity in &self.selected_entities {
            if let Some(scene_entity) = self.scene_entities.iter().find(|e| e.entity == entity) {
                let (screen_x, screen_y) = self.viewport.world_to_screen((scene_entity.world_x, scene_entity.world_y));
                let screen_w = scene_entity.width * self.viewport.zoom;
                let screen_h = scene_entity.height * self.viewport.zoom;

                let has_transform = scene_entity.rotation != 0.0 || scene_entity.scale_x != 1.0 || scene_entity.scale_y != 1.0;

                if has_transform {
                    let corners = Self::compute_transformed_corners(
                        screen_x - 2.0,
                        screen_y - 2.0,
                        screen_w + 4.0,
                        screen_h + 4.0,
                        scene_entity.rotation,
                        scene_entity.scale_x,
                        scene_entity.scale_y,
                    );

                    context.draw(DrawCommand::Line {
                        start: [corners[0].0, corners[0].1],
                        end: [corners[1].0, corners[1].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [corners[1].0, corners[1].1],
                        end: [corners[2].0, corners[2].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [corners[2].0, corners[2].1],
                        end: [corners[3].0, corners[3].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [corners[3].0, corners[3].1],
                        end: [corners[0].0, corners[0].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                }
                else {
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(screen_x - 2.0, screen_y - 2.0, screen_w + 4.0, screen_h + 4.0),
                        color: highlight_color,
                        corner_radius: 0.0,
                    });
                }
            }
        }
    }

    /// 构建缩放指示器文本
    pub(crate) fn zoom_indicator_text(&self) -> String {
        format!("{:.0}%", self.viewport.zoom * 100.0)
    }

    /// 从 GameWorld 中获取实体的快照数据
    pub(crate) fn take_entity_snapshot(world: &GameWorld, entity: Entity) -> Option<EntitySnapshot> {
        let transform = world.get_component::<Transform2D>(entity)?.clone();
        let rect_renderer = world.get_component::<RectRenderer>(entity).cloned();
        let sprite_renderer = world.get_component::<SpriteRenderer>(entity).cloned();
        Some(EntitySnapshot { transform, rect_renderer, sprite_renderer })
    }

    /// 从 GameWorld 中获取实体的剪贴板数据
    pub(crate) fn take_clipboard_entity(world: &GameWorld, entity: Entity) -> Option<ClipboardEntity> {
        let transform = world.get_component::<Transform2D>(entity)?.clone();
        let rect_renderer = world.get_component::<RectRenderer>(entity).cloned();
        let sprite_renderer = world.get_component::<SpriteRenderer>(entity).cloned();
        let kind = if sprite_renderer.is_some() { SceneEntityKind::Sprite } else { SceneEntityKind::Rect };
        Some(ClipboardEntity { transform, rect_renderer, sprite_renderer, kind })
    }
}
