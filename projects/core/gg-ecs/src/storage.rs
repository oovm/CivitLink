#![warn(missing_docs)]

//! 组件存储模块
//!
//! 提供类型擦除的组件存储机制，使用手动内存管理实现高性能连续存储。

use std::{
    alloc::{Layout, alloc, dealloc},
    any::TypeId,
    ptr::{self, NonNull},
};

use crate::Component;

/// 动态组件列，存储任意类型的组件
///
/// 使用手动内存管理实现类型擦除的连续存储，
/// 支持动态扩容、swap-remove 和跨 Archetype 迁移。
pub struct ComponentColumn {
    /// 组件类型 ID
    type_id: TypeId,
    /// 组件类型名称
    type_name: &'static str,
    /// 单个组件的字节大小
    size: usize,
    /// 组件的对齐要求
    align: usize,
    /// 数据缓冲区指针
    data: NonNull<u8>,
    /// 容量（组件数量）
    capacity: usize,
    /// 长度（组件数量）
    len: usize,
    /// 析构函数指针
    drop_fn: Option<unsafe fn(*mut u8, usize)>,
}

impl ComponentColumn {
    /// 为组件类型 `T` 创建新的组件列
    pub fn new<T: Component>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            drop_fn: Some(Self::drop_impl::<T>),
        }
    }

    /// 析构函数实现
    ///
    /// 逐个调用组件的 drop 方法。
    unsafe fn drop_impl<T>(ptr: *mut u8, len: usize) {
        let size = std::mem::size_of::<T>();
        if size == 0 {
            return;
        }
        for i in 0..len {
            unsafe {
                let item_ptr = ptr.add(i * size) as *mut T;
                std::ptr::drop_in_place(item_ptr);
            }
        }
    }

    /// 获取组件类型 ID
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// 获取组件类型名称
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// 获取单个组件的字节大小
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取组件的对齐要求
    pub fn align(&self) -> usize {
        self.align
    }

    /// 获取析构函数指针
    pub fn drop_fn(&self) -> Option<unsafe fn(*mut u8, usize)> {
        self.drop_fn
    }

    /// 获取组件数量
    pub fn len(&self) -> usize {
        self.len
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 确保容量足够
    ///
    /// 容量策略为 `max(8, capacity * 2)`。
    fn ensure_capacity(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }

        let new_capacity = new_capacity.max(8).max(self.capacity * 2);
        if self.size == 0 {
            self.capacity = new_capacity;
            return;
        }

        let layout = Layout::from_size_align(new_capacity * self.size, self.align).unwrap();
        let new_data = unsafe { alloc(layout) };
        let new_data = NonNull::new(new_data).expect("allocation failed");

        if self.capacity > 0 && self.len > 0 {
            unsafe {
                ptr::copy_nonoverlapping(self.data.as_ptr(), new_data.as_ptr(), self.len * self.size);
                let old_layout = Layout::from_size_align(self.capacity * self.size, self.align).unwrap();
                dealloc(self.data.as_ptr(), old_layout);
            }
        }

        self.data = new_data;
        self.capacity = new_capacity;
    }

    /// 确保长度达到指定值
    ///
    /// 新增的空间会被零初始化。
    pub fn ensure_len(&mut self, new_len: usize) {
        if new_len > self.len {
            self.ensure_capacity(new_len);
            if self.size > 0 {
                unsafe {
                    for i in self.len..new_len {
                        let ptr = self.data.as_ptr().add(i * self.size);
                        ptr::write_bytes(ptr, 0, self.size);
                    }
                }
            }
            self.len = new_len;
        }
    }

    /// 推入一个组件到列末尾
    pub fn push<T: Component>(&mut self, component: T) {
        debug_assert_eq!(TypeId::of::<T>(), self.type_id);
        self.ensure_capacity(self.len + 1);
        if self.size > 0 {
            unsafe {
                let ptr = self.data.as_ptr().add(self.len * self.size) as *mut T;
                ptr.write(component);
            }
        }
        self.len += 1;
    }

    /// 设置指定位置的组件
    pub fn set<T: Component>(&mut self, index: usize, component: T) {
        debug_assert_eq!(TypeId::of::<T>(), self.type_id);
        debug_assert!(index < self.len);
        if self.size > 0 {
            unsafe {
                let ptr = self.data.as_ptr().add(index * self.size) as *mut T;
                ptr.write(component);
            }
        }
    }

    /// 设置指定位置的组件（类型擦除版本）
    ///
    /// 从原始指针拷贝组件数据到指定行。
    /// 调用方必须确保 `src` 指向的数据类型与列的类型匹配，
    /// 且 `size` 与列的 `size` 一致。
    pub fn set_raw(&mut self, index: usize, src: *const u8, size: usize) {
        debug_assert_eq!(size, self.size);
        debug_assert!(index < self.len);
        if self.size > 0 {
            unsafe {
                let dst = self.data.as_ptr().add(index * self.size);
                std::ptr::copy_nonoverlapping(src, dst, self.size);
            }
        }
    }

    /// 获取指定位置的组件引用
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        if index >= self.len || TypeId::of::<T>() != self.type_id || self.size == 0 {
            return None;
        }
        unsafe {
            let ptr = self.data.as_ptr().add(index * self.size) as *const T;
            Some(&*ptr)
        }
    }

    /// 获取指定位置的组件可变引用
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len || TypeId::of::<T>() != self.type_id || self.size == 0 {
            return None;
        }
        unsafe {
            let ptr = self.data.as_ptr().add(index * self.size) as *mut T;
            Some(&mut *ptr)
        }
    }

    /// 交换移除指定位置的组件
    ///
    /// 将末尾元素拷贝到被删位置，保持数据紧凑。
    /// 注意：调用方需要更新被移动实体的 EntityLocation。
    pub fn swap_remove(&mut self, index: usize) {
        if index >= self.len || self.size == 0 {
            return;
        }
        self.len -= 1;
        if index < self.len {
            unsafe {
                let src = self.data.as_ptr().add(self.len * self.size);
                let dst = self.data.as_ptr().add(index * self.size);
                ptr::copy_nonoverlapping(src, dst, self.size);
            }
        }
    }

    /// 将指定位置的组件移动到另一个列
    ///
    /// 组件数据会被拷贝到目标列末尾，源列执行 swap-remove。
    pub fn move_to(&mut self, index: usize, other: &mut Self) {
        if index >= self.len || self.type_id != other.type_id || self.size == 0 {
            return;
        }
        other.ensure_capacity(other.len + 1);
        unsafe {
            let src = self.data.as_ptr().add(index * self.size);
            let dst = other.data.as_ptr().add(other.len * self.size);
            ptr::copy_nonoverlapping(src, dst, self.size);
        }
        other.len += 1;
        self.swap_remove(index);
    }

    /// 读取指定位置的组件原始字节（用于迁移）
    ///
    /// 返回组件数据的裸指针和大小，调用方负责确保安全性。
    pub unsafe fn read_raw(&self, index: usize) -> Option<(*const u8, usize)> {
        if index >= self.len || self.size == 0 {
            return None;
        }
        unsafe { Some((self.data.as_ptr().add(index * self.size), self.size)) }
    }

    /// 获取数据缓冲区的可变裸指针
    ///
    /// 仅用于迁移场景，调用方负责确保安全性。
    pub unsafe fn data_as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }
}

