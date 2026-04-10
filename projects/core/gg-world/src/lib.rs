#![warn(missing_docs)]

//! GG 引擎世界管理模块
//! 提供游戏世界集成和多个世界管理功能

use gg_asset::AssetServer;
use gg_ecs::{Entity, EntityBuilder, GgWorld, Resource};
use gg_error::GResult;
use gg_reflection::ReflectionRegistry;

/// 游戏世界，集成 ECS 世界、资源服务器和反射注册表
///
/// 提供统一的实体管理、资源管理和类型反射接口，
/// 是游戏运行时的核心容器。
pub struct GameWorld {
    /// ECS 世界实例，管理实体和组件
    pub ecs_world: GgWorld,
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
            ecs_world: GgWorld::new(),
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
        self.ecs_world = GgWorld::new();
        self.asset_server = AssetServer::new();
        self.reflection_registry = ReflectionRegistry::new();
        self.name.clear();
    }

    /// 清空所有实体和资源
    pub fn clear(&mut self) {
        self.ecs_world = GgWorld::new();
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
}

/// 世界管理器，管理多个游戏世界
///
/// 使用槽位存储管理多个 `GameWorld` 实例，
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
        Self {
            worlds: Vec::new(),
            active_world: None,
            next_id: 0,
        }
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
        } else {
            false
        }
    }

    /// 获取所有未销毁世界的 ID 列表
    pub fn world_ids(&self) -> Vec<usize> {
        self.worlds
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|_| i))
            .collect()
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
