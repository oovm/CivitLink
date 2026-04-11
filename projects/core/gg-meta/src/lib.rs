#![feature(new_range_api)]
#![warn(missing_docs)]

//! GG Meta 库
//!
//! 用于处理游戏资源的元数据文件

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
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
    map.insert("script", "Script");

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
        Self { type_map: default_type_map() }
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
    pub fn generate_for_directory(&self, path: &Path, recursive: bool) -> Result<usize, Box<dyn std::error::Error>> {
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
                let is_meta = entry_path.extension().is_some_and(|ext| ext == "meta");

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

        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();

        let asset_type = self.get_asset_type(&extension);
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let asset_path = path.to_string_lossy().replace('\\', "/");

        let meta = MetaFile::new(asset_type, &asset_path, &name, size);
        meta.to_file(&meta_path)?;

        Ok(true)
    }
}

/// VON 资产类型验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// 字段类型不匹配
    TypeMismatch {
        /// 字段名
        field: String,
        /// 期望类型
        expected: String,
        /// 实际类型
        actual: String,
    },
    /// 必需字段缺失
    MissingRequiredField {
        /// 字段名
        field: String,
    },
    /// GUID 引用无效
    InvalidGuidReference {
        /// 字段名
        field: String,
        /// 无效的 GUID
        guid: String,
    },
    /// 未知资产类型
    UnknownAssetType {
        /// 类型名称
        type_name: String,
    },
}

