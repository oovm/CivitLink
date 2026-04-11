#![feature(new_range_api)]
#![warn(missing_docs)]

//! GG Meta 库
//!
//! 用于处理游戏资源的元数据文件

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    #[serde(rename = "type")]
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
        let timestamp = chrono::Utc::now();
        let guid = Uuid::now_v7().to_string();
        let timestamp_str = timestamp.to_rfc3339();

        Self {
            version: "1.0".to_string(),
            asset: Asset {
                r#type: asset_type.to_string(),
                path: asset_path.to_string(),
                guid,
                name: asset_name.to_string(),
                size,
                modified: timestamp_str.clone(),
            },
            import_settings: None,
            dependencies: Vec::new(),
            references: Vec::new(),
            timestamp: timestamp_str,
            hash: None,
        }
    }

    /// 从文件读取元数据
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let meta: Self = oak_von::from_str(&content)?;
        Ok(meta)
    }

    /// 写入元数据到文件
    pub fn to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = oak_von::to_string(self)?;
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
    Uuid::now_v7().to_string()
}

/// 元数据生成器
///
/// 自动为项目目录中的资源文件生成 .meta 文件。
/// 支持递归目录扫描、按扩展名自动检测资源类型和增量生成。
pub struct MetaGenerator {
    /// 扩展名到资源类型的映射表
    type_map: HashMap<&'static str, &'static str>,
}

/// 获取默认的扩展名到资源类型映射表
fn default_type_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    map.insert("png", "Texture");
    map.insert("jpg", "Texture");
    map.insert("jpeg", "Texture");
    map.insert("bmp", "Texture");
    map.insert("webp", "Texture");

    map.insert("wav", "Audio");
    map.insert("mp3", "Audio");
    map.insert("ogg", "Audio");
    map.insert("flac", "Audio");

    map.insert("ttf", "Font");
    map.insert("otf", "Font");
    map.insert("woff", "Font");
    map.insert("woff2", "Font");

    map.insert("v", "Script");
    map.insert("vx", "Script");
    map.insert("gscript", "Script");

    map.insert("glsl", "Shader");
    map.insert("vert", "Shader");
    map.insert("frag", "Shader");
    map.insert("gs", "Shader");

    map.insert("scene", "Scene");

    map.insert("prefab", "Prefab");

    map.insert("toml", "Config");
    map.insert("json", "Config");
    map.insert("yaml", "Config");
    map.insert("yml", "Config");

    map.insert("anim", "Animation");

    map.insert("spine", "Spine");
    map.insert("atlas", "Spine");

    map.insert("xlsx", "Sheet");
    map.insert("csv", "Sheet");
    map.insert("tsv", "Sheet");

    map
}

impl MetaGenerator {
    /// 创建新的元数据生成器，使用默认扩展名映射
    pub fn new() -> Self {
        Self {
            type_map: default_type_map(),
        }
    }

    /// 注册自定义扩展名映射
    ///
    /// # 参数
    ///
    /// - `extension` - 文件扩展名（不含点号），如 "png"
    /// - `asset_type` - 资源类型名称，如 "Texture"
    pub fn register_type(&mut self, extension: &'static str, asset_type: &'static str) {
        self.type_map.insert(extension, asset_type);
    }

    /// 根据文件扩展名获取资源类型
    ///
    /// 如果扩展名未在映射表中注册，返回 "Unknown"。
    pub fn get_asset_type(&self, extension: &str) -> &'static str {
        self.type_map.get(extension).copied().unwrap_or("Unknown")
    }

    /// 为指定目录生成 .meta 文件
    ///
    /// # 参数
    ///
    /// - `path` - 目标目录路径
    /// - `recursive` - 是否递归处理子目录
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的 .meta 文件数量
    pub fn generate_for_directory(
        &self,
        path: &Path,
        recursive: bool,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0usize;

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                if recursive {
                    count += self.generate_for_directory(&entry_path, true)?;
                }
            }
            else if entry_path.is_file() {
                let is_meta = entry_path
                    .extension()
                    .is_some_and(|ext| ext == "meta");

                if !is_meta {
                    if self.generate_for_file(&entry_path)? {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// 为指定文件生成 .meta 文件
    ///
    /// 如果该文件已有对应的 .meta 文件，则跳过（增量生成）。
    ///
    /// # 参数
    ///
    /// - `path` - 目标文件路径
    ///
    /// # 返回值
    ///
    /// 成功时返回 true（已生成），false（已存在，跳过）
    pub fn generate_for_file(&self, path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        let meta_path = {
            let original_ext = path.extension().map(|e| e.to_string_lossy().to_string());
            match original_ext {
                Some(ext) => path.with_extension(format!("{}.meta", ext)),
                None => path.with_extension("meta"),
            }
        };

        if meta_path.exists() {
            return Ok(false);
        }

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let asset_type = self.get_asset_type(&extension);
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let asset_path = path.to_string_lossy().replace('\\', "/");

        let meta = MetaFile::new(asset_type, &asset_path, &name, size);
        meta.to_file(&meta_path)?;

        Ok(true)
    }
}

/// 预导入模块
pub mod prelude {
    /// 元数据生成器
    pub use crate::MetaGenerator;
    /// 元数据文件结构
    pub use crate::MetaFile;
    /// 资源信息
    pub use crate::Asset;
    /// 导入设置
    pub use crate::ImportSettings;
    /// 依赖关系
    pub use crate::Dependency;
    /// 引用关系
    pub use crate::Reference;
    /// 生成新的GUID
    pub use crate::generate_guid;
}


