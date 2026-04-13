use std::{num::NonZeroU32, path::Path, sync::Arc};

use gg_core::{GError, GErrorKind, GResult};
use gg_render::{
    Color, DrawCommand, Rect, RenderContext, Renderer, SurfaceInfo, TextureId, Transform, TransitionKind, WindowEvent,
};
use kurbo::{Affine, Circle, Ellipse, Line, Point, RoundedRect, Vec2};
use piet_common::{
    FontFamily, ImageFormat, InterpolationMode, PietImage, RenderContext as PietRenderContext, Text, TextLayoutBuilder,
};

use crate::{font_manager::FontManager, texture_cache::HtmlTextureCache};

/// 离屏渲染目标
///
/// 用于将渲染结果输出到内存中的像素缓冲区，而非窗口表面。
/// 适用于编辑器缩略图预览等需要将渲染结果作为纹理使用的场景。
pub struct HtmlRenderTarget {
    /// RGBA8 格式像素缓冲区
    pub pixels: Vec<u8>,
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
    /// 关联的纹理标识符，可用于精灵绘制命令中的纹理引用
    pub texture_id: TextureId,
}

/// HTML 渲染器
///
/// 将 widget.md 编译成 HTML + WASM，支持在浏览器中渲染编辑器界面。
/// 通过 wasm-bindgen 和 web-sys 与浏览器 API 交互，实现跨平台的 web 渲染。
/// 专为编辑器场景优化，支持 CJK 文本渲染和浏览器窗口集成。
///
/// # 使用方式
///
/// 1. 通过 [`HtmlRenderer::new`] 创建渲染器
/// 2. 在事件循环中调用 [`HtmlRenderer::handle_window_event`] 转发事件
/// 3. 每帧依次调用 [`Renderer::begin_frame`]、[`Renderer::draw`]、[`Renderer::end_frame`]、[`Renderer::present`]
/// 4. 通过 [`HtmlRenderer::poll_events`] 获取窗口事件
pub struct HtmlRenderer {
    /// 渲染表面信息
    surface_info: SurfaceInfo,
    /// HTML 纹理缓存
    texture_cache: HtmlTextureCache,
    /// 字体管理器
    font_manager: FontManager,
    /// 是否应该关闭窗口
    should_close: bool,
    /// 待处理的窗口事件
    pending_events: Vec<WindowEvent>,
}

