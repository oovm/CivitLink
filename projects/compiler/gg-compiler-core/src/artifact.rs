//! 编译产物模块
//! 定义编译产物的键、产物本身及产物集合类型

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

/// 编译产物键，用于在 ArtifactSet 中唯一标识一个产物
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ArtifactKey {
    /// 产物类型名称
    pub type_name: String,
    /// 产物标识符（如文件名、模块名）
    pub id: String,
}

impl ArtifactKey {
    /// 创建一个新的产物键
    pub fn new(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self { type_name: type_name.into(), id: id.into() }
    }
}

/// 编译产物，包装任意类型的编译结果
pub struct Artifact {
    /// 产物的类型标识
    pub key: ArtifactKey,
    /// 产物的内容哈希，用于增量编译判断
    pub content_hash: u64,
    /// 产物的二进制数据
    pub data: Vec<u8>,
}

impl Artifact {
    /// 创建一个新的编译产物
    pub fn new(key: ArtifactKey, data: Vec<u8>) -> Self {
        let content_hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            data.hash(&mut hasher);
            hasher.finish()
        };
        Self { key, content_hash, data }
    }
}

/// 编译产物集合，按 ArtifactKey 索引
pub struct ArtifactSet {
    /// 内部存储映射
    artifacts: HashMap<ArtifactKey, Artifact>,
}

impl ArtifactSet {
    /// 创建一个空的产物集合
    pub fn new() -> Self {
        Self { artifacts: HashMap::new() }
    }

    /// 插入一个产物，如果已存在则替换并返回旧产物
    pub fn insert(&mut self, artifact: Artifact) -> Option<Artifact> {
        self.artifacts.insert(artifact.key.clone(), artifact)
    }

    /// 根据键获取产物引用
    pub fn get(&self, key: &ArtifactKey) -> Option<&Artifact> {
        self.artifacts.get(key)
    }

    /// 根据键移除并返回产物
    pub fn remove(&mut self, key: &ArtifactKey) -> Option<Artifact> {
        self.artifacts.remove(key)
    }

    /// 检查是否包含指定键的产物
    pub fn contains(&self, key: &ArtifactKey) -> bool {
        self.artifacts.contains_key(key)
    }

    /// 获取所有产物键的迭代器
    pub fn keys(&self) -> impl Iterator<Item = &ArtifactKey> {
        self.artifacts.keys()
    }

    /// 检查产物集合是否为空
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    /// 获取产物数量
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    /// 将另一个产物集合并入此集合，已有键的产物会被覆盖
    pub fn merge(&mut self, other: ArtifactSet) {
        for (key, artifact) in other.artifacts {
            self.artifacts.insert(key, artifact);
        }
    }

    /// 计算所有产物的组合哈希，用于增量编译判断
    pub fn content_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let mut keys: Vec<&ArtifactKey> = self.artifacts.keys().collect();
        keys.sort_by(|a, b| a.type_name.cmp(&b.type_name).then(a.id.cmp(&b.id)));
        for key in keys {
            key.hash(&mut hasher);
            self.artifacts[key].content_hash.hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl Default for ArtifactSet {
    fn default() -> Self {
        Self::new()
    }
}
