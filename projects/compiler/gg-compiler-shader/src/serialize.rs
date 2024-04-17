//! naga IR 序列化模块
//!
//! 提供 naga Module 的序列化和反序列化功能，
//! 使用 SPIR-V 二进制格式进行序列化，确保跨平台兼容性。

use gg_core::{GError, GErrorKind, GResult};
use naga;

/// 将 naga Module 序列化为 SPIR-V 二进制数据
///
/// SPIR-V 是标准的着色器中间格式，跨平台兼容性好。
pub fn serialize_module(module: &naga::Module) -> GResult<Vec<u8>> {
    let module_info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(module)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("naga Module 验证失败: {:?}", e) })?;

    let spirv = naga::back::spv::write_vec(module, &module_info, &naga::back::spv::Options::default(), None)
        .map_err(|e| GError { kind: GErrorKind::Other, message: format!("SPIR-V 序列化失败: {:?}", e) })?;

    let bytes: Vec<u8> = spirv.iter().flat_map(|word| word.to_le_bytes()).collect();

    Ok(bytes)
}

/// 从 SPIR-V 二进制数据反序列化 naga Module
///
/// 支持从序列化的 SPIR-V 二进制重建 naga Module。
pub fn deserialize_module(bytes: &[u8]) -> GResult<naga::Module> {
    if bytes.len() % 4 != 0 {
        return Err(GError { kind: GErrorKind::Other, message: "SPIR-V 数据长度必须是 4 的倍数".to_string() });
    }

    let words: Vec<u32> =
        bytes.chunks_exact(4).map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect();

    let module = naga::front::spv::parse_u8_slice(
        &words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(),
        &naga::front::spv::Options::default(),
    )
    .map_err(|e| GError { kind: GErrorKind::Other, message: format!("SPIR-V 反序列化失败: {:?}", e) })?;

    Ok(module)
}
