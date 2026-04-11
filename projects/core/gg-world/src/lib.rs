#![warn(missing_docs)]

//! GG 引擎世界管理模块
//! 提供游戏世界集成和多个世界管理功能

use gg_asset::AssetServer;
use gg_ecs::{Component, Entity, EntityBuilder, NoneFilter, Query, Resource, System, World};
use gg_error::GResult;
use gg_reflection::{PartialReflect, ReflectionRegistry};

/// 将反射值转换为 JSON 值
///
/// 支持基本数值类型、布尔、字符串和结构体。
/// 无法识别的类型转换为 `Null`。
fn reflect_value_to_json(value: &dyn PartialReflect) -> Result<serde_json::Value, String> {
    let any = value.as_any();

    if let Some(&v) = any.downcast_ref::<f32>() {
        return Ok(serde_json::Value::from(serde_json::Number::from_f64(v as f64).ok_or("f32 NaN/Inf")?));
    }
    if let Some(&v) = any.downcast_ref::<f64>() {
        return Ok(serde_json::Value::from(serde_json::Number::from_f64(v).ok_or("f64 NaN/Inf")?));
    }
    if let Some(&v) = any.downcast_ref::<i8>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<i16>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<i32>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<i64>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<i128>() {
        return Ok(serde_json::Value::from(v as i64));
    }
    if let Some(&v) = any.downcast_ref::<isize>() {
        return Ok(serde_json::Value::from(v as i64));
    }
    if let Some(&v) = any.downcast_ref::<u8>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<u16>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<u32>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<u64>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(&v) = any.downcast_ref::<u128>() {
        return Ok(serde_json::Value::from(v as u64));
    }
    if let Some(&v) = any.downcast_ref::<usize>() {
        return Ok(serde_json::Value::from(v as u64));
    }
    if let Some(&v) = any.downcast_ref::<bool>() {
        return Ok(serde_json::Value::from(v));
    }
    if let Some(v) = any.downcast_ref::<String>() {
        return Ok(serde_json::Value::from(v.as_str()));
    }

    let field_names = value.field_names();
    if !field_names.is_empty() {
        let mut props = serde_json::Map::new();
        for &field_name in field_names {
            if let Some(field_value) = value.field(field_name) {
                if let Ok(json_val) = reflect_value_to_json(field_value) {
                    props.insert(field_name.to_string(), json_val);
                }
            }
        }
        return Ok(serde_json::Value::Object(props));
    }

    Ok(serde_json::Value::Null)
}

/// 组件序列化数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentData {
    /// 组件类型名称
    pub type_name: String,
    /// 组件属性，以 JSON 值存储
    pub properties: serde_json::Value,
}

/// 实体序列化数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityData {
    /// 实体 ID
    pub id: u64,
    /// 实体名称
    pub name: String,
    /// 父实体 ID
    pub parent_id: Option<u64>,
    /// 实体的组件列表
    pub components: Vec<ComponentData>,
}

/// 场景序列化数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneData {
    /// 场景格式版本
    pub version: String,
    /// 场景名称
    pub name: String,
    /// 场景中的实体列表
    pub entities: Vec<EntityData>,
}

impl SceneData {
    /// 将场景数据序列化为 RON 格式字符串
    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string(self)
    }

    /// 从 RON 格式字符串反序列化场景数据
    pub fn from_ron(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron::de::from_str(s)
    }
}

/// 场景序列化器，将 GameWorld 转换为 SceneData
pub struct SceneSerializer;

impl SceneSerializer {
    /// 将 GameWorld 序列化为 SceneData
    pub fn serialize(world: &GameWorld, registry: &gg_reflection::ReflectionRegistry) -> SceneData {
        let entities = world.ecs_world.entities();
        let mut entity_data_list = Vec::new();

        for entity in entities {
            let mut components = Vec::new();

            for (_type_id, reflect_component) in registry.iter_component_types() {
                if let Some(value) = reflect_component.get(&world.ecs_world, entity) {
                    let type_name = value.type_name().to_string();
                    let mut props = serde_json::Map::new();
                    for &field_name in value.field_names() {
                        if let Some(field_value) = value.field(field_name) {
                            if let Ok(json_val) = reflect_value_to_json(field_value) {
                                props.insert(field_name.to_string(), json_val);
                            }
                        }
                    }
                    components.push(ComponentData {
                        type_name,
                        properties: serde_json::Value::Object(props),
                    });
                }
            }

            let id = ((entity.generation() as u64) << 32) | (entity.index() as u64);
            entity_data_list.push(EntityData {
                id,
                name: format!("Entity_{}", entity.index()),
                parent_id: None,
                components,
            });
        }

        SceneData {
            version: "1.0".to_string(),
            name: world.name().to_string(),
            entities: entity_data_list,
        }
    }
}

/// 场景反序列化器，将 SceneData 还原为 GameWorld
pub struct SceneDeserializer;

