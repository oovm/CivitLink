# GG Shader Compiler

GG Shader Compiler is a shader compilation tool for the GG Game Engine, providing shader parsing, compilation, and optimization capabilities.

## Features
- Shader parsing and compilation
- Cross-platform shader support
- Shader optimization
- Integration with GG Render

## Usage
```rust
use gg_shader::compile_shader;

let shader_source = r#"
    #version 450
    void main() {
        // Shader code
    }
"#;

let compiled_shader = compile_shader(shader_source, "vertex");
```

## Requirements
- Rust 1.60+
- SPIR-V compiler
