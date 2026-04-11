use std::{path::Path, sync::Arc};

use ab_glyph::{Font, ScaleFont};
use gg_core::{GError, GErrorKind, GResult};
use gg_render::{Camera, DrawCommand, RenderContext, Renderer, SurfaceInfo, TextureId, Transform, TransitionKind, WindowEvent};
use wgpu::util::DeviceExt;

#[cfg(not(target_arch = "wasm32"))]
use winit::{
    event::Event,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    glyph_cache::GlyphCache,
    pipeline::{
        BatchSpritePipeline, EllipsePipeline, EllipseUniforms, RenderItem, RenderItemType, RoundedRectPipeline,
        RoundedRectUniforms, SpritePipeline, SpriteUniforms, TransitionPipeline, TransitionUniforms,
    },
    sprite_batch::{SpriteBatch, SpriteBatcher},
    texture_cache::TextureCache,
    uniform_pool::UniformPool,
};

/// 裁剪矩形
///
/// 用于渲染通道中的像素裁剪区域。
struct ScissorRect {
    /// x 坐标
    x: u32,
    /// y 坐标
    y: u32,
    /// 宽度
    w: u32,
    /// 高度
    h: u32,
}

/// 离屏渲染目标
///
/// 封装一个可渲染的离屏纹理视图。
/// 渲染目标的纹理已注册到纹理缓存中，可通过 `texture_id` 作为精灵纹理使用。
pub struct RenderTarget {
    /// 纹理视图，用于渲染通道的颜色附件
    view: wgpu::TextureView,
    /// 纹理标识符，可用于精灵绘制
    texture_id: TextureId,
    /// 渲染目标宽度（像素）
    width: u32,
    /// 渲染目标高度（像素）
    height: u32,
}

impl RenderTarget {
    /// 获取纹理标识符
    ///
    /// 返回的标识符可用于精灵绘制命令中的纹理引用。
    pub fn texture_id(&self) -> TextureId {
        self.texture_id
    }

    /// 获取渲染目标宽度（像素）
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取渲染目标高度（像素）
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// WGPU 渲染器
///
/// 基于 WGPU 实现的渲染器，提供跨平台的 2D 图形渲染能力。
/// 支持精灵批渲染、文本渲染、几何图形绘制、场景过渡动画、
/// 相机变换和裁剪矩形。
///
/// # 使用方式
///
/// 1. 通过 [`WgpuRenderer::new`] 创建渲染器（桌面平台，需要传入 winit 事件循环）
///    或通过 [`WgpuRenderer::new_from_surface`] 从已有的 wgpu 对象创建（所有平台）
/// 2. 在 winit 事件循环中调用 [`WgpuRenderer::handle_window_event`] 转发事件（桌面平台）
/// 3. 每帧依次调用 [`Renderer::begin_frame`]、[`Renderer::draw`]、[`Renderer::end_frame`]、[`Renderer::present`]
/// 4. 通过 [`WgpuRenderer::poll_events`] 获取窗口事件
pub struct WgpuRenderer {
    /// 窗口（桌面平台）
    #[cfg(not(target_arch = "wasm32"))]
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
    /// 精灵渲染管线（非批渲染，用于过渡动画）
    sprite_pipeline: SpritePipeline,
    /// 批渲染精灵管线
    batch_sprite_pipeline: BatchSpritePipeline,
    /// 过渡渲染管线
    transition_pipeline: TransitionPipeline,
    /// 圆角矩形渲染管线
    rounded_rect_pipeline: RoundedRectPipeline,
    /// 椭圆渲染管线
    ellipse_pipeline: EllipsePipeline,
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
    /// Uniform 缓冲区池
    uniform_pool: UniformPool,
}

impl WgpuRenderer {
    /// 检查窗口是否应该关闭
    pub fn should_close(&self) -> bool {
        self.should_close
    }

    /// 轮询窗口事件
    ///
    /// 返回并清空内部事件缓冲区中的所有窗口事件。
    /// 桌面平台通过 [`WgpuRenderer::handle_window_event`] 转发 winit 事件，
    /// Web 平台通过 [`WgpuRenderer::push_event`] 推送事件。
    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        std::mem::take(&mut self.pending_events)
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

