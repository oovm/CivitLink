#![warn(missing_docs)]

//! 构建配置管理
//!
//! 提供构建模式与工具链配置的映射关系，以及构建参数的默认值。

use gg_core::platform::BuildProfile;

/// 获取构建模式的默认优化参数
///
/// 返回传递给 Cargo 的优化级别参数列表。
pub fn get_optimization_args(profile: BuildProfile) -> &'static [&'static str] {
    match profile {
        BuildProfile::Debug => &[],
        BuildProfile::Release => &["--release"],
        BuildProfile::Profile => &["--profile", "release"],
    }
}

/// 获取构建模式对应的 Cargo profile 名称
pub fn get_cargo_profile_name(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Debug => "dev",
        BuildProfile::Release => "release",
        BuildProfile::Profile => "release",
    }
}

/// 获取构建模式是否启用调试断言
pub fn is_debug_assertions_enabled(profile: BuildProfile) -> bool {
    matches!(profile, BuildProfile::Debug)
}

/// 获取构建模式是否启用 LTO（链接时优化）
pub fn is_lto_enabled(profile: BuildProfile) -> bool {
    matches!(profile, BuildProfile::Release)
}

/// 获取构建模式的调试信息级别
///
/// 返回调试信息的级别描述：`"full"` 表示完整调试信息，`"line-tables-only"` 表示仅行号表，`"none"` 表示无调试信息。
pub fn get_debug_info_level(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Debug => "full",
        BuildProfile::Release => "none",
        BuildProfile::Profile => "line-tables-only",
    }
}
