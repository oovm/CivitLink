//! Archetype 实现
//!
//! Archetype 是具有相同组件集合的所有实体的存储容器。

use std::any::TypeId;
use std::collections::HashSet;

use gwg_types::prelude::*;
use crate::storage::ComponentStorage;

/// Archetype ID
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub u32);

impl ArchetypeId {
    /// 创建新的 Archetype ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Archetype，存储具有相同组件集合的实体
pub struct Archetype {
    /// Archetype ID
    id: ArchetypeId,
    /// 组件类型集合
    component_types: HashSet<TypeId>,
    /// 组件存储
    storage: ComponentStorage,
    /// 实体 ID 列表
    entities: Vec<Entity>,
    /// 实体数量
    len: usize,
}

impl Archetype {
    /// 创建新的 Archetype
    pub fn new(id: ArchetypeId) -> Self {
        Self {
            id,
            component_types: HashSet::new(),
            storage: ComponentStorage::new(),
            entities: Vec::new(),
            len: 0,
        }
    }

    /// 创建具有指定组件类型的 Archetype
    pub fn with_components(id: ArchetypeId, component_types: HashSet<TypeId>) -> Self {
        Self {
            id,
            component_types,
            storage: ComponentStorage::new(),
            entities: Vec::new(),
            len: 0,
        }
    }

    /// 获取 Archetype ID
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.len
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 检查是否包含指定组件类型
    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }

    /// 获取组件类型集合
    pub fn component_types(&self) -> &HashSet<TypeId> {
        &self.component_types
    }

    /// 添加实体
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let row = self.len;
        self.entities.push(entity);
        self.len += 1;
        row
    }

    /// 移除实体（交换移除）
    pub fn remove_entity(&mut self, row: usize) -> Option<Entity> {
        if row >= self.len {
            return None;
        }

        self.len -= 1;
        
        for type_id in self.component_types.iter() {
            if let Some(column) = self.storage.get_column_mut(*type_id) {
                column.swap_remove(row);
            }
        }

        if row < self.len {
            self.entities[row] = self.entities[self.len];
        }
        self.entities.pop()
    }

    /// 获取指定行的实体
    pub fn get_entity(&self, row: usize) -> Option<Entity> {
        self.entities.get(row).copied()
    }

    /// 添加组件到实体
    pub fn add_component<T: Component>(&mut self, row: usize, component: T) {
        let column = self.storage.get_or_add_column::<T>();
        column.ensure_len(row + 1);
        column.set(row, component);
        self.component_types.insert(TypeId::of::<T>());
    }

    /// 确保组件列存在
    pub fn ensure_column<T: Component>(&mut self) {
        self.storage.get_or_add_column::<T>();
    }

    /// 标记拥有组件类型
    pub fn mark_has_component<T: Component>(&mut self) {
        self.component_types.insert(TypeId::of::<T>());
    }

    /// 获取组件引用
    pub fn get_component<T: Component>(&self, row: usize) -> Option<&T> {
        self.storage
            .get_column(TypeId::of::<T>())?
            .get(row)
    }

    /// 获取组件可变引用
    pub fn get_component_mut<T: Component>(&mut self, row: usize) -> Option<&mut T> {
        self.storage
            .get_column_mut(TypeId::of::<T>())?
            .get_mut(row)
    }

    /// 检查是否匹配指定的组件类型集合
    pub fn matches(&self, types: &HashSet<TypeId>) -> bool {
        self.component_types == *types
    }

    /// 检查是否包含所有指定的组件类型
    pub fn contains_all(&self, types: &[TypeId]) -> bool {
        types.iter().all(|t| self.component_types.contains(t))
    }

    /// 检查是否不包含任何指定的组件类型
    pub fn contains_none(&self, types: &[TypeId]) -> bool {
        types.iter().all(|t| !self.component_types.contains(t))
    }

    /// 获取组件存储
    pub fn storage(&self) -> &ComponentStorage {
        &self.storage
    }

    /// 获取组件存储可变引用
    pub fn storage_mut(&mut self) -> &mut ComponentStorage {
        &mut self.storage
    }

    /// 遍历所有实体
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().take(self.len).copied()
    }
}

/// 将 HashSet<TypeId> 转换为可哈希的键
fn type_set_to_key(types: &HashSet<TypeId>) -> Vec<TypeId> {
    let mut vec: Vec<TypeId> = types.iter().copied().collect();
    vec.sort_by_key(|id| format!("{:?}", id));
    vec
}

/// Archetype 图，管理 Archetype 之间的转换关系
pub struct ArchetypeGraph {
    /// Archetype 集合
    archetypes: Vec<Archetype>,
    /// 组件类型集合到 Archetype ID 的映射
    type_set_to_archetype: std::collections::HashMap<Vec<TypeId>, ArchetypeId>,
}

impl ArchetypeGraph {
    /// 创建新的 Archetype 图
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            type_set_to_archetype: std::collections::HashMap::new(),
        }
    }

    /// 获取或创建 Archetype
    pub fn get_or_create(&mut self, component_types: HashSet<TypeId>) -> ArchetypeId {
        let key = type_set_to_key(&component_types);
        if let Some(&id) = self.type_set_to_archetype.get(&key) {
            return id;
        }

        let id = ArchetypeId::new(self.archetypes.len() as u32);
        let archetype = Archetype::with_components(id, component_types);
        self.archetypes.push(archetype);
        self.type_set_to_archetype.insert(key, id);
        id
    }

    /// 获取 Archetype
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.0 as usize)
    }

    /// 获取 Archetype 可变引用
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.0 as usize)
    }

    /// 获取空 Archetype（无组件）
    pub fn empty_archetype(&mut self) -> ArchetypeId {
        self.get_or_create(HashSet::new())
    }

    /// 根据 ID 集合获取 Archetype
    pub fn get_by_types(&self, types: &HashSet<TypeId>) -> Option<ArchetypeId> {
        let key = type_set_to_key(types);
        self.type_set_to_archetype.get(&key).copied()
    }

    /// 遍历所有 Archetype
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    /// 遍历所有 Archetype 可变引用
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.iter_mut()
    }

    /// 获取 Archetype 数量
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}

impl Default for ArchetypeGraph {
    fn default() -> Self {
        Self::new()
    }
}
