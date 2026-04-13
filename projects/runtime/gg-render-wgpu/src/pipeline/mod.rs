use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

use crate::sprite_batch::BatchedSpriteInstance;

/// 顶点格式
///
/// 包含二维位置和纹理坐标，用于四边形绘制。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    /// 位置 `[x, y]`
    pub position: [f32; 2],
    /// 纹理坐标 `[u, v]`
    pub uv: [f32; 2],
}

/// 精灵着色器 uniform 数据
///
/// 包含 MVP 变换矩阵、着色颜色和 UV 变换参数。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SpriteUniforms {
    /// MVP 变换矩阵
    pub mvp: [[f32; 4]; 4],
    /// 着色颜色 RGBA
    pub tint: [f32; 4],
    /// UV 偏移和缩放 `[u_offset, v_offset, u_scale, v_scale]`
    pub uv_transform: [f32; 4],
}

/// 过渡着色器 uniform 数据
///
/// 包含 MVP 变换矩阵、过渡参数和着色颜色。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TransitionUniforms {
    /// MVP 变换矩阵
    pub mvp: [[f32; 4]; 4],
    /// 过渡参数 `[progress, kind, 0.0, 0.0]`
    pub params: [f32; 4],
    /// 着色颜色 RGBA
    pub tint: [f32; 4],
}

/// 单位四边形的顶点数据
const QUAD_VERTICES: [Vertex; 4] = [
    Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
    Vertex { position: [1.0, 0.0], uv: [1.0, 0.0] },
    Vertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
    Vertex { position: [0.0, 1.0], uv: [0.0, 1.0] },
];

/// 单位四边形的索引数据
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

/// 精灵渲染管线
///
/// 封装了精灵绘制所需的 wgpu 渲染管线、绑定组布局和顶点/索引缓冲区。
pub struct SpritePipeline {
    /// wgpu 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// uniform 缓冲区绑定组布局
    uniform_layout: wgpu::BindGroupLayout,
    /// 纹理绑定组布局
    texture_layout: wgpu::BindGroupLayout,
    /// 顶点缓冲区
    vertex_buffer: wgpu::Buffer,
    /// 索引缓冲区
    index_buffer: wgpu::Buffer,
}

impl SpritePipeline {
    /// 创建精灵渲染管线
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `format` - 渲染目标纹理格式
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite_shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                gg_compiler_shader::shaders::load_sprite_shader().expect("内置精灵着色器编译失败"),
            )),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&texture_layout)],
            immediate_size: 0,
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            &shader,
            format,
            wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_index_buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { pipeline, uniform_layout, texture_layout, vertex_buffer, index_buffer }
    }

    /// 获取渲染管线的引用
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// 获取 uniform 绑定组布局的引用
    pub fn uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_layout
    }

    /// 获取纹理绑定组布局的引用
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    /// 获取顶点缓冲区的引用
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// 获取索引缓冲区的引用
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// 创建 uniform 绑定组
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `buffer` - uniform 缓冲区
    pub fn create_uniform_bind_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }
}

/// 过渡渲染管线
///
/// 封装了场景过渡动画所需的 wgpu 渲染管线和绑定组布局。
pub struct TransitionPipeline {
    /// wgpu 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// uniform 缓冲区绑定组布局
    uniform_layout: wgpu::BindGroupLayout,
    /// 纹理绑定组布局（包含两个纹理）
    texture_layout: wgpu::BindGroupLayout,
}

impl TransitionPipeline {
    /// 创建过渡渲染管线
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `format` - 渲染目标纹理格式
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("transition_shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                gg_compiler_shader::shaders::load_transition_shader().expect("内置过渡着色器编译失败"),
            )),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("transition_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("transition_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("transition_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&texture_layout)],
            immediate_size: 0,
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            &shader,
            format,
            wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
        );

        Self { pipeline, uniform_layout, texture_layout }
    }

    /// 获取渲染管线的引用
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// 获取 uniform 绑定组布局的引用
    pub fn uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_layout
    }

    /// 获取纹理绑定组布局的引用
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    /// 创建 uniform 绑定组
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `buffer` - uniform 缓冲区
    pub fn create_uniform_bind_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transition_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }
}

/// 渲染项类型
///
/// 标识渲染项使用的渲染管线类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderItemType {
    /// 精灵渲染
    Sprite,
    /// 过渡渲染
    Transition,
}

