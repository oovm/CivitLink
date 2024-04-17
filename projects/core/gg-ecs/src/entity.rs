#![warn(missing_docs)]

//! 实体分配器模块
//!
//! 负责实体的创建、销毁和代数管理，支持实体 ID 复用。

use std::collections::VecDeque;

/// 实体标识符
///
/// 由索引和代数组成，代数用于检测悬空引用。
/// 当实体被销毁后，相同索引的新实体会获得递增的代数，
/// 使旧的实体引用失效。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    /// 实体索引
    index: u32,
    /// 实体代数
    generation: u32,
}

impl Entity {
    /// 创建新的实体标识符
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// 获取实体索引
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// 获取实体代数
    pub const fn generation(&self) -> u32 {
        self.generation
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({}:{})", self.index, self.generation)
    }
}

/// 实体位置信息
///
/// 记录实体在 Archetype 中的存储位置，
/// 用于快速定位实体的组件数据。
#[derive(Clone, Copy, Debug)]
pub struct EntityLocation {
    /// 实体所在的 Archetype ID
    pub archetype_id: u32,
    /// 实体在 Archetype 中的行索引
    pub row: usize,
}

impl EntityLocation {
    /// 创建新的实体位置
    pub const fn new(archetype_id: u32, row: usize) -> Self {
        Self { archetype_id, row }
    }
}

/// 实体元数据（内部使用）
#[derive(Clone, Copy, Debug)]
struct EntityMeta {
    /// 实体代数
    generation: u32,
    /// 实体位置，None 表示实体已销毁
    location: Option<EntityLocation>,
}

/// 实体分配器
///
/// 管理实体的创建、销毁和代数追踪。
/// 支持实体 ID 复用：销毁后的实体索引会被加入空闲队列，
/// 下次分配时优先复用已释放的索引。
pub struct EntityAllocator {
    /// 实体元数据列表
    entities: Vec<EntityMeta>,
    /// 空闲实体索引队列（FIFO，用于复用）
    free_indices: VecDeque<u32>,
}

impl EntityAllocator {
    /// 创建新的实体分配器
    pub fn new() -> Self {
        Self { entities: Vec::new(), free_indices: VecDeque::new() }
    }

    /// 分配一个新实体
    ///
    /// 优先复用已释放的索引，否则追加新索引。
    /// 新实体的代数从 0 开始（复用索引时代数已在销毁时递增）。
    pub fn allocate(&mut self) -> Entity {
        if let Some(index) = self.free_indices.pop_front() {
            let meta = &mut self.entities[index as usize];
            meta.location = Some(EntityLocation::new(0, 0));
            Entity::new(index, meta.generation)
        }
        else {
            let index = self.entities.len() as u32;
            self.entities.push(EntityMeta { generation: 0, location: Some(EntityLocation::new(0, 0)) });
            Entity::new(index, 0)
        }
    }

    /// 销毁一个实体
    ///
    /// 验证代数匹配后，将位置置空、代数递增、索引加入空闲队列。
    /// 返回实体之前的位置信息，用于从 Archetype 中移除数据。
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

    /// 获取实体的位置信息
    ///
    /// 如果实体不存在或代数不匹配，返回 None。
    pub fn get_location(&self, entity: Entity) -> Option<EntityLocation> {
        let index = entity.index() as usize;
        self.entities.get(index).and_then(|meta| if meta.generation == entity.generation() { meta.location } else { None })
    }

    /// 设置实体的位置信息
    ///
    /// 如果实体不存在或代数不匹配，返回 false。
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
        self.entities.get(index).map(|meta| meta.generation == entity.generation() && meta.location.is_some()).unwrap_or(false)
    }

    /// 获取存活实体数量
    pub fn len(&self) -> usize {
        self.entities.len() - self.free_indices.len()
    }

    /// 检查是否没有任何存活实体
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 遍历所有存活实体
    pub fn iter_alive(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().enumerate().filter_map(|(index, meta)| {
            if meta.location.is_some() { Some(Entity::new(index as u32, meta.generation)) } else { None }
        })
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}
