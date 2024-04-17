#![doc = include_str!("readme.md")]

/// 渲染后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendType {
    /// WebGPU 后端
    WebGPU,
    /// WebGL 后端（回退）
    WebGL,
}

/// 渲染后端降级策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderFallbackStrategy {
    /// 优先 WebGPU，不可用时回退到 WebGL2
    WebGpuFirst,
    /// 仅使用 WebGL2
    WebGlOnly,
    /// 仅使用 WebGPU，不回退
    Strict,
}

/// 渲染能力检测结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderCapability {
    /// 检测到的最佳后端
    pub backend: RenderBackendType,
    /// 可用的回退后端
    pub fallback: Option<RenderBackendType>,
    /// 是否有可用的渲染后端
    pub is_available: bool,
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
        "(() => { try { return !!document.createElement('canvas').getContext('webgl2'); } catch(e) { return false; } })()",
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
    }
    else if is_webgl2_available() {
        Some(RenderBackendType::WebGL)
    }
    else {
        None
    }
}

/// 根据降级策略检测渲染能力
///
/// 根据指定的降级策略检测可用的渲染后端，返回包含最佳后端、
/// 回退后端以及可用性信息的 `RenderCapability`。
pub fn detect_with_fallback(strategy: RenderFallbackStrategy) -> RenderCapability {
    match strategy {
        RenderFallbackStrategy::WebGpuFirst => {
            let webgpu = is_webgpu_available();
            let webgl2 = is_webgl2_available();
            if webgpu {
                RenderCapability {
                    backend: RenderBackendType::WebGPU,
                    fallback: if webgl2 { Some(RenderBackendType::WebGL) } else { None },
                    is_available: true,
                }
            }
            else if webgl2 {
                RenderCapability { backend: RenderBackendType::WebGL, fallback: None, is_available: true }
            }
            else {
                RenderCapability { backend: RenderBackendType::WebGL, fallback: None, is_available: false }
            }
        }
        RenderFallbackStrategy::WebGlOnly => {
            let webgl2 = is_webgl2_available();
            if webgl2 {
                RenderCapability { backend: RenderBackendType::WebGL, fallback: None, is_available: true }
            }
            else {
                RenderCapability { backend: RenderBackendType::WebGL, fallback: None, is_available: false }
            }
        }
        RenderFallbackStrategy::Strict => {
            let webgpu = is_webgpu_available();
            if webgpu {
                RenderCapability { backend: RenderBackendType::WebGPU, fallback: None, is_available: true }
            }
            else {
                RenderCapability { backend: RenderBackendType::WebGPU, fallback: None, is_available: false }
            }
        }
    }
}
