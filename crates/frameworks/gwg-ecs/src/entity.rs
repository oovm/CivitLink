//! 实体分配器
//!
//! 负责实体的创建、销毁和代数管理。

use gwg_types::prelude::*;
use std::collections::VecDeque;

/// 实体位置信息
#[derive(Clone, Copy, Debug)]
pub struct EntityLocation {
    /// 实体所在的 Archetype 索引
    pub archetype_id: u32,
    /// 实体在 Archetype 中的行索引
    pub row: u32,
}

impl EntityLocation {
    /// 创建新的实体位置
    pub const fn new(archetype_id: u32, row: u32) -> Self {
        Self { archetype_id, row }
    }
}

/// 实体元数据
#[derive(Clone, Copy, Debug)]
struct EntityMeta {
    /// 实体代数
    generation: u32,
    /// 实体位置（None 表示实体已销毁）
    location: Option<EntityLocation>,
}

/// 实体分配器
///
/// 管理实体的创建、销毁和代数追踪。
pub struct EntityAllocator {
    /// 实体元数据列表
    entities: Vec<EntityMeta>,
    /// 空闲实体索引队列（用于重用）
    free_indices: VecDeque<u32>,
}

impl EntityAllocator {
    /// 创建新的实体分配器
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_indices: VecDeque::new(),
        }
    }

    /// 分配一个新实体
    pub fn allocate(&mut self) -> Entity {
        if let Some(index) = self.free_indices.pop_front() {
            let meta = &mut self.entities[index as usize];
            meta.location = Some(EntityLocation::new(0, 0));
            Entity::new(index, meta.generation)
        } else {
            let index = self.entities.len() as u32;
            self.entities.push(EntityMeta {
                generation: 0,
                location: Some(EntityLocation::new(0, 0)),
            });
            Entity::new(index, 0)
        }
    }

    /// 销毁一个实体
    ///
    /// 返回实体的位置信息（如果实体存在）
    pub fn destroy(&mut self, entity: Entity) -> Option<EntityLocation> {
        let index = entity.index() as usize;
        if index >= self.entities.len() {
            return None;
        }

        let meta = &mut self.entities[index];
        if meta.generation != entity.generation() {
            return None;
        }

        let location = meta.location.take();
        meta.generation = meta.generation.wrapping_add(1);
        self.free_indices.push_back(entity.index());
        location
    }

    /// 获取实体的位置
    pub fn get_location(&self, entity: Entity) -> Option<EntityLocation> {
        let index = entity.index() as usize;
        self.entities.get(index).and_then(|meta| {
            if meta.generation == entity.generation() {
                meta.location
            } else {
                None
            }
        })
    }

    /// 设置实体的位置
    pub fn set_location(&mut self, entity: Entity, location: EntityLocation) -> bool {
        let index = entity.index() as usize;
        if let Some(meta) = self.entities.get_mut(index) {
            if meta.generation == entity.generation() {
                meta.location = Some(location);
                return true;
            }
        }
        false
    }

    /// 检查实体是否存活
    pub fn is_alive(&self, entity: Entity) -> bool {
        let index = entity.index() as usize;
        self.entities
            .get(index)
            .map(|meta| meta.generation == entity.generation() && meta.location.is_some())
            .unwrap_or(false)
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.entities.len() - self.free_indices.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}
