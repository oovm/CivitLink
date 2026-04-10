use std::{path::Path, sync::Arc};

use ab_glyph::{Font, ScaleFont};
use gg_core::{GError, GErrorKind, GResult};
use gg_render::{DrawCommand, RenderContext, Renderer, SurfaceInfo, TextureId, Transform, TransitionKind, WindowEvent};
use wgpu::util::DeviceExt;
use winit::{
    event::Event,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    glyph_cache::GlyphCache,
    pipeline::{RenderItem, RenderItemType, SpritePipeline, SpriteUniforms, TransitionPipeline, TransitionUniforms},
    texture_cache::TextureCache,
};

/// WGPU 渲染器
///
/// 基于 WGPU 实现的渲染器，提供跨平台的 2D 图形渲染能力。
/// 支持精灵绘制、文本渲染、矩形绘制和场景过渡动画。
///
/// # 使用方式
///
/// 1. 通过 [`WgpuRenderer::new`] 创建渲染器（需要传入 winit 事件循环）
/// 2. 在 winit 事件循环中调用 [`WgpuRenderer::handle_window_event`] 转发事件
/// 3. 每帧依次调用 [`Renderer::begin_frame`]、[`Renderer::draw`]、[`Renderer::end_frame`]、[`Renderer::present`]
/// 4. 通过 [`WgpuRenderer::poll_events`] 获取窗口事件
pub struct WgpuRenderer {
    /// 窗口
    window: Arc<Window>,
    /// 渲染表面
    surface: wgpu::Surface<'static>,
    /// wgpu 设备
    device: wgpu::Device,
    /// 命令队列
    queue: wgpu::Queue,
    /// 表面配置
    config: wgpu::SurfaceConfiguration,
    /// 渲染表面信息
    surface_info: SurfaceInfo,
    /// 纹理缓存
    texture_cache: TextureCache,
    /// 字形缓存
    glyph_cache: GlyphCache,
    /// 精灵渲染管线
    sprite_pipeline: SpritePipeline,
    /// 过渡渲染管线
    transition_pipeline: TransitionPipeline,
    /// 是否应该关闭窗口
    should_close: bool,
    /// 待处理的窗口事件
    pending_events: Vec<WindowEvent>,
    /// 当前帧的表面纹理
    frame_output: Option<wgpu::SurfaceTexture>,
    /// 命令编码器
    command_encoder: Option<wgpu::CommandEncoder>,
    /// 1x1 白色像素纹理的标识符
    white_pixel_texture: TextureId,
}

impl WgpuRenderer {
    /// 创建新的 WGPU 渲染器
    ///
    /// 使用指定的 winit 事件循环和渲染表面信息创建渲染器。
    /// 内部会创建窗口、初始化 WGPU 设备和渲染管线。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环引用
    /// - `surface_info` - 渲染表面信息
    ///
    /// # 返回值
    ///
    /// 成功时返回渲染器实例
    pub fn new(event_loop: &EventLoop<()>, surface_info: SurfaceInfo) -> GResult<Self> {
        let window_attrs = WindowAttributes::default()
            .with_title(&surface_info.title)
            .with_inner_size(winit::dpi::LogicalSize::new(surface_info.width, surface_info.height));

        #[allow(deprecated)]
        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建窗口: {}", e) })?,
        );

        let (surface, device, queue, config) = pollster::block_on(async {
            let instance =
                wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });

