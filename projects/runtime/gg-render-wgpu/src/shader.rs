/// 精灵着色器 WGSL 源码
///
/// 顶点着色器将单位四边形通过 MVP 矩阵变换到屏幕空间，
/// 片段着色器对纹理进行采样并乘以着色颜色。
pub const SPRITE_SHADER: &str = r#"
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
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var tex: texture_2d<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.mvp * vec4f(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let tex_color = textureSample(tex, tex_sampler, input.uv);
    return tex_color * uniforms.tint;
}
"#;

/// 过渡着色器 WGSL 源码
///
/// 与精灵着色器类似，但支持两个纹理和一个进度参数，
/// 用于场景切换时的过渡动画效果。
pub const TRANSITION_SHADER: &str = r#"
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