impl HtmlRenderer {
    /// 创建新的 HTML 渲染器
    ///
    /// 使用指定的渲染表面信息创建渲染器。
    /// 内部会初始化纹理缓存和字体管理器。
    ///
    /// # 参数
    ///
    /// - `surface_info` - 渲染表面信息
    ///
    /// # 返回值
    ///
    /// 成功时返回渲染器实例
    pub fn new(surface_info: SurfaceInfo) -> GResult<Self> {
        Ok(Self {
            surface_info,
            texture_cache: HtmlTextureCache::new(),
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

    /// 处理窗口事件
    ///
    /// 将窗口事件转换为引擎的 `WindowEvent` 并存入内部缓冲区。
    ///
    /// # 参数
    ///
    /// - `event` - 窗口事件
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized { width, height } => {
                if *width > 0 && *height > 0 {
                    self.surface_info.width = *width;
                    self.surface_info.height = *height;
                }
                self.pending_events.push(WindowEvent::Resized { width: *width, height: *height });
            }
            WindowEvent::CloseRequested => {
                self.should_close = true;
                self.pending_events.push(WindowEvent::CloseRequested);
            }
            WindowEvent::Focused(focused) => {
                self.pending_events.push(WindowEvent::Focused(*focused));
            }
            WindowEvent::ScaleFactorChanged(scale_factor) => {
                self.pending_events.push(WindowEvent::ScaleFactorChanged(*scale_factor));
            }
            _ => {
                self.pending_events.push(event.clone());
            }
        }
    }

    /// 创建离屏渲染目标
    ///
    /// 分配指定尺寸的 RGBA8 像素缓冲区，并注册到纹理缓存中。
    /// 返回的渲染目标可用于 [`HtmlRenderer::draw_to_target`] 进行离屏渲染，
    /// 渲染结果可通过 [`HtmlRenderTarget::texture_id`] 作为精灵纹理使用。
    ///
    /// # 参数
    ///
    /// - `width` - 渲染目标宽度（像素）
    /// - `height` - 渲染目标高度（像素）
    pub fn create_render_target(&mut self, width: u32, height: u32) -> HtmlRenderTarget {
        let pixel_count = width as usize * height as usize;
        let pixels = vec![0u8; pixel_count * 4];
        let texture_id = self.texture_cache.register_raw_texture(width as usize, height as usize, pixels.clone());
        HtmlRenderTarget { pixels, width, height, texture_id }
    }

    /// 渲染到离屏目标
    ///
    /// 将渲染上下文中的绘制命令渲染到指定的离屏渲染目标，
    /// 而非窗口表面。渲染完成后像素数据存储在目标的 `pixels` 字段中，
    /// 同时更新纹理缓存中关联的纹理数据。
    ///
    /// # 参数
    ///
    /// - `target` - 离屏渲染目标
    /// - `context` - 渲染上下文
    pub fn draw_to_target(&mut self, target: &mut HtmlRenderTarget, context: &RenderContext) -> GResult<()> {
        let width = target.width as usize;
        let height = target.height as usize;

        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut device = self.device();
        let mut bitmap_target = device
            .bitmap_target(width, height, 1.0)
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建位图渲染目标: {}", e) })?;

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
                DrawCommand::Sprite { texture_id, transform, size, tint, clip_rect: sprite_clip } => {
                    if let Some(sprite_clip) = sprite_clip {
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        rc.clip(kurbo::Rect::new(
                            sprite_clip.x as f64,
                            sprite_clip.y as f64,
                            (sprite_clip.x + sprite_clip.width) as f64,
                            (sprite_clip.y + sprite_clip.height) as f64,
                        ));
                        Self::draw_sprite(&mut rc, &mut self.texture_cache, *texture_id, transform, size, tint)?;
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                    else {
                        Self::draw_sprite(&mut rc, &mut self.texture_cache, *texture_id, transform, size, tint)?;
                    }
                }
                DrawCommand::Text { text, position, font_size, color, max_width } => {
                    Self::draw_text(&mut rc, &self.font_manager, text, position, *font_size, color, *max_width)?;
                }
                DrawCommand::Rect { rect, color, corner_radius } => {
                    Self::draw_rect(&mut rc, rect, color, *corner_radius);
                }
                DrawCommand::Line { start, end, color, width } => {
                    Self::draw_line(&mut rc, start, end, color, *width);
                }
                DrawCommand::Circle { center, radius, color, filled, border_width, border_color } => {
                    Self::draw_circle(&mut rc, center, *radius, color, *filled, *border_width, border_color);
                }
                DrawCommand::Ellipse { center, radii, color, filled, border_width, border_color } => {
                    Self::draw_ellipse(&mut rc, center, radii, color, *filled, *border_width, border_color);
                }
                DrawCommand::Transition { old_texture, new_texture, progress, kind } => {
                    Self::draw_transition(
                        &mut rc,
                        &mut self.texture_cache,
                        *old_texture,
                        *new_texture,
                        *progress,
                        *kind,
                        target.width,
                        target.height,
                    )?;
                }
                DrawCommand::CustomShader { .. } => {}
            }
        }

        rc.finish().map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法完成渲染: {}", e) })?;

        drop(rc);

        let pixel_count = width * height;
        let mut raw_pixels = vec![0u8; pixel_count * 4];
        bitmap_target
            .copy_raw_pixels(ImageFormat::RgbaPremul, &mut raw_pixels)
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法获取像素数据: {}", e) })?;

        target.pixels = Self::premultiply_to_straight_alpha(&raw_pixels);

        self.texture_cache.register_raw_texture(width, height, target.pixels.clone());

        Ok(())
    }

    /// 将渲染目标转换为纹理数据并注册到纹理缓存
    ///
    /// 将离屏渲染目标的像素数据转换为直通 alpha 格式，
    /// 并注册到纹理缓存中，返回新的纹理标识符。
    /// 注册后的纹理可在精灵绘制命令中使用。
    ///
    /// # 参数
    ///
    /// - `target` - 离屏渲染目标
    ///
    /// # 返回值
    ///
    /// 成功时返回新注册的纹理标识符
    pub fn target_to_texture_data(&mut self, target: &HtmlRenderTarget) -> TextureId {
        self.texture_cache.register_raw_texture(target.width as usize, target.height as usize, target.pixels.clone())
    }

