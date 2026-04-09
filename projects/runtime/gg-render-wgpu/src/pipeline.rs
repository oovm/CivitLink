use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::shader;

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
/// 包含 MVP 变换矩阵和着色颜色。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SpriteUniforms {
    /// MVP 变换矩阵
    pub mvp: [[f32; 4]; 4],
    /// 着色颜色 RGBA
    pub tint: [f32; 4],
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
    Vertex {
        position: [0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0],
        uv: [0.0, 1.0],
    },
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
            source: wgpu::ShaderSource::Wgsl(shader::SPRITE_SHADER.into()),
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
            bind_group_layouts: &[&uniform_layout, &texture_layout],
            push_constant_ranges: &[],
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

        Self {
            pipeline,
            uniform_layout,
            texture_layout,
            vertex_buffer,
            index_buffer,
        }
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
    pub fn create_uniform_bind_group(
        &self,
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
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
            source: wgpu::ShaderSource::Wgsl(shader::TRANSITION_SHADER.into()),
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
            bind_group_layouts: &[&uniform_layout, &texture_layout],
            push_constant_ranges: &[],
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

        Self {
            pipeline,
            uniform_layout,
            texture_layout,
        }
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
    pub fn create_uniform_bind_group(
        &self,
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transition_uniform_bind_group"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
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
/// 包含一次绘制调用所需的所有绑定组信息。
pub struct RenderItem {
    /// 渲染项类型
    pub item_type: RenderItemType,
    /// uniform 绑定组
    pub uniform_bind_group: wgpu::BindGroup,
    /// 纹理绑定组
    pub texture_bind_group: wgpu::BindGroup,
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
    vertex_states: &[wgpu::VertexState<'_>],
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(layout),
        vertex: vertex_states[0].clone(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
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
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