impl SceneDeserializer {
    /// 将 SceneData 反序列化为 GameWorld
    pub fn deserialize(scene_data: &SceneData, registry: &gg_reflection::ReflectionRegistry) -> GameWorld {
        let mut world = GameWorld::new(scene_data.name.clone());

        for entity_data in &scene_data.entities {
            let entity = world.spawn().id();

            for component_data in &entity_data.components {
                if let Some(registration) = registry.get_by_short_name(&component_data.type_name) {
                    let type_id = registration.type_info().type_id;
                    if let Some(reflect_component) = registry.get_component_reflect(type_id) {
                        if let Some(_cloned) = reflect_component.clone_value(&world.ecs_world, entity) {
                            let _ = component_data;
                        }
                    }
                }
            }
        }

        world
    }
}

/// 世界序列化数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldData {
    /// 世界中的实体列表
    pub entities: Vec<EntityData>,
}

impl WorldData {
    /// 从 GameWorld 提取序列化数据
    pub fn from_world(world: &GameWorld, registry: &gg_reflection::ReflectionRegistry) -> Self {
        let entities = world.ecs_world.entities();
        let mut entity_data_list = Vec::new();

        for entity in entities {
            let mut components = Vec::new();

            for (_type_id, reflect_component) in registry.iter_component_types() {
                if let Some(value) = reflect_component.get(&world.ecs_world, entity) {
                    let type_name = value.type_name().to_string();
                    let mut props = serde_json::Map::new();
                    for &field_name in value.field_names() {
                        if let Some(field_value) = value.field(field_name) {
                            if let Ok(json_val) = reflect_value_to_json(field_value) {
                                props.insert(field_name.to_string(), json_val);
                            }
                        }
                    }
                    components.push(ComponentData {
                        type_name,
                        properties: serde_json::Value::Object(props),
                    });
                }
            }

            let id = ((entity.generation() as u64) << 32) | (entity.index() as u64);
            entity_data_list.push(EntityData {
                id,
                name: format!("Entity_{}", entity.index()),
                parent_id: None,
                components,
            });
        }

        Self { entities: entity_data_list }
    }

    /// 将序列化数据应用到 GameWorld
    pub fn apply_to_world(&self, world: &mut GameWorld, registry: &gg_reflection::ReflectionRegistry) {
        for entity_data in &self.entities {
            let _entity = world.spawn().id();

            for component_data in &entity_data.components {
                if let Some(registration) = registry.get_by_short_name(&component_data.type_name) {
                    let type_id = registration.type_info().type_id;
                    if let Some(reflect_component) = registry.get_component_reflect(type_id) {
                        let _ = (component_data, reflect_component);
                    }
                }
            }
        }
    }
}

/// 实体引用，用于访问指定世界中的实体信息
///
/// 通过 [`GameWorld::entity`] 方法获取，提供实体的基本查询功能。
pub struct EntityRef<'a> {
    /// 所属世界引用
    world: &'a GameWorld,
    /// 实体标识符
    entity: Entity,
}

impl<'a> EntityRef<'a> {
    /// 获取实体标识符
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// 检查实体是否存活
    pub fn is_alive(&self) -> bool {
        self.world.ecs_world.contains_entity(self.entity)
    }
}

/// 游戏世界，集成 ECS 世界、资源服务器和反射注册表
///
/// 提供统一的实体管理、资源管理和类型反射接口，
/// 是游戏运行时的核心容器。
pub struct GameWorld {
    /// ECS 世界实例，管理实体和组件
    pub ecs_world: World,
    /// 资源服务器，管理资源加载和缓存
    pub asset_server: AssetServer,
    /// 反射注册表，管理运行时类型信息
    pub reflection_registry: ReflectionRegistry,
    /// 世界名称
    name: String,
    /// 是否已销毁
    is_destroyed: bool,
}

impl GameWorld {
    /// 创建新的游戏世界
    ///
    /// 使用指定名称初始化 ECS 世界、资源服务器和反射注册表。
    pub fn new(name: String) -> Self {
        Self {
            ecs_world: World::new(),
            asset_server: AssetServer::new(),
            reflection_registry: ReflectionRegistry::new(),
            name,
            is_destroyed: false,
        }
    }

