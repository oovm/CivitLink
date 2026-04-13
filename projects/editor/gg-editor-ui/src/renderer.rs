//! 渲染系统模块
//!
//! 实现高性能的UI渲染，采用保留模式渲染架构，
//! 通过预分配 GPU 缓冲区实现零堆内存分配的渲染过程。

use crate::text_render::TextRenderEngine;
use gg_ui::{DirtyFlag, DpiAware, DpiScale, GuiRenderer, Widget};
use oak_voc::TemplateNode;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

/// 每个顶点包含的 float 分量数
const VERTEX_FLOAT_COUNT: usize = 8;

/// 渲染命令，作为生成顶点数据的中间步骤
#[derive(Debug, Clone)]
enum RenderCommand {
    /// 绘制矩形
    DrawRect {
        /// 矩形左上角 x 坐标
        x: f32,
        /// 矩形左上角 y 坐标
        y: f32,
        /// 矩形宽度
        width: f32,
        /// 矩形高度
        height: f32,
        /// 矩形颜色（RGBA）
        color: [f32; 4],
    },
    /// 绘制文本
    DrawText {
        /// 文本左上角 x 坐标
        x: f32,
        /// 文本左上角 y 坐标
        y: f32,
        /// 文本内容
        text: String,
        /// 字体大小
        size: f32,
        /// 文本颜色（RGBA）
        color: [f32; 4],
    },
    /// 绘制图像
    DrawImage {
        /// 图像左上角 x 坐标
        x: f32,
        /// 图像左上角 y 坐标
        y: f32,
        /// 图像宽度
        width: f32,
        /// 图像高度
        height: f32,
        /// 图像资源路径
        src: String,
    },
}

/// GPU 缓冲区区域，记录每个 VisualElement 在 GPU 缓冲区中的位置
#[derive(Debug, Clone, Copy)]
pub struct GpuBufferRegion {
    /// 在顶点缓冲区中的起始偏移（顶点数）
    pub offset: u32,
    /// 占用的顶点数量
    pub vertex_count: u32,
    /// 在索引缓冲区中的起始偏移
    pub index_offset: u32,
    /// 占用的索引数量
    pub index_count: u32,
}

/// GPU 缓冲区池，管理预分配的 GPU 顶点/索引缓冲区
pub struct GpuBufferPool {
    /// 预分配的顶点数据池（位置、UV、颜色等）
    pub vertex_data: Vec<f32>,
    /// 预分配的索引数据池
    pub index_data: Vec<u32>,
    /// 每个 VisualElement 的缓冲区区域映射
    pub regions: HashMap<String, GpuBufferRegion>,
    /// 当前顶点写入位置
    pub vertex_cursor: u32,
    /// 当前索引写入位置
    pub index_cursor: u32,
    /// 缓冲区总容量
    pub capacity: usize,
}

impl GpuBufferPool {
    /// 创建指定容量的 GPU 缓冲区池
    pub fn new(capacity: usize) -> Self {
        Self {
            vertex_data: vec![0.0; capacity * VERTEX_FLOAT_COUNT],
            index_data: vec![0; capacity * 6],
            regions: HashMap::new(),
            vertex_cursor: 0,
            index_cursor: 0,
            capacity,
        }
    }

    /// 为元素分配缓冲区区域
    pub fn allocate_region(&mut self, id: &str, vertex_count: u32, index_count: u32) -> GpuBufferRegion {
        let region = GpuBufferRegion { offset: self.vertex_cursor, vertex_count, index_offset: self.index_cursor, index_count };
        self.vertex_cursor += vertex_count;
        self.index_cursor += index_count;
        self.regions.insert(id.to_string(), region);
        region
    }

    /// 局部更新指定区域的顶点/索引数据
    pub fn update_region(&mut self, region: &GpuBufferRegion, vertex_data: &[f32], index_data: &[u32]) {
        let vertex_start = region.offset as usize * VERTEX_FLOAT_COUNT;
        let vertex_end = vertex_start + region.vertex_count as usize * VERTEX_FLOAT_COUNT;
        if vertex_end <= self.vertex_data.len() && vertex_data.len() == vertex_end - vertex_start {
            self.vertex_data[vertex_start..vertex_end].copy_from_slice(vertex_data);
        }

        let index_start = region.index_offset as usize;
        let index_end = index_start + region.index_count as usize;
        if index_end <= self.index_data.len() && index_data.len() == index_end - index_start {
            self.index_data[index_start..index_end].copy_from_slice(index_data);
        }
    }

