#![warn(missing_docs)]

//! Archetype 模块
//!
//! Archetype 是具有相同组件集合的所有实体的存储容器。
//! 通过将组件集合相同的实体组织在一起，实现缓存友好的行式存储。

use std::{any::TypeId, collections::HashSet};

use crate::{Component, entity::Entity, storage::ComponentStorage};

/// Archetype 标识符
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub u32);

impl ArchetypeId {
    /// 创建新的 Archetype 标识符
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Archetype，存储具有相同组件集合的实体
///
/// 每个实体占一行，同一列存储相同类型的组件。
/// 组件数据在内存中连续排列，提高缓存命中率。
pub struct Archetype {
    /// Archetype 标识符
    id: ArchetypeId,
    /// 组件类型集合
    component_types: HashSet<TypeId>,
    /// 组件存储
    storage: ComponentStorage,
    /// 实体 ID 列表（行到实体的映射）
    entities: Vec<Entity>,
}

impl Archetype {
    /// 创建新的空 Archetype
    pub fn new(id: ArchetypeId) -> Self {
        Self { id, component_types: HashSet::new(), storage: ComponentStorage::new(), entities: Vec::new() }
    }

    /// 创建具有指定组件类型的 Archetype
    pub fn with_components(id: ArchetypeId, component_types: HashSet<TypeId>) -> Self {
        Self { id, component_types, storage: ComponentStorage::new(), entities: Vec::new() }
    }

    /// 获取 Archetype 标识符
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// 检查是否包含指定组件类型
    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }

    /// 获取组件类型集合
    pub fn component_types(&self) -> &HashSet<TypeId> {
        &self.component_types
    }

    /// 添加实体到 Archetype
    ///
    /// 返回新实体所在的行索引。
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);
        for type_id in self.component_types.iter() {
            if let Some(column) = self.storage.get_column_mut(*type_id) {
                column.ensure_len(row + 1);
            }
        }
        row
    }

    /// 移除实体（交换移除）
    ///
    /// 将末尾实体移动到被删位置，保持数据紧凑。
    /// 返回被移动的实体（如果有），调用方需要更新其 EntityLocation。
    pub fn remove_entity(&mut self, row: usize) -> Option<SwappedEntity> {
        if row >= self.entities.len() {
            return None;
        }

        let last_row = self.entities.len() - 1;

        for type_id in self.component_types.iter() {
            if let Some(column) = self.storage.get_column_mut(*type_id) {
                column.swap_remove(row);
            }
        }

        let swapped = if row < last_row {
            self.entities[row] = self.entities[last_row];
            Some(SwappedEntity { entity: self.entities[row], new_row: row })
        }
        else {
            None
        };

        self.entities.pop();
        swapped
    }

    /// 获取指定行的实体
    pub fn get_entity(&self, row: usize) -> Option<Entity> {
        self.entities.get(row).copied()
    }

    /// 添加组件到指定行
    pub fn add_component<T: Component>(&mut self, row: usize, component: T) {
        let column = self.storage.get_or_add_column::<T>();
        column.ensure_len(row + 1);
        column.set(row, component);
        self.component_types.insert(TypeId::of::<T>());
    }

    /// 添加类型擦除的组件到指定行
    ///
    /// 与 add_component 类似，但接受已装箱的类型擦除组件。
    /// 通过类型 ID 查找或创建组件列，然后将组件数据写入对应位置。
    pub fn add_component_raw(&mut self, row: usize, component: Box<dyn std::any::Any + Send + Sync>, type_id: std::any::TypeId) {
        let column = self.storage.get_column_mut(type_id);
        if let Some(column) = column {
            column.ensure_len(row + 1);
            let size = column.size();
            if size > 0 {
                unsafe {
                    let src = &*component as *const (dyn std::any::Any + Send + Sync) as *const u8;
                    column.set_raw(row, src, size);
                }
            }
        }
        self.component_types.insert(type_id);
    }

    /// 确保组件列存在
    pub fn ensure_column<T: Component>(&mut self) {
        self.storage.get_or_add_column::<T>();
    }

    /// 标记拥有组件类型（不创建列）
    pub fn mark_has_component<T: Component>(&mut self) {
        self.component_types.insert(TypeId::of::<T>());
    }

    /// 获取指定行的组件引用
    pub fn get_component<T: Component>(&self, row: usize) -> Option<&T> {
        self.storage.get_column(TypeId::of::<T>())?.get(row)
    }

    /// 获取指定行的组件可变引用
    pub fn get_component_mut<T: Component>(&mut self, row: usize) -> Option<&mut T> {
        self.storage.get_column_mut(TypeId::of::<T>())?.get_mut(row)
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

    /// 获取组件存储引用
    pub fn storage(&self) -> &ComponentStorage {
        &self.storage
    }

    /// 获取组件存储可变引用
    pub fn storage_mut(&mut self) -> &mut ComponentStorage {
        &mut self.storage
    }

    /// 遍历所有实体
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }
}

/// 交换移除后被移动的实体信息
///
/// 当从 Archetype 中移除实体时，末尾实体会被移动到被删位置，
/// 此结构体记录被移动的实体及其新行索引，
/// 调用方需要更新该实体的 EntityLocation。
#[derive(Clone, Copy, Debug)]
pub struct SwappedEntity {
    /// 被移动的实体
    pub entity: Entity,
    /// 被移动到的新行索引
    pub new_row: usize,
}

/// 将 HashSet<TypeId> 转换为可哈希的排序键
fn type_set_to_key(types: &HashSet<TypeId>) -> Vec<TypeId> {
    let mut vec: Vec<TypeId> = types.iter().copied().collect();
    vec.sort_by_key(|id| format!("{:?}", id));
    vec
}

/// Archetype 图，管理 Archetype 的创建和查找
///
/// 维护组件类型集合到 Archetype 的映射，
/// 当实体添加/移除组件时，自动查找或创建目标 Archetype。
pub struct ArchetypeGraph {
    /// Archetype 集合
    archetypes: Vec<Archetype>,
    /// 组件类型集合到 Archetype ID 的映射
    type_set_to_archetype: std::collections::HashMap<Vec<TypeId>, ArchetypeId>,
}

impl ArchetypeGraph {
    /// 创建新的 Archetype 图
    pub fn new() -> Self {
        Self { archetypes: Vec::new(), type_set_to_archetype: std::collections::HashMap::new() }
    }

    /// 获取或创建具有指定组件类型的 Archetype
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

    /// 获取指定 ID 的 Archetype
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.0 as usize)
    }

    /// 获取指定 ID 的 Archetype 可变引用
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.0 as usize)
    }

    /// 获取或创建空 Archetype（无组件）
    pub fn empty_archetype(&mut self) -> ArchetypeId {
        self.get_or_create(HashSet::new())
    }

    /// 根据组件类型集合查找 Archetype ID
    pub fn get_by_types(&self, types: &HashSet<TypeId>) -> Option<ArchetypeId> {
        let key = type_set_to_key(types);
        self.type_set_to_archetype.get(&key).copied()
    }

    /// 遍历所有 Archetype
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    /// 遍历所有 Archetype（可变）
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
