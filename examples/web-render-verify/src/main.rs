//! Web 平台渲染验证示例
//!
//! 验证 Web 平台的渲染后端检测能力。
//! 在非 wasm32 目标上输出检测结果，在 wasm32 目标上可进一步验证渲染管线。

use gg_platform_web::{RenderBackendType, detect_best_backend, is_webgl2_available, is_webgpu_available};

fn main() {
    println!("=== GG Engine Web Platform Render Verification ===");
    println!();

    let webgpu_available = is_webgpu_available();
    let webgl2_available = is_webgl2_available();

    println!("WebGPU available: {}", webgpu_available);
    println!("WebGL2 available: {}", webgl2_available);

    match detect_best_backend() {
        Some(RenderBackendType::WebGPU) => {
            println!("Best backend: WebGPU");
        }
        Some(RenderBackendType::WebGL) => {
            println!("Best backend: WebGL (fallback)");
        }
        None => {
            println!("Best backend: None (no rendering support)");
        }
    }

    println!();
    println!("Verification complete.");
}
