//! GG-Sheet 项目配置模块
//! 提供从 GGSheet.toml 加载和保存项目配置的功能

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{SheetError, SheetResult};

/// GG-Sheet 项目配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetConfig {
    /// 配置表目录，默认 "asset/sheet"
    #[serde(default = "default_sheet_dir")]
    pub sheet_dir: String,
    /// 输出目录，默认 "asset/script/table"
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    /// 全局命名空间前缀
    #[serde(default)]
    pub namespace: Option<String>,
    /// 表格级别配置，key 为表名
    #[serde(default)]
    pub tables: HashMap<String, TableConfig>,
}

/// 表格级别配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// 表格级别命名空间
    #[serde(default)]
    pub namespace: Option<String>,
    /// 是否跳过该表，默认 false
    #[serde(default)]
    pub skip: bool,
}

fn default_sheet_dir() -> String {
    "asset/sheet".to_string()
}

fn default_output_dir() -> String {
    "asset/script/table".to_string()
}

impl Default for SheetConfig {
    fn default() -> Self {
        Self { sheet_dir: default_sheet_dir(), output_dir: default_output_dir(), namespace: None, tables: HashMap::new() }
    }
}

impl SheetConfig {
    /// 从 TOML 文件加载配置
    pub fn load(path: &Path) -> SheetResult<SheetConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法读取配置文件: {}", e) })?;

        toml::from_str(&content).map_err(|e| SheetError::Config { message: format!("配置文件解析失败: {}", e) })
    }

    /// 生成默认配置文件内容
    pub fn default_toml() -> String {
        let config = SheetConfig::default();
        toml::to_string_pretty(&config).unwrap_or_default()
    }

    /// 保存配置到文件
    pub fn save(&self, path: &Path) -> SheetResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| SheetError::Config { message: format!("配置序列化失败: {}", e) })?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SheetError::Io {
                    path: parent.to_path_buf(), message: format!("无法创建配置目录: {}", e)
                })?;
        }

        std::fs::write(path, &content)
            .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法写入配置文件: {}", e) })
    }
}
