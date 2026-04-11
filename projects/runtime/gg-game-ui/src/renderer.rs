//! 游戏UI渲染系统模块
//! 
//! 实现游戏UI的渲染功能，包括Canvas渲染、批处理等

use gg_ecs::{World, Entity};
use gg_render::{Color, DrawCommand, RenderContext, TextureId, Transform};
use crate::components::{Canvas, RectTransform, Graphic, Image, Text, CanvasRenderMode};

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
        Self {
            render_context: RenderContext::new(width, height),
            screen_width: width,
            screen_height: height,
        }
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
        // 检查是否有RectTransform组件
        if let Some(rect_transform) = world.get_component::<RectTransform>(entity) {
            // 检查是否有Graphic组件
            if let Some(graphic) = world.get_component::<Graphic>(entity) {
                if !graphic.visible {
                    return;
                }

                // 计算实际位置和大小
                let (x, y) = self.calculate_position(rect_transform);
                let (width, height) = self.calculate_size(rect_transform);

                // 检查是否有Image组件
                if let Some(image) = world.get_component::<Image>(entity) {
                    self.render_image(entity, image, graphic, x, y, width, height);
                }
                // 检查是否有Text组件
                else if let Some(text) = world.get_component::<Text>(entity) {
                    self.render_text(entity, text, graphic, x, y, width, height);
                }
                // 其他Graphic类型
                else {
                    self.render_graphic(graphic, x, y, width, height);
                }

                // 递归渲染子元素
                for child in &rect_transform.children {
                    self.render_ui_element(*child, world);
                }
            }
        }
    }

    /// 计算位置
    fn calculate_position(&self, rect_transform: &RectTransform) -> (f32, f32) {
        // 简单实现，实际需要考虑父元素的变换
        (rect_transform.anchored_position[0], rect_transform.anchored_position[1])
    }

    /// 计算大小
    fn calculate_size(&self, rect_transform: &RectTransform) -> (f32, f32) {
        // 简单实现，实际需要考虑锚点和父元素的大小
        (rect_transform.size_delta[0], rect_transform.size_delta[1])
    }

    /// 渲染图形
    fn render_graphic(&mut self, graphic: &Graphic, x: f32, y: f32, width: f32, height: f32) {
        // 添加绘制命令
        self.render_context.draw(DrawCommand::Sprite {
            texture_id: TextureId::INVALID,
            transform: Transform {
                position: [x, y],
                scale: [1.0, 1.0],
                rotation: 0.0,
                z_index: 0.0,
            },
            size: [width, height],
            tint: graphic.color,
            clip_rect: None,
        });
    }

    /// 渲染图片
    fn render_image(&mut self, entity: Entity, image: &Image, graphic: &Graphic, x: f32, y: f32, width: f32, height: f32) {
        // 简单实现，实际需要考虑精灵和填充方式
        self.render_context.draw(DrawCommand::Sprite {
            texture_id: TextureId::INVALID,
            transform: Transform {
                position: [x, y],
                scale: [1.0, 1.0],
                rotation: 0.0,
                z_index: 0.0,
            },
            size: [width, height],
            tint: graphic.color,
            clip_rect: None,
        });
    }

    /// 渲染文本
    fn render_text(&mut self, entity: Entity, text: &Text, graphic: &Graphic, x: f32, y: f32, width: f32, height: f32) {
        // 简单实现，实际需要考虑字体和文本布局
        println!("Rendering text: {} at ({}, {}) with size ({}, {})", text.text, x, y, width, height);
    }

    /// 渲染Canvas
    fn render_canvas(&mut self, canvas: &Canvas, world: &mut World) {
        // 遍历所有UI元素
        for entity in world.entities() {
            if let Some(rect_transform) = world.get_component::<RectTransform>(entity) {
                // 检查是否是根元素
                if rect_transform.parent.is_none() {
                    self.render_ui_element(entity, world);
                }
            }
        }
    }
}

impl CanvasRenderer for BasicCanvasRenderer {
    fn render(&self, world: &mut World) -> gg_error::GResult<()> {
        // 克隆自身以修改render_context
        let mut renderer = self.clone();
        
        // 遍历所有Canvas
        for entity in world.entities() {
            if let Some(canvas) = world.get_component::<Canvas>(entity) {
                renderer.render_canvas(canvas, world);
            }
        }
        
        // 这里应该将render_context的内容提交给实际的渲染器
        Ok(())
    }
}

impl Clone for BasicCanvasRenderer {
    fn clone(&self) -> Self {
        Self {
            render_context: self.render_context.clone(),
            screen_width: self.screen_width,
            screen_height: self.screen_height,
        }
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
        Self {
            renderer,
        }
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