impl Drop for ComponentColumn {
    fn drop(&mut self) {
        if self.capacity > 0 && self.size > 0 {
            if let Some(drop_fn) = self.drop_fn {
                unsafe {
                    drop_fn(self.data.as_ptr(), self.len);
                }
            }
            let layout = Layout::from_size_align(self.capacity * self.size, self.align).unwrap();
            unsafe {
                dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}

unsafe impl Send for ComponentColumn {}
unsafe impl Sync for ComponentColumn {}

/// 组件存储，管理多个组件列
///
/// 每个组件类型对应一个 `ComponentColumn`，
/// 通过 `TypeId` 索引实现类型安全的存储和查询。
pub struct ComponentStorage {
    /// 组件列集合
    columns: Vec<ComponentColumn>,
    /// 类型 ID 到列索引的映射
    type_to_column: std::collections::HashMap<TypeId, usize>,
}

impl ComponentStorage {
    /// 创建新的组件存储
    pub fn new() -> Self {
        Self { columns: Vec::new(), type_to_column: std::collections::HashMap::new() }
    }

    /// 添加组件列
    pub fn add_column<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        if self.type_to_column.contains_key(&type_id) {
            return;
        }
        let index = self.columns.len();
        self.columns.push(ComponentColumn::new::<T>());
        self.type_to_column.insert(type_id, index);
    }

    /// 根据类型 ID 添加组件列（类型擦除版本）
    pub fn add_column_raw(
        &mut self,
        type_id: TypeId,
        type_name: &'static str,
        size: usize,
        align: usize,
        drop_fn: Option<unsafe fn(*mut u8, usize)>,
    ) {
        if self.type_to_column.contains_key(&type_id) {
            return;
        }
        let index = self.columns.len();
        self.columns.push(ComponentColumn {
            type_id,
            type_name,
            size,
            align,
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            drop_fn,
        });
        self.type_to_column.insert(type_id, index);
    }

    /// 根据类型 ID 获取组件列
    pub fn get_column(&self, type_id: TypeId) -> Option<&ComponentColumn> {
        self.type_to_column.get(&type_id).map(|&i| &self.columns[i])
    }

    /// 根据类型 ID 获取组件列可变引用
    pub fn get_column_mut(&mut self, type_id: TypeId) -> Option<&mut ComponentColumn> {
        self.type_to_column.get(&type_id).map(|&i| &mut self.columns[i])
    }

    /// 获取或添加组件列
    pub fn get_or_add_column<T: Component>(&mut self) -> &mut ComponentColumn {
        let type_id = TypeId::of::<T>();
        if !self.type_to_column.contains_key(&type_id) {
            self.add_column::<T>();
        }
        let index = self.type_to_column[&type_id];
        &mut self.columns[index]
    }

    /// 检查是否包含指定类型的组件列
    pub fn contains(&self, type_id: TypeId) -> bool {
        self.type_to_column.contains_key(&type_id)
    }

    /// 获取所有类型 ID
    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.type_to_column.keys().copied()
    }

    /// 遍历所有组件列
    pub fn iter_columns(&self) -> impl Iterator<Item = &ComponentColumn> {
        self.columns.iter()
    }

    /// 遍历所有组件列（可变）
    pub fn iter_columns_mut(&mut self) -> impl Iterator<Item = &mut ComponentColumn> {
        self.columns.iter_mut()
    }
}

impl Default for ComponentStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件列迭代器
///
/// 遍历所有匹配的实体和组件引用，
/// 用于 `Query` 的底层实现。
pub struct ComponentColumnIter<'a, T> {
    /// 预收集的 (Entity, *const T) 条目
    entries: Vec<(crate::entity::Entity, *const T)>,
    /// 当前位置
    pos: usize,
    /// 生命周期标记
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Component> ComponentColumnIter<'a, T> {
    /// 创建新的组件列迭代器
    pub fn new(entries: Vec<(crate::entity::Entity, *const T)>) -> Self {
        Self { entries, pos: 0, _marker: std::marker::PhantomData }
    }
}

impl<'a, T: Component> Iterator for ComponentColumnIter<'a, T> {
    type Item = (crate::entity::Entity, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.entries.len() {
            return None;
        }
        let (entity, ptr) = self.entries[self.pos];
        self.pos += 1;
        unsafe { Some((entity, &*ptr)) }
    }
}