    /// 计算视图投影矩阵
    ///
    /// 根据相机参数计算视图投影矩阵。
    /// 无相机时返回正交投影矩阵，有相机时叠加相机变换。
    fn compute_view_projection(surface_width: u32, surface_height: u32, camera: Option<&Camera>) -> [[f32; 4]; 4] {
        let projection = Self::orthographic(surface_width as f32, surface_height as f32);

        match camera {
            None => projection,
            Some(cam) => {
                let half_w = surface_width as f32 * 0.5;
                let half_h = surface_height as f32 * 0.5;
                let view_t = Self::translate(-cam.position[0], -cam.position[1]);
                let view_r = Self::rotate(-cam.rotation);
                let view_s = Self::scale(cam.zoom, cam.zoom);
                let center_t = Self::translate(half_w, half_h);
                let center_t_inv = Self::translate(-half_w, -half_h);

                let view = Self::mat4_mul(
                    &center_t,
                    &Self::mat4_mul(&view_r, &Self::mat4_mul(&view_s, &Self::mat4_mul(&center_t_inv, &view_t))),
                );
                Self::mat4_mul(&projection, &view)
            }
        }
    }

    /// 计算精灵的 MVP 矩阵
    fn compute_sprite_mvp(transform: &Transform, size: [f32; 2], view_projection: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let model_t = Self::translate(transform.position[0], transform.position[1]);
        let model_r = Self::rotate(transform.rotation);
        let model_s = Self::scale(size[0] * transform.scale[0], size[1] * transform.scale[1]);
        let model = Self::mat4_mul(&model_t, &Self::mat4_mul(&model_r, &model_s));
        Self::mat4_mul(view_projection, &model)
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

    /// 创建离屏渲染目标
    ///
    /// 创建一个指定尺寸的离屏纹理，可作为渲染目标使用。
    /// 纹理格式与当前渲染表面格式一致，支持渲染附件和纹理绑定。
    /// 创建后纹理已注册到纹理缓存，可通过 `texture_id` 作为精灵纹理引用。
    ///
    /// # 参数
    ///
    /// - `width` - 渲染目标宽度（像素）
    /// - `height` - 渲染目标高度（像素）
    ///
    /// # 返回值
    ///
    /// 成功时返回渲染目标，包含纹理标识符
    pub fn create_render_target(&mut self, width: u32, height: u32) -> GResult<RenderTarget> {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render_target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_id = self.texture_cache.register_texture(texture);

        Ok(RenderTarget { view, texture_id, width, height })
    }

    /// 渲染到离屏纹理
    ///
    /// 将渲染上下文中的绘制命令渲染到指定的离屏渲染目标，
    /// 而非屏幕表面。渲染完成后命令自动提交到 GPU 队列。
    ///
    /// # 参数
    ///
    /// - `target` - 离屏渲染目标
    /// - `context` - 渲染上下文
    pub fn draw_to_target(&mut self, target: &RenderTarget, context: &RenderContext) -> GResult<()> {
        let surface_width = target.width;
        let surface_height = target.height;
        let view_projection = Self::compute_view_projection(surface_width, surface_height, context.camera());

        {
            for cmd in context.commands() {
                if let DrawCommand::Text { text, font_size, .. } = cmd {
                    let font = match self.glyph_cache.font() {
                        Some(f) => f.clone(),
                        None => continue,
                    };

                    let px_scale = ab_glyph::PxScale { x: *font_size, y: *font_size };

                    for c in text.chars() {
                        let glyph_id = font.glyph_id(c);
                        let glyph = glyph_id.with_scale(px_scale);
                        let _ = self.glyph_cache.get_or_rasterize(glyph, &self.device, &self.queue, &mut self.texture_cache);
                    }
                }
            }
        }

        let commands = context.commands();
        let mut indexed: Vec<IndexedCommand> = commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let (z, is_t) = match cmd {
                    DrawCommand::Sprite { transform, .. } => (transform.z_index, false),
                    DrawCommand::Text { .. } => (0.0, false),
                    DrawCommand::Rect { .. } => (0.0, false),
                    DrawCommand::Line { .. } => (0.0, false),
                    DrawCommand::Circle { .. } => (0.0, false),
                    DrawCommand::Ellipse { .. } => (0.0, false),
                    DrawCommand::Transition { .. } => (f32::MAX, true),
                };
                IndexedCommand { index: i, z_index: z, is_transition: is_t }
            })
            .collect();

