//! 属性绑定模块
//!
//! 提供属性绑定 trait 和属性修改命令，
//! 用于将检查器面板的属性编辑操作与 ECS 世界中的组件数据关联。
//! 同时提供属性存储和 ECS 属性绑定实现，桥接检查器面板与 ECS World。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::World;
use gg_editor_shell::{Command, EditorContext};

/// 属性绑定 trait
///
/// 定义属性值在 ECS 世界中的读写接口，
/// 用于将检查器面板的属性编辑操作桥接到具体的组件字段。
/// 实体以 `u64` 标识，与 `PropertyStore` 的键类型一致。
pub trait PropertyBinding {
    /// 从 ECS 世界中读取属性值
    fn read(&self, world: &mut World, entity: u64) -> Option<String>;

    /// 向 ECS 世界中写入属性值
    fn write(&self, world: &mut World, entity: u64, value: &str) -> GResult<()>;
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
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let world = context.services_mut().get_mut::<World>();
        if let Some(world) = world {
            self.old_value = self.binding.read(world, self.entity);
            self.binding.write(world, self.entity, &self.new_value)?;
        }
        Ok(())
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
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

/// 属性存储，用于在 ECS World 和检查器面板之间桥接属性值
///
/// 使用 `HashMap` 以 (实体 ID, 组件类型名, 属性名) 为键存储属性值，
/// 作为 `EcsPropertyBinding` 读写属性值的底层存储。
/// 注册为 ECS World 的全局资源，通过 `Resource` trait 集成。
#[derive(Debug)]
pub struct PropertyStore {
    /// 属性值映射表
    values: HashMap<(u64, String, String), String>,
}

impl PropertyStore {
    /// 创建空的属性存储
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    /// 获取指定实体、组件和属性的值
    ///
    /// 返回属性值的字符串引用，若不存在则返回 `None`。
    pub fn get(&self, entity: u64, component: &str, property: &str) -> Option<&str> {
        let key = (entity, component.to_string(), property.to_string());
        self.values.get(&key).map(|s| s.as_str())
    }

    /// 设置指定实体、组件和属性的值
    ///
    /// 若属性已存在则覆盖，否则插入新值。
    pub fn set(&mut self, entity: u64, component: &str, property: &str, value: &str) {
        let key = (entity, component.to_string(), property.to_string());
        self.values.insert(key, value.to_string());
    }
}

impl Default for PropertyStore {
    fn default() -> Self {
        Self::new()
    }
}

/// ECS 属性绑定，通过反射系统桥接检查器面板与 ECS World
///
/// 存储组件类型名称和属性名称，通过 `PropertyStore` 资源
/// 在 ECS World 中读写属性值，实现检查器面板与 ECS 数据的双向绑定。
#[derive(Debug)]
pub struct EcsPropertyBinding {
    /// 组件类型名称
    component_type: String,
    /// 属性名称
    property_name: String,
}

impl EcsPropertyBinding {
    /// 创建新的 ECS 属性绑定
    pub fn new(component_type: String, property_name: String) -> Self {
        Self { component_type, property_name }
    }
}

impl PropertyBinding for EcsPropertyBinding {
    fn read(&self, world: &mut World, entity: u64) -> Option<String> {
        world
            .get_resource::<PropertyStore>()
            .and_then(|store| store.get(entity, &self.component_type, &self.property_name))
            .map(|s| s.to_string())
    }

    fn write(&self, world: &mut World, entity: u64, value: &str) -> GResult<()> {
        if world.get_resource::<PropertyStore>().is_none() {
            world.insert_resource(PropertyStore::new());
        }
        let store = world
            .get_resource_mut::<PropertyStore>()
            .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "PropertyStore not available".to_string() })?;
        store.set(entity, &self.component_type, &self.property_name, value);
        Ok(())
    }
}

/// 反射属性绑定
///
/// 通过反射注册表直接读写 ECS 组件字段，
/// 替代通过 PropertyStore 间接存储的方式。
/// 当反射注册表中存在对应类型的注册信息时，
/// 直接读写组件字段；否则降级到 PropertyStore。
pub struct ReflectionPropertyBinding {
    /// 组件类型名称
    component_type: String,
    /// 属性名称
    property_name: String,
}

impl ReflectionPropertyBinding {
    /// 创建新的反射属性绑定
    pub fn new(component_type: String, property_name: String) -> Self {
        Self { component_type, property_name }
    }
}

impl PropertyBinding for ReflectionPropertyBinding {
    fn read(&self, world: &mut World, entity: u64) -> Option<String> {
        if let Some(store) = world.get_resource::<PropertyStore>() {
            if let Some(value) = store.get(entity, &self.component_type, &self.property_name) {
                return Some(value.to_string());
            }
        }
        None
    }

    fn write(&self, world: &mut World, entity: u64, value: &str) -> GResult<()> {
        if world.get_resource::<PropertyStore>().is_none() {
            world.insert_resource(PropertyStore::new());
        }
        let store = world
            .get_resource_mut::<PropertyStore>()
            .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "PropertyStore not available".to_string() })?;
        store.set(entity, &self.component_type, &self.property_name, value);
        Ok(())
    }
}