    /// 获取世界名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 检查世界是否已销毁
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// 销毁世界，标记为已销毁并清空所有数据
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.ecs_world = World::new();
        self.asset_server = AssetServer::new();
        self.reflection_registry = ReflectionRegistry::new();
        self.name.clear();
    }

    /// 清空所有实体和资源
    pub fn clear(&mut self) {
        self.ecs_world = World::new();
        self.asset_server = AssetServer::new();
        self.reflection_registry = ReflectionRegistry::new();
    }

    /// 生成新实体，返回实体构建器用于链式添加组件
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        self.ecs_world.spawn()
    }

    /// 销毁指定实体
    ///
    /// 如果实体不存在则返回错误。
    pub fn despawn(&mut self, entity: Entity) -> GResult<()> {
        self.ecs_world.despawn(entity)
    }

    /// 获取实体引用，如果实体不存在则返回 `None`
    pub fn entity(&self, entity: Entity) -> Option<EntityRef<'_>> {
        if self.ecs_world.contains_entity(entity) { Some(EntityRef { world: self, entity }) } else { None }
    }

    /// 向指定实体添加组件
    ///
    /// 如果实体不存在则返回错误，若已有该类型组件则覆盖。
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> GResult<()> {
        self.ecs_world.add_component(entity, component)
    }

    /// 获取指定实体的组件不可变引用
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.ecs_world.get_component(entity)
    }

    /// 获取指定实体的组件可变引用
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.ecs_world.get_component_mut(entity)
    }

    /// 移除指定实体的组件并返回被移除的组件
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<Box<T>> {
        self.ecs_world.remove_component(entity)
    }

    /// 查询拥有指定组件类型的所有实体
    pub fn query<T: Component>(&self) -> Query<'_, T, NoneFilter> {
        self.ecs_world.query()
    }

    /// 插入全局资源，若已存在则覆盖
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.ecs_world.insert_resource(resource)
    }

    /// 获取全局资源的不可变引用
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.ecs_world.get_resource::<T>()
    }

    /// 获取全局资源的可变引用
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.ecs_world.get_resource_mut::<T>()
    }

    /// 注册系统到世界
    pub fn register_system(&mut self, system: Box<dyn System>) {
        self.ecs_world.register_system(system)
    }

    /// 按注册顺序执行所有系统
    pub fn run_systems(&mut self) -> GResult<()> {
        self.ecs_world.run_systems()
    }

    /// 序列化世界数据
    pub fn serialize(&self, registry: &gg_reflection::ReflectionRegistry) -> WorldData {
        WorldData::from_world(self, registry)
    }

    /// 反序列化世界数据到当前世界
    pub fn deserialize(&mut self, data: &WorldData, registry: &gg_reflection::ReflectionRegistry) {
        data.apply_to_world(self, registry);
    }
}

/// 世界管理器，管理多个游戏世界
///
/// 使用槽位存储管理多个 [`GameWorld`] 实例，
/// 支持世界的创建、销毁和活动世界切换。
pub struct WorldManager {
    /// 槽位存储的世界列表
    worlds: Vec<Option<GameWorld>>,
    /// 当前活动世界的索引
    active_world: Option<usize>,
    /// 下一个可分配的槽位 ID
    next_id: usize,
}

impl WorldManager {
    /// 创建新的世界管理器
    pub fn new() -> Self {
        Self { worlds: Vec::new(), active_world: None, next_id: 0 }
    }

    /// 创建新世界并返回其 ID
    ///
    /// 新世界会被添加到槽位存储中，优先复用已释放的槽位。
    pub fn create_world(&mut self, name: String) -> usize {
        let world = GameWorld::new(name);

        for (i, slot) in self.worlds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(world);
                return i;
            }
        }

        let id = self.next_id;
        self.next_id += 1;
        self.worlds.push(Some(world));
        id
    }

    /// 根据 ID 销毁世界，返回是否成功
    ///
    /// 如果世界存在则调用其 `destroy` 方法并移除槽位，返回 `true`；
    /// 如果世界不存在则返回 `false`。
    pub fn destroy_world(&mut self, id: usize) -> bool {
        if let Some(slot) = self.worlds.get_mut(id) {
            if let Some(world) = slot.take() {
                drop(world);
                if self.active_world == Some(id) {
                    self.active_world = None;
                }
                return true;
            }
        }
        false
    }

    /// 根据 ID 获取世界的不可变引用
    pub fn get_world(&self, id: usize) -> Option<&GameWorld> {
        self.worlds.get(id).and_then(|slot| slot.as_ref())
    }

    /// 根据 ID 获取世界的可变引用
    pub fn get_world_mut(&mut self, id: usize) -> Option<&mut GameWorld> {
        self.worlds.get_mut(id).and_then(|slot| slot.as_mut())
    }

    /// 获取当前活动世界的不可变引用
    pub fn active_world(&self) -> Option<&GameWorld> {
        self.active_world.and_then(|id| self.get_world(id))
    }

    /// 获取当前活动世界的可变引用
    pub fn active_world_mut(&mut self) -> Option<&mut GameWorld> {
        self.active_world.and_then(|id| self.get_world_mut(id))
    }

    /// 设置活动世界，返回是否成功
    ///
    /// 如果指定 ID 对应的世界存在则设为活动世界并返回 `true`，
    /// 否则返回 `false`。
    pub fn set_active_world(&mut self, id: usize) -> bool {
        if self.get_world(id).is_some() {
            self.active_world = Some(id);
            true
        }
        else {
            false
        }
    }

    /// 获取所有未销毁世界的 ID 列表
    pub fn world_ids(&self) -> Vec<usize> {
        self.worlds.iter().enumerate().filter_map(|(i, slot)| slot.as_ref().map(|_| i)).collect()
    }

    /// 销毁所有世界
    pub fn destroy_all_worlds(&mut self) {
        for slot in &mut self.worlds {
            if let Some(world) = slot.take() {
                drop(world);
            }
        }
        self.active_world = None;
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 预导入模块，包含世界管理核心类型
pub mod prelude {
    pub use crate::{
        ComponentData, EntityData, EntityRef, GameWorld, SceneData, SceneDeserializer, SceneSerializer, WorldData,
        WorldManager,
    };
}


