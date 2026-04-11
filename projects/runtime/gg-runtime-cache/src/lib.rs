#![warn(missing_docs)]

//! GG 引擎缓存模块
//! 提供缓存驱动抽象与内存缓存实现

use gg_core::GResult;
use std::time::Duration;

/// 缓存值类型
///
/// 表示缓存中存储的值，支持多种基本数据类型。
#[derive(Debug, Clone, PartialEq)]
pub enum CacheValue {
    /// 空值
    Null,
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Real(f64),
    /// 文本字符串值
    Text(String),
    /// 二进制数据值
    Blob(Vec<u8>),
    /// 布尔值
    Bool(bool),
}

/// 缓存驱动 trait
///
/// 定义缓存操作的标准接口，所有缓存驱动都需要实现此 trait。
pub trait CacheDriver {
    /// 获取缓存值
    ///
    /// 根据键名获取对应的缓存值，如果键不存在或已过期则返回 None。
    fn get(&mut self, key: &str) -> GResult<Option<CacheValue>>;

    /// 设置缓存值
    ///
    /// 将键值对存入缓存，可指定可选的生存时间（TTL）。
    fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()>;

    /// 删除缓存值
    ///
    /// 根据键名删除对应的缓存条目，返回是否成功删除。
    fn delete(&mut self, key: &str) -> GResult<bool>;

    /// 检查缓存键是否存在
    ///
    /// 检查指定键名是否存在且未过期。
    fn exists(&mut self, key: &str) -> GResult<bool>;

    /// 清空所有缓存
    ///
    /// 移除缓存中的所有条目。
    fn clear(&mut self) -> GResult<()>;
}

/// 内存缓存驱动实现
pub mod memory;

pub use memory::MemoryCacheDriver;