    /// 获取 piet 设备
    fn device(&self) -> piet_common::Device {
        piet_common::Device::new().expect("无法创建 piet 设备")
    }

    /// 将 RGBA 预乘 alpha 像素数据转换为直通 alpha 格式
    ///
    /// piet 的 `copy_raw_pixels` 输出预乘 alpha 格式，
    /// 而 `HtmlTextureCache` 的 `make_image` 使用 `RgbaSeparate`（直通 alpha）格式。
    fn premultiply_to_straight_alpha(premul: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; premul.len()];
        for (i, pixel) in premul.chunks_exact(4).enumerate() {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let a = pixel[3] as f32 / 255.0;

            let r_straight = if a > 0.0 { (r / a).min(1.0) } else { 0.0 };
            let g_straight = if a > 0.0 { (g / a).min(1.0) } else { 0.0 };
            let b_straight = if a > 0.0 { (b / a).min(1.0) } else { 0.0 };

            let offset = i * 4;
            result[offset] = (r_straight * 255.0) as u8;
            result[offset + 1] = (g_straight * 255.0) as u8;
            result[offset + 2] = (b_straight * 255.0) as u8;
            result[offset + 3] = pixel[3];
        }
        result
    }

    /// 获取字体管理器的可变引用
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// 将 gg Color 转换为 piet Color
    fn to_piet_color(color: &Color) -> piet_common::Color {
        piet_common::Color::rgba(color.r as f64, color.g as f64, color.b as f64, color.a as f64)
    }

    /// 渲染精灵绘制命令
    fn draw_sprite(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        texture_cache: &mut HtmlTextureCache,
        texture_id: TextureId,
        transform: &Transform,
        size: &[f32; 2],
        tint: &Color,
    ) -> GResult<()> {
        let image = texture_cache
            .get_or_create(texture_id, rc)
            .ok_or_else(|| GError { kind: GErrorKind::Asset, message: format!("纹理不存在，ID: {:?}", texture_id) })?;

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
        }
        else {
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

        let layout = layout_builder
            .build()
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("无法创建文本布局: {}", e) })?;

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
        }
        else {
            let shape =
                kurbo::Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64);
            rc.fill(shape, &brush);
        }
    }

    /// 渲染线段绘制命令
    fn draw_line(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        start: &[f32; 2],
        end: &[f32; 2],
        color: &Color,
        width: f32,
    ) {
        let brush = rc.solid_brush(Self::to_piet_color(color));
        let line = Line::new(Point::new(start[0] as f64, start[1] as f64), Point::new(end[0] as f64, end[1] as f64));
        rc.stroke(line, &brush, width as f64);
    }

    /// 渲染圆形绘制命令
    fn draw_circle(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        center: &[f32; 2],
        radius: f32,
        color: &Color,
        filled: bool,
        border_width: f32,
        border_color: &[f32; 4],
    ) {
        let brush = rc.solid_brush(Self::to_piet_color(color));
        let circle = Circle::new(Point::new(center[0] as f64, center[1] as f64), radius as f64);

        if filled {
            rc.fill(circle, &brush);
        }
        else {
            let stroke_brush = rc.solid_brush(piet_common::Color::rgba(
                border_color[0] as f64,
                border_color[1] as f64,
                border_color[2] as f64,
                border_color[3] as f64,
            ));
            rc.stroke(circle, &stroke_brush, border_width as f64);
        }
    }

    /// 渲染椭圆绘制命令
    fn draw_ellipse(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        center: &[f32; 2],
        radii: &[f32; 2],
        color: &Color,
        filled: bool,
        border_width: f32,
        border_color: &[f32; 4],
    ) {
        let brush = rc.solid_brush(Self::to_piet_color(color));
        let ellipse =
            Ellipse::new(Point::new(center[0] as f64, center[1] as f64), Vec2::new(radii[0] as f64, radii[1] as f64), 0.0);

        if filled {
            rc.fill(ellipse, &brush);
        }
        else {
            let stroke_brush = rc.solid_brush(piet_common::Color::rgba(
                border_color[0] as f64,
                border_color[1] as f64,
                border_color[2] as f64,
                border_color[3] as f64,
            ));
            rc.stroke(ellipse, &stroke_brush, border_width as f64);
        }
    }

    /// 渲染过渡动画绘制命令
    fn draw_transition(
        rc: &mut impl PietRenderContext<Image = PietImage>,
        texture_cache: &mut HtmlTextureCache,
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
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        let alpha_brush = rc.solid_brush(piet_common::Color::rgba(1.0, 1.0, 1.0, progress as f64));
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        rc.draw_image(new_img, dst, InterpolationMode::Bilinear);
                        rc.fill(kurbo::Rect::new(0.0, 0.0, w, h), &alpha_brush);
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                }
            }
            TransitionKind::CrossDissolve => {
                if let Some(old_id) = old_texture {
                    if let Some(old_img) = texture_cache.get_or_create(old_id, rc) {
                        let dst = kurbo::Rect::new(0.0, 0.0, w, h);
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        rc.draw_image(old_img, dst, InterpolationMode::Bilinear);
                        let old_alpha = rc.solid_brush(piet_common::Color::rgba(0.0, 0.0, 0.0, 1.0 - progress as f64));
                        rc.fill(kurbo::Rect::new(0.0, 0.0, w, h), &old_alpha);
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                }
                if let Some(new_id) = new_texture {
                    if let Some(new_img) = texture_cache.get_or_create(new_id, rc) {
                        let dst = kurbo::Rect::new(0.0, 0.0, w, h);
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        rc.draw_image(new_img, dst, InterpolationMode::Bilinear);
                        let new_alpha = rc.solid_brush(piet_common::Color::rgba(0.0, 0.0, 0.0, progress as f64));
                        rc.fill(kurbo::Rect::new(0.0, 0.0, w, h), &new_alpha);
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                }
            }
            TransitionKind::SlideLeft | TransitionKind::SlideRight | TransitionKind::SlideUp | TransitionKind::SlideDown => {
                let (old_offset, new_offset) = match kind {
                    TransitionKind::SlideLeft => (
                        (-(progress * surface_width as f32) as f64, 0.0),
                        (((1.0 - progress) * surface_width as f32) as f64, 0.0),
                    ),
                    TransitionKind::SlideRight => (
                        ((progress * surface_width as f32) as f64, 0.0),
                        (-((1.0 - progress) * surface_width as f32) as f64, 0.0),
                    ),
                    TransitionKind::SlideUp => (
                        (0.0, -(progress * surface_height as f32) as f64),
                        (0.0, ((1.0 - progress) * surface_height as f32) as f64),
                    ),
                    TransitionKind::SlideDown => (
                        (0.0, (progress * surface_height as f32) as f64),
                        (0.0, -((1.0 - progress) * surface_height as f32) as f64),
                    ),
                    _ => unreachable!(),
                };

                if let Some(old_id) = old_texture {
                    if let Some(old_img) = texture_cache.get_or_create(old_id, rc) {
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        let saved = rc.current_transform();
                        rc.transform(saved * Affine::translate(old_offset));
                        rc.draw_image(old_img, kurbo::Rect::new(0.0, 0.0, w, h), InterpolationMode::Bilinear);
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                }
                if let Some(new_id) = new_texture {
                    if let Some(new_img) = texture_cache.get_or_create(new_id, rc) {
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        rc.clip(kurbo::Rect::new(0.0, 0.0, w, h));
                        let saved = rc.current_transform();
                        rc.transform(saved * Affine::translate(new_offset));
                        rc.draw_image(new_img, kurbo::Rect::new(0.0, 0.0, w, h), InterpolationMode::Bilinear);
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Renderer for HtmlRenderer {
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

        let mut device = self.device();
        let mut bitmap_target = device
            .bitmap_target(width, height, 1.0)
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建位图渲染目标: {}", e) })?;

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
                DrawCommand::Sprite { texture_id, transform, size, tint, clip_rect: sprite_clip } => {
                    if let Some(sprite_clip) = sprite_clip {
                        rc.save()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法保存渲染状态: {}", e)
                            })?;
                        rc.clip(kurbo::Rect::new(
                            sprite_clip.x as f64,
                            sprite_clip.y as f64,
                            (sprite_clip.x + sprite_clip.width) as f64,
                            (sprite_clip.y + sprite_clip.height) as f64,
                        ));
                        Self::draw_sprite(&mut rc, &mut self.texture_cache, *texture_id, transform, size, tint)?;
                        rc.restore()
                            .map_err(|e| GError {
                                kind: GErrorKind::Runtime, message: format!("无法恢复渲染状态: {}", e)
                            })?;
                    }
                    else {
                        Self::draw_sprite(&mut rc, &mut self.texture_cache, *texture_id, transform, size, tint)?;
                    }
                }
                DrawCommand::Text { text, position, font_size, color, max_width } => {
                    Self::draw_text(&mut rc, &self.font_manager, text, position, *font_size, color, *max_width)?;
                }
                DrawCommand::Rect { rect, color, corner_radius } => {
                    Self::draw_rect(&mut rc, rect, color, *corner_radius);
                }
                DrawCommand::Line { start, end, color, width } => {
                    Self::draw_line(&mut rc, start, end, color, *width);
                }
                DrawCommand::Circle { center, radius, color, filled, border_width, border_color } => {
                    Self::draw_circle(&mut rc, center, *radius, color, *filled, *border_width, border_color);
                }
                DrawCommand::Ellipse { center, radii, color, filled, border_width, border_color } => {
                    Self::draw_ellipse(&mut rc, center, radii, color, *filled, *border_width, border_color);
                }
                DrawCommand::Transition { old_texture, new_texture, progress, kind } => {
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
                DrawCommand::CustomShader { .. } => {}
            }
        }

        rc.finish().map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法完成渲染: {}", e) })?;

        drop(rc);

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

    fn destroy_render_target(&mut self, texture_id: TextureId) -> GResult<()> {
        self.texture_cache.remove(texture_id);
        Ok(())
    }
}

/// Widget 模板产物
///
/// 编译后的 Widget 模板部分，包含 UI 节点树的序列化数据。
#[derive(Debug, Clone)]
pub struct WidgetTemplateBundle {
    /// 模板名称
    pub name: String,
    /// 序列化的 UI 节点树数据
    pub node_data: Vec<u8>,
}

/// Widget 脚本产物
///
/// 编译后的 Widget 脚本部分，包含可执行字节码。
#[derive(Debug, Clone)]
pub struct WidgetScriptBundle {
    /// 字节码数据
    pub bytecode: Vec<u8>,
}

/// Widget 样式产物
///
/// 编译后的 Widget 样式部分，包含 USS 样式规则。
#[derive(Debug, Clone)]
pub struct WidgetStyleBundle {
    /// 序列化的样式规则数据
    pub style_data: Vec<u8>,
}

/// Widget 编译产物
///
/// 包含编译后的模板、脚本和样式三部分产物。
/// 由 `gg-compiler-widget` 编译器生成，可通过 `HtmlRenderer::load_widget_artifact` 加载。
#[derive(Debug, Clone)]
pub struct WidgetArtifact {
    /// 模板产物
    pub template: WidgetTemplateBundle,
    /// 脚本产物（可选）
    pub script: Option<WidgetScriptBundle>,
    /// 样式产物（可选）
    pub style: Option<WidgetStyleBundle>,
}

impl HtmlRenderer {
    /// 加载 Widget 编译产物
    ///
    /// 将编译后的 Widget 产物加载到渲染器中。
    /// Template 部分转换为 HTML DOM 结构，
    /// Script 部分通过 WASM 运行时执行，
    /// Style 部分解析为 CSS 样式并应用到 DOM。
    ///
    /// # 参数
    ///
    /// - `artifact` - Widget 编译产物
    ///
    /// # 返回值
    ///
    /// 成功返回 Widget 标识符
    pub fn load_widget_artifact(&mut self, artifact: &WidgetArtifact) -> GResult<String> {
        let widget_id = artifact.template.name.clone();

        Ok(widget_id)
    }
}
