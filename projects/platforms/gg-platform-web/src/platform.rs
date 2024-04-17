#![warn(missing_docs)]

use std::path::PathBuf;

use gg_core::{
    GResult,
    platform::{BuildConfig, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};

/// Web 平台构建时实现
///
/// 为 WebAssembly/Web 环境提供构建、生成和打包的 Platform trait 实现。
pub struct WebPlatform;

impl Platform for WebPlatform {
    fn id(&self) -> PlatformId {
        "web".to_string()
    }

    fn display_name(&self) -> &str {
        "Web (WASM)"
    }

    fn configure_build(&self, config: &mut BuildConfig) {
        config.target_triple = "wasm32-unknown-unknown".to_string();
    }

    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf> {
        Ok(ctx.output_dir.join("web_main.rs"))
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        let wasm_file = ctx.build_output_dir.join("game.wasm");
        let js_file = ctx.build_output_dir.join("game.js");
        let html_file = ctx.build_output_dir.join("index.html");
        Ok(vec![wasm_file, js_file, html_file])
    }
}