        indexed.sort_by(|a, b| {
            if a.is_transition != b.is_transition {
                if a.is_transition { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less }
            }
            else {
                a.z_index.partial_cmp(&b.z_index).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let mut sprite_batcher = SpriteBatcher::new();
        let mut transition_items: Vec<RenderItem> = Vec::new();
        let mut rounded_rect_items: Vec<RoundedRectRenderItem> = Vec::new();
        let mut ellipse_items: Vec<EllipseRenderItem> = Vec::new();

        for ic in &indexed {
            let cmd = &commands[ic.index];
            match cmd {
                DrawCommand::Sprite { texture_id, transform, size, tint, .. } => {
                    let mvp = Self::compute_sprite_mvp(transform, *size, &view_projection);
                    sprite_batcher.push(*texture_id, mvp, [tint.r, tint.g, tint.b, tint.a], [0.0, 0.0, 1.0, 1.0]);
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

                            let mvp = Self::compute_sprite_mvp(&glyph_transform, glyph_info.size, &view_projection);
                            let uv_transform = [
                                glyph_info.uv_rect[0],
                                glyph_info.uv_rect[1],
                                glyph_info.uv_rect[2] - glyph_info.uv_rect[0],
                                glyph_info.uv_rect[3] - glyph_info.uv_rect[1],
                            ];
                            sprite_batcher.push(glyph_info.texture_id, mvp, [color.r, color.g, color.b, color.a], uv_transform);
                        }

                        cursor_x += advance;
                    }
                }
                DrawCommand::Rect { rect, color, corner_radius } => {
                    if *corner_radius > 0.0 {
                        let clamped_radius = corner_radius.min(rect.width.min(rect.height) * 0.5);
                        let transform = Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
                        let mvp = Self::compute_sprite_mvp(&transform, [rect.width, rect.height], &view_projection);
                        let uniforms = RoundedRectUniforms {
                            mvp,
                            rect_size: [rect.width, rect.height, clamped_radius, 0.0],
                            color: [color.r, color.g, color.b, color.a],
                        };
                        let uniform_buffer =
                            self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                        let uniform_bind_group =
                            self.rounded_rect_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                        rounded_rect_items.push(RoundedRectRenderItem { uniform_bind_group, uniform_buffer });
                    } else {
                        let transform = Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
                        let mvp = Self::compute_sprite_mvp(&transform, [rect.width, rect.height], &view_projection);
                        sprite_batcher.push(
                            self.white_pixel_texture,
                            mvp,
                            [color.r, color.g, color.b, color.a],
                            [0.0, 0.0, 1.0, 1.0],
                        );
                    }
                }
                DrawCommand::Line { start, end, color, width } => {
                    let dx = end[0] - start[0];
                    let dy = end[1] - start[1];
                    let length = (dx * dx + dy * dy).sqrt();
                    if length < 0.001 {
                        continue;
                    }
                    let angle = dy.atan2(dx);
                    let transform = Transform { position: *start, scale: [1.0, 1.0], rotation: angle, z_index: 0.0 };
                    let mvp = Self::compute_sprite_mvp(&transform, [length, *width], &view_projection);
                    sprite_batcher.push(
                        self.white_pixel_texture,
                        mvp,
                        [color.r, color.g, color.b, color.a],
                        [0.0, 0.0, 1.0, 1.0],
                    );
                }
                DrawCommand::Circle { center, radius, color, filled, border_width, border_color } => {
                    let size = [radius * 2.0, radius * 2.0];
                    let transform = Transform {
                        position: [center[0] - radius, center[1] - radius],
                        scale: [1.0, 1.0],
                        rotation: 0.0,
                        z_index: 0.0,
                    };
                    let mvp = Self::compute_sprite_mvp(&transform, size, &view_projection);
                    let effective_border_width = if *filled { 0.0 } else { *border_width };
                    let uniforms = EllipseUniforms {
                        mvp,
                        ellipse_params: [0.0, 0.0, size[0], size[1]],
                        fill_color: [color.r, color.g, color.b, color.a],
                        border_params: [effective_border_width, border_color[0], border_color[1], border_color[2]],
                    };
                    let uniform_buffer =
                        self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                    let uniform_bind_group =
                        self.ellipse_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    ellipse_items.push(EllipseRenderItem { uniform_bind_group, uniform_buffer });
                }
                DrawCommand::Ellipse { center, radii, color, filled, border_width, border_color } => {
                    let size = [radii[0] * 2.0, radii[1] * 2.0];
                    let transform = Transform {
                        position: [center[0] - radii[0], center[1] - radii[1]],
                        scale: [1.0, 1.0],
                        rotation: 0.0,
                        z_index: 0.0,
                    };
                    let mvp = Self::compute_sprite_mvp(&transform, size, &view_projection);
                    let effective_border_width = if *filled { 0.0 } else { *border_width };
                    let uniforms = EllipseUniforms {
                        mvp,
                        ellipse_params: [0.0, 0.0, size[0], size[1]],
                        fill_color: [color.r, color.g, color.b, color.a],
                        border_params: [effective_border_width, border_color[0], border_color[1], border_color[2]],
                    };
                    let uniform_buffer =
                        self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                    let uniform_bind_group =
                        self.ellipse_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    ellipse_items.push(EllipseRenderItem { uniform_bind_group, uniform_buffer });
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
                    let uniform_buffer =
                        self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                    let uniform_bind_group = self.transition_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    let texture_bind_group = self
                        .texture_cache
                        .create_transition_bind_group(old_id, new_id, &self.device, self.transition_pipeline.texture_layout())
                        .ok_or_else(|| GError {
                            kind: GErrorKind::Asset, message: "无法创建过渡纹理绑定组".to_string()
                        })?;

                    transition_items.push(RenderItem {
                        item_type: RenderItemType::Transition,
                        uniform_bind_group,
                        texture_bind_group,
                        uniform_buffer,
                    });
                }
            }
        }

        let sprite_batches = sprite_batcher.flush();

        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_target_encoder") });

        let scissor_rect = context.clip_rect().map(|r| ScissorRect {
            x: r.x.max(0.0) as u32,
            y: r.y.max(0.0) as u32,
            w: r.width.max(0.0) as u32,
            h: r.height.max(0.0) as u32,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_target_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Some(rect) = &scissor_rect {
            render_pass.set_scissor_rect(rect.x, rect.y, rect.w, rect.h);
        }

        for SpriteBatch { texture_id, instances } in &sprite_batches {
            let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sprite_instance_buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let texture_bind_group = self
                .texture_cache
                .create_bind_group(*texture_id, &self.device, self.batch_sprite_pipeline.texture_layout())
                .ok_or_else(|| GError {
                    kind: GErrorKind::Asset,
                    message: format!("无法创建精灵纹理绑定组，纹理 ID: {:?}", texture_id),
                })?;

            render_pass.set_pipeline(self.batch_sprite_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.batch_sprite_pipeline.vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(self.batch_sprite_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
        }

        for item in &transition_items {
            render_pass.set_pipeline(self.transition_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.sprite_pipeline.vertex_buffer().slice(..));
            render_pass.set_index_buffer(self.sprite_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &item.texture_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        for item in &rounded_rect_items {
            render_pass.set_pipeline(self.rounded_rect_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.rounded_rect_pipeline.vertex_buffer().slice(..));
            render_pass.set_index_buffer(self.rounded_rect_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        for item in &ellipse_items {
            render_pass.set_pipeline(self.ellipse_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.ellipse_pipeline.vertex_buffer().slice(..));
            render_pass.set_index_buffer(self.ellipse_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        drop(render_pass);

        for RenderItem { uniform_buffer, .. } in transition_items {
            self.uniform_pool.mark_used(uniform_buffer);
        }

        for RoundedRectRenderItem { uniform_buffer, .. } in rounded_rect_items {
            self.uniform_pool.mark_used(uniform_buffer);
        }

        for EllipseRenderItem { uniform_buffer, .. } in ellipse_items {
            self.uniform_pool.mark_used(uniform_buffer);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// 重新加载纹理（用于 HMR 热更新）
    ///
    /// 从指定路径重新加载纹理数据并更新 GPU 纹理对象。
    /// 如果加载失败，保留旧纹理不变。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    pub fn reload_texture(&mut self, path: &str) -> GResult<()> {
        let path = std::path::Path::new(path);
        self.texture_cache.load_texture(path, &self.device, &self.queue)?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl WgpuRenderer {
    /// 创建新的 WGPU 渲染器（桌面平台）
    ///
    /// 使用指定的 winit 事件循环和渲染表面信息创建渲染器。
    /// 内部会创建窗口、初始化 WGPU 设备和渲染管线。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环引用
    /// - `surface_info` - 渲染表面信息
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
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

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
                .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法找到合适的图形适配器: {}", e) })?;

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("gg_render_device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        ..Default::default()
                    },
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
        let batch_sprite_pipeline = BatchSpritePipeline::new(&device, config.format);
        let transition_pipeline = TransitionPipeline::new(&device, config.format);
        let rounded_rect_pipeline = RoundedRectPipeline::new(&device, config.format);
        let ellipse_pipeline = EllipsePipeline::new(&device, config.format);

        let mut texture_cache = TextureCache::new();
        let white_pixel_data: [u8; 4] = [255, 255, 255, 255];
        let white_pixel_texture =
            texture_cache.create_texture_from_data(1, 1, &white_pixel_data, &device, &queue, "white_pixel").map_err(|e| {
                GError { kind: GErrorKind::Platform, message: format!("无法创建白色像素纹理: {}", e.message) }
            })?;

        let glyph_cache = GlyphCache::new();

        let buffer_size = std::cmp::max(
            std::mem::size_of::<SpriteUniforms>(),
            std::cmp::max(std::mem::size_of::<TransitionUniforms>(), std::cmp::max(std::mem::size_of::<RoundedRectUniforms>(), std::mem::size_of::<EllipseUniforms>())),
        );
        let uniform_pool = UniformPool::new(buffer_size);

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
            batch_sprite_pipeline,
            transition_pipeline,
            rounded_rect_pipeline,
            ellipse_pipeline,
            should_close: false,
            pending_events: Vec::new(),
            frame_output: None,
            command_encoder: None,
            white_pixel_texture,
            uniform_pool,
        })
    }

    /// 从已有的 wgpu 对象创建渲染器（桌面平台）
    ///
    /// 适用于自定义窗口系统集成场景，
    /// 调用者负责创建 wgpu 实例、适配器、设备和表面。
    ///
    /// # 参数
    ///
    /// - `surface` - wgpu 渲染表面
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `config` - 表面配置
    /// - `surface_info` - 渲染表面信息
    /// - `window` - winit 窗口
    pub fn new_from_surface(
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        surface_info: SurfaceInfo,
        window: Arc<Window>,
    ) -> GResult<Self> {
        let sprite_pipeline = SpritePipeline::new(&device, config.format);
        let batch_sprite_pipeline = BatchSpritePipeline::new(&device, config.format);
        let transition_pipeline = TransitionPipeline::new(&device, config.format);
        let rounded_rect_pipeline = RoundedRectPipeline::new(&device, config.format);
        let ellipse_pipeline = EllipsePipeline::new(&device, config.format);

        let mut texture_cache = TextureCache::new();
        let white_pixel_data: [u8; 4] = [255, 255, 255, 255];
        let white_pixel_texture =
            texture_cache.create_texture_from_data(1, 1, &white_pixel_data, &device, &queue, "white_pixel").map_err(|e| {
                GError { kind: GErrorKind::Platform, message: format!("无法创建白色像素纹理: {}", e.message) }
            })?;

        let glyph_cache = GlyphCache::new();

        let buffer_size = std::cmp::max(
            std::mem::size_of::<SpriteUniforms>(),
            std::cmp::max(std::mem::size_of::<TransitionUniforms>(), std::cmp::max(std::mem::size_of::<RoundedRectUniforms>(), std::mem::size_of::<EllipseUniforms>())),
        );
        let uniform_pool = UniformPool::new(buffer_size);

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
            batch_sprite_pipeline,
            transition_pipeline,
            rounded_rect_pipeline,
            ellipse_pipeline,
            should_close: false,
            pending_events: Vec::new(),
            frame_output: None,
            command_encoder: None,
            white_pixel_texture,
            uniform_pool,
        })
    }

    /// 处理 winit 窗口事件
    ///
    /// 将 winit 的窗口事件转换为引擎的 `WindowEvent` 并存入内部缓冲区。
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
}

#[cfg(target_arch = "wasm32")]
impl WgpuRenderer {
    /// 从已有的 wgpu 对象创建渲染器（Web 平台）
    ///
    /// 适用于 Web 平台，调用者负责通过异步方式
    /// 创建 wgpu 实例、适配器、设备和表面。
    ///
    /// # 参数
    ///
    /// - `surface` - wgpu 渲染表面
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `config` - 表面配置
    /// - `surface_info` - 渲染表面信息
    pub fn new_from_surface(
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        surface_info: SurfaceInfo,
    ) -> GResult<Self> {
        let sprite_pipeline = SpritePipeline::new(&device, config.format);
        let batch_sprite_pipeline = BatchSpritePipeline::new(&device, config.format);
        let transition_pipeline = TransitionPipeline::new(&device, config.format);
        let rounded_rect_pipeline = RoundedRectPipeline::new(&device, config.format);
        let ellipse_pipeline = EllipsePipeline::new(&device, config.format);

        let mut texture_cache = TextureCache::new();
        let white_pixel_data: [u8; 4] = [255, 255, 255, 255];
        let white_pixel_texture =
            texture_cache.create_texture_from_data(1, 1, &white_pixel_data, &device, &queue, "white_pixel").map_err(|e| {
                GError { kind: GErrorKind::Platform, message: format!("无法创建白色像素纹理: {}", e.message) }
            })?;

        let glyph_cache = GlyphCache::new();

        let buffer_size = std::cmp::max(
            std::mem::size_of::<SpriteUniforms>(),
            std::cmp::max(std::mem::size_of::<TransitionUniforms>(), std::cmp::max(std::mem::size_of::<RoundedRectUniforms>(), std::mem::size_of::<EllipseUniforms>())),
        );
        let uniform_pool = UniformPool::new(buffer_size);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            surface_info,
            texture_cache,
            glyph_cache,
            sprite_pipeline,
            batch_sprite_pipeline,
            transition_pipeline,
            rounded_rect_pipeline,
            ellipse_pipeline,
            should_close: false,
            pending_events: Vec::new(),
            frame_output: None,
            command_encoder: None,
            white_pixel_texture,
            uniform_pool,
        })
    }

    /// 推送窗口事件到内部缓冲区
    ///
    /// Web 平台通过此方法将 JavaScript 端的事件转换为引擎事件。
    ///
    /// # 参数
    ///
    /// - `event` - 窗口事件
    pub fn push_event(&mut self, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) {
            self.should_close = true;
        }
        self.pending_events.push(event);
    }
}

/// 带索引的绘制命令，用于 Z 排序时保持原始顺序
struct IndexedCommand {
    /// 原始命令索引
    index: usize,
    /// Z 层级值
    z_index: f32,
    /// 是否为过渡命令（过渡命令始终最后渲染）
    is_transition: bool,
}

/// 圆角矩形渲染项
///
/// 包含圆角矩形绘制所需的 uniform 绑定组和缓冲区。
struct RoundedRectRenderItem {
    /// uniform 绑定组
    uniform_bind_group: wgpu::BindGroup,
    /// uniform 缓冲区（用于帧间复用）
    uniform_buffer: wgpu::Buffer,
}

/// 椭圆渲染项
///
/// 包含椭圆绘制所需的 uniform 绑定组和缓冲区。
struct EllipseRenderItem {
    /// uniform 绑定组
    uniform_bind_group: wgpu::BindGroup,
    /// uniform 缓冲区（用于帧间复用）
    uniform_buffer: wgpu::Buffer,
}

impl Renderer for WgpuRenderer {
    fn begin_frame(&mut self) -> GResult<()> {
        self.uniform_pool.recycle_frame();
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(GError { kind: GErrorKind::Platform, message: "获取当前帧纹理超时".to_string() });
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(GError { kind: GErrorKind::Platform, message: "获取当前帧纹理: 窗口被遮挡".to_string() });
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Err(GError { kind: GErrorKind::Platform, message: "获取当前帧纹理: 表面已过时".to_string() });
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(GError { kind: GErrorKind::Platform, message: "获取当前帧纹理: 表面已丢失".to_string() });
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(GError { kind: GErrorKind::Platform, message: "获取当前帧纹理: 验证错误".to_string() });
            }
        };
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
        let view_projection = Self::compute_view_projection(surface_width, surface_height, context.camera());

        // 阶段 1：预光栅化所有文本字形
        {
            for cmd in context.commands() {
                if let DrawCommand::Text { text, font_size, .. } = cmd {
                    let font = match self.glyph_cache.font() {
                        Some(f) => f.clone(),
                        None => continue,
                    };

                    let px_scale = ab_glyph::PxScale { x: *font_size, y: *font_size };

                    for c in text.chars() {
                        let glyph_id = font.glyph_id(c);
                        let glyph = glyph_id.with_scale(px_scale);
                        let _ = self.glyph_cache.get_or_rasterize(glyph, &self.device, &self.queue, &mut self.texture_cache);
                    }
                }
            }
        }

        let commands = context.commands();
        let mut indexed: Vec<IndexedCommand> = commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let (z, is_t) = match cmd {
                    DrawCommand::Sprite { transform, .. } => (transform.z_index, false),
                    DrawCommand::Text { .. } => (0.0, false),
                    DrawCommand::Rect { .. } => (0.0, false),
                    DrawCommand::Line { .. } => (0.0, false),
                    DrawCommand::Circle { .. } => (0.0, false),
                    DrawCommand::Ellipse { .. } => (0.0, false),
                    DrawCommand::Transition { .. } => (f32::MAX, true),
                };
                IndexedCommand { index: i, z_index: z, is_transition: is_t }
            })
            .collect();

        indexed.sort_by(|a, b| {
            if a.is_transition != b.is_transition {
                if a.is_transition { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less }
            }
            else {
                a.z_index.partial_cmp(&b.z_index).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        // 阶段 3：收集精灵批次和过渡渲染项
        let mut sprite_batcher = SpriteBatcher::new();
        let mut transition_items: Vec<RenderItem> = Vec::new();
        let mut rounded_rect_items: Vec<RoundedRectRenderItem> = Vec::new();
        let mut ellipse_items: Vec<EllipseRenderItem> = Vec::new();

        for ic in &indexed {
            let cmd = &commands[ic.index];
            match cmd {
                DrawCommand::Sprite { texture_id, transform, size, tint, .. } => {
                    let mvp = Self::compute_sprite_mvp(transform, *size, &view_projection);
                    sprite_batcher.push(*texture_id, mvp, [tint.r, tint.g, tint.b, tint.a], [0.0, 0.0, 1.0, 1.0]);
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

                            let mvp = Self::compute_sprite_mvp(&glyph_transform, glyph_info.size, &view_projection);
                            let uv_transform = [
                                glyph_info.uv_rect[0],
                                glyph_info.uv_rect[1],
                                glyph_info.uv_rect[2] - glyph_info.uv_rect[0],
                                glyph_info.uv_rect[3] - glyph_info.uv_rect[1],
                            ];
                            sprite_batcher.push(glyph_info.texture_id, mvp, [color.r, color.g, color.b, color.a], uv_transform);
                        }

                        cursor_x += advance;
                    }
                }
                DrawCommand::Rect { rect, color, corner_radius } => {
                    if *corner_radius > 0.0 {
                        let clamped_radius = corner_radius.min(rect.width.min(rect.height) * 0.5);
                        let transform = Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
                        let mvp = Self::compute_sprite_mvp(&transform, [rect.width, rect.height], &view_projection);
                        let uniforms = RoundedRectUniforms {
                            mvp,
                            rect_size: [rect.width, rect.height, clamped_radius, 0.0],
                            color: [color.r, color.g, color.b, color.a],
                        };
                        let uniform_buffer =
                            self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                        let uniform_bind_group =
                            self.rounded_rect_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                        rounded_rect_items.push(RoundedRectRenderItem { uniform_bind_group, uniform_buffer });
                    } else {
                        let transform = Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
                        let mvp = Self::compute_sprite_mvp(&transform, [rect.width, rect.height], &view_projection);
                        sprite_batcher.push(
                            self.white_pixel_texture,
                            mvp,
                            [color.r, color.g, color.b, color.a],
                            [0.0, 0.0, 1.0, 1.0],
                        );
                    }
                }
                DrawCommand::Line { start, end, color, width } => {
                    let dx = end[0] - start[0];
                    let dy = end[1] - start[1];
                    let length = (dx * dx + dy * dy).sqrt();
                    if length < 0.001 {
                        continue;
                    }
                    let angle = dy.atan2(dx);
                    let transform = Transform { position: *start, scale: [1.0, 1.0], rotation: angle, z_index: 0.0 };
                    let mvp = Self::compute_sprite_mvp(&transform, [length, *width], &view_projection);
                    sprite_batcher.push(
                        self.white_pixel_texture,
                        mvp,
                        [color.r, color.g, color.b, color.a],
                        [0.0, 0.0, 1.0, 1.0],
                    );
                }
                DrawCommand::Circle { center, radius, color, filled, border_width, border_color } => {
                    let size = [radius * 2.0, radius * 2.0];
                    let transform = Transform {
                        position: [center[0] - radius, center[1] - radius],
                        scale: [1.0, 1.0],
                        rotation: 0.0,
                        z_index: 0.0,
                    };
                    let mvp = Self::compute_sprite_mvp(&transform, size, &view_projection);
                    let effective_border_width = if *filled { 0.0 } else { *border_width };
                    let uniforms = EllipseUniforms {
                        mvp,
                        ellipse_params: [0.0, 0.0, size[0], size[1]],
                        fill_color: [color.r, color.g, color.b, color.a],
                        border_params: [effective_border_width, border_color[0], border_color[1], border_color[2]],
                    };
                    let uniform_buffer =
                        self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                    let uniform_bind_group =
                        self.ellipse_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    ellipse_items.push(EllipseRenderItem { uniform_bind_group, uniform_buffer });
                }
                DrawCommand::Ellipse { center, radii, color, filled, border_width, border_color } => {
                    let size = [radii[0] * 2.0, radii[1] * 2.0];
                    let transform = Transform {
                        position: [center[0] - radii[0], center[1] - radii[1]],
                        scale: [1.0, 1.0],
                        rotation: 0.0,
                        z_index: 0.0,
                    };
                    let mvp = Self::compute_sprite_mvp(&transform, size, &view_projection);
                    let effective_border_width = if *filled { 0.0 } else { *border_width };
                    let uniforms = EllipseUniforms {
                        mvp,
                        ellipse_params: [0.0, 0.0, size[0], size[1]],
                        fill_color: [color.r, color.g, color.b, color.a],
                        border_params: [effective_border_width, border_color[0], border_color[1], border_color[2]],
                    };
                    let uniform_buffer =
                        self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                    let uniform_bind_group =
                        self.ellipse_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    ellipse_items.push(EllipseRenderItem { uniform_bind_group, uniform_buffer });
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
                    let uniform_buffer =
                        self.uniform_pool.allocate(&self.device, &self.queue, bytemuck::cast_slice(&[uniforms]));
                    let uniform_bind_group = self.transition_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
                    let texture_bind_group = self
                        .texture_cache
                        .create_transition_bind_group(old_id, new_id, &self.device, self.transition_pipeline.texture_layout())
                        .ok_or_else(|| GError {
                            kind: GErrorKind::Asset, message: "无法创建过渡纹理绑定组".to_string()
                        })?;

                    transition_items.push(RenderItem {
                        item_type: RenderItemType::Transition,
                        uniform_bind_group,
                        texture_bind_group,
                        uniform_buffer,
                    });
                }
            }
        }

        let sprite_batches = sprite_batcher.flush();

        // 阶段 4：记录渲染通道
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

        let scissor_rect = context.clip_rect().map(|r| ScissorRect {
            x: r.x.max(0.0) as u32,
            y: r.y.max(0.0) as u32,
            w: r.width.max(0.0) as u32,
            h: r.height.max(0.0) as u32,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Some(rect) = &scissor_rect {
            render_pass.set_scissor_rect(rect.x, rect.y, rect.w, rect.h);
        }

        // 绘制精灵批次
        for SpriteBatch { texture_id, instances } in &sprite_batches {
            let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sprite_instance_buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let texture_bind_group = self
                .texture_cache
                .create_bind_group(*texture_id, &self.device, self.batch_sprite_pipeline.texture_layout())
                .ok_or_else(|| GError {
                    kind: GErrorKind::Asset,
                    message: format!("无法创建精灵纹理绑定组，纹理 ID: {:?}", texture_id),
                })?;

            render_pass.set_pipeline(self.batch_sprite_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.batch_sprite_pipeline.vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(self.batch_sprite_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
        }

        // 绘制过渡动画
        for item in &transition_items {
            render_pass.set_pipeline(self.transition_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.sprite_pipeline.vertex_buffer().slice(..));
            render_pass.set_index_buffer(self.sprite_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &item.texture_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        // 绘制圆角矩形
        for item in &rounded_rect_items {
            render_pass.set_pipeline(self.rounded_rect_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.rounded_rect_pipeline.vertex_buffer().slice(..));
            render_pass.set_index_buffer(self.rounded_rect_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        for item in &ellipse_items {
            render_pass.set_pipeline(self.ellipse_pipeline.pipeline());
            render_pass.set_vertex_buffer(0, self.ellipse_pipeline.vertex_buffer().slice(..));
            render_pass.set_index_buffer(self.ellipse_pipeline.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        drop(render_pass);

        for RenderItem { uniform_buffer, .. } in transition_items {
            self.uniform_pool.mark_used(uniform_buffer);
        }

        for RoundedRectRenderItem { uniform_buffer, .. } in rounded_rect_items {
            self.uniform_pool.mark_used(uniform_buffer);
        }

        for EllipseRenderItem { uniform_buffer, .. } in ellipse_items {
            self.uniform_pool.mark_used(uniform_buffer);
        }

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

    fn reload_texture(&mut self, path: &str) -> GResult<()> {
        let path_ref = std::path::Path::new(path);
        self.texture_cache.load_texture(path_ref, &self.device, &self.queue)?;
        Ok(())
    }
}
