//! Cranelift AOT 编译后端模块
//! 使用 Cranelift 将字节码模块编译为目标平台原生代码

use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use gg_bytecode::format::BytecodeModule;
use gg_core::{GError, GErrorKind, GResult};

use crate::{backend::AotBackend, error::AotError, target::TargetPlatform};

/// Cranelift AOT 编译后端
/// 使用 Cranelift JIT 编译器将字节码模块编译为目标平台原生代码
pub struct CraneliftBackend {
    /// 支持的目标平台列表
    supported: Vec<TargetPlatform>,
}

impl CraneliftBackend {
    /// 创建新的 Cranelift 编译后端
    pub fn new() -> Self {
        Self {
            supported: vec![
                TargetPlatform::WindowsX64,
                TargetPlatform::LinuxX64,
                TargetPlatform::MacOSArm64,
                TargetPlatform::WebWasm32,
            ],
        }
    }
}

impl Default for CraneliftBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AotBackend for CraneliftBackend {
    fn name(&self) -> &str {
        "cranelift"
    }

    fn supported_targets(&self) -> Vec<TargetPlatform> {
        self.supported.clone()
    }

    fn compile(&self, module: &BytecodeModule, target: &TargetPlatform) -> GResult<Vec<u8>> {
        if !self.supported.contains(target) {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: AotError::UnsupportedTarget { target: *target }.to_string(),
            });
        }

        let triple_str = match target {
            TargetPlatform::WindowsX64 => "x86_64-pc-windows-msvc",
            TargetPlatform::LinuxX64 => "x86_64-unknown-linux-gnu",
            TargetPlatform::MacOSArm64 => "aarch64-apple-darwin",
            TargetPlatform::WebWasm32 => {
                return Err(GError {
                    kind: GErrorKind::Runtime,
                    message: AotError::CompilationFailed {
                        target: *target,
                        message: "WASM target is not yet supported by Cranelift backend".to_string(),
                    }
                    .to_string(),
                });
            }
            _ => {
                return Err(GError {
                    kind: GErrorKind::Runtime,
                    message: AotError::UnsupportedTarget { target: *target }.to_string(),
                });
            }
        };

        let triple = triple_str.parse::<target_lexicon::Triple>().map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("Failed to parse target triple '{}': {}", triple_str, e),
        })?;

        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to set cranelift flag: {}", e) })?;
        flag_builder
            .set("is_pic", "true")
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to set cranelift flag: {}", e) })?;

        let isa_builder = cranelift_codegen::isa::lookup(triple).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("Failed to lookup ISA for target '{}': {}", target.triple(), e),
        })?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to create ISA: {}", e) })?;

        let builder = ObjectModule::new(
            ObjectBuilder::new(isa, module.name.as_str(), cranelift_module::default_libcall_names())
                .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to create ObjectBuilder: {}", e) })?,
        );

        let mut cranelift_module: ObjectModule = builder;

        for function in &module.functions {
            compile_function(&mut cranelift_module, function, module)?;
        }

        let product = cranelift_module.finish();
        let obj_bytes = product
            .emit()
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to emit object file: {}", e) })?;

        Ok(obj_bytes)
    }
}

/// 编译单个字节码函数为 Cranelift IR
fn compile_function(
    cranelift_module: &mut ObjectModule,
    function: &gg_bytecode::format::BytecodeFunction,
    _module: &BytecodeModule,
) -> GResult<()> {
    let mut ctx = cranelift_module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let ptr_type = cranelift_module.target_config().pointer_type();

    for _ in 0..function.param_count {
        ctx.func.signature.params.push(AbiParam::new(ptr_type));
    }
    ctx.func.signature.returns.push(AbiParam::new(ptr_type));

    let func_id = cranelift_module.declare_function(&function.name, Linkage::Export, &ctx.func.signature).map_err(|e| {
        GError { kind: GErrorKind::Runtime, message: format!("Failed to declare function '{}': {}", function.name, e) }
    })?;

    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let return_val = builder.ins().iconst(ptr_type, 0);
        builder.ins().return_(&[return_val]);

        builder.finalize();
    }

    cranelift_module.define_function(func_id, &mut ctx).map_err(|e| GError {
        kind: GErrorKind::Runtime,
        message: format!("Failed to define function '{}': {}", function.name, e),
    })?;

    cranelift_module.clear_context(&mut ctx);

    Ok(())
}
