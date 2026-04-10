use std::{num::NonZeroU32, path::Path, sync::Arc};

use gg_core::{GError, GErrorKind, GResult};
use gg_render::{Color, DrawCommand, Rect, RenderContext, Renderer, SurfaceInfo, TextureId, Transform, TransitionKind, WindowEvent};
use kurbo::{Affine, Circle, Ellipse, Line, Point, RoundedRect, Vec2};
use piet_common::{FontFamily, ImageFormat, InterpolationMode, PietImage, RenderContext as PietRenderContext, Text, TextLayoutBuilder};
use winit::{event::Event, event_loop::EventLoop, window::{Window, WindowAttributes}};

use crate::{font_manager::FontManager, texture_cache::NativeTextureCache};

/// 原生渲染器
///
/// 基于平台原生 2D 图形 API（Direct2D/CoreGraphics/Cairo）实现的渲染器，
/// 通过 piet-common 进行渲染，使用 softbuffer 将渲染结果呈现到 winit 窗口。
/// 专为编辑器场景优化，支持 CJK 文本渲染和原生窗口集成。
///
/// # 使用方式
///
/// 1. 通过 [`NativeRenderer::new`] 创建渲染器
/// 2. 在 winit 事件循环中调用 [`NativeRenderer::handle_window_event`] 转发事件
/// 3. 每帧依次调用 [`Renderer::begin_frame`]、[`Renderer::draw`]、[`Renderer::end_frame`]、[`Renderer::present`]
/// 4. 通过 [`NativeRenderer::poll_events`] 获取窗口事件
pub struct NativeRenderer {
    /// winit 窗口
    window: Arc<Window>,
    /// softbuffer 渲染表面，用于将像素数据呈现到窗口
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// piet 设备，用于创建位图渲染目标
    device: piet_common::Device,
    /// 渲染表面信息
    surface_info: SurfaceInfo,
    /// 原生纹理缓存
    texture_cache: NativeTextureCache,
    /// 字体管理器
    font_manager: FontManager,
    /// 是否应该关闭窗口
    should_close: bool,
    /// 待处理的窗口事件
    pending_events: Vec<WindowEvent>,
}

impl NativeRenderer {
    /// 创建新的原生渲染器
    ///
    /// 使用指定的 winit 事件循环和渲染表面信息创建渲染器。
    /// 内部会创建窗口、初始化 softbuffer 表面和 piet 设备。
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

        let context = softbuffer::Context::new(Arc::clone(&window))
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建 softbuffer 上下文: {}", e) })?;
        let surface = softbuffer::Surface::new(&context, Arc::clone(&window))
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建 softbuffer 表面: {}", e) })?;

        let device = piet_common::Device::new()
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建 piet 设备: {}", e) })?;

