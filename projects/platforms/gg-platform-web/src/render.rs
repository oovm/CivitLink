//! WebGPU 渲染后端适配模块
//!
//! 为 Web 平台提供 WebGPU 渲染后端的可用性检测和回退逻辑。
//! 实际的渲染器创建由 `gg-render-wgpu` 模块负责。

/// 渲染后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendType {
    /// WebGPU 后端
    WebGPU,
    /// WebGL 后端（回退）
    WebGL,
}

/// 检测浏览器是否支持 WebGPU
///
/// 通过检查 `navigator.gpu` 是否存在来判断 WebGPU 支持。
#[cfg(target_arch = "wasm32")]
pub fn is_webgpu_available() -> bool {
    js_sys::eval("typeof navigator !== 'undefined' && navigator.gpu !== undefined")
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// 检测浏览器是否支持 WebGPU
///
/// 非 wasm32 目标始终返回 `false`。
#[cfg(not(target_arch = "wasm32"))]
pub fn is_webgpu_available() -> bool {
    false
}

/// 检测浏览器是否支持 WebGL2
///
/// 通过尝试创建 WebGL2 上下文来判断 WebGL2 支持。
#[cfg(target_arch = "wasm32")]
pub fn is_webgl2_available() -> bool {
    js_sys::eval(
        "(() => { try { return !!document.createElement('canvas').getContext('webgl2'); } catch(e) { return false; } })()"
    )
    .ok()
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
}

/// 检测浏览器是否支持 WebGL2
///
/// 非 wasm32 目标始终返回 `false`。
#[cfg(not(target_arch = "wasm32"))]
pub fn is_webgl2_available() -> bool {
    false
}

/// 检测最佳可用渲染后端
///
/// 优先返回 WebGPU，若不可用则回退到 WebGL2。
/// 若两者均不可用，返回 `None`。
pub fn detect_best_backend() -> Option<RenderBackendType> {
    if is_webgpu_available() {
        Some(RenderBackendType::WebGPU)
    } else if is_webgl2_available() {
        Some(RenderBackendType::WebGL)
    } else {
        None
    }
}