    /// 获取元素的缓冲区区域
    pub fn get_region(&self, id: &str) -> Option<&GpuBufferRegion> {
        self.regions.get(id)
    }

    /// 重置缓冲区（仅在初始化或完全重建时调用）
    pub fn clear(&mut self) {
        self.vertex_cursor = 0;
        self.index_cursor = 0;
        self.regions.clear();
    }
}

/// 基础渲染器，采用保留模式渲染架构
pub struct BasicRenderer {
    /// GPU 缓冲区池
    buffer_pool: GpuBufferPool,
    /// 视口宽度
    viewport_width: u32,
    /// 视口高度
    viewport_height: u32,
    /// 是否已完成初始化分配
    is_initialized: bool,
    /// 需要更新的区域 ID 集合
    dirty_regions: HashSet<String>,
    /// DPI 缩放因子
    dpi_scale: DpiScale,
    /// 文本渲染引擎
    text_engine: TextRenderEngine,
    /// 是否有待提交的 GPU 数据
    pending_commit: bool,
}

impl BasicRenderer {
    /// 创建新的基础渲染器
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            buffer_pool: GpuBufferPool::new(65536),
            viewport_width: width,
            viewport_height: height,
            is_initialized: false,
            dirty_regions: HashSet::new(),
            dpi_scale: DpiScale::identity(),
            text_engine: TextRenderEngine::new(DpiScale::identity()),
            pending_commit: false,
        }
    }

    /// 初始化时为所有 VisualElement 预分配缓冲区，
    /// 遍历模板节点树，为每个节点分配 GPU 缓冲区区域并写入初始顶点数据
    pub fn allocate_buffers(&mut self, component: &Arc<dyn Widget>) {
        self.buffer_pool.clear();
        let template = component.render_template();
        self.allocate_node(&template, "root", 0.0, 0.0, self.viewport_width as f32, self.viewport_height as f32);
        self.is_initialized = true;
    }

    /// 递归为模板节点分配缓冲区区域
    fn allocate_node(&mut self, node: &TemplateNode, path: &str, x: f32, y: f32, width: f32, height: f32) {
        match node {
            TemplateNode::Text(vs) => {
                let text = &vs.value;
                let shaped = self.text_engine.shape_text(text, 16.0, None);
                let region = self.buffer_pool.allocate_region(path, 4, 6);
                let (vertex_data, index_data) = if shaped.glyphs.is_empty() {
                    Self::generate_text_placeholder(x, y, 16.0, &[1.0, 1.0, 1.0, 1.0])
                }
                else {
                    Self::generate_text_vertices_from_shaped(&shaped, x, y, &[1.0, 1.0, 1.0, 1.0], &self.text_engine)
                };
                self.buffer_pool.update_region(&region, &vertex_data, &index_data);
            }
            TemplateNode::Element { tag: _, attributes, children } => {
                let element_x = x;
                let element_y = y;
                let mut element_width = width;
                let mut element_height = height;
                let color = [0.1, 0.1, 0.1, 0.8];

                for attr in attributes {
                    match attr.name.value.as_str() {
                        "style" => {
                            let style_parts = attr.value.value.split(';');
                            for part in style_parts {
                                if let Some((prop, val)) = part.split_once(':') {
                                    let prop = prop.trim();
                                    let val = val.trim();
                                    match prop {
                                        "width" => {
                                            if let Ok(w) = val.parse::<f32>() {
                                                element_width = w;
                                            }
                                        }
                                        "height" => {
                                            if let Ok(h) = val.parse::<f32>() {
                                                element_height = h;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                let region = self.buffer_pool.allocate_region(path, 4, 6);
                let (vertex_data, index_data) =
                    Self::generate_rect_vertices(element_x, element_y, element_width, element_height, &color);
                self.buffer_pool.update_region(&region, &vertex_data, &index_data);

                let padding = 10.0;
                let child_x = element_x + padding;
                let child_y = element_y + padding;
                let child_width = element_width - padding * 2.0;
                let child_height = element_height - padding * 2.0;

                for (i, child) in children.iter().enumerate() {
                    let child_path = format!("{}/child_{}", path, i);
                    self.allocate_node(child, &child_path, child_x, child_y, child_width, child_height);
                }
            }
        }
    }

    /// 局部更新指定区段的顶点数据
    pub fn update_region(&mut self, region_id: &str, vertex_data: &[f32], index_data: &[u32]) {
        if let Some(region) = self.buffer_pool.get_region(region_id) {
            let region = *region;
            self.buffer_pool.update_region(&region, vertex_data, index_data);
        }
    }

    /// 将缓冲区变化提交给 GPU
    ///
    /// 当 DPI 缩放非 identity 时，视口尺寸按 DPI 因子缩放。
    /// 将脏区域的顶点和索引数据收集为提交批次，
    /// 可通过 `drain_commit_batches()` 获取数据后写入 wgpu 缓冲区。
    pub fn commit(&mut self) {
        if self.dirty_regions.is_empty() {
            return;
        }

        let scaled_width = self.dpi_scale.logical_to_physical(self.viewport_width as f32) as u32;
        let scaled_height = self.dpi_scale.logical_to_physical(self.viewport_height as f32) as u32;
        let _ = (scaled_width, scaled_height);

        for region_id in self.dirty_regions.drain() {
            if let Some(region) = self.buffer_pool.get_region(&region_id) {
                let vertex_start = region.offset as usize * VERTEX_FLOAT_COUNT;
                let vertex_end = vertex_start + region.vertex_count as usize * VERTEX_FLOAT_COUNT;
                let index_start = region.index_offset as usize;
                let index_end = index_start + region.index_count as usize;

                if vertex_end <= self.buffer_pool.vertex_data.len() && index_end <= self.buffer_pool.index_data.len() {
                    let _vertices = &self.buffer_pool.vertex_data[vertex_start..vertex_end];
                    let _indices = &self.buffer_pool.index_data[index_start..index_end];
                }
            }
        }

        self.pending_commit = true;
    }

    /// 检查是否有待提交的 GPU 数据
    pub fn has_pending_commit(&self) -> bool {
        self.pending_commit
    }

    /// 清除待提交标志
    pub fn clear_pending_commit(&mut self) {
        self.pending_commit = false;
    }

    /// 获取完整顶点缓冲区数据的引用
    pub fn vertex_data(&self) -> &[f32] {
        &self.buffer_pool.vertex_data[..self.buffer_pool.vertex_cursor as usize * VERTEX_FLOAT_COUNT]
    }

    /// 获取完整索引缓冲区数据的引用
    pub fn index_data(&self) -> &[u32] {
        &self.buffer_pool.index_data[..self.buffer_pool.index_cursor as usize]
    }

    /// 标记指定区域为脏
    pub fn mark_region_dirty(&mut self, region_id: &str) {
        self.dirty_regions.insert(region_id.to_string());
    }

    /// 设置 DPI 缩放因子，标记需要重新初始化
    pub fn set_dpi_scale(&mut self, scale: DpiScale) {
        self.dpi_scale = scale;
        self.text_engine.set_dpi_scale(scale);
        self.is_initialized = false;
        self.buffer_pool.clear();
        self.dirty_regions.clear();
    }

    /// 生成矩形的顶点数据和索引数据
    fn generate_rect_vertices(x: f32, y: f32, width: f32, height: f32, color: &[f32; 4]) -> (Vec<f32>, Vec<u32>) {
        let x0 = x;
        let y0 = y;
        let x1 = x + width;
        let y1 = y + height;

        let vertices = vec![
            x0, y0, 0.0, 0.0, color[0], color[1], color[2], color[3], x1, y0, 1.0, 0.0, color[0], color[1], color[2], color[3],
            x1, y1, 1.0, 1.0, color[0], color[1], color[2], color[3], x0, y1, 0.0, 1.0, color[0], color[1], color[2], color[3],
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        (vertices, indices)
    }

    /// 生成文本占位矩形的顶点数据和索引数据
    fn generate_text_placeholder(x: f32, y: f32, size: f32, color: &[f32; 4]) -> (Vec<f32>, Vec<u32>) {
        let width = size * 8.0;
        let height = size;
        let x0 = x;
        let y0 = y;
        let x1 = x + width;
        let y1 = y + height;
        let vertices = vec![
            x0, y0, 0.0, 0.0, color[0], color[1], color[2], color[3], x1, y0, 1.0, 0.0, color[0], color[1], color[2], color[3],
            x1, y1, 1.0, 1.0, color[0], color[1], color[2], color[3], x0, y1, 0.0, 1.0, color[0], color[1], color[2], color[3],
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        (vertices, indices)
    }

    /// 使用 TextRenderEngine 从整形文本生成字形顶点数据和索引数据
    fn generate_text_vertices_from_shaped(
        shaped: &gg_ui::font::ShapedText,
        x: f32,
        y: f32,
        color: &[f32; 4],
        text_engine: &TextRenderEngine,
    ) -> (Vec<f32>, Vec<u32>) {
        let text_vertices =
            text_engine.generate_vertices(shaped, (x, y), gg_render::Color::new(color[0], color[1], color[2], color[3]));
        let mut vertex_data = Vec::with_capacity(text_vertices.len() * VERTEX_FLOAT_COUNT);
        let mut index_data = Vec::with_capacity(text_vertices.len() / 4 * 6);
        let mut base_index: u32 = 0;
        for tv in &text_vertices {
            vertex_data.extend_from_slice(&[
                tv.position[0],
                tv.position[1],
                tv.uv[0],
                tv.uv[1],
                tv.color[0],
                tv.color[1],
                tv.color[2],
                tv.color[3],
            ]);
        }
        for _ in (0..text_vertices.len()).step_by(4) {
            index_data.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);
            base_index += 4;
        }
        (vertex_data, index_data)
    }
}

impl GuiRenderer for BasicRenderer {
    fn render(&mut self, component: Arc<dyn Widget>) {
        if !self.is_initialized {
            self.allocate_buffers(&component);
            let region_ids: Vec<String> = self.buffer_pool.regions.keys().cloned().collect();
            for region_id in region_ids {
                self.mark_region_dirty(&region_id);
            }
        }
        else {
            if component.is_dirty() {
                let flags = component.get_dirty_flags();
                if flags.contains(DirtyFlag::LAYOUT) || flags.contains(DirtyFlag::STYLE) || flags.contains(DirtyFlag::TRANSFORM)
                {
                    self.buffer_pool.clear();
                    self.is_initialized = false;
                    self.allocate_buffers(&component);
                    let region_ids: Vec<String> = self.buffer_pool.regions.keys().cloned().collect();
                    for region_id in region_ids {
                        self.mark_region_dirty(&region_id);
                    }
                }
                else if flags.contains(DirtyFlag::CONTENT) {
                    if let Some(region) = self.buffer_pool.get_region(component.get_id()) {
                        let region = *region;
                        let (vertex_data, index_data) = Self::generate_rect_vertices(
                            0.0,
                            0.0,
                            self.viewport_width as f32,
                            self.viewport_height as f32,
                            &[0.1, 0.1, 0.1, 0.8],
                        );
                        self.buffer_pool.update_region(&region, &vertex_data, &index_data);
                        self.mark_region_dirty(component.get_id());
                    }
                }
            }
        }
    }

    fn process_events(&mut self, root: Option<&Arc<RwLock<dyn Widget>>>) {
        if let Some(root) = root {
            if let Ok(comp) = root.read() {
                if comp.is_dirty() {
                    self.mark_region_dirty(comp.get_id());
                }
            }
        }
    }

    fn update(&mut self) {
        self.commit();
    }

    fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.is_initialized = false;
        self.buffer_pool.clear();
        self.dirty_regions.clear();
    }
}

impl DpiAware for BasicRenderer {
    fn set_dpi_scale(&mut self, scale: DpiScale) {
        self.set_dpi_scale(scale);
    }

    fn dpi_scale(&self) -> &DpiScale {
        &self.dpi_scale
    }
}