        Ok(Self {
            window,
            surface,
            device,
            surface_info,
            texture_cache: NativeTextureCache::new(),
            font_manager: FontManager::new(),
            should_close: false,
            pending_events: Vec::new(),
        })
    }

    /// 检查窗口是否应该关闭
    pub fn should_close(&self) -> bool {
        self.should_close
    }

    /// 轮询窗口事件
    ///
    /// 返回并清空内部事件缓冲区中的所有窗口事件。
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

    /// 获取字体管理器的可变引用
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// 将 gg Color 转换为 piet Color
    fn to_piet_color(color: &Color) -> piet_common::Color {
        piet_common::Color::rgba(color.r as f64, color.g as f64, color.b as f64, color.a as f64)
    }

    /// 将 RGBA 预乘像素数据转换为 softbuffer 所需的 u32 像素格式
    ///
    /// softbuffer 使用 0x00RRGGBB 格式，piet 输出 RGBA 预乘格式。
    /// 对于预乘 alpha 的像素，需要先反预乘再转换。
    fn convert_rgba_to_softbuffer(rgba_data: &[u8], width: usize, height: usize) -> Vec<u32> {
        let mut buffer = Vec::with_capacity(width * height);
        for pixel in rgba_data.chunks_exact(4) {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let a = pixel[3] as f32 / 255.0;

            let r_unmul = if a > 0.0 { (r / a).min(1.0) } else { 0.0 };
            let g_unmul = if a > 0.0 { (g / a).min(1.0) } else { 0.0 };
            let b_unmul = if a > 0.0 { (b / a).min(1.0) } else { 0.0 };

            let r_val = (r_unmul * 255.0) as u32;
            let g_val = (g_unmul * 255.0) as u32;
            let b_val = (b_unmul * 255.0) as u32;

            buffer.push((r_val << 16) | (g_val << 8) | b_val);
        }
        buffer
    }

    /// 渲染精灵绘制命令
    fn draw_sprite(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        texture_cache: &mut NativeTextureCache,
        texture_id: TextureId,
        transform: &Transform,
        size: &[f32; 2],
        tint: &Color,
    ) -> GResult<()> {
        let image = texture_cache.get_or_create(texture_id, rc).ok_or_else(|| GError {
            kind: GErrorKind::Asset,
            message: format!("纹理不存在，ID: {:?}", texture_id),
        })?;

        let current_transform = rc.current_transform();
        let sprite_transform = Affine::translate((transform.position[0] as f64, transform.position[1] as f64))
            * Affine::rotate(transform.rotation as f64)
            * Affine::scale_non_uniform(size[0] as f64 * transform.scale[0] as f64, size[1] as f64 * transform.scale[1] as f64);

        rc.transform(current_transform * sprite_transform);

        let dst_rect = kurbo::Rect::new(0.0, 0.0, size[0] as f64, size[1] as f64);
        rc.draw_image(image, dst_rect, InterpolationMode::Bilinear);

        if tint.a > 0.0 && (tint.r < 1.0 || tint.g < 1.0 || tint.b < 1.0 || tint.a < 1.0) {
            let tint_brush = rc.solid_brush(Self::to_piet_color(tint));
            rc.fill(dst_rect, &tint_brush);
        }

        rc.transform(current_transform * current_transform.inverse());

        Ok(())
    }

    /// 渲染文本绘制命令
    fn draw_text(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        font_manager: &FontManager,
        text: &str,
        position: &[f32; 2],
        font_size: f32,
        color: &Color,
        max_width: Option<f32>,
    ) -> GResult<()> {
        let font_family = if let Some(name) = font_manager.font_family() {
            rc.text().font_family(name).unwrap_or_default()
        } else {
            FontFamily::default()
        };

        let mut layout_builder = rc
            .text()
            .new_text_layout(text.to_string())
            .font(font_family, font_size as f64)
            .text_color(Self::to_piet_color(color));

        if let Some(max_w) = max_width {
            layout_builder = layout_builder.max_width(max_w as f64);
        }

        let layout = layout_builder.build().map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("无法创建文本布局: {}", e),
        })?;

        rc.draw_text(&layout, Point::new(position[0] as f64, position[1] as f64));

        Ok(())
    }

    /// 渲染矩形绘制命令
    fn draw_rect(rc: &mut impl PietRenderContext<Image = PietImage>, rect: &Rect, color: &Color, corner_radius: f32) {
        let brush = rc.solid_brush(Self::to_piet_color(color));

        if corner_radius > 0.0 {
            let rounded = RoundedRect::new(
                rect.x as f64,
                rect.y as f64,
                (rect.x + rect.width) as f64,
                (rect.y + rect.height) as f64,
                corner_radius as f64,
            );
            rc.fill(rounded, &brush);
        } else {
            let shape = kurbo::Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64);
            rc.fill(shape, &brush);
        }
    }

    /// 渲染线段绘制命令
    fn draw_line(rc: &mut impl PietRenderContext<Image = PietImage>, start: &[f32; 2], end: &[f32; 2], color: &Color, width: f32) {
        let brush = rc.solid_brush(Self::to_piet_color(color));
        let line = Line::new(
            Point::new(start[0] as f64, start[1] as f64),
            Point::new(end[0] as f64, end[1] as f64),
        );
        rc.stroke(line, &brush, width as f64);
    }

    /// 渲染圆形绘制命令
    fn draw_circle(rc: &mut impl PietRenderContext<Image = PietImage>, center: &[f32; 2], radius: f32, color: &Color, filled: bool) {
        let brush = rc.solid_brush(Self::to_piet_color(color));
        let circle = Circle::new(Point::new(center[0] as f64, center[1] as f64), radius as f64);

        if filled {
            rc.fill(circle, &brush);
        } else {
            rc.stroke(circle, &brush, 1.0);
        }
    }

    /// 渲染椭圆绘制命令
    fn draw_ellipse(rc: &mut impl PietRenderContext<Image = PietImage>, center: &[f32; 2], radii: &[f32; 2], color: &Color, filled: bool) {
        let brush = rc.solid_brush(Self::to_piet_color(color));
        let ellipse = Ellipse::new(
            Point::new(center[0] as f64, center[1] as f64),
            Vec2::new(radii[0] as f64, radii[1] as f64),
            0.0,
        );

        if filled {
            rc.fill(ellipse, &brush);
        } else {
            rc.stroke(ellipse, &brush, 1.0);
        }
    }

    /// 渲染过渡动画绘制命令
    fn draw_transition(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        texture_cache: &mut NativeTextureCache,
        old_texture: Option<TextureId>,
        new_texture: Option<TextureId>,
        progress: f32,
        kind: TransitionKind,
        surface_width: u32,
        surface_height: u32,
    ) -> GResult<()> {
        let w = surface_width as f64;
        let h = surface_height as f64;

        match kind {
            TransitionKind::Fade => {
                if let Some(old_id) = old_texture {
                    if let Some(old_img) = texture_cache.get_or_create(old_id, rc) {
                        let dst = kurbo::Rect::new(0.0, 0.0, w, h);
                        rc.draw_image(old_img, dst, InterpolationMode::Bilinear);
                    }
                }
                if let Some(new_id) = new_texture {
                    if let Some(new_img) = texture_cache.get_or_create(new_id, rc) {
                        let dst = kurbo::Rect::new(0.0, 0.0, w, h);
                        rc.save().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e) })?;
                        let alpha_brush = rc.solid_brush(piet_common::Color::rgba(1.0, 1.0, 1.0, progress as f64));
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        rc.draw_image(new_img, dst, InterpolationMode::Bilinear);
                        rc.fill(kurbo::Rect::new(0.0, 0.0, w, h), &alpha_brush);
                        rc.restore().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e) })?;
                    }
                }
            }
            TransitionKind::CrossDissolve => {
                if let Some(old_id) = old_texture {
                    if let Some(old_img) = texture_cache.get_or_create(old_id, rc) {
                        let dst = kurbo::Rect::new(0.0, 0.0, w, h);
                        rc.save().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e) })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        rc.draw_image(old_img, dst, InterpolationMode::Bilinear);
                        let old_alpha = rc.solid_brush(piet_common::Color::rgba(0.0, 0.0, 0.0, 1.0 - progress as f64));
                        rc.fill(kurbo::Rect::new(0.0, 0.0, w, h), &old_alpha);
                        rc.restore().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e) })?;
                    }
                }
                if let Some(new_id) = new_texture {
                    if let Some(new_img) = texture_cache.get_or_create(new_id, rc) {
                        let dst = kurbo::Rect::new(0.0, 0.0, w, h);
                        rc.save().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e) })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        rc.draw_image(new_img, dst, InterpolationMode::Bilinear);
                        let new_alpha = rc.solid_brush(piet_common::Color::rgba(0.0, 0.0, 0.0, progress as f64));
                        rc.fill(kurbo::Rect::new(0.0, 0.0, w, h), &new_alpha);
                        rc.restore().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e) })?;
                    }
                }
            }
            TransitionKind::SlideLeft | TransitionKind::SlideRight | TransitionKind::SlideUp | TransitionKind::SlideDown => {
                let (old_offset, new_offset) = match kind {
                    TransitionKind::SlideLeft => ((-(progress * surface_width as f32) as f64, 0.0), (((1.0 - progress) * surface_width as f32) as f64, 0.0)),
                    TransitionKind::SlideRight => (((progress * surface_width as f32) as f64, 0.0), (-((1.0 - progress) * surface_width as f32) as f64, 0.0)),
                    TransitionKind::SlideUp => ((0.0, -(progress * surface_height as f32) as f64), (0.0, ((1.0 - progress) * surface_height as f32) as f64)),
                    TransitionKind::SlideDown => ((0.0, (progress * surface_height as f32) as f64), (0.0, -((1.0 - progress) * surface_height as f32) as f64)),
                    _ => unreachable!(),
                };

                if let Some(old_id) = old_texture {
                    if let Some(old_img) = texture_cache.get_or_create(old_id, rc) {
                        rc.save().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e) })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        let saved = rc.current_transform();
                        rc.transform(saved * Affine::translate(old_offset));
                        rc.draw_image(old_img, kurbo::Rect::new(0.0, 0.0, w, h), InterpolationMode::Bilinear);
                        rc.restore().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e) })?;
                    }
                }
                if let Some(new_id) = new_texture {
                    if let Some(new_img) = texture_cache.get_or_create(new_id, rc) {
                        rc.save().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e) })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        let saved = rc.current_transform();
                        rc.transform(saved * Affine::translate(new_offset));
                        rc.draw_image(new_img, kurbo::Rect::new(0.0, 0.0, w, h), InterpolationMode::Bilinear);
                        rc.restore().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e) })?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Renderer for NativeRenderer {
    fn begin_frame(&mut self) -> GResult<()> {
        self.texture_cache.invalidate_piet_images();
        Ok(())
    }

    fn end_frame(&mut self) -> GResult<()> {
        Ok(())
    }

    fn draw(&mut self, context: &RenderContext) -> GResult<()> {
        let width = context.surface_width() as usize;
        let height = context.surface_height() as usize;

        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut bitmap_target = self.device.bitmap_target(width, height, 1.0).map_err(|e| GError {
            kind: GErrorKind::Platform,
            message: format!("无法创建位图渲染目标: {}", e),
        })?;

        let mut rc = bitmap_target.render_context();

        rc.clear(kurbo::Rect::new(0.0, 0.0, width as f64, height as f64), piet_common::Color::BLACK);

        if let Some(camera) = context.camera() {
            rc.transform(
                Affine::translate((-camera.position[0] as f64, -camera.position[1] as f64))
                    * Affine::rotate(camera.rotation as f64)
                    * Affine::scale(camera.zoom as f64),
            );
        }

        if let Some(clip_rect) = context.clip_rect() {
            rc.clip(kurbo::Rect::new(
                clip_rect.x as f64,
                clip_rect.y as f64,
                (clip_rect.x + clip_rect.width) as f64,
                (clip_rect.y + clip_rect.height) as f64,
            ));
        }

        for cmd in context.commands() {
            match cmd {
                DrawCommand::Sprite {
                    texture_id,
                    transform,
                    size,
                    tint,
                    clip_rect: sprite_clip,
                } => {
                    if let Some(sprite_clip) = sprite_clip {
                        rc.save().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e) })?;
                        rc.clip(kurbo::Rect::new(
                            sprite_clip.x as f64,
                            sprite_clip.y as f64,
                            (sprite_clip.x + sprite_clip.width) as f64,
                            (sprite_clip.y + sprite_clip.height) as f64,
                        ));
                        Self::draw_sprite(&mut rc, &mut self.texture_cache, *texture_id, transform, size, tint)?;
                        rc.restore().map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e) })?;
                    } else {
                        Self::draw_sprite(&mut rc, &mut self.texture_cache, *texture_id, transform, size, tint)?;
                    }
                }
                DrawCommand::Text {
                    text,
                    position,
                    font_size,
                    color,
                    max_width,
                } => {
                    Self::draw_text(&mut rc, &self.font_manager, text, position, *font_size, color, *max_width)?;
                }
                DrawCommand::Rect {
                    rect,
                    color,
                    corner_radius,
                } => {
                    Self::draw_rect(&mut rc, rect, color, *corner_radius);
                }
                DrawCommand::Line {
                    start,
                    end,
                    color,
                    width,
                } => {
                    Self::draw_line(&mut rc, start, end, color, *width);
                }
                DrawCommand::Circle {
                    center,
                    radius,
                    color,
                    filled,
                } => {
                    Self::draw_circle(&mut rc, center, *radius, color, *filled);
                }
                DrawCommand::Ellipse {
                    center,
                    radii,
                    color,
                    filled,
                } => {
                    Self::draw_ellipse(&mut rc, center, radii, color, *filled);
                }
                DrawCommand::Transition {
                    old_texture,
                    new_texture,
                    progress,
                    kind,
                } => {
                    Self::draw_transition(
                        &mut rc,
                        &mut self.texture_cache,
                        *old_texture,
                        *new_texture,
                        *progress,
                        *kind,
                        context.surface_width(),
                        context.surface_height(),
                    )?;
                }
            }
        }

        rc.finish().map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法完成渲染: {}", e) })?;

        drop(rc);

        let pixel_count = width * height;
        let mut raw_pixels = vec![0u8; pixel_count * 4];
        bitmap_target.copy_raw_pixels(ImageFormat::RgbaPremul, &mut raw_pixels).map_err(|e| GError {
            kind: GErrorKind::Platform,
            message: format!("无法获取像素数据: {}", e),
        })?;

        let surface_width = context.surface_width();
        let surface_height = context.surface_height();

        let buffer = Self::convert_rgba_to_softbuffer(&raw_pixels, width, height);

        if let (Some(w), Some(h)) = (NonZeroU32::new(surface_width), NonZeroU32::new(surface_height)) {
            let _ = self.surface.resize(w, h);
            let mut surface_buffer = self.surface.buffer_mut().map_err(|e| GError {
                kind: GErrorKind::Platform,
                message: format!("无法获取 softbuffer 缓冲区: {}", e),
            })?;
            surface_buffer.copy_from_slice(&buffer);
            surface_buffer.present().map_err(|e| GError {
                kind: GErrorKind::Platform,
                message: format!("无法呈现渲染结果: {}", e),
            })?;
        }

        Ok(())
    }

    fn present(&mut self) -> GResult<()> {
        Ok(())
    }

    fn load_texture(&mut self, path: &Path) -> GResult<TextureId> {
        self.texture_cache.load_texture(path)
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_info.width = width;
            self.surface_info.height = height;
        }
    }

    fn surface_info(&self) -> &SurfaceInfo {
        &self.surface_info
    }

    fn reload_texture(&mut self, path: &str) -> GResult<()> {
        self.texture_cache.reload_texture(path)
    }
}
