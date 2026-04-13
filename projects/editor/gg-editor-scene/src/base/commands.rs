//! 场景编辑器命令定义

use gg_core::{GError, GErrorKind, GResult};
use gg_editor_shell::{Command, EditorContext};

use crate::components::{RectRenderer, SpriteRenderer, Transform2D};

use super::types::{SceneEntityKind, TransformKind, TransformValue, entity_to_u64, u64_to_entity};

/// 变换命令
///
/// 可撤销/重做的变换操作命令，记录实体的变换类型、旧值和新值。
/// 执行时将新值应用到实体的 `Transform2D` 组件，撤销时恢复旧值。
pub struct TransformCommand {
    /// 被变换的实体 ID
    pub entity_id: u64,
    /// 变换类型
    pub transform_kind: TransformKind,
    /// 变换前的值
    pub old_value: TransformValue,
    /// 变换后的值
    pub new_value: TransformValue,
    /// 命令描述
    pub description: String,
}

impl Command for TransformCommand {
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        let world = context.world_mut();
        if let Some(transform) = world.get_component_mut::<Transform2D>(entity) {
            match self.new_value {
                TransformValue::Position((x, y)) => {
                    transform.x = x;
                    transform.y = y;
                }
                TransformValue::Rotation(r) => {
                    transform.rotation = r;
                }
                TransformValue::Scale((sx, sy)) => {
                    transform.scale_x = sx;
                    transform.scale_y = sy;
                }
            }
            Ok(())
        }
        else {
            Err(GError {
                kind: GErrorKind::Ecs, message: format!("实体 {} 不存在或缺少 Transform2D 组件", self.entity_id)
            })
        }
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        let world = context.world_mut();
        if let Some(transform) = world.get_component_mut::<Transform2D>(entity) {
            match self.old_value {
                TransformValue::Position((x, y)) => {
                    transform.x = x;
                    transform.y = y;
                }
                TransformValue::Rotation(r) => {
                    transform.rotation = r;
                }
                TransformValue::Scale((sx, sy)) => {
                    transform.scale_x = sx;
                    transform.scale_y = sy;
                }
            }
            Ok(())
        }
        else {
            Err(GError {
                kind: GErrorKind::Ecs, message: format!("实体 {} 不存在或缺少 Transform2D 组件", self.entity_id)
            })
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 删除实体命令
///
/// 可撤销/重做的删除实体操作命令。
/// 执行时从世界中移除实体，撤销时重新生成实体并恢复所有组件。
pub struct DeleteEntityCommand {
    /// 被删除的实体 ID
    pub entity_id: u64,
    /// 实体组件快照，用于撤销时恢复
    pub snapshot: super::clipboard::EntitySnapshot,
    /// 命令描述
    pub description: String,
}

impl Command for DeleteEntityCommand {
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        context.world_mut().despawn(entity)
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        let world = context.world_mut();
        world.spawn_with_entity(entity);
        world.add_component(entity, self.snapshot.transform.clone())?;
        if let Some(ref rect) = self.snapshot.rect_renderer {
            world.add_component(entity, rect.clone())?;
        }
        if let Some(ref sprite) = self.snapshot.sprite_renderer {
            world.add_component(entity, sprite.clone())?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 创建实体命令
///
/// 可撤销/重做的创建实体操作命令。
/// 执行时在世界中生成新实体并添加 `Transform2D` 和对应渲染器组件，
/// 撤销时从世界中移除该实体。
pub struct CreateEntityCommand {
    /// 创建的实体 ID，执行后设置
    pub entity_id: Option<u64>,
    /// 实体的世界坐标位置
    pub position: (f32, f32),
    /// 实体类型
    pub entity_kind: SceneEntityKind,
    /// 命令描述
    pub description: String,
}

impl Command for CreateEntityCommand {
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let world = context.world_mut();
        let entity =
            world.spawn().insert(Transform2D { x: self.position.0, y: self.position.1, ..Transform2D::default() }).id();
        match self.entity_kind {
            SceneEntityKind::Rect => {
                world.add_component(entity, RectRenderer::default())?;
            }
            SceneEntityKind::Sprite => {
                world.add_component(entity, SpriteRenderer::default())?;
            }
        }
        self.entity_id = Some(entity_to_u64(entity));
        Ok(())
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        if let Some(id) = self.entity_id {
            let entity = u64_to_entity(id);
            context.world_mut().despawn(entity)
        }
        else {
            Err(GError { kind: GErrorKind::Other, message: "实体尚未创建，无法撤销".to_string() })
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}
