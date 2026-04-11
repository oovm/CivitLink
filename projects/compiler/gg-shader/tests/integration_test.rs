//! gg-shader 集成测试
//!
//! 测试 gs → naga 的完整转换流程

use gg_shader::compiler::ShaderCompiler;
use gg_shader::lower::GslLowerer;

#[test]
fn test_lowerer_new() {
    let lowerer = GslLowerer::new();
    let _ = lowerer;
}

#[test]
fn test_lowerer_default() {
    let lowerer = GslLowerer::default();
    let _ = lowerer;
}

#[test]
fn test_compile_no_shader_block() {
    let source = r#"
let x = 42
"#;
    let compiler = ShaderCompiler::new();
    let result = compiler.compile(source);
    assert!(result.is_err(), "应该编译失败：没有 shader 块");
}

#[test]
fn test_compile_minimal_shader() {
    let source = r#"
shader MinimalShader by Unlit {
    @vertex
    micro vs_main() -> f32 {
        return 1.0
    }
}
"#;
    let compiler = ShaderCompiler::no_optimize();
    let result = compiler.compile(source);
    match &result {
        Ok(module) => {
            assert_eq!(module.entry_points.len(), 1);
            assert_eq!(module.entry_points[0].stage, naga::ShaderStage::Vertex);
        }
        Err(e) => panic!("编译失败: {:?}", e),
    }
}

#[test]
fn test_compile_vertex_and_fragment() {
    let source = r#"
shader SimpleShader by Unlit {
    @vertex
    micro vs_main() -> f32 {
        return 1.0
    }

    @fragment
    micro fs_main() -> f32 {
        return 1.0
    }
}
"#;
    let compiler = ShaderCompiler::no_optimize();
    let result = compiler.compile(source);
    match &result {
        Ok(module) => {
            assert_eq!(module.entry_points.len(), 2);
        }
        Err(e) => panic!("编译失败: {:?}", e),
    }
}

#[test]
fn test_compile_with_uniforms() {
    let source = r#"
shader UniformShader by PBR {
    structure Uniforms {
        mvp: mat44
        tint: vec4
    }

    @vertex
    micro vs_main() -> f32 {
        return 1.0
    }
}
"#;
    let compiler = ShaderCompiler::no_optimize();
    let result = compiler.compile(source);
    match &result {
        Ok(module) => {
            assert!(module.types.iter().count() > 0, "应该有类型定义");
        }
        Err(e) => panic!("编译失败: {:?}", e),
    }
}

#[test]
fn test_compile_with_texture_property() {
    let source = r#"
shader TexturedShader by Unlit {
    let albedo: texture = "white"

    @vertex
    micro vs_main() -> f32 {
        return 1.0
    }
}
"#;
    let compiler = ShaderCompiler::no_optimize();
    let result = compiler.compile(source);
    match &result {
        Ok(module) => {
            let gv_count = module.global_variables.iter().count();
            assert!(gv_count >= 2, "应该有 sampler 和 texture 全局变量");
        }
        Err(e) => panic!("编译失败: {:?}", e),
    }
}

#[test]
fn test_compile_compute_shader() {
    let source = r#"
shader ComputeShader by Compute {
    @compute
    micro cs_main() -> f32 {
        return 1.0
    }
}
"#;
    let compiler = ShaderCompiler::no_optimize();
    let result = compiler.compile(source);
    match &result {
        Ok(module) => {
            assert_eq!(module.entry_points.len(), 1);
            assert_eq!(module.entry_points[0].stage, naga::ShaderStage::Compute);
        }
        Err(e) => panic!("编译失败: {:?}", e),
    }
}

#[test]
fn test_compile_with_params() {
    let source = r#"
shader ParamShader by Unlit {
    @vertex
    micro vs_main(position: vec2) -> vec4f {
        return vec4(1.0, 1.0, 1.0, 1.0)
    }
}
"#;
    let compiler = ShaderCompiler::no_optimize();
    let result = compiler.compile(source);
    match &result {
        Ok(module) => {
            assert_eq!(module.entry_points[0].function.arguments.len(), 1);
        }
        Err(e) => panic!("编译失败: {:?}", e),
    }
}
