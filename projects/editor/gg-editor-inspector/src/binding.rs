//! 属性绑定模块
//!
//! 提供属性绑定 trait 和属性修改命令，
//! 用于将检查器面板的属性编辑操作与 ECS 世界中的组件数据关联。

use gg_core::GResult;
use gg_ecs::{Entity, World};
use gg_editor_shell::{Command, EditorContext};

/// 属性绑定 trait
///
/// 定义属性值在 ECS 世界中的读写接口，
/// 用于将检查器面板的属性编辑操作桥接到具体的组件字段。
pub trait PropertyBinding {
    /// 从 ECS 世界中读取属性值
    fn read(&self, world: &mut World, entity: Entity) -> Option<String>;

    /// 向 ECS 世界中写入属性值
    fn write(&self, world: &mut World, entity: Entity, value: &str) -> GResult<()>;
}

/// 设置属性命令
///
/// 可撤销的属性修改命令，通过属性绑定读写 ECS 世界中的组件属性值，
/// 支持撤销和重做操作。
pub struct SetPropertyCommand {
    /// 属性绑定
    binding: Box<dyn PropertyBinding>,
    /// 目标实体 ID
    entity: u64,
    /// 旧值
    old_value: Option<String>,
    /// 新值
    new_value: String,
    /// 命令描述
    description: String,
}

impl SetPropertyCommand {
    /// 创建新的设置属性命令
    pub fn new(binding: Box<dyn PropertyBinding>, entity: u64, new_value: String, description: String) -> Self {
        Self { binding, entity, old_value: None, new_value, description }
    }
}

impl Command for SetPropertyCommand {
    fn execute(&mut self, context: &mut EditorContext) -> gg_core::GResult<()> {
        let world = context.services_mut().get_mut::<World>();
        if let Some(world) = world {
            self.old_value = self.binding.read(world, self.entity);
            self.binding.write(world, self.entity, &self.new_value)?;
        }
        Ok(())
    }

    fn undo(&mut self, context: &mut EditorContext) -> gg_core::GResult<()> {
        if let Some(ref old_value) = self.old_value {
            let world = context.services_mut().get_mut::<World>();
            if let Some(world) = world {
                self.binding.write(world, self.entity, old_value)?;
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}
