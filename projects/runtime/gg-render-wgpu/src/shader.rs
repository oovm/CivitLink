//! 着色器源码与加载模块
//!
//! 定义内置着色器源码和 GPU 着色器源抽象，
//! 支持从 naga IR 或 WGSL 文本创建着色器模块。

use std::borrow::Cow;

/// GPU 着色器源
///
/// 封装 naga IR 和 WGSL 两种着色器源，
/// 提供统一的着色器模块创建接口。
pub enum GpuShaderSource {
    /// naga IR 着色器源
    Naga(Cow<'static, naga::Module>),
    /// WGSL 文本着色器源
    Wgsl(Cow<'static, str>),
}

impl GpuShaderSource {
    /// 从 naga Module 创建着色器源
    pub fn from_naga(module: naga::Module) -> Self {
        GpuShaderSource::Naga(Cow::Owned(module))
    }

    /// 从 WGSL 字符串创建着色器源
    pub fn from_wgsl(source: impl Into<Cow<'static, str>>) -> Self {
        GpuShaderSource::Wgsl(source.into())
    }

    /// 创建 wgpu 着色器模块
    pub fn create_shader_module(&self, device: &wgpu::Device, label: &str) -> wgpu::ShaderModule {
        match self {
            GpuShaderSource::Naga(module) => device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Naga(module.clone()),
            }),
            GpuShaderSource::Wgsl(source) => device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.clone()),
            }),
        }
    }
}

/// 精灵着色器 WGSL 源码（保留作为文档参考和回退方案）
///
/// 顶点着色器将单位四边形通过 MVP 矩阵变换到屏幕空间，
/// 并通过 `uv_transform` 对纹理坐标进行偏移和缩放，
/// 片段着色器对纹理进行采样并乘以着色颜色。
#[allow(dead_code)]
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

/// 过渡着色器 WGSL 源码（保留作为文档参考和回退方案）
///
/// 与精灵着色器类似，但支持两个纹理和一个进度参数，
/// 用于场景切换时的过渡动画效果。
#[allow(dead_code)]
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

/// 批渲染精灵着色器 WGSL 源码（保留作为文档参考和回退方案）
///
/// 与精灵着色器功能相同，但通过实例化顶点属性传递 per-instance 数据，
/// 支持一次绘制调用渲染多个同纹理精灵。
#[allow(dead_code)]
pub const SPRITE_BATCH_SHADER: &str = r#"
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

/// 圆角矩形着色器 WGSL 源码（保留作为文档参考和回退方案）
///
/// 使用 SDF（有符号距离场）技术渲染圆角矩形，
/// 顶点着色器将单位四边形通过 MVP 矩阵变换到屏幕空间，
/// 片段着色器计算圆角矩形的 SDF 并进行抗锯齿处理。
#[allow(dead_code)]
pub const ROUNDED_RECT_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) local_pos: vec2f,
}

struct Uniforms {
    mvp: mat4x4f,
    rect_size: vec4f,
    color: vec4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.mvp * vec4f(input.position, 0.0, 1.0);
    output.local_pos = input.uv * uniforms.rect_size.xy;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let half_size = uniforms.rect_size.xy * 0.5;
    let r = uniforms.rect_size.z;
    let center = half_size;
    let d = length(max(abs(input.local_pos - center) - half_size + r, vec2f(0.0))) - r;
    let aa_width = 1.0;
    let alpha = 1.0 - smoothstep(-aa_width, aa_width, d);
    if (alpha <= 0.0) {
        discard;
    }
    return vec4f(uniforms.color.rgb, uniforms.color.a * alpha);
}
"#;

/// 椭圆着色器 WGSL 源码（保留作为文档参考和回退方案）
///
/// 使用 SDF（有符号距离场）技术渲染椭圆和圆形，
/// 支持填充和描边模式，以及抗锯齿边缘处理。
/// 圆形是椭圆的特例（radius_x == radius_y）。
#[allow(dead_code)]
pub const ELLIPSE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) local_pos: vec2f,
}

struct Uniforms {
    mvp: mat4x4f,
    ellipse_params: vec4f,
    fill_color: vec4f,
    border_params: vec4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.mvp * vec4f(input.position, 0.0, 1.0);
    output.local_pos = input.uv * uniforms.ellipse_params.zw;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let center = uniforms.ellipse_params.zw * 0.5;
    let radius_x = uniforms.ellipse_params.z;
    let radius_y = uniforms.ellipse_params.w;
    let border_width = uniforms.border_params.x;
    let border_color = uniforms.border_params.yzw;

    let offset = input.local_pos - center;
    let normalized = offset / vec2f(radius_x, radius_y);
    let d = length(normalized) - 1.0;

    let aa_width = 1.0 / min(radius_x, radius_y);

    if (border_width > 0.0) {
        let inner_d = length(offset / vec2f(max(radius_x - border_width, 0.0), max(radius_y - border_width, 0.0))) - 1.0;
        let outer_alpha = 1.0 - smoothstep(-aa_width, aa_width, d);
        let inner_alpha = smoothstep(-aa_width, aa_width, inner_d);

        let ring_alpha = outer_alpha * inner_alpha;
        let fill_alpha = outer_alpha * (1.0 - inner_alpha);

        let final_alpha = ring_alpha + fill_alpha;
        if (final_alpha <= 0.0) {
            discard;
        }
        let color = border_color * ring_alpha + uniforms.fill_color.rgb * fill_alpha;
        return vec4f(color, final_alpha * max(uniforms.fill_color.a, ring_alpha > 0.0 ? 1.0 : 0.0));
    } else {
        let alpha = 1.0 - smoothstep(-aa_width, aa_width, d);
        if (alpha <= 0.0) {
            discard;
        }
        return vec4f(uniforms.fill_color.rgb, uniforms.fill_color.a * alpha);
    }
}
"#;
