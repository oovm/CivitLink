//! 资产类型定义模块
//! 定义资产系统的核心类型，包括 GUID、资产类型枚举、资产条目、依赖和引用

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 全局唯一标识符类型
pub type Guid = String;

/// 资产类型枚举，表示引擎支持的所有资源格式
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// 动画资源
    Animation,
    /// 配置资源
    Config,
    /// Galgame 资源
    Galgame,
    /// 材质资源
    Material,
    /// 元数据资源
    Meta,
    /// 预制体资源
    Prefab,
    /// 场景资源
    Scene,
    /// VON 格式资源
    Von,
    /// Schema 资源
    Schema,
    /// 脚本资源
    Script,
    /// 着色器资源
    Shader,
    /// Widget 资源
    Widget,
    /// 未知资源类型
    Unknown(String),
}

impl AssetType {
    /// 从字符串解析资产类型
    pub fn from_str(s: &str) -> Self {
        match s {
            "Animation" => AssetType::Animation,
            "Config" => AssetType::Config,
            "Galgame" => AssetType::Galgame,
            "Material" => AssetType::Material,
            "Meta" => AssetType::Meta,
            "Prefab" => AssetType::Prefab,
            "Scene" => AssetType::Scene,
            "Von" => AssetType::Von,
            "Schema" => AssetType::Schema,
            "Script" => AssetType::Script,
            "Shader" => AssetType::Shader,
            "Widget" => AssetType::Widget,
            other => AssetType::Unknown(other.to_string()),
        }
    }

    /// 获取资产类型的字符串表示
    pub fn as_str(&self) -> &str {
        match self {
            AssetType::Animation => "Animation",
            AssetType::Config => "Config",
            AssetType::Galgame => "Galgame",
            AssetType::Material => "Material",
            AssetType::Meta => "Meta",
            AssetType::Prefab => "Prefab",
            AssetType::Scene => "Scene",
            AssetType::Von => "Von",
            AssetType::Schema => "Schema",
            AssetType::Script => "Script",
            AssetType::Shader => "Shader",
            AssetType::Widget => "Widget",
            AssetType::Unknown(s) => s,
        }
    }

    /// 获取资产类型的编译优先级，数值越小优先级越高
    ///
    /// 优先级排序：Meta(0) > Schema(1) > Shader(2) > Script(3) > Widget(4) > VON 类资产(5) > Unknown(99)
    pub fn compile_priority(&self) -> u32 {
        match self {
            AssetType::Meta => 0,
            AssetType::Schema => 1,
            AssetType::Shader => 2,
            AssetType::Script => 3,
            AssetType::Widget => 4,
            AssetType::Animation => 5,
            AssetType::Config => 5,
            AssetType::Galgame => 5,
            AssetType::Material => 5,
            AssetType::Prefab => 5,
            AssetType::Scene => 5,
            AssetType::Von => 5,
            AssetType::Unknown(_) => 99,
        }
    }

    /// 获取资产类型对应的文件扩展名（不含点号）
    pub fn file_extension(&self) -> &str {
        match self {
            AssetType::Animation => "animation",
            AssetType::Config => "config",
            AssetType::Galgame => "galgame",
            AssetType::Material => "material",
            AssetType::Meta => "meta",
            AssetType::Prefab => "prefab",
            AssetType::Scene => "scene",
            AssetType::Von => "von",
            AssetType::Schema => "schema",
            AssetType::Script => "script",
            AssetType::Shader => "shader",
            AssetType::Widget => "widget",
            AssetType::Unknown(_) => "unknown",
        }
    }
}

/// 资产条目，描述一个已注册资产的完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// 资产的全局唯一标识符
    pub guid: Guid,
    /// 资产的文件路径
    pub path: PathBuf,
    /// 资产类型
    pub asset_type: AssetType,
    /// 资产内容的哈希值
    pub hash: String,
    /// 资产依赖的其他资产的 GUID 列表
    pub dependencies: Vec<Guid>,
    /// 引用此资产的其他资产的 GUID 列表
    pub references: Vec<Guid>,
    /// 编译产物的输出路径
    pub compiled_artifact: Option<PathBuf>,
}

/// 依赖关系，表示从源资产到目标资产的有向边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// 依赖的源资产 GUID
    pub source: Guid,
    /// 依赖的目标资产 GUID
    pub target: Guid,
}

/// 引用关系，表示一个资产对另一个资产的字段级引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// 被引用资产的 GUID
    pub asset: Guid,
    /// 引用的具体字段名称
    pub field: Option<String>,
}