/// 渲染项
///
/// 包含一次绘制调用所需的所有绑定组信息和 uniform 缓冲区。
pub struct RenderItem {
    /// 渲染项类型
    pub item_type: RenderItemType,
    /// uniform 绑定组
    pub uniform_bind_group: wgpu::BindGroup,
    /// 纹理绑定组
    pub texture_bind_group: wgpu::BindGroup,
    /// uniform 缓冲区（用于帧间复用）
    pub uniform_buffer: wgpu::Buffer,
}

/// 批渲染精灵管线
///
/// 封装了实例化精灵绘制所需的 wgpu 渲染管线和绑定组布局。
/// 支持一次绘制调用渲染多个同纹理精灵。
pub struct BatchSpritePipeline {
    /// wgpu 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// 纹理绑定组布局
    texture_layout: wgpu::BindGroupLayout,
    /// 顶点缓冲区
    vertex_buffer: wgpu::Buffer,
    /// 索引缓冲区
    index_buffer: wgpu::Buffer,
}

impl BatchSpritePipeline {
    /// 创建批渲染精灵管线
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `format` - 渲染目标纹理格式
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite_batch_shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                gg_compiler_shader::shaders::load_batch_sprite_shader().expect("内置批渲染精灵着色器编译失败"),
            )),
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("batch_sprite_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("batch_sprite_pipeline_layout"),
            bind_group_layouts: &[Some(&texture_layout)],
            immediate_size: 0,
        });

        let instance_size = std::mem::size_of::<BatchedSpriteInstance>() as wgpu::BufferAddress;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("batch_sprite_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2,
                            1 => Float32x2,
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: instance_size,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            2 => Float32x4,
                            3 => Float32x4,
                            4 => Float32x4,
                            5 => Float32x4,
                            6 => Float32x4,
                            7 => Float32x4,
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("batch_sprite_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("batch_sprite_index_buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { pipeline, texture_layout, vertex_buffer, index_buffer }
    }

    /// 获取渲染管线的引用
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// 获取纹理绑定组布局的引用
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    /// 获取顶点缓冲区的引用
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// 获取索引缓冲区的引用
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
}

/// 圆角矩形着色器 uniform 数据
///
/// 包含 MVP 变换矩阵、矩形尺寸和圆角半径、填充颜色。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RoundedRectUniforms {
    /// MVP 变换矩阵
    pub mvp: [[f32; 4]; 4],
    /// 矩形参数 `[width, height, corner_radius, 0.0]`
    pub rect_size: [f32; 4],
    /// 填充颜色 RGBA
    pub color: [f32; 4],
}

/// 圆角矩形渲染管线
///
/// 使用 SDF（有符号距离场）技术渲染圆角矩形，
/// 支持可配置的圆角半径和抗锯齿边缘。
pub struct RoundedRectPipeline {
    /// wgpu 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// uniform 缓冲区绑定组布局
    uniform_layout: wgpu::BindGroupLayout,
    /// 顶点缓冲区
    vertex_buffer: wgpu::Buffer,
    /// 索引缓冲区
    index_buffer: wgpu::Buffer,
}

impl RoundedRectPipeline {
    /// 创建圆角矩形渲染管线
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `format` - 渲染目标纹理格式
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rounded_rect_shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                gg_compiler_shader::shaders::load_rounded_rect_shader().expect("内置圆角矩形着色器编译失败"),
            )),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rounded_rect_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rounded_rect_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_layout)],
            immediate_size: 0,
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            &shader,
            format,
            wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rounded_rect_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rounded_rect_index_buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { pipeline, uniform_layout, vertex_buffer, index_buffer }
    }

    /// 获取渲染管线的引用
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// 获取 uniform 绑定组布局的引用
    pub fn uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_layout
    }

    /// 获取顶点缓冲区的引用
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// 获取索引缓冲区的引用
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// 创建 uniform 绑定组
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `buffer` - uniform 缓冲区
    pub fn create_uniform_bind_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rounded_rect_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }
}

/// 椭圆着色器 uniform 数据
///
/// 包含 MVP 变换矩阵、椭圆参数、填充颜色和边框参数。
/// 圆形是椭圆的特例（radius_x == radius_y）。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct EllipseUniforms {
    /// MVP 变换矩阵
    pub mvp: [[f32; 4]; 4],
    /// 椭圆参数 `[0.0, 0.0, radius_x, radius_y]`
    pub ellipse_params: [f32; 4],
    /// 填充颜色 RGBA
    pub fill_color: [f32; 4],
    /// 边框参数 `[border_width, border_color_r, border_color_g, border_color_b]`
    pub border_params: [f32; 4],
}

