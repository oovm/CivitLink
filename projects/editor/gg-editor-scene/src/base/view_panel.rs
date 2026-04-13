//! 基础场景视图 EditorPanel trait 实现

use std::rc::Rc;

use gg_editor_shell::{DragData, EditorContext, EditorEvent, EditorPanel, Key, PanelLayoutHint, PanelPosition};
use gg_render::Color;
use gg_ui::{FlexDirection, FontStyle, LayoutStyle, Style, UiNodeData};

use crate::components::Transform2D;

use super::{
    commands::{CreateEntityCommand, DeleteEntityCommand, TransformCommand},
    context_menu::SceneContextMenu,
    types::{CLIPBOARD_OFFSET, TransformGizmo, TransformKind, TransformValue, entity_to_u64, u64_to_entity},
    view::BaseSceneView,
};
use crate::scene_view::SceneView;

impl EditorPanel for BaseSceneView {
    fn name(&self) -> &str {
        "Scene View"
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn on_register(&mut self, context: &mut EditorContext) {
        let incoming_selected = Rc::clone(&self.incoming_selected);
        context.events_mut().subscribe(Box::new(move |event| {
            if let EditorEvent::EntitySelected { entity } = event {
                incoming_selected.borrow_mut().push(*entity);
            }
        }));

        let incoming_deselected = Rc::clone(&self.incoming_deselected);
        context.events_mut().subscribe(Box::new(move |event| {
            if matches!(event, EditorEvent::EntityDeselected) {
                *incoming_deselected.borrow_mut() = true;
            }
        }));
    }

    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::KeyDown { key } => match key {
                Key::W => {
                    if !self.ctrl_pressed && !self.alt_pressed {
                        self.set_transform_mode(TransformGizmo::Translate);
                    }
                }
                Key::E => {
                    if !self.ctrl_pressed && !self.alt_pressed {
                        self.set_transform_mode(TransformGizmo::Rotate);
                    }
                }
                Key::R => {
                    if !self.ctrl_pressed && !self.alt_pressed {
                        self.set_transform_mode(TransformGizmo::Scale);
                    }
                }
                Key::F => self.frame_selection(),
                Key::Escape => {
                    if self.is_gizmo_dragging {
                        self.is_gizmo_dragging = false;
                        self.gizmo_state.active_axis = None;
                    }
                    if self.selection_box.is_active {
                        self.selection_box.is_active = false;
                    }
                    self.context_menu.hide();
                }
                Key::Z => {
                    if self.ctrl_pressed {
                        let _ = context.undo_command();
                    }
                }
                Key::Y => {
                    if self.ctrl_pressed {
                        let _ = context.redo_command();
                    }
                }
                Key::Delete => {
                    if !self.selected_entities.is_empty() {
                        let entities_to_delete: Vec<_> = self.selected_entities.clone();
                        for entity in entities_to_delete {
                            let entity_id = entity_to_u64(entity);
                            if let Some(snapshot) = Self::take_entity_snapshot(context.world(), entity) {
                                let command = Box::new(DeleteEntityCommand {
                                    entity_id,
                                    snapshot,
                                    description: format!("删除实体 {}", entity_id),
                                });
                                context.execute_command(command);
                            }
                        }
                        self.selected_entities.clear();
                        context.events_mut().publish(EditorEvent::EntityDeselected);
                    }
                }
                Key::C => {
                    if self.ctrl_pressed && !self.selected_entities.is_empty() {
                        self.clipboard.clear();
                        for &entity in &self.selected_entities {
                            if let Some(clip) = Self::take_clipboard_entity(context.world(), entity) {
                                self.clipboard.push(clip);
                            }
                        }
                    }
                }
                Key::V => {
                    if self.ctrl_pressed && !self.clipboard.is_empty() {
                        for clip in &self.clipboard {
                            let position = (clip.transform.x + CLIPBOARD_OFFSET, clip.transform.y + CLIPBOARD_OFFSET);
                            let command = Box::new(CreateEntityCommand {
                                entity_id: None,
                                position,
                                entity_kind: clip.kind,
                                description: "粘贴实体".to_string(),
                            });
                            context.execute_command(command);
                        }
                    }
                }
                Key::Control => {
                    self.ctrl_pressed = true;
                }
                Key::Shift => {
                    self.shift_pressed = true;
                }
                Key::Alt => {
                    self.alt_pressed = true;
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
                Key::Alt => {
                    self.alt_pressed = false;
                }
                _ => {}
            },
            EditorEvent::MouseDown { button, position } => match button {
                gg_editor_shell::MouseButton::Left => {
                    self.context_menu.hide();
                    if self.handle_gizmo_interaction(*position, context) {
                    }
                    else {
                        let world_pos = self.viewport.screen_to_world(*position);
                        let world = context.world();
                        let hit = Self::hit_test_world(world, world_pos);

                        if let Some((entity, wx, wy)) = hit {
                            self.select_entity(entity);
                            self.is_dragging = true;
                            self.drag_start_screen = *position;
                            self.drag_entity_start_pos = (wx, wy);
                            self.dragged_entity = Some(entity);
                            context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
                        }
                        else {
                            self.deselect_all();
                            self.is_dragging = false;
                            self.dragged_entity = None;
                            self.selection_box.start_pos = *position;
                            self.selection_box.current_pos = *position;
                            self.selection_box.is_active = true;
                            context.events_mut().publish(EditorEvent::EntityDeselected);
                        }
                    }
                }
                gg_editor_shell::MouseButton::Middle => {
                    self.handle_middle_button_down(*position);
                }
                gg_editor_shell::MouseButton::Right => {
                    let world_pos = self.viewport.screen_to_world(*position);
                    self.context_menu.show(*position, world_pos);
                }
            },
            EditorEvent::MouseUp { button, .. } => match button {
                gg_editor_shell::MouseButton::Left => {
                    if self.is_gizmo_dragging {
                        if let Some(entity) = self.selected_entities.first() {
                            let entity_id = entity_to_u64(*entity);
                            if let Some(old_value) = self.gizmo_drag_old_value.take() {
                                let world = context.world();
                                if let Some(transform) = world.get_component::<Transform2D>(*entity) {
                                    let new_value = match self.transform_gizmo {
                                        TransformGizmo::Translate => TransformValue::Position((transform.x, transform.y)),
                                        TransformGizmo::Rotate => TransformValue::Rotation(transform.rotation),
                                        TransformGizmo::Scale => TransformValue::Scale((transform.scale_x, transform.scale_y)),
                                    };
                                    let transform_kind = match self.transform_gizmo {
                                        TransformGizmo::Translate => TransformKind::Translate,
                                        TransformGizmo::Rotate => TransformKind::Rotate,
                                        TransformGizmo::Scale => TransformKind::Scale,
                                    };
                                    let desc = match transform_kind {
                                        TransformKind::Translate => "移动实体",
                                        TransformKind::Rotate => "旋转实体",
                                        TransformKind::Scale => "缩放实体",
                                    };
                                    let command = Box::new(TransformCommand {
                                        entity_id,
                                        transform_kind,
                                        old_value,
                                        new_value,
                                        description: format!("{} {}", desc, entity_id),
                                    });
                                    context.execute_command(command);
                                }
                            }
                        }
                    }
                    self.is_dragging = false;
                    self.dragged_entity = None;
                    self.is_gizmo_dragging = false;
                    self.gizmo_state.active_axis = None;
                    self.gizmo_drag_old_value = None;
                    if self.selection_box.is_active {
                        self.selection_box.is_active = false;
                    }
                }
                gg_editor_shell::MouseButton::Middle => {
                    self.handle_middle_button_up();
                }
                gg_editor_shell::MouseButton::Right => {}
            },
            EditorEvent::MouseMove { position } => {
                if self.is_gizmo_dragging {
                    if let Some(entity) = self.selected_entities.first() {
                        let current_world = self.viewport.screen_to_world(*position);
                        if let Some(start_world) = self.gizmo_state.drag_start_world {
                            if let Some(start_pos) = self.gizmo_state.drag_entity_start_pos {
                                let dx = current_world.0 - start_world.0;
                                let dy = current_world.1 - start_world.1;

                                let world = context.world_mut();
                                if let Some(transform) = world.get_component_mut::<Transform2D>(*entity) {
                                    match self.transform_gizmo {
                                        TransformGizmo::Translate => match self.gizmo_state.active_axis.as_deref() {
                                            Some("x") => transform.x = start_pos.0 + dx,
                                            Some("y") => transform.y = start_pos.1 + dy,
                                            Some("xy") => {
                                                transform.x = start_pos.0 + dx;
                                                transform.y = start_pos.1 + dy;
                                            }
                                            _ => {}
                                        },
                                        TransformGizmo::Rotate => {
                                            if let Some(se) = self.scene_entities.iter().find(|e| e.entity == *entity) {
                                                let (cx, cy) = self.viewport.world_to_screen((se.world_x, se.world_y));
                                                let start_angle = (self.gizmo_state.drag_start_world.unwrap().1 - cy)
                                                    .atan2(self.gizmo_state.drag_start_world.unwrap().0 - cx);
                                                let current_angle = (position.1 - cy).atan2(position.0 - cx);
                                                transform.rotation += current_angle - start_angle;
                                            }
                                        }
                                        TransformGizmo::Scale => {
                                            if let Some(start_scale) = self.gizmo_state.drag_entity_start_scale {
                                                match self.gizmo_state.active_axis.as_deref() {
                                                    Some("x") => {
                                                        let delta = 1.0 + dx * 0.01;
                                                        transform.scale_x = (start_scale.0 * delta).max(0.01);
                                                    }
                                                    Some("y") => {
                                                        let delta = 1.0 + dy * 0.01;
                                                        transform.scale_y = (start_scale.1 * delta).max(0.01);
                                                    }
                                                    Some("xy") => {
                                                        let delta = 1.0 + (dx + dy) * 0.005;
                                                        transform.scale_x = (start_scale.0 * delta).max(0.01);
                                                        transform.scale_y = (start_scale.1 * delta).max(0.01);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                else if self.is_dragging {
                    if let Some(entity) = self.dragged_entity {
                        let (start_wx, start_wy) = self.viewport.screen_to_world(self.drag_start_screen);
                        let (current_wx, current_wy) = self.viewport.screen_to_world(*position);
                        let dx = current_wx - start_wx;
                        let dy = current_wy - start_wy;

                        let new_x = self.drag_entity_start_pos.0 + dx;
                        let new_y = self.drag_entity_start_pos.1 + dy;

                        let world = context.world_mut();
                        if let Some(transform) = world.get_component_mut::<Transform2D>(entity) {
                            transform.x = new_x;
                            transform.y = new_y;
                        }
                    }
                }
                else if self.selection_box.is_active {
                    self.selection_box.current_pos = *position;
                }
                else if self.is_panning {
                    self.handle_mouse_move(*position);
                }
                self.last_mouse_pos = Some(*position);
            }
            EditorEvent::MouseWheel { delta, position } => {
                self.handle_scroll(delta.1, *position);
            }
            EditorEvent::DragEnd { position, data } => match data {
                DragData::AssetPath(path) => {
                    let world_pos = self.viewport.screen_to_world(*position);
                    let world = context.world_mut();
                    let entity = world
                        .spawn()
                        .insert(Transform2D { x: world_pos.0, y: world_pos.1, ..Transform2D::default() })
                        .insert(crate::components::SpriteRenderer {
                            texture_path: path.clone(),
                            ..crate::components::SpriteRenderer::default()
                        })
                        .id();
                    context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
                }
                DragData::MultiAsset(paths) => {
                    let world_pos = self.viewport.screen_to_world(*position);
                    for (i, path) in paths.iter().enumerate() {
                        let offset_x = (i as f32 % 5.0) * 60.0;
                        let offset_y = (i as f32 / 5.0).floor() * 60.0;
                        let world = context.world_mut();
                        let entity = world
                            .spawn()
                            .insert(Transform2D {
                                x: world_pos.0 + offset_x,
                                y: world_pos.1 + offset_y,
                                ..Transform2D::default()
                            })
                            .insert(crate::components::SpriteRenderer {
                                texture_path: path.clone(),
                                ..crate::components::SpriteRenderer::default()
                            })
                            .id();
                        context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
                    }
                }
                DragData::Entity(_) | DragData::Custom { .. } => {}
            },
            EditorEvent::Custom { name, .. } => match name.as_str() {
                "reset_view" => self.reset_viewport(),
                "zoom_in" => self.zoom_in(),
                "zoom_out" => self.zoom_out(),
                "toggle_grid" => self.toggle_grid(),
                _ => {}
            },
            _ => {}
        }
    }

    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut gg_ui::UiTree) -> Option<gg_ui::UiNodeId> {
        for entity in self.pending_select_events.drain(..) {
            context.events_mut().publish(EditorEvent::EntitySelected { entity });
        }
        if self.pending_deselect_event {
            context.events_mut().publish(EditorEvent::EntityDeselected);
            self.pending_deselect_event = false;
        }

        if *self.incoming_deselected.borrow() {
            self.selected_entities.clear();
            *self.incoming_deselected.borrow_mut() = false;
        }
        let mut incoming = self.incoming_selected.borrow_mut();
        for entity_id in incoming.drain(..) {
            let entity = u64_to_entity(entity_id);
            if !self.selected_entities.contains(&entity) {
                self.selected_entities.push(entity);
            }
        }
        drop(incoming);

        let viewport_style = Style::new()
            .with_background_color(Color::new(0.1, 0.1, 0.1, 1.0))
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column));
        let viewport_id = ui_tree.create_node(
            "scene_viewport",
            viewport_style,
            UiNodeData::Custom { kind: format!("scene_render_target:{}", self.render_target_name) },
        );

        let zoom_text = self.zoom_indicator_text();
        let zoom_indicator_id = ui_tree.create_node(
            "zoom_indicator",
            Style::new().with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.9, 0.9, 0.9, 1.0))),
            UiNodeData::Text { content: zoom_text },
        );

        let toolbar_id = ui_tree.create_node(
            "scene_toolbar",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(4.0)),
            UiNodeData::Container,
        );

        let toggle_grid_id = ui_tree.create_node(
            "btn_toggle_grid",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        let reset_view_id = ui_tree.create_node(
            "btn_reset_view",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        let zoom_in_id = ui_tree.create_node(
            "btn_zoom_in",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        let zoom_out_id = ui_tree.create_node(
            "btn_zoom_out",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        ui_tree.add_child(toolbar_id, toggle_grid_id);
        ui_tree.add_child(toolbar_id, reset_view_id);
        ui_tree.add_child(toolbar_id, zoom_in_id);
        ui_tree.add_child(toolbar_id, zoom_out_id);

        ui_tree.add_child(viewport_id, zoom_indicator_id);
        ui_tree.add_child(viewport_id, toolbar_id);

        if self.context_menu.visible {
            let menu_style = Style::new()
                .with_background_color(Color::new(0.18, 0.18, 0.18, 0.95))
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0));
            let context_menu_id =
                ui_tree.create_node("context_menu", menu_style, UiNodeData::Custom { kind: "context_menu".to_string() });

            let create_empty_id = ui_tree.create_node(
                "menu_create_empty",
                Style::new().with_background_color(Color::new(0.25, 0.25, 0.25, 1.0)),
                UiNodeData::Text { content: "创建空实体".to_string() },
            );

            let create_rect_id = ui_tree.create_node(
                "menu_create_rect",
                Style::new().with_background_color(Color::new(0.25, 0.25, 0.25, 1.0)),
                UiNodeData::Text { content: "创建矩形".to_string() },
            );

            let create_sprite_id = ui_tree.create_node(
                "menu_create_sprite",
                Style::new().with_background_color(Color::new(0.25, 0.25, 0.25, 1.0)),
                UiNodeData::Text { content: "创建精灵".to_string() },
            );

            ui_tree.add_child(context_menu_id, create_empty_id);
            ui_tree.add_child(context_menu_id, create_rect_id);
            ui_tree.add_child(context_menu_id, create_sprite_id);
            ui_tree.add_child(viewport_id, context_menu_id);
        }

        Some(viewport_id)
    }

    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Center,
            preferred_size: Some((800.0, 600.0)),
            min_size: Some((400.0, 300.0)),
        }
    }
}

impl SceneView for BaseSceneView {}