            let surface = instance
                .create_surface(Arc::clone(&window))
                .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建渲染表面: {}", e) })?;

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok_or(GError { kind: GErrorKind::Platform, message: "无法找到合适的图形适配器".to_string() })?;

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("gg_render_device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        ..Default::default()
                    },
                    None,
                )
                .await
                .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建图形设备: {}", e) })?;

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(surface_caps.formats[0]);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: surface_info.width,
                height: surface_info.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(&device, &config);

            Ok::<_, GError>((surface, device, queue, config))
        })?;

        let sprite_pipeline = SpritePipeline::new(&device, config.format);
        let transition_pipeline = TransitionPipeline::new(&device, config.format);

        let mut texture_cache = TextureCache::new();
        let white_pixel_data: [u8; 4] = [255, 255, 255, 255];
        let white_pixel_texture =
            texture_cache.create_texture_from_data(1, 1, &white_pixel_data, &device, &queue, "white_pixel").map_err(|e| {
                GError { kind: GErrorKind::Platform, message: format!("无法创建白色像素纹理: {}", e.message) }
            })?;

        let glyph_cache = GlyphCache::new();

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            surface_info,
            texture_cache,
            glyph_cache,
            sprite_pipeline,
            transition_pipeline,
            should_close: false,
            pending_events: Vec::new(),
            frame_output: None,
            command_encoder: None,
            white_pixel_texture,
        })
    }

    /// 检查窗口是否应该关闭
    pub fn should_close(&self) -> bool {
        self.should_close
    }

    /// 轮询窗口事件
    ///
    /// 返回并清空内部事件缓冲区中的所有窗口事件。
    /// 需要通过 [`WgpuRenderer::handle_window_event`] 方法将 winit 事件转发到此渲染器。
    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// 处理 winit 窗口事件
    ///
    /// 将 winit 的窗口事件转换为引擎的 `WindowEvent` 并存入内部缓冲区。
    /// 应在 winit 事件循环的回调中调用此方法。
    ///
    /// # 参数
    ///
    /// - `event` - winit 窗口事件引用
    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::Resized(physical_size) => {
                let width = physical_size.width;
                let height = physical_size.height;
                if width > 0 && height > 0 {
                    self.config.width = width;
                    self.config.height = height;
                    self.surface.configure(&self.device, &self.config);
                    self.surface_info.width = width;
                    self.surface_info.height = height;
                }
                self.pending_events.push(WindowEvent::Resized { width, height });
            }
            winit::event::WindowEvent::CloseRequested => {
                self.should_close = true;
                self.pending_events.push(WindowEvent::CloseRequested);
            }
            winit::event::WindowEvent::Focused(focused) => {
                self.pending_events.push(WindowEvent::Focused(*focused));
            }
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.pending_events.push(WindowEvent::ScaleFactorChanged(*scale_factor));
            }
            _ => {}
        }
    }

    /// 处理 winit 事件
    ///
    /// 从 winit 事件中提取窗口事件并转发到内部缓冲区。
    ///
    /// # 参数
    ///
    /// - `event` - winit 事件引用
    pub fn handle_event<T>(&mut self, event: &Event<T>) {
        if let Event::WindowEvent { event, .. } = event {
            self.handle_window_event(event);
        }
    }

    /// 获取窗口的引用
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    /// 获取 wgpu 设备的引用
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// 获取字形缓存的可变引用
    pub fn glyph_cache_mut(&mut self) -> &mut GlyphCache {
        &mut self.glyph_cache
    }

    /// 计算正交投影矩阵
    ///
    /// 创建一个将像素坐标映射到裁剪空间的正交投影矩阵。
    /// 原点在左上角，X 轴向右，Y 轴向下。
    fn orthographic(width: f32, height: f32) -> [[f32; 4]; 4] {
        [[2.0 / width, 0.0, 0.0, 0.0], [0.0, -2.0 / height, 0.0, 0.0], [0.0, 0.0, 0.5, 0.0], [-1.0, 1.0, 0.5, 1.0]]
    }

    /// 计算平移矩阵
    fn translate(tx: f32, ty: f32) -> [[f32; 4]; 4] {
        [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [tx, ty, 0.0, 1.0]]
    }

    /// 计算旋转矩阵
    fn rotate(angle: f32) -> [[f32; 4]; 4] {
        let (s, c) = (angle.sin(), angle.cos());
        [[c, s, 0.0, 0.0], [-s, c, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }

    /// 计算缩放矩阵
    fn scale(sx: f32, sy: f32) -> [[f32; 4]; 4] {
        [[sx, 0.0, 0.0, 0.0], [0.0, sy, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }

    /// 4x4 矩阵乘法
    fn mat4_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
            }
        }
        result
    }

    /// 计算精灵的 MVP 矩阵
    fn compute_sprite_mvp(transform: &Transform, size: [f32; 2], surface_width: u32, surface_height: u32) -> [[f32; 4]; 4] {
        let projection = Self::orthographic(surface_width as f32, surface_height as f32);
        let model_t = Self::translate(transform.position[0], transform.position[1]);
        let model_r = Self::rotate(transform.rotation);
        let model_s = Self::scale(size[0] * transform.scale[0], size[1] * transform.scale[1]);
        let model = Self::mat4_mul(&model_t, &Self::mat4_mul(&model_r, &model_s));
        Self::mat4_mul(&projection, &model)
    }

    /// 计算全屏四边形的 MVP 矩阵
    fn compute_fullscreen_mvp(surface_width: u32, surface_height: u32) -> [[f32; 4]; 4] {
        let projection = Self::orthographic(surface_width as f32, surface_height as f32);
        let model = Self::scale(surface_width as f32, surface_height as f32);
        Self::mat4_mul(&projection, &model)
    }

    /// 将过渡类型转换为着色器参数值
    fn transition_kind_to_param(kind: TransitionKind) -> f32 {
        match kind {
            TransitionKind::Fade => 0.0,
            TransitionKind::CrossDissolve => 1.0,
            TransitionKind::SlideLeft => 2.0,
            TransitionKind::SlideRight => 3.0,
            TransitionKind::SlideUp => 4.0,
            TransitionKind::SlideDown => 5.0,
        }
    }
}