/// 椭圆渲染管线
///
/// 使用 SDF（有符号距离场）技术渲染椭圆和圆形，
/// 支持填充和描边模式，以及抗锯齿边缘处理。
/// 圆形是椭圆的特例（radius_x == radius_y）。
pub struct EllipsePipeline {
    /// wgpu 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// uniform 缓冲区绑定组布局
    uniform_layout: wgpu::BindGroupLayout,
    /// 顶点缓冲区
    vertex_buffer: wgpu::Buffer,
    /// 索引缓冲区
    index_buffer: wgpu::Buffer,
}

impl EllipsePipeline {
    /// 创建椭圆渲染管线
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `format` - 渲染目标纹理格式
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ellipse_shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                gg_compiler_shader::shaders::load_ellipse_shader().expect("内置椭圆着色器编译失败"),
            )),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ellipse_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ellipse_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_layout)],
            immediate_size: 0,
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            &shader,
            format,
            wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ellipse_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ellipse_index_buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { pipeline, uniform_layout, vertex_buffer, index_buffer }
    }

    /// 获取渲染管线的引用
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// 获取 uniform 绑定组布局的引用
    pub fn uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_layout
    }

    /// 获取顶点缓冲区的引用
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// 获取索引缓冲区的引用
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// 创建 uniform 绑定组
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `buffer` - uniform 缓冲区
    pub fn create_uniform_bind_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ellipse_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }
}

/// 创建渲染管线
///
/// 辅助函数，创建启用了 alpha 混合的 wgpu 渲染管线。
///
/// # 参数
///
/// - `device` - wgpu 设备
/// - `layout` - 管线布局
/// - `shader` - 着色器模块
/// - `format` - 渲染目标纹理格式
/// - `vertex_states` - 顶点状态列表
fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    vertex_state: wgpu::VertexState<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(layout),
        vertex: vertex_state,
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
        multiview_mask: None,
        cache: None,
    })
}

/// SDF 文本着色器 uniform 数据
///
/// 包含 MVP 变换矩阵、SDF 参数、颜色、描边和阴影设置。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SdfTextUniforms {
    /// MVP 变换矩阵
    pub mvp: [[f32; 4]; 4],
    /// SDF 参数 `[smooth_min, smooth_max, font_size, 0.0]`
    pub sdf_params: [f32; 4],
    /// 文本颜色 RGBA
    pub color: [f32; 4],
    /// 描边颜色 RGBA
    pub outline_color: [f32; 4],
    /// 描边宽度和填充 `[outline_width, 0.0, 0.0, 0.0]`
    pub outline_params: [f32; 4],
    /// 阴影颜色 RGBA
    pub shadow_color: [f32; 4],
    /// 阴影偏移 `[shadow_offset_x, shadow_offset_y, 0.0, 0.0]`
    pub shadow_params: [f32; 4],
}

/// SDF 文本渲染管线
///
/// 使用 SDF（有符号距离场）技术渲染文本，
/// 支持任意缩放不模糊、描边和阴影效果。
/// 着色器通过 gs 编译器从 `.shader` 文件编译。
pub struct SdfTextPipeline {
    /// wgpu 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// uniform 缓冲区绑定组布局
    uniform_layout: wgpu::BindGroupLayout,
    /// 纹理绑定组布局
    texture_layout: wgpu::BindGroupLayout,
    /// 顶点缓冲区
    vertex_buffer: wgpu::Buffer,
    /// 索引缓冲区
    index_buffer: wgpu::Buffer,
}

impl SdfTextPipeline {
    /// 创建 SDF 文本渲染管线
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `format` - 渲染目标纹理格式
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sdf_text_shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                gg_compiler_shader::shaders::load_sdf_text_shader().expect("内置 SDF 文本着色器编译失败"),
            )),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_text_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_text_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sdf_text_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&texture_layout)],
            immediate_size: 0,
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            &shader,
            format,
            wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_text_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_text_index_buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { pipeline, uniform_layout, texture_layout, vertex_buffer, index_buffer }
    }

    /// 获取渲染管线的引用
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// 获取 uniform 绑定组布局的引用
    pub fn uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_layout
    }

    /// 获取纹理绑定组布局的引用
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    /// 获取顶点缓冲区的引用
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// 获取索引缓冲区的引用
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// 创建 uniform 绑定组
    ///
    /// # 参数
    ///
    /// - `device` - wgpu 设备
    /// - `buffer` - uniform 缓冲区
    pub fn create_uniform_bind_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_text_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }
}
