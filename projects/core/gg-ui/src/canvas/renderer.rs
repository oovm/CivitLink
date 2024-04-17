//! 游戏UI渲染系统模块
//!
//! 实现游戏UI的渲染功能，包括Canvas渲染、批处理等

use super::components::{Canvas, CanvasRenderMode, Graphic, Image, RectTransform, Text};
use gg_ecs::{Entity, World};
use gg_render::{Color, DrawCommand, RenderContext, TextureId, Transform};

/// Canvas渲染器
pub trait CanvasRenderer: Send + Sync {
    /// 渲染Canvas
    fn render(&self, world: &mut World) -> gg_error::GResult<()>;
}

/// 基础Canvas渲染器
pub struct BasicCanvasRenderer {
    /// 渲染上下文
    render_context: RenderContext,
    /// 屏幕宽度
    screen_width: u32,
    /// 屏幕高度
    screen_height: u32,
}

impl BasicCanvasRenderer {
    /// 创建新的基础Canvas渲染器
    pub fn new(width: u32, height: u32) -> Self {
        Self { render_context: RenderContext::new(width, height), screen_width: width, screen_height: height }
    }

    /// 设置屏幕大小
    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
        self.render_context = RenderContext::new(width, height);
    }

    /// 获取渲染上下文
    pub fn get_render_context(&mut self) -> &mut RenderContext {
        &mut self.render_context
    }

    /// 计算世界坐标到屏幕坐标
    fn world_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        (x, self.screen_height as f32 - y)
    }

    /// 计算屏幕坐标到世界坐标
    fn screen_to_world(&self, x: f32, y: f32) -> (f32, f32) {
        (x, self.screen_height as f32 - y)
    }

    /// 渲染UI元素
    fn render_ui_element(&mut self, entity: Entity, world: &mut World) {
        let ((x, y), (width, height), children, image, text, graphic) = {
            let Some(rect_transform) = world.get_component::<RectTransform>(entity)
            else {
                return;
            };
            let Some(graphic) = world.get_component::<Graphic>(entity)
            else {
                return;
            };
            if !graphic.visible {
                return;
            }
            (
                self.calculate_position(rect_transform),
                self.calculate_size(rect_transform),
                rect_transform.children.clone(),
                world.get_component::<Image>(entity).cloned(),
                world.get_component::<Text>(entity).cloned(),
                graphic.clone(),
            )
        };

        if let Some(image) = image {
            self.render_image(entity, &image, &graphic, x, y, width, height);
        }
        else if let Some(text) = text {
            self.render_text(entity, &text, &graphic, x, y, width, height);
        }
        else {
            self.render_graphic(&graphic, x, y, width, height);
        }

        for child in children {
            self.render_ui_element(child, world);
        }
    }

    /// 计算位置
    fn calculate_position(&self, rect_transform: &RectTransform) -> (f32, f32) {
        (rect_transform.anchored_position[0], rect_transform.anchored_position[1])
    }

    /// 计算大小
    fn calculate_size(&self, rect_transform: &RectTransform) -> (f32, f32) {
        (rect_transform.size_delta[0], rect_transform.size_delta[1])
    }

    /// 渲染图形
    fn render_graphic(&mut self, graphic: &Graphic, x: f32, y: f32, width: f32, height: f32) {
        self.render_context.draw(DrawCommand::Sprite {
            texture_id: TextureId::INVALID,
            transform: Transform { position: [x, y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 },
            size: [width, height],
            tint: graphic.color,
            clip_rect: None,
        });
    }

    /// 渲染图片
    fn render_image(&mut self, _entity: Entity, _image: &Image, graphic: &Graphic, x: f32, y: f32, width: f32, height: f32) {
        self.render_context.draw(DrawCommand::Sprite {
            texture_id: TextureId::INVALID,
            transform: Transform { position: [x, y], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 },
            size: [width, height],
            tint: graphic.color,
            clip_rect: None,
        });
    }

    /// 渲染文本
    fn render_text(&mut self, _entity: Entity, text: &Text, _graphic: &Graphic, x: f32, y: f32, width: f32, height: f32) {
        println!("Rendering text: {} at ({}, {}) with size ({}, {})", text.text, x, y, width, height);
    }

    /// 渲染Canvas
    fn render_canvas(&mut self, _canvas: &Canvas, world: &mut World) {
        let root_entities: Vec<Entity> = world
            .entities()
            .into_iter()
            .filter(|&entity| world.get_component::<RectTransform>(entity).map_or(false, |rt| rt.parent.is_none()))
            .collect();

        for entity in root_entities {
            self.render_ui_element(entity, world);
        }
    }
}

impl CanvasRenderer for BasicCanvasRenderer {
    fn render(&self, world: &mut World) -> gg_error::GResult<()> {
        let mut renderer = self.clone();

        let canvases: Vec<Canvas> =
            world.entities().into_iter().filter_map(|entity| world.get_component::<Canvas>(entity).cloned()).collect();

        for canvas in canvases {
            renderer.render_canvas(&canvas, world);
        }

        Ok(())
    }
}

impl Clone for BasicCanvasRenderer {
    fn clone(&self) -> Self {
        Self { render_context: self.render_context.clone(), screen_width: self.screen_width, screen_height: self.screen_height }
    }
}

/// Canvas渲染系统
pub struct CanvasRenderSystem {
    /// Canvas渲染器
    renderer: Box<dyn CanvasRenderer>,
}

impl CanvasRenderSystem {
    /// 创建新的Canvas渲染系统
    pub fn new(renderer: Box<dyn CanvasRenderer>) -> Self {
        Self { renderer }
    }
}

impl gg_ecs::System for CanvasRenderSystem {
    fn name(&self) -> &str {
        "CanvasRenderSystem"
    }

    fn execute(&mut self, world: &mut World) -> gg_error::GResult<()> {
        self.renderer.render(world)?;
        Ok(())
    }
}