/// 验证错误集合
#[derive(Debug)]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            match err {
                ValidationError::TypeMismatch { field, expected, actual } => {
                    write!(f, "type mismatch on '{}': expected {}, got {}", field, expected, actual)?;
                }
                ValidationError::MissingRequiredField { field } => {
                    write!(f, "missing required field '{}'", field)?;
                }
                ValidationError::InvalidGuidReference { field, guid } => {
                    write!(f, "invalid guid reference on '{}': '{}'", field, guid)?;
                }
                ValidationError::UnknownAssetType { type_name } => {
                    write!(f, "unknown asset type '{}'", type_name)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// VON 编译器，提供 oak-voc AST 接收接口和完整类型验证
pub struct VonCompiler {
    /// 已知的资产类型集合
    known_asset_types: HashSet<&'static str>,
}

impl VonCompiler {
    /// 创建新的 VON 编译器
    pub fn new() -> Self {
        let known_asset_types =
            HashSet::from(["AnimationFile", "ConfigFile", "MaterialFile", "PrefabFile", "SceneFile", "MetaFile", "VonAsset"]);
        Self { known_asset_types }
    }

    /// 从 VON 字符串编译资产
    pub fn compile(&self, source: &str) -> Result<MetaFile, Box<dyn std::error::Error>> {
        let meta: MetaFile = oak_von::from_str(source)?;
        self.validate(&meta).map_err(|e| Box::new(ValidationErrors(e)) as Box<dyn std::error::Error>)?;
        Ok(meta)
    }

    /// 从 JSON AST 数据编译资产
    ///
    /// 接收 VON AST 的 JSON 表示，提取资产类型和基本字段，
    /// 然后根据资产类型分派到对应的专用编译方法。
    pub fn compile_from_ast(&self, data: serde_json::Value) -> Result<MetaFile, Box<dyn std::error::Error>> {
        let asset_type = data.get("type").or_else(|| data.get("asset_type")).and_then(|v| v.as_str()).unwrap_or("Unknown");

        let result = match asset_type {
            "Animation" | "AnimationFile" => self.compile_animation(&data),
            "Config" | "ConfigFile" => self.compile_config(&data),
            "Material" | "MaterialFile" => self.compile_material(&data),
            "Prefab" | "PrefabFile" => self.compile_prefab(&data),
            "Scene" | "SceneFile" => self.compile_scene(&data),
            _ => Err(ValidationError::UnknownAssetType { type_name: asset_type.to_string() }),
        };

        result.map_err(|e| Box::new(ValidationErrors(vec![e])) as Box<dyn std::error::Error>)
    }

    /// 验证元数据文件的类型完整性
    ///
    /// 检查必需字段、资产类型注册状态、GUID 格式有效性以及依赖引用完整性。
    pub fn validate(&self, meta: &MetaFile) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if meta.asset.path.is_empty() {
            errors.push(ValidationError::MissingRequiredField { field: "asset.path".to_string() });
        }

        if meta.asset.guid.is_empty() {
            errors.push(ValidationError::MissingRequiredField { field: "asset.guid".to_string() });
        }
        else if Uuid::parse_str(&meta.asset.guid).is_err() {
            errors
                .push(ValidationError::InvalidGuidReference { field: "asset.guid".to_string(), guid: meta.asset.guid.clone() });
        }

        if meta.asset.name.is_empty() {
            errors.push(ValidationError::MissingRequiredField { field: "asset.name".to_string() });
        }

        if !self.is_known_asset_type(&meta.asset.r#type) {
            errors.push(ValidationError::UnknownAssetType { type_name: meta.asset.r#type.clone() });
        }

        for dep in &meta.dependencies {
            if dep.guid.is_empty() {
                errors.push(ValidationError::InvalidGuidReference {
                    field: format!("dependency[{}]", dep.path),
                    guid: dep.guid.clone(),
                });
            }
            else if Uuid::parse_str(&dep.guid).is_err() {
                errors.push(ValidationError::InvalidGuidReference {
                    field: format!("dependency[{}]", dep.path),
                    guid: dep.guid.clone(),
                });
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// 注册自定义资产类型
    pub fn register_asset_type(&mut self, type_name: &'static str) {
        self.known_asset_types.insert(type_name);
    }

    /// 检查资产类型是否已知
    pub fn is_known_asset_type(&self, type_name: &str) -> bool {
        self.known_asset_types.contains(type_name)
    }

    /// 验证 Animation 资产结构
    ///
    /// 检查 "animation"、"tracks"、"events" 字段是否存在且类型正确。
    pub fn validate_animation(&self, data: &serde_json::Value) -> ValidationErrors {
        let mut errors = Vec::new();
        self.validate_required_field(data, "animation", "object", &mut errors);
        self.validate_required_field(data, "tracks", "array", &mut errors);
        self.validate_required_field(data, "events", "array", &mut errors);
        ValidationErrors(errors)
    }

    /// 验证 Config 资产结构
    ///
    /// 检查 "config"、"settings" 字段是否存在且类型正确。
    pub fn validate_config(&self, data: &serde_json::Value) -> ValidationErrors {
        let mut errors = Vec::new();
        self.validate_required_field(data, "config", "object", &mut errors);
        self.validate_required_field(data, "settings", "object", &mut errors);
        ValidationErrors(errors)
    }

    /// 验证 Material 资产结构
    ///
    /// 检查 "material"、"properties"、"render_states" 字段是否存在且类型正确。
    pub fn validate_material(&self, data: &serde_json::Value) -> ValidationErrors {
        let mut errors = Vec::new();
        self.validate_required_field(data, "material", "object", &mut errors);
        self.validate_required_field(data, "properties", "object", &mut errors);
        self.validate_required_field(data, "render_states", "object", &mut errors);
        ValidationErrors(errors)
    }

    /// 验证 Prefab 资产结构
    ///
    /// 检查 "prefab"、"entities" 字段是否存在且类型正确。
    pub fn validate_prefab(&self, data: &serde_json::Value) -> ValidationErrors {
        let mut errors = Vec::new();
        self.validate_required_field(data, "prefab", "object", &mut errors);
        self.validate_required_field(data, "entities", "array", &mut errors);
        ValidationErrors(errors)
    }

    /// 验证 Scene 资产结构
    ///
    /// 检查 "scene"、"environment"、"entities" 字段是否存在且类型正确。
    pub fn validate_scene(&self, data: &serde_json::Value) -> ValidationErrors {
        let mut errors = Vec::new();
        self.validate_required_field(data, "scene", "object", &mut errors);
        self.validate_required_field(data, "environment", "object", &mut errors);
        self.validate_required_field(data, "entities", "array", &mut errors);
        ValidationErrors(errors)
    }

    /// 编译 Animation 资产
    ///
    /// 验证 Animation 结构后，从 JSON 数据中提取字段构建 MetaFile。
    pub fn compile_animation(&self, data: &serde_json::Value) -> Result<MetaFile, ValidationError> {
        let errors = self.validate_animation(data);
        if !errors.0.is_empty() {
            return Err(errors.0.into_iter().next().unwrap());
        }
        Ok(self.build_meta_from_json(data, "Animation"))
    }

    /// 编译 Config 资产
    ///
    /// 验证 Config 结构后，从 JSON 数据中提取字段构建 MetaFile。
    pub fn compile_config(&self, data: &serde_json::Value) -> Result<MetaFile, ValidationError> {
        let errors = self.validate_config(data);
        if !errors.0.is_empty() {
            return Err(errors.0.into_iter().next().unwrap());
        }
        Ok(self.build_meta_from_json(data, "Config"))
    }

    /// 编译 Material 资产
    ///
    /// 验证 Material 结构后，从 JSON 数据中提取字段构建 MetaFile。
    pub fn compile_material(&self, data: &serde_json::Value) -> Result<MetaFile, ValidationError> {
        let errors = self.validate_material(data);
        if !errors.0.is_empty() {
            return Err(errors.0.into_iter().next().unwrap());
        }
        Ok(self.build_meta_from_json(data, "Material"))
    }

    /// 编译 Prefab 资产
    ///
    /// 验证 Prefab 结构后，从 JSON 数据中提取字段构建 MetaFile。
    pub fn compile_prefab(&self, data: &serde_json::Value) -> Result<MetaFile, ValidationError> {
        let errors = self.validate_prefab(data);
        if !errors.0.is_empty() {
            return Err(errors.0.into_iter().next().unwrap());
        }
        Ok(self.build_meta_from_json(data, "Prefab"))
    }

    /// 编译 Scene 资产
    ///
    /// 验证 Scene 结构后，从 JSON 数据中提取字段构建 MetaFile。
    pub fn compile_scene(&self, data: &serde_json::Value) -> Result<MetaFile, ValidationError> {
        let errors = self.validate_scene(data);
        if !errors.0.is_empty() {
            return Err(errors.0.into_iter().next().unwrap());
        }
        Ok(self.build_meta_from_json(data, "Scene"))
    }

    fn validate_required_field(
        &self,
        data: &serde_json::Value,
        field: &str,
        expected_type: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        match data.get(field) {
            None => {
                errors.push(ValidationError::MissingRequiredField { field: field.to_string() });
            }
            Some(v) => {
                let type_matches = match expected_type {
                    "object" => v.is_object(),
                    "array" => v.is_array(),
                    "string" => v.is_string(),
                    "number" => v.is_number(),
                    "boolean" => v.is_boolean(),
                    _ => true,
                };
                if !type_matches {
                    errors.push(ValidationError::TypeMismatch {
                        field: field.to_string(),
                        expected: expected_type.to_string(),
                        actual: Self::json_type_name(v),
                    });
                }
            }
        }
    }

    fn json_type_name(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(_) => "boolean".to_string(),
            serde_json::Value::Number(_) => "number".to_string(),
            serde_json::Value::String(_) => "string".to_string(),
            serde_json::Value::Array(_) => "array".to_string(),
            serde_json::Value::Object(_) => "object".to_string(),
        }
    }

    fn build_meta_from_json(&self, data: &serde_json::Value, default_type: &str) -> MetaFile {
        let asset_type = data.get("type").or_else(|| data.get("asset_type")).and_then(|v| v.as_str()).unwrap_or(default_type);
        let path = data.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let guid = data.get("guid").and_then(|v| v.as_str()).unwrap_or("");
        let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let size = data.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

        let mut meta = MetaFile::new(asset_type, path, name, size);

        if !guid.is_empty() {
            meta.asset.guid = guid.to_string();
        }

        if let Some(deps) = data.get("dependencies").and_then(|v| v.as_array()) {
            for dep in deps {
                let dep_path = dep.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let dep_guid = dep.get("guid").and_then(|v| v.as_str()).unwrap_or("");
                if !dep_path.is_empty() && !dep_guid.is_empty() {
                    meta.add_dependency(dep_path, dep_guid);
                }
            }
        }

        meta
    }
}

impl Default for VonCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 预导入模块
pub mod prelude {
    /// 资源信息
    pub use crate::Asset;
    /// 依赖关系
    pub use crate::Dependency;
    /// 导入设置
    pub use crate::ImportSettings;
    /// 元数据文件结构
    pub use crate::MetaFile;
    /// 元数据生成器
    pub use crate::MetaGenerator;
    /// 引用关系
    pub use crate::Reference;
    /// 类型验证错误
    pub use crate::ValidationError;
    /// 验证错误集合
    pub use crate::ValidationErrors;
    /// VON 编译器
    pub use crate::VonCompiler;
    /// 生成新的GUID
    pub use crate::generate_guid;
}
