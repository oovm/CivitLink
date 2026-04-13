//! 动态库插件加载器
//!
//! 提供运行时加载 .so/.dll 动态库插件的功能，
//! 通过 C-ABI 接口与动态库交互，支持插件清单解析和 ABI 版本校验。

use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};

/// 编辑器插件 ABI 版本
///
/// 动态库插件必须声明与此版本兼容的 ABI 版本才能被加载。
pub const EDITOR_PLUGIN_ABI_VERSION: &str = "0.1.0";

/// C-ABI 插件入口函数类型
///
/// 动态库必须导出名为 `gg_editor_plugin_entry` 的函数，
/// 签名为 `unsafe extern "C" fn() -> *mut c_void`。
pub type PluginEntryFn = unsafe extern "C" fn() -> *mut std::ffi::c_void;

/// 插件入口函数导出符号名
const PLUGIN_ENTRY_SYMBOL: &str = "gg_editor_plugin_entry";

/// 插件清单
///
/// 从 `plugin.json` 文件解析而来，包含插件的元数据信息。
#[derive(Clone, Debug, serde::Deserialize)]
pub struct PluginManifest {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: Option<String>,
    /// 插件依赖列表
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// 动态库文件名
    pub entry_library: String,
    /// 插件 ABI 版本
    pub abi_version: String,
}

impl PluginManifest {
    /// 从 JSON 文件解析插件清单
    pub fn from_file(path: &Path) -> GResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GError {
                kind: GErrorKind::Io, message: format!("无法读取插件清单文件 {:?}: {}", path, e)
            })?;
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| GError { kind: GErrorKind::Plugin, message: format!("插件清单格式错误 {:?}: {}", path, e) })?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// 验证插件清单的必填字段和 ABI 版本兼容性
    pub fn validate(&self) -> GResult<()> {
        if self.name.is_empty() {
            return Err(GError { kind: GErrorKind::Plugin, message: "插件清单缺少 name 字段".to_string() });
        }
        if self.version.is_empty() {
            return Err(GError { kind: GErrorKind::Plugin, message: "插件清单缺少 version 字段".to_string() });
        }
        if self.entry_library.is_empty() {
            return Err(GError { kind: GErrorKind::Plugin, message: "插件清单缺少 entry_library 字段".to_string() });
        }
        if self.abi_version != EDITOR_PLUGIN_ABI_VERSION {
            return Err(GError {
                kind: GErrorKind::Plugin,
                message: format!(
                    "插件 '{}' 的 ABI 版本 '{}' 与编辑器期望版本 '{}' 不兼容",
                    self.name, self.abi_version, EDITOR_PLUGIN_ABI_VERSION
                ),
            });
        }
        Ok(())
    }
}

/// 动态库插件加载器
///
/// 负责在运行时加载 .so（Linux/macOS）/ .dll（Windows）动态库，
/// 通过 C-ABI 入口函数获取插件实例。
#[cfg(not(target_arch = "wasm32"))]
pub struct DynamicPluginLoader;

#[cfg(not(target_arch = "wasm32"))]
impl DynamicPluginLoader {
    /// 从指定路径加载动态库插件
    ///
    /// 加载动态库并调用其导出的 `gg_editor_plugin_entry` 入口函数，
    /// 返回动态库句柄和插件实例指针。
    ///
    /// # Safety
    ///
    /// 此方法涉及不安全的动态库加载和符号解析，
    /// 调用者需确保动态库来源可信且与编辑器 ABI 兼容。
    pub unsafe fn load(path: &Path) -> GResult<(libloading::Library, *mut std::ffi::c_void)> {
        let library = libloading::Library::new(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("无法加载动态库 {:?}: {}", path, e) })?;

        let symbol: Result<libloading::Symbol<PluginEntryFn>, _> = library.get(PLUGIN_ENTRY_SYMBOL.as_bytes());
        let entry_fn = symbol.map_err(|e| GError {
            kind: GErrorKind::Plugin,
            message: format!("动态库 {:?} 缺少入口函数 '{}': {}", path, PLUGIN_ENTRY_SYMBOL, e),
        })?;

        let plugin_ptr = entry_fn();
        if plugin_ptr.is_null() {
            return Err(GError {
                kind: GErrorKind::Plugin, message: format!("动态库 {:?} 的入口函数返回了空指针", path)
            });
        }

        Ok((library, plugin_ptr))
    }

    /// 释放动态库句柄
    ///
    /// # Safety
    ///
    /// 调用此方法后，从该动态库获取的所有符号均不可再使用。
    pub unsafe fn unload(library: libloading::Library) -> GResult<()> {
        library.close().map_err(|e| GError { kind: GErrorKind::Plugin, message: format!("释放动态库失败: {}", e) })
    }
}
