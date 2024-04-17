//! Galgame 编译器模块
//! 整合解析和代码生成阶段，提供完整的编译流程

use std::path::Path;

use crate::{
    codegen::GalgameCodegen,
    error::GalgameResult,
    ir::{DialogueDB, GalgameIr, StorySequence},
    parser::GalgameParser,
};

/// Galgame 编译器
pub struct GalgameCompiler {
    /// 是否禁用优化
    optimize: bool,
}

impl GalgameCompiler {
    /// 创建新的 Galgame 编译器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 禁用优化并返回自身
    pub fn no_optimize(mut self) -> Self {
        self.optimize = false;
        self
    }

    /// 编译 .galgame 源码为字节码
    pub fn compile(&self, source: &str, module_name: &str) -> GalgameResult<Vec<u8>> {
        let ir = self.parse(source)?;

        let _ = module_name;

        let bytecode = GalgameCodegen::generate(&ir)?;

        Ok(bytecode)
    }

    /// 解析源码为 GalgameIr
    pub fn parse(&self, source: &str) -> GalgameResult<GalgameIr> {
        let mut parser = GalgameParser::new();
        parser.parse(source)
    }

    /// 编译单个文件为 StorySequence
    pub fn compile_file(path: &Path) -> GalgameResult<StorySequence> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::GalgameError::ParseError(format!("Failed to read file '{}': {}", path.display(), e)))?;
        let compiler = Self::new();
        let ir = compiler.parse(&content)?;
        Ok(StorySequence { nodes: ir.dialogues })
    }

    /// 编译目录下所有 .galgame 文件为 DialogueDB
    pub fn compile_directory(dir: &Path) -> GalgameResult<DialogueDB> {
        let mut db = DialogueDB::new();
        let entries = std::fs::read_dir(dir).map_err(|e| {
            crate::error::GalgameError::ParseError(format!("Failed to read directory '{}': {}", dir.display(), e))
        })?;

        for entry in entries {
            let entry =
                entry.map_err(|e| crate::error::GalgameError::ParseError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("galgame") {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
                let sequence = Self::compile_file(&path)?;
                for node in &sequence.nodes {
                    db.all_node_ids.insert(node.id.clone());
                }
                db.sequences.insert(file_name, sequence);
            }
        }

        Ok(db)
    }
}

impl Default for GalgameCompiler {
    fn default() -> Self {
        Self::new()
    }
}
