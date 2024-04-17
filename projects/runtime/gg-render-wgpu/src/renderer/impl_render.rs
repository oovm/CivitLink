use std::path::Path;

use ab_glyph::{Font, ScaleFont};
use gg_core::{GError, GErrorKind, GResult};
use gg_render::{DrawCommand, RenderContext, Renderer, SurfaceInfo, TextureId, Transform};
use wgpu::util::DeviceExt;

use crate::{
    WgpuRenderer,
    pipeline::{EllipseUniforms, RenderItem, RenderItemType, RoundedRectUniforms, TransitionUniforms},
    sprite_batch::SpriteBatch,
};

use super::{EllipseRenderItem, IndexedCommand, RoundedRectRenderItem, ScissorRect};

impl Renderer for WgpuRenderer {
    fn begin_frame(&mut self) -> GResult<()> {
        self.uniform_pool.recycle_frame();
        self.instance_pool.recycle_frame();
        self.profiler.begin_frame();
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(GError { kind: GErrorKind::Platform, message: "获取当前帧纹理超时".to_string() });
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(GError {
                    kind: GErrorKind::Platform, message: "获取当前帧纹理: 窗口被遮挡".to_string()
                });
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Err(GError {
                    kind: GErrorKind::Platform, message: "获取当前帧纹理: 表面已过时".to_string()
                });
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(GError {
                    kind: GErrorKind::Platform, message: "获取当前帧纹理: 表面已丢失".to_string()
                });
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
        self.profiler.end_frame();
        Ok(())
    }

    fn draw(&mut self, context: &RenderContext) -> GResult<()> {
        let surface_width = context.surface_width();
        let surface_height = context.surface_height();
        let view_projection = Self::compute_view_projection(surface_width, surface_height, context.camera());

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
                    DrawCommand::CustomShader { z_index, .. } => (*z_index, false),
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

        let mut sprite_batcher = crate::sprite_batch::SpriteBatcher::new();
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
                        let transform =
                            Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
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
                    }
                    else {
                        let transform =
                            Transform { position: [rect.x, rect.y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };
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
                    let uniform_bind_group = self.ellipse_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
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
                    let uniform_bind_group = self.ellipse_pipeline.create_uniform_bind_group(&self.device, &uniform_buffer);
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
                DrawCommand::CustomShader { .. } => {}
            }
        }

        let sprite_batches = sprite_batcher.flush();

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

    fn destroy_render_target(&mut self, texture_id: TextureId) -> GResult<()> {
        self.texture_cache.remove(texture_id);
        Ok(())
    }

    fn draw_pipeline(&mut self, pipeline: &mut dyn gg_render::RenderPipeline, context: &mut RenderContext) -> GResult<()> {
        let pass_contexts = pipeline.execute(context);
        for pass_ctx in &pass_contexts {
            self.draw(pass_ctx)?;
        }
        Ok(())
    }
}
