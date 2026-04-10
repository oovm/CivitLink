//! `gg generate` 命令实现
//!
//! 读取 Engine.toml，解析并验证清单，调用工厂生成引擎代码。

use crate::GResult;
use gg_factory::EngineFactory;
use gg_manifest::EngineManifest;
use std::path::PathBuf;

/// 执行 `generate` 子命令
///
/// 读取 Engine.toml，解析并验证清单，调用工厂生成引擎代码。
pub fn cmd_generate(manifest_path: &str) -> GResult<()> {
    let path = PathBuf::from(manifest_path).join("Engine.toml");

    let manifest = EngineManifest::load_from_file(&path)?;
    manifest.validate()?;

    let output_dir = PathBuf::from(manifest_path).join("generated");
    let files = EngineFactory::generate(&manifest, &output_dir)?;

    println!("Generated engine project for '{}'", manifest.engine.name);
    println!("  Plugins: {}", manifest.modules.plugins.join(", "));
    println!("  Generated files:");
    for file in &files {
        println!("    {}", file.display());
    }
    Ok(())
}
