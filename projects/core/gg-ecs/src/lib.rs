//! GG 引擎 ECS 核心模块
//! 提供实体-组件-系统架构

use gg_core::{GResult, GError, GErrorKind};
use std::collections::{HashMap, HashSet};
use std::any::{Any, TypeId};

/// 实体 ID 类型
pub type Entity = u64;

/// 组件 trait
pub trait Component: Any + Send + Sync {
    /// 获取组件类型 ID
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// 系统 trait
pub trait System {
    /// 系统名称
    fn name(&self) -> &str;
    
    /// 执行系统
    fn execute(&mut self, world: &mut World) -> GResult<()>;
}

/// 组件存储
struct ComponentStore {
    components: HashMap<Entity, Box<dyn Any + Send + Sync>>,
}

impl ComponentStore {
    /// 创建新的组件存储
    fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    /// 添加组件
    fn add(&mut self, entity: Entity, component: Box<dyn Any + Send + Sync>) {
        self.components.insert(entity, component);
    }
    
    /// 获取组件
    fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.components.get(&entity)
            .and_then(|c| c.downcast_ref::<T>())
    }
    
    /// 获取可变组件
    fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(&entity)
            .and_then(|c| c.downcast_mut::<T>())
    }
    
    /// 移除组件
    fn remove(&mut self, entity: Entity) -> Option<Box<dyn Any + Send + Sync>> {
        self.components.remove(&entity)
    }
}

/// 世界结构
pub struct World {
    /// 实体计数器
    next_entity: Entity,
    /// 组件存储映射
    component_stores: HashMap<TypeId, ComponentStore>,
    /// 活动实体
    entities: HashSet<Entity>,
    /// 系统列表
    systems: Vec<Box<dyn System>>,
}

impl World {
    /// 创建新的世界
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            component_stores: HashMap::new(),
            entities: HashSet::new(),
            systems: Vec::new(),
        }
    }
    
    /// 创建实体
    pub fn spawn(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.insert(entity);
        entity
    }
    
    /// 销毁实体
    pub fn despawn(&mut self, entity: Entity) -> GResult<()> {
        if !self.entities.contains(&entity) {
            return Err(GError {
                kind: GErrorKind::Ecs,
                message: format!("Entity {} does not exist", entity),
            });
        }
        
        // 移除所有组件
        for store in self.component_stores.values_mut() {
            store.remove(entity);
        }
        
        // 移除实体
        self.entities.remove(&entity);
        Ok(())
    }
    
    /// 添加组件
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> GResult<()> {
        if !self.entities.contains(&entity) {
            return Err(GError {
                kind: GErrorKind::Ecs,
                message: format!("Entity {} does not exist", entity),
            });
        }
        
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.entry(type_id)
            .or_insert_with(ComponentStore::new);
        store.add(entity, Box::new(component));
        Ok(())
    }
    
    /// 获取组件
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get(&type_id)
            .and_then(|store| store.get::<T>(entity))
    }
    
    /// 获取可变组件
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get_mut(&type_id)
            .and_then(|store| store.get_mut::<T>(entity))
    }
    
    /// 移除组件
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<Box<T>> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get_mut(&type_id)
            .and_then(|store| store.remove(entity))
            .and_then(|c| c.downcast::<T>().ok())
    }
    
    /// 注册系统
    pub fn register_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }
    
    /// 执行所有系统
    pub fn run_systems(&mut self) -> GResult<()> {
        // 临时取出系统列表，避免同时可变借用
        let mut systems = std::mem::take(&mut self.systems);
        for system in &mut systems {
            system.execute(self)?;
        }
        // 放回系统列表
        self.systems = systems;
        Ok(())
    }
    
    /// 获取所有实体
    pub fn entities(&self) -> &HashSet<Entity> {
        &self.entities
    }
}

/// 系统调度器
pub struct Scheduler {
    /// 世界
    world: World,
}

impl Scheduler {
    /// 创建新的调度器
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
    
    /// 获取世界
    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }
    
    /// 执行一帧
    pub fn tick(&mut self) -> GResult<()> {
        self.world.run_systems()
    }
}
