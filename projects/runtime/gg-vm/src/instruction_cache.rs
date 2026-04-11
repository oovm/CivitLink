//! VM 指令缓存模块
//! 缓存已解码的指令序列，避免重复解码

use std::collections::HashMap;

/// 函数缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionKey {
    /// 模块名称
    pub module_name: String,
    /// 函数名称
    pub function_name: String,
}

/// 缓存的指令
#[derive(Debug, Clone)]
pub struct CachedInstruction {
    /// 操作码
    pub opcode: u8,
    /// 操作数
    pub operands: Vec<u64>,
    /// 字节码偏移
    pub byte_offset: usize,
}

/// 指令缓存
pub struct InstructionCache {
    /// 缓存条目
    cache: HashMap<FunctionKey, Vec<CachedInstruction>>,
    /// 缓存命中次数
    hits: u64,
    /// 缓存未命中次数
    misses: u64,
}

impl InstructionCache {
    /// 创建新的指令缓存
    pub fn new() -> Self {
        Self { cache: HashMap::new(), hits: 0, misses: 0 }
    }

    /// 获取缓存的指令或解码并缓存
    pub fn get_or_decode<F>(&mut self, key: FunctionKey, decoder: F) -> &[CachedInstruction]
    where
        F: FnOnce() -> Vec<CachedInstruction>,
    {
        if self.cache.contains_key(&key) {
            self.hits += 1;
        }
        else {
            self.misses += 1;
            let instructions = decoder();
            self.cache.insert(key.clone(), instructions);
        }
        self.cache.get(&key).unwrap()
    }

    /// 检查缓存中是否存在指定函数
    pub fn contains(&self, key: &FunctionKey) -> bool {
        self.cache.contains_key(key)
    }

    /// 按函数键清除缓存条目
    pub fn invalidate(&mut self, key: &FunctionKey) -> bool {
        self.cache.remove(key).is_some()
    }

    /// 按模块名清除所有相关缓存条目
    pub fn invalidate_module(&mut self, module_name: &str) -> usize {
        let keys_to_remove: Vec<FunctionKey> = self.cache.keys().filter(|k| k.module_name == module_name).cloned().collect();
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.cache.remove(&key);
        }
        count
    }

    /// 清除所有缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// 获取缓存命中次数
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// 获取缓存未命中次数
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    /// 获取缓存条目数
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for InstructionCache {
    fn default() -> Self {
        Self::new()
    }
}
