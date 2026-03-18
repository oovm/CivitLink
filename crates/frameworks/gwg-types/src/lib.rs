//! GWG Engine 基础类型系统
//!
//! 提供实体 ID、组件 trait、资源 trait 等核心抽象。

use std::any::TypeId;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// 实体 ID，用于唯一标识游戏世界中的实体
///
/// 由索引和代数组成，索引用于快速定位，代数用于检测实体是否已被销毁并重用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
    /// 实体索引
    index: u32,
    /// 实体代数，用于检测实体是否已被销毁
    generation: u32,
}

impl Entity {
    /// 创建一个新的实体 ID
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

    /// 创建一个占位符实体（索引为 0，代数为 0）
    pub const PLACEHOLDER: Entity = Entity::new(0, 0);
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::PLACEHOLDER
    }
}

/// 实体 ID 别名
pub type EntityId = Entity;

/// 组件 trait，所有组件都必须实现此 trait
///
/// 组件是存储在实体上的数据片段，用于描述实体的属性和行为。
pub trait Component: Send + Sync + 'static {}

/// 自动为所有满足条件的类型实现 Component
impl<T: Send + Sync + 'static> Component for T {}

/// 资源 trait，全局单例数据
///
/// 资源是存储在世界中的全局数据，不与任何实体关联。
pub trait Resource: Send + Sync + 'static {}

/// 自动为所有满足条件的类型实现 Resource
impl<T: Send + Sync + 'static> Resource for T {}

/// 组件类型信息
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentInfo {
    /// 组件类型 ID
    type_id: TypeId,
    /// 组件类型名称
    type_name: &'static str,
    /// 组件大小（字节）
    size: usize,
    /// 组件对齐要求
    align: usize,
}

impl ComponentInfo {
    /// 创建新的组件信息
    pub fn new<T: Component>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
        }
    }

    /// 获取类型 ID
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// 获取组件大小
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取组件对齐要求
    pub fn align(&self) -> usize {
        self.align
    }
}

/// 资源类型信息
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceInfo {
    /// 资源类型 ID
    type_id: TypeId,
    /// 资源类型名称
    type_name: &'static str,
}

impl ResourceInfo {
    /// 创建新的资源信息
    pub fn new<T: Resource>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }

    /// 获取类型 ID
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
}

/// ECS 操作错误
#[derive(thiserror::Error, Debug)]
pub enum EcsError {
    /// 实体不存在
    #[error("Entity not found: {0:?}")]
    EntityNotFound(Entity),

    /// 组件不存在
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// 资源不存在
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// 类型不匹配
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },

    /// 实体已销毁
    #[error("Entity has been destroyed: {0:?}")]
    EntityDestroyed(Entity),

    /// 无效操作
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// 变更检测标记
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChangeTick {
    tick: u32,
}

impl ChangeTick {
    /// 创建新的变更标记
    pub const fn new(tick: u32) -> Self {
        Self { tick }
    }

    /// 获取标记值
    pub const fn tick(&self) -> u32 {
        self.tick
    }
}

/// 变更检测器
#[derive(Clone, Copy, Debug, Default)]
pub struct ChangeDetector {
    added: ChangeTick,
    changed: ChangeTick,
}

impl ChangeDetector {
    /// 创建新的变更检测器
    pub const fn new(added: ChangeTick, changed: ChangeTick) -> Self {
        Self { added, changed }
    }

    /// 获取添加时间
    pub const fn added(&self) -> ChangeTick {
        self.added
    }

    /// 获取最后修改时间
    pub const fn changed(&self) -> ChangeTick {
        self.changed
    }

    /// 设置添加时间
    pub fn set_added(&mut self, tick: ChangeTick) {
        self.added = tick;
    }

    /// 设置修改时间
    pub fn set_changed(&mut self, tick: ChangeTick) {
        self.changed = tick;
    }
}

/// 系统标签 trait，用于标识和排序系统
pub trait SystemLabel: Clone + Debug + Eq + Hash + Send + Sync + 'static {}

impl<T: Clone + Debug + Eq + Hash + Send + Sync + 'static> SystemLabel for T {}

/// 调度标签 trait，用于标识调度阶段
pub trait ScheduleLabel: Clone + Debug + Eq + Hash + Send + Sync + 'static {}

impl<T: Clone + Debug + Eq + Hash + Send + Sync + 'static> ScheduleLabel for T {}

/// 系统集合 trait，用于组织相关系统
pub trait SystemSet: Clone + Debug + Eq + Hash + Send + Sync + 'static {}

impl<T: Clone + Debug + Eq + Hash + Send + Sync + 'static> SystemSet for T {}

pub mod prelude {
    //! 基础类型的预导入模块

    pub use super::{
        ChangeDetector, ChangeTick, Component, ComponentInfo, EcsError, Entity, EntityId,
        Resource, ResourceInfo, ScheduleLabel, SystemLabel, SystemSet,
    };
}
