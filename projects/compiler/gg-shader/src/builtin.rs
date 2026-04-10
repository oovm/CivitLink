//! 内置着色器 naga IR 构建模块
//!
//! 提供内置着色器（精灵、过渡、批渲染精灵）的 naga IR 构建函数，
//! 通过 naga 的 WGSL 前端解析预定义的 WGSL 源码生成 naga Module。

use gg_core::{GError, GErrorKind, GResult};
use naga;

/// 精灵着色器 WGSL 源码
pub const SPRITE_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
}

struct Uniforms {
    mvp: mat4x4f,
    tint: vec4f,
    uv_transform: vec4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var tex: texture_2d<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.mvp * vec4f(input.position, 0.0, 1.0);
    output.uv = vec2f(
        uniforms.uv_transform.x + input.uv.x * uniforms.uv_transform.z,
        uniforms.uv_transform.y + input.uv.y * uniforms.uv_transform.w
    );
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let tex_color = textureSample(tex, tex_sampler, input.uv);
    return tex_color * uniforms.tint;
}
"#;

/// 过渡着色器 WGSL 源码
pub const TRANSITION_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
}

struct Uniforms {
    mvp: mat4x4f,
    params: vec4f,
    tint: vec4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var old_tex: texture_2d<f32>;
@group(1) @binding(2) var new_tex: texture_2d<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.mvp * vec4f(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let old_color = textureSample(old_tex, tex_sampler, input.uv);
    let new_color = textureSample(new_tex, tex_sampler, input.uv);
    let progress = uniforms.params.x;
    let kind = uniforms.params.y;

    var result: vec4f;
    if (kind < 0.5) {
        result = mix(old_color, new_color, progress);
    } else if (kind < 1.5) {
        result = mix(old_color, new_color, progress);
    } else if (kind < 2.5) {
        let offset = vec2f(progress, 0.0);
        let uv_old = input.uv + offset;
        let uv_new = input.uv - vec2f(1.0 - progress, 0.0);
        let c_old = textureSample(old_tex, tex_sampler, uv_old);
        let c_new = textureSample(new_tex, tex_sampler, uv_new);
        result = mix(c_old, c_new, step(1.0 - progress, input.uv.x));
    } else if (kind < 3.5) {
        let offset = vec2f(-progress, 0.0);
        let uv_old = input.uv + offset;
        let uv_new = input.uv + vec2f(1.0 - progress, 0.0);
        let c_old = textureSample(old_tex, tex_sampler, uv_old);
        let c_new = textureSample(new_tex, tex_sampler, uv_new);
        result = mix(c_old, c_new, step(progress, input.uv.x));
    } else if (kind < 4.5) {
        let offset = vec2f(0.0, -progress);
        let uv_old = input.uv + offset;
        let uv_new = input.uv + vec2f(0.0, 1.0 - progress);
        let c_old = textureSample(old_tex, tex_sampler, uv_old);
        let c_new = textureSample(new_tex, tex_sampler, uv_new);
        result = mix(c_old, c_new, step(progress, input.uv.y));
    } else {
        let offset = vec2f(0.0, progress);
        let uv_old = input.uv + offset;
        let uv_new = input.uv - vec2f(0.0, 1.0 - progress);
        let c_old = textureSample(old_tex, tex_sampler, uv_old);
        let c_new = textureSample(new_tex, tex_sampler, uv_new);
        result = mix(c_old, c_new, step(1.0 - progress, input.uv.y));
    }

    return result * uniforms.tint;
}
"#;

/// 批渲染精灵着色器 WGSL 源码
pub const BATCH_SPRITE_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) instance_mvp_row0: vec4f,
    @location(3) instance_mvp_row1: vec4f,
    @location(4) instance_mvp_row2: vec4f,
    @location(5) instance_mvp_row3: vec4f,
    @location(6) instance_tint: vec4f,
    @location(7) instance_uv_transform: vec4f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) tint: vec4f,
}

@group(0) @binding(0) var tex_sampler: sampler;
@group(0) @binding(1) var tex: texture_2d<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mvp = mat4x4f(
        input.instance_mvp_row0,
        input.instance_mvp_row1,
        input.instance_mvp_row2,
        input.instance_mvp_row3
    );
    output.clip_position = mvp * vec4f(input.position, 0.0, 1.0);
    output.uv = vec2f(
        input.instance_uv_transform.x + input.uv.x * input.instance_uv_transform.z,
        input.instance_uv_transform.y + input.uv.y * input.instance_uv_transform.w
    );
    output.tint = input.instance_tint;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let tex_color = textureSample(tex, tex_sampler, input.uv);
    return tex_color * input.tint;
}
"#;

/// 从 WGSL 源码构建 naga Module
fn parse_wgsl(source: &str, label: &str) -> GResult<naga::Module> {
    naga::front::wgsl::parse_str(source).map_err(|e| GError {
        kind: GErrorKind::Other,
        message: format!("内置着色器 '{}' WGSL 解析失败: {:?}", label, e),
    })
}

/// 构建精灵着色器的 naga Module
pub fn builtin_sprite_shader() -> GResult<naga::Module> {
    parse_wgsl(SPRITE_SHADER_WGSL, "sprite")
}

/// 构建过渡着色器的 naga Module
pub fn builtin_transition_shader() -> GResult<naga::Module> {
    parse_wgsl(TRANSITION_SHADER_WGSL, "transition")
}

/// 构建批渲染精灵着色器的 naga Module
pub fn builtin_batch_sprite_shader() -> GResult<naga::Module> {
    parse_wgsl(BATCH_SPRITE_SHADER_WGSL, "batch_sprite")
}
