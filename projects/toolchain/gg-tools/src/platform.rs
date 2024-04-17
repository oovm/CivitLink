#![warn(missing_docs)]

//! 平台映射模块
//!
//! 提供平台名称到 Cargo 目标三元组的映射，以及平台工具链检查功能。

use gg_manifest::EngineManifest;
use once_cell::sync::Lazy;

/// 平台类别
///
/// 表示平台的大类划分，用于区分桌面、Web 和移动平台。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// 桌面平台（Windows、macOS、Linux）
    Desktop,
    /// Web 平台（WASM）
    Web,
    /// 移动平台（Android、iOS）
    Mobile,
}

impl Platform {
    /// 根据 Cargo 目标三元组推断平台类别
    ///
    /// 包含 `wasm32` 的目标归为 Web，包含 `android` 或 `ios` 的归为 Mobile，其余归为 Desktop。
    pub fn from_target(target: &str) -> Self {
        if target.contains("wasm32") {
            Platform::Web
        }
        else if target.contains("android") || target.contains("ios") {
            Platform::Mobile
        }
        else {
            Platform::Desktop
        }
    }
}

/// 平台目标信息
///
/// 包含平台的显示名称、Cargo 目标三元组和平台特定的 Cargo features。
#[derive(Debug, Clone)]
pub struct PlatformTarget {
    /// 平台显示名称（如 "Windows"）
    pub name: String,
    /// Cargo 目标三元组（如 "x86_64-pc-windows-msvc"）
    pub target: String,
    /// 平台特定的 Cargo features
    pub features: Vec<String>,
}

/// 平台名称到目标信息的映射表
pub static PLATFORM_MAP: Lazy<Vec<(&'static str, PlatformTarget)>> = Lazy::new(|| {
    vec![
        (
            "windows",
            PlatformTarget {
                name: "Windows".to_string(),
                target: "x86_64-pc-windows-msvc".to_string(),
                features: vec!["desktop".to_string()],
            },
        ),
        (
            "windows-gnu",
            PlatformTarget {
                name: "Windows (GNU)".to_string(),
                target: "x86_64-pc-windows-gnu".to_string(),
                features: vec!["desktop".to_string()],
            },
        ),
        (
            "macos",
            PlatformTarget {
                name: "macOS".to_string(),
                target: "aarch64-apple-darwin".to_string(),
                features: vec!["desktop".to_string()],
            },
        ),
        (
            "macos-x86",
            PlatformTarget {
                name: "macOS (x86)".to_string(),
                target: "x86_64-apple-darwin".to_string(),
                features: vec!["desktop".to_string()],
            },
        ),
        (
            "linux",
            PlatformTarget {
                name: "Linux".to_string(),
                target: "x86_64-unknown-linux-gnu".to_string(),
                features: vec!["desktop".to_string()],
            },
        ),
        (
            "web",
            PlatformTarget {
                name: "Web".to_string(),
                target: "wasm32-unknown-unknown".to_string(),
                features: vec!["web".to_string()],
            },
        ),
        (
            "android",
            PlatformTarget {
                name: "Android".to_string(),
                target: "aarch64-linux-android".to_string(),
                features: vec!["mobile".to_string()],
            },
        ),
        (
            "android-x86",
            PlatformTarget {
                name: "Android (x86)".to_string(),
                target: "x86_64-linux-android".to_string(),
                features: vec!["mobile".to_string()],
            },
        ),
        (
            "ios",
            PlatformTarget {
                name: "iOS".to_string(),
                target: "aarch64-apple-ios".to_string(),
                features: vec!["mobile".to_string()],
            },
        ),
        (
            "ios-sim",
            PlatformTarget {
                name: "iOS Simulator".to_string(),
                target: "aarch64-apple-ios-sim".to_string(),
                features: vec!["mobile".to_string()],
            },
        ),
    ]
});

/// 根据平台名称查找映射的目标信息
///
/// 在 `PLATFORM_MAP` 中查找与给定名称匹配的条目，返回对应的 `PlatformTarget` 引用。
pub fn resolve_platform(name: &str) -> Option<&PlatformTarget> {
    (&*PLATFORM_MAP).iter().find(|(key, _)| *key == name).map(|(_, target)| target)
}

/// 从引擎清单的 `platforms` 字段解析平台列表
///
/// 将清单中的每个 `PlatformEntry` 转换为 `PlatformTarget`。
pub fn resolve_from_manifest(manifest: &EngineManifest) -> Vec<PlatformTarget> {
    manifest
        .platforms
        .iter()
        .map(|entry| PlatformTarget {
            name: entry.name.clone(),
            target: entry.target.clone(),
            features: entry.features.clone(),
        })
        .collect()
}

/// 检查目标平台工具链是否已安装
///
/// 运行 `rustup target list --installed` 并检查输出中是否包含指定的目标三元组。
pub fn check_target_installed(target: &str) -> bool {
    let output = match std::process::Command::new("rustup").args(["target", "list", "--installed"]).output() {
        Ok(output) => output,
        Err(_) => return false,
    };

    let installed_list = String::from_utf8_lossy(&output.stdout);
    installed_list.lines().any(|line| line.trim() == target)
}
