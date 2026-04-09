#![warn(missing_docs)]

//! GG 引擎脚本编译模块
//! 使用 oak-valkyrie 前端将 Valkyrie 源码编译为字节码模块

pub mod compiler;

use std::path::Path;

use gg_bytecode::format::BytecodeModule;
use gg_bytecode::reader::BytecodeReader;
use gg_bytecode::writer::BytecodeWriter;
use gg_core::{GError, GErrorKind, GResult};
use gg_ir::IrModule;
use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

use crate::compiler::ValkyrieCompiler;

/// 脚本编译器，将 Valkyrie 源码编译为字节码模块
///
/// 编译管线：源码 → ValkyrieBuilder → ValkyrieRoot AST → ValkyrieCompiler → IrModule → 优化 → BytecodeModule
pub struct ScriptCompiler {
    /// 是否启用 IR 优化
    pub optimize: bool,
}

impl ScriptCompiler {
    /// 创建新的脚本编译器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 创建不启用优化的脚本编译器
    pub fn no_optimize() -> Self {
        Self { optimize: false }
    }

    /// 编译 Valkyrie 脚本源码为字节码模块
    ///
    /// 通过 ValkyrieBuilder 将源码解析为 AST，再通过 ValkyrieCompiler 编译为 IR，
    /// 可选地执行 IR 优化，最后序列化为 BytecodeModule。
    pub fn compile(&self, source: &str, module_name: &str) -> GResult<BytecodeModule> {
        let mut ir_module = self.compile_to_ir(source, module_name)?;

        if self.optimize {
            let optimizer = gg_ir::default_optimizer();
            optimizer.optimize(&mut ir_module)?;
        }

        let bytecode_data = BytecodeWriter::write(&ir_module)?;
        BytecodeReader::read(&bytecode_data)
    }

    /// 编译 Valkyrie 脚本源码为 IR 模块（跳过优化和字节码序列化）
    pub fn compile_to_ir(&self, source: &str, module_name: &str) -> GResult<IrModule> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let source_text = SourceText::new(source);
        let mut cache = oak_core::parser::ParseSession::<ValkyrieLanguage>::default();
        let diagnostics = builder.build(&source_text, &[], &mut cache);

        match diagnostics.result {
            Ok(root) => {
                let compiler = ValkyrieCompiler::new(module_name);
                compiler.compile(&root, module_name)
            }
            Err(e) => Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("Valkyrie parse error: {}", e),
            }),
        }
    }
}

impl Default for ScriptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 脚本加载器，从文件系统加载脚本并编译为字节码模块
pub struct ScriptLoader {
    /// 脚本编译器
    compiler: ScriptCompiler,
}

impl ScriptLoader {
    /// 创建新的脚本加载器
    pub fn new() -> Self {
        Self {
            compiler: ScriptCompiler::new(),
        }
    }

    /// 从文件加载脚本并编译为字节码模块
    pub fn load_file(&self, path: &Path) -> GResult<BytecodeModule> {
        let source = std::fs::read_to_string(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read script file: {}", e),
        })?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.compiler.compile(&source, module_name)
    }

    /// 从字符串加载脚本并编译为字节码模块
    pub fn load_string(&self, source: &str, module_name: &str) -> GResult<BytecodeModule> {
        self.compiler.compile(source, module_name)
    }

    /// 从文件加载脚本并编译为 IR 模块
    pub fn load_file_as_ir(&self, path: &Path) -> GResult<IrModule> {
        let source = std::fs::read_to_string(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read script file: {}", e),
        })?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.compiler.compile_to_ir(&source, module_name)
    }
}

impl Default for ScriptLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_compiler_basic() {
        let source = r#"
            micro init() {
                let entity1 = spawn_entity()
                add_component(entity1, "Position")
                set_field(entity1, "Position", "x", 0.0)
                set_field(entity1, "Position", "y", 0.0)
                print("Hello from script!")
            }
        "#;

        let compiler = ScriptCompiler::new();
        let module = compiler.compile(source, "test").unwrap();

        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "init");
        assert_eq!(module.functions[0].param_count, 0);
    }

    #[test]
    fn test_script_compiler_two_micros() {
        let source = r#"
            micro init() {
                print("init")
            }

            micro update() {
                print("update")
            }
        "#;

        let compiler = ScriptCompiler::new();
        let module = compiler.compile(source, "test").unwrap();

        assert_eq!(module.functions.len(), 2);
        assert_eq!(module.functions[0].name, "init");
        assert_eq!(module.functions[1].name, "update");
    }

    #[test]
    fn test_script_compiler_let_and_arithmetic() {
        let source = r#"
            micro calc() {
                let x = 10
                let y = x + 5
                let z = x * y
            }
        "#;

        let compiler = ScriptCompiler::new();
        let module = compiler.compile(source, "test").unwrap();

        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "calc");
        assert!(!module.functions[0].instructions.is_empty());
    }

    #[test]
    fn test_script_compiler_if_else() {
        let source = r#"
            micro check() {
                if (true) {
                    print("yes")
                } else {
                    print("no")
                }
            }
        "#;

        let compiler = ScriptCompiler::new();
        let module = compiler.compile(source, "test").unwrap();

        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_script_compiler_micro_with_params() {
        let source = r#"
            micro greet(name) {
                print(name)
            }
        "#;

        let compiler = ScriptCompiler::new();
        let module = compiler.compile(source, "test").unwrap();

        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].param_count, 1);
    }

    #[test]
    fn test_script_loader_load_string() {
        let loader = ScriptLoader::new();
        let source = r#"
            micro main() {
                print("hello")
            }
        "#;

        let module = loader.load_string(source, "test").unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "main");
    }

    #[test]
    fn test_script_compiler_no_optimize() {
        let source = r#"
            micro main() {
                print("hello")
            }
        "#;

        let compiler = ScriptCompiler::no_optimize();
        let module = compiler.compile(source, "test").unwrap();

        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "main");
    }

    #[test]
    fn test_script_compiler_compile_to_ir() {
        let source = r#"
            micro main() {
                print("hello")
            }
        "#;

        let compiler = ScriptCompiler::new();
        let ir_module = compiler.compile_to_ir(source, "test").unwrap();

        assert_eq!(ir_module.functions.len(), 1);
        assert_eq!(ir_module.functions[0].name, "main");
        assert!(!ir_module.functions[0].instructions.is_empty());
    }
}
