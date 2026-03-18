//! GWG Engine 世界管理
//!
//! 提供世界创建、销毁和管理功能。

use gwg_ecs::prelude::*;
use gwg_asset::prelude::*;
use gwg_reflection::prelude::*;

/// 游戏世界，整合 ECS 世界、资源服务器和反射注册表
pub struct GameWorld {
    /// ECS 世界
    pub ecs_world: GwgWorld,
    /// 资源服务器
    pub asset_server: AssetServer,
    /// 反射注册表
    pub reflection_registry: ReflectionRegistry,
    /// 世界名称
    name: String,
    /// 世界是否已销毁
    is_destroyed: bool,
}

impl GameWorld {
    /// 创建一个新的游戏世界
    pub fn new(name: String) -> Self {
        Self {
            ecs_world: GwgWorld::new(),
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

    /// 销毁世界
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.ecs_world = GwgWorld::new();
            self.is_destroyed = true;
        }
    }

    /// 清空世界中的所有实体和资源
    pub fn clear(&mut self) {
        self.ecs_world = GwgWorld::new();
        self.asset_server.cache().clear();
    }

    /// 插入全局资源到 ECS 世界
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.ecs_world.insert_resource(resource);
    }

    /// 获取 ECS 世界中的全局资源
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.ecs_world.get_resource()
    }

    /// 获取 ECS 世界中的全局资源的可变引用
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<Mut<'_, T>> {
        self.ecs_world.get_resource_mut()
    }

    /// 生成一个新实体
    pub fn spawn(&mut self) -> EntityWorldMut<'_> {
        self.ecs_world.spawn()
    }

    /// 根据 ID 获取实体
    pub fn entity(&self, entity: Entity) -> EntityRef<'_> {
        self.ecs_world.entity(entity)
    }

    /// 根据 ID 获取实体的可变引用
    pub fn entity_mut(&mut self, entity: Entity) -> EntityWorldMut<'_> {
        self.ecs_world.entity_mut(entity)
    }

    /// 销毁指定的实体
    pub fn despawn(&mut self, entity: Entity) {
        self.ecs_world.despawn(entity);
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new("Unnamed World".to_string())
    }
}

/// 世界管理器，负责管理多个游戏世界
pub struct WorldManager {
    worlds: Vec<Option<GameWorld>>,
    active_world: Option<usize>,
    next_id: usize,
}

impl WorldManager {
    /// 创建一个新的世界管理器
    pub fn new() -> Self {
        Self {
            worlds: Vec::new(),
            active_world: None,
            next_id: 0,
        }
    }

    /// 创建一个新的游戏世界
    pub fn create_world(&mut self, name: String) -> usize {
        let world = GameWorld::new(name);
        let id = self.next_id;
        self.next_id += 1;
        
        if id >= self.worlds.len() {
            self.worlds.resize_with(id + 1, || None);
        }
        
        self.worlds[id] = Some(world);
        
        if self.active_world.is_none() {
            self.active_world = Some(id);
        }
        
        id
    }

    /// 销毁指定 ID 的世界
    pub fn destroy_world(&mut self, id: usize) -> bool {
        if let Some(world) = self.worlds.get_mut(id) {
            if let Some(mut w) = world.take() {
                w.destroy();
                if self.active_world == Some(id) {
                    self.active_world = None;
                }
                return true;
            }
        }
        false
    }

    /// 获取指定 ID 的世界
    pub fn get_world(&self, id: usize) -> Option<&GameWorld> {
        self.worlds.get(id).and_then(|w| w.as_ref())
    }

    /// 获取指定 ID 的世界的可变引用
    pub fn get_world_mut(&mut self, id: usize) -> Option<&mut GameWorld> {
        self.worlds.get_mut(id).and_then(|w| w.as_mut())
    }

    /// 获取当前活动的世界
    pub fn active_world(&self) -> Option<&GameWorld> {
        self.active_world.and_then(|id| self.get_world(id))
    }

    /// 获取当前活动的世界的可变引用
    pub fn active_world_mut(&mut self) -> Option<&mut GameWorld> {
        self.active_world.and_then(|id| self.get_world_mut(id))
    }

    /// 设置活动世界
    pub fn set_active_world(&mut self, id: usize) -> bool {
        if self.get_world(id).is_some() {
            self.active_world = Some(id);
            true
        } else {
            false
        }
    }

    /// 获取所有世界的 ID 列表
    pub fn world_ids(&self) -> Vec<usize> {
        self.worlds
            .iter()
            .enumerate()
            .filter(|(_, w)| w.is_some())
            .map(|(id, _)| id)
            .collect()
    }

    /// 销毁所有世界
    pub fn destroy_all_worlds(&mut self) {
        for world in &mut self.worlds {
            if let Some(mut w) = world.take() {
                w.destroy();
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

pub mod prelude {
    //! 世界管理的预导入模块

    pub use super::{GameWorld, WorldManager};
    pub use gwg_ecs::prelude::*;
    pub use gwg_asset::prelude::*;
    pub use gwg_reflection::prelude::*;
}
