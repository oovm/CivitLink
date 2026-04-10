#![feature(new_range_api)]
#![warn(missing_docs)]

use oak_core::{
    source::{SourceBuffer, ToSource},
};
use oak_von::{
    VonValue,
    parse,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value};
use std::path::Path;
use uuid::Uuid;

/// 元数据文件结构
#[derive(Debug, Serialize, Deserialize)]
pub struct MetaFile {
    /// 版本信息
    pub version: String,
    /// 资源信息
    pub asset: Asset,
    /// 导入设置
    pub import_settings: Option<ImportSettings>,
    /// 依赖关系
    pub dependencies: Vec<Dependency>,
    /// 引用关系
    pub references: Vec<Reference>,
    /// 时间戳
    pub timestamp: String,
    /// 哈希值（可选，用于检测文件变更）
    pub hash: Option<String>,
}

/// 资源信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    /// 资源类型
    pub r#type: String,
    /// 资源路径
    pub path: String,
    /// 资源GUID
    pub guid: String,
    /// 资源名称
    pub name: String,
    /// 资源大小
    pub size: u64,
    /// 资源修改时间
    pub modified: String,
}

/// 导入设置
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSettings {
    /// 导入选项
    pub options: serde_json::Value,
}

/// 依赖关系
#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    /// 依赖路径
    pub path: String,
    /// 依赖GUID
    pub guid: String,
}

/// 引用关系
#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    /// 引用路径
    pub path: String,
    /// 引用字段
    pub field: Option<String>,
}

impl MetaFile {
    /// 创建新的元数据文件
    pub fn new(asset_type: &str, asset_path: &str, asset_name: &str, size: u64) -> Self {
        let guid = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        Self {
            version: "1.0".to_string(),
            asset: Asset {
                r#type: asset_type.to_string(),
                path: asset_path.to_string(),
                guid,
                name: asset_name.to_string(),
                size,
                modified: timestamp.clone(),
            },
            import_settings: None,
            dependencies: Vec::new(),
            references: Vec::new(),
            timestamp,
            hash: None,
        }
    }

    /// 从文件读取元数据
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let von_value = parse(&content)?;
        let meta: Self = serde::from_value(serde_json::to_value(von_value)?)?;
        Ok(meta)
    }

    /// 写入元数据到文件
    pub fn to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json_value = serde_json::to_value(self)?;
        let von_value: VonValue = serde::from_value(json_value)?;
        let mut buffer = SourceBuffer::new();
        von_value.to_source(&mut buffer);
        let content = buffer.to_string();
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, path: &str, guid: &str) {
        self.dependencies.push(Dependency { path: path.to_string(), guid: guid.to_string() });
    }

    /// 添加引用
    pub fn add_reference(&mut self, path: &str, field: Option<&str>) {
        self.references.push(Reference { path: path.to_string(), field: field.map(|s| s.to_string()) });
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.timestamp = chrono::Utc::now().to_rfc3339();
        self.asset.modified = self.timestamp.clone();
    }

    /// 更新哈希值
    pub fn update_hash(&mut self, hash: &str) {
        self.hash = Some(hash.to_string());
    }
}

/// 生成新的GUID
pub fn generate_guid() -> String {
    Uuid::new_v4().to_string()
}