impl Renderer for WgpuRenderer {
    fn begin_frame(&mut self) -> GResult<()> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法获取当前帧纹理: {:?}", e) })?;
        self.frame_output = Some(output);
        self.command_encoder =
            Some(self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_encoder") }));
        Ok(())
    }

    fn end_frame(&mut self) -> GResult<()> {
        Ok(())
    }

    fn draw(&mut self, context: &RenderContext) -> GResult<()> {
        let surface_width = context.surface_width();
        let surface_height = context.surface_height();

        // 阶段 1：预光栅化所有文本字形
        {
            for cmd in context.commands() {
                if let DrawCommand::Text { text, font_size, .. } = cmd {
                    let font = self
                        .glyph_cache
                        .font()
                        .ok_or_else(|| GError {
                            kind: GErrorKind::Runtime, message: "未加载字体，无法渲染文本".to_string()
                        })?
                        .clone();

                    let px_scale = ab_glyph::PxScale { x: *font_size, y: *font_size };

                    for c in text.chars() {
                        let glyph_id = font.glyph_id(c);
                        let glyph = glyph_id.with_scale(px_scale);
                        self.glyph_cache.get_or_rasterize(glyph, &self.device, &self.queue, &mut self.texture_cache)?;
                    }
                }
            }
        }

        // 阶段 2：创建渲染项
        let mut render_items: Vec<RenderItem> = Vec::new();

        for cmd in context.commands() {
            match cmd {
                DrawCommand::Sprite { texture_id, transform, size, tint, .. } => {
                    let mvp = Self::compute_sprite_mvp(transform, *size, surface_width, surface_height);
                    let uniforms = SpriteUniforms { mvp, tint: [tint.r, tint.g, tint.b, tint.a] };
                    let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("sprite_uniform_buffer"),
                        contents: bytemuck::cast_slice(&[uniforms]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                    let uniform_bind_group = self.sprite_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    let texture_bind_group = self
                        .texture_cache
                        .create_bind_group(*texture_id, &self.device, self.sprite_pipeline.texture_layout())
                        .ok_or_else(|| GError {
                            kind: GErrorKind::Asset,
                            message: format!("无法创建精灵纹理绑定组，纹理 ID: {:?}", texture_id),
                        })?;

                    render_items.push(RenderItem { item_type: RenderItemType::Sprite, uniform_bind_group, texture_bind_group });
                }
                DrawCommand::Text { text, position, font_size, color, .. } => {
                    let font = match self.glyph_cache.font() {
                        Some(f) => f.clone(),
                        None => continue,
                    };

                    let px_scale = ab_glyph::PxScale { x: *font_size, y: *font_size };
                    let scaled_font = font.as_scaled(px_scale);

                    let mut cursor_x = position[0];
                    let cursor_y = position[1];

                    for c in text.chars() {
                        let glyph_id = font.glyph_id(c);
                        let advance = scaled_font.h_advance(glyph_id);
                        let glyph = glyph_id.with_scale(px_scale);

                        let glyph_info = match self.glyph_cache.get_or_rasterize(
                            glyph,
                            &self.device,
                            &self.queue,
                            &mut self.texture_cache,
                        ) {
                            Ok(info) => info,
                            Err(_) => {
                                cursor_x += advance;
                                continue;
                            }
                        };

                        if glyph_info.texture_id != TextureId::INVALID && glyph_info.size[0] > 0.0 && glyph_info.size[1] > 0.0 {
                            let glyph_transform = Transform {
                                position: [cursor_x + glyph_info.offset[0], cursor_y + glyph_info.offset[1]],
                                scale: [1.0, 1.0],
                                rotation: 0.0,
                                z_index: 0.0,
                            };

                            let mvp =
                                Self::compute_sprite_mvp(&glyph_transform, glyph_info.size, surface_width, surface_height);
                            let uniforms = SpriteUniforms { mvp, tint: [color.r, color.g, color.b, color.a] };
                            let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("text_uniform_buffer"),
                                contents: bytemuck::cast_slice(&[uniforms]),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                            let uniform_bind_group =
                                self.sprite_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);

                            if let Some(texture_bind_group) = self.texture_cache.create_bind_group(
                                glyph_info.texture_id,
                                &self.device,
                                self.sprite_pipeline.texture_layout(),
                            ) {
                                render_items.push(RenderItem {
                                    item_type: RenderItemType::Sprite,
                                    uniform_bind_group,
                                    texture_bind_group,
                                });
                            }
                        }

                        cursor_x += advance;
                    }
                }
                DrawCommand::Rect { rect, color, .. } => {
                    let transform = Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
                    let mvp = Self::compute_sprite_mvp(&transform, [rect.width, rect.height], surface_width, surface_height);
                    let uniforms = SpriteUniforms { mvp, tint: [color.r, color.g, color.b, color.a] };
                    let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("rect_uniform_buffer"),
                        contents: bytemuck::cast_slice(&[uniforms]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                    let uniform_bind_group = self.sprite_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    let texture_bind_group = self
                        .texture_cache
                        .create_bind_group(self.white_pixel_texture, &self.device, self.sprite_pipeline.texture_layout())
                        .ok_or_else(|| GError {
                            kind: GErrorKind::Asset, message: "无法创建矩形纹理绑定组".to_string()
                        })?;

                    render_items.push(RenderItem { item_type: RenderItemType::Sprite, uniform_bind_group, texture_bind_group });
                }
                DrawCommand::Transition { old_texture, new_texture, progress, kind } => {
                    let old_id = old_texture.unwrap_or(self.white_pixel_texture);
                    let new_id = new_texture.unwrap_or(self.white_pixel_texture);

                    let mvp = Self::compute_fullscreen_mvp(surface_width, surface_height);
                    let uniforms = TransitionUniforms {
                        mvp,
                        params: [*progress, Self::transition_kind_to_param(*kind), 0.0, 0.0],
                        tint: [1.0, 1.0, 1.0, 1.0],
                    };
                    let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("transition_uniform_buffer"),
                        contents: bytemuck::cast_slice(&[uniforms]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                    let uniform_bind_group = self.transition_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    let texture_bind_group = self
                        .texture_cache
                        .create_transition_bind_group(old_id, new_id, &self.device, self.transition_pipeline.texture_layout())
                        .ok_or_else(|| GError {
                            kind: GErrorKind::Asset, message: "无法创建过渡纹理绑定组".to_string()
                        })?;

                    render_items.push(RenderItem {
                        item_type: RenderItemType::Transition,
                        uniform_bind_group,
                        texture_bind_group,
                    });
                }
            }
        }

        // 阶段 3：记录渲染通道
        let encoder = self.command_encoder.as_mut().ok_or_else(|| GError {
            kind: GErrorKind::Runtime,
            message: "命令编码器不存在，请先调用 begin_frame".to_string(),
        })?;
        let frame =
            self.frame_output.as_ref().ok_or_else(|| GError {
                kind: GErrorKind::Runtime,
                message: "帧输出不存在，请先调用 begin_frame".to_string(),
            })?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sprite_pipeline = &self.sprite_pipeline;
        let transition_pipeline = &self.transition_pipeline;

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        for item in &render_items {
            match item.item_type {
                RenderItemType::Sprite => {
                    render_pass.set_pipeline(sprite_pipeline.pipeline());
                    render_pass.set_vertex_buffer(0, sprite_pipeline.vertex_buffer().slice(..));
                    render_pass.set_index_buffer(sprite_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
                }
                RenderItemType::Transition => {
                    render_pass.set_pipeline(transition_pipeline.pipeline());
                    render_pass.set_vertex_buffer(0, sprite_pipeline.vertex_buffer().slice(..));
                    render_pass.set_index_buffer(sprite_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
                }
            }
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &item.texture_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        drop(render_pass);
        Ok(())
    }

    fn present(&mut self) -> GResult<()> {
        if let Some(encoder) = self.command_encoder.take() {
            self.queue.submit(std::iter::once(encoder.finish()));
        }
        if let Some(frame) = self.frame_output.take() {
            frame.present();
        }
        Ok(())
    }

    fn load_texture(&mut self, path: &Path) -> GResult<TextureId> {
        self.texture_cache.load_texture(path, &self.device, &self.queue)
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.surface_info.width = width;
            self.surface_info.height = height;
        }
    }

    fn surface_info(&self) -> &SurfaceInfo {
        &self.surface_info
    }
}
