//! naga IR 序列化模块
//!
//! 提供 naga Module 的序列化和反序列化功能，
//! 用于构建时编译产物存储和运行时加载。

use gg_core::{GError, GErrorKind, GResult};
use naga;

/// 将 naga Module 序列化为二进制数据
///
/// 使用 WGSL 文本格式进行序列化，确保跨平台兼容性。
pub fn serialize_module(module: &naga::Module) -> GResult<Vec<u8>> {
    let wgsl = naga::back::wgsl::write_string(
        module,
        &naga::back::wgsl::Options::default(),
        naga::back::wgsl::WriterFlags::empty(),
    )
    .map_err(|e| GError {
        kind: GErrorKind::Other,
        message: format!("naga Module 序列化失败: {:?}", e),
    })?;
    Ok(wgsl.into_bytes())
}

/// 从二进制数据反序列化 naga Module
///
/// 支持从序列化的 WGSL 文本重建 naga Module。
pub fn deserialize_module(bytes: &[u8]) -> GResult<naga::Module> {
    let source = std::str::from_utf8(bytes).map_err(|e| GError {
        kind: GErrorKind::Other,
        message: format!("反序列化失败: 数据不是有效的 UTF-8 文本: {}", e),
    })?;

    let module = naga::front::wgsl::parse_str(source).map_err(|e| GError {
        kind: GErrorKind::Other,
        message: format!("WGSL 反序列化失败: {:?}", e),
    })?;

    Ok(module)
}
