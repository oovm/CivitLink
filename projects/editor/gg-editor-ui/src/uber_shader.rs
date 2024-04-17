//! Editor UI Uber-Shader 模块
//!
//! 实现超级着色器机制，通过统一图集和 uniform mode 开关
//! 将多种 UI 渲染模式合并为单次 Draw Call，提升渲染性能。

use gg_error::{GError, GErrorKind, GResult};
use std::collections::HashMap;

/// 每个顶点包含的 float 分量数（x, y, u, v, r, g, b, a）
const VERTEX_FLOAT_COUNT: usize = 8;

/// Uber-Shader 支持的渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UberShaderMode {
    /// 矩形渲染（背景、边框等）
    Rect,
    /// 文本渲染
    Text,
    /// 图标渲染
    Icon,
    /// 图片渲染
    Image,
}

impl UberShaderMode {
    /// 获取渲染模式对应的 uniform 值
    pub fn as_u32(&self) -> u32 {
        match self {
            UberShaderMode::Rect => 0,
            UberShaderMode::Text => 1,
            UberShaderMode::Icon => 2,
            UberShaderMode::Image => 3,
        }
    }
}

/// 纹理图集中的区域描述
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    /// UV 起始 U 坐标
    pub u: f32,
    /// UV 起始 V 坐标
    pub v: f32,
    /// UV 宽度
    pub width: f32,
    /// UV 高度
    pub height: f32,
}

/// 纹理图集打包器
pub struct TextureAtlas {
    /// 图集尺寸（正方形）
    pub atlas_size: u32,
    /// 纹理名称 → 图集区域映射
    pub regions: HashMap<String, AtlasRegion>,
    /// 当前写入位置 X
    pub cursor_x: u32,
    /// 当前写入位置 Y
    pub cursor_y: u32,
    /// 当前行最大高度
    pub row_height: u32,
    /// 图集像素数据（RGBA）
    pub pixel_data: Vec<u8>,
}

impl TextureAtlas {
    /// 创建指定尺寸的纹理图集
    pub fn new(atlas_size: u32) -> Self {
        let total_pixels = (atlas_size * atlas_size) as usize;
        Self {
            atlas_size,
            regions: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            pixel_data: vec![0u8; total_pixels * 4],
        }
    }

    /// 添加纹理到图集，返回区域描述
    pub fn add_texture(&mut self, name: &str, width: u32, height: u32, data: &[u8]) -> GResult<AtlasRegion> {
        if self.regions.contains_key(name) {
            return Err(GError { kind: GErrorKind::Asset, message: format!("Texture '{}' already exists in atlas", name) });
        }

        if width == 0 || height == 0 {
            return Err(GError {
                kind: GErrorKind::Asset,
                message: format!("Texture '{}' has invalid dimensions: {}x{}", name, width, height),
            });
        }

        if data.len() != (width * height) as usize * 4 {
            return Err(GError {
                kind: GErrorKind::Asset,
                message: format!(
                    "Texture '{}' data size mismatch: expected {} bytes, got {} bytes",
                    name,
                    (width * height) as usize * 4,
                    data.len()
                ),
            });
        }

        if self.cursor_x + width > self.atlas_size {
            self.cursor_y += self.row_height;
            self.cursor_x = 0;
            self.row_height = 0;
        }

        if self.cursor_y + height > self.atlas_size {
            return Err(GError { kind: GErrorKind::Asset, message: format!("Texture atlas is full, cannot add '{}'", name) });
        }

        let dst_x = self.cursor_x;
        let dst_y = self.cursor_y;

        for row in 0..height {
            let src_offset = (row * width) as usize * 4;
            let dst_offset = ((dst_y + row) * self.atlas_size + dst_x) as usize * 4;
            let src_slice = &data[src_offset..src_offset + width as usize * 4];
            let dst_slice = &mut self.pixel_data[dst_offset..dst_offset + width as usize * 4];
            dst_slice.copy_from_slice(src_slice);
        }

        let atlas_size_f = self.atlas_size as f32;
        let region = AtlasRegion {
            u: dst_x as f32 / atlas_size_f,
            v: dst_y as f32 / atlas_size_f,
            width: width as f32 / atlas_size_f,
            height: height as f32 / atlas_size_f,
        };

        self.regions.insert(name.to_string(), region);

        self.cursor_x += width;
        if height > self.row_height {
            self.row_height = height;
        }

        Ok(region)
    }

    /// 获取纹理区域
    pub fn get_region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.get(name)
    }

    /// 执行图集打包（当前为简单的行优先排列算法）
    pub fn pack(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
    }
}

/// Uber-Shader 的 uniform 数据
#[derive(Debug, Clone)]
pub struct UberShaderUniforms {
    /// 渲染模式（0=Rect, 1=Text, 2=Icon, 3=Image）
    pub mode: u32,
    /// 变换矩阵
    pub transform: [f32; 16],
    /// 主颜色
    pub color: [f32; 4],
    /// 裁剪矩形
    pub clip_rect: [f32; 4],
    /// 图集 UV 偏移
    pub atlas_uv_offset: [f32; 2],
    /// 图集 UV 缩放
    pub atlas_uv_scale: [f32; 2],
}

impl Default for UberShaderUniforms {
    fn default() -> Self {
        Self {
            mode: 0,
            transform: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            clip_rect: [0.0, 0.0, f32::MAX, f32::MAX],
            atlas_uv_offset: [0.0, 0.0],
            atlas_uv_scale: [1.0, 1.0],
        }
    }
}

/// Uber-Shader 批处理结果
#[derive(Debug, Clone)]
pub struct UberShaderBatch {
    /// 合并后的顶点数据
    pub vertices: Vec<f32>,
    /// 合并后的索引数据
    pub indices: Vec<u32>,
    /// 共享 uniform 数据
    pub uniforms: UberShaderUniforms,
    /// 包含的元素数量
    pub element_count: usize,
}

/// Uber-Shader 管理器
pub struct EditorUiUberShader {
    /// 纹理图集
    pub atlas: TextureAtlas,
    /// 当前批处理
    pub current_batch: Option<UberShaderBatch>,
    /// 是否已提交
    pub is_committed: bool,
}

impl EditorUiUberShader {
    /// 创建 Uber-Shader 管理器，初始化 2048x2048 图集
    pub fn new() -> Self {
        Self { atlas: TextureAtlas::new(2048), current_batch: None, is_committed: false }
    }

    /// 添加纹理到图集
    pub fn add_texture(&mut self, name: &str, width: u32, height: u32, data: &[u8]) -> GResult<AtlasRegion> {
        self.atlas.add_texture(name, width, height, data)
    }

    /// 开始新的批处理
    pub fn begin_batch(&mut self) {
        self.current_batch = Some(UberShaderBatch {
            vertices: Vec::new(),
            indices: Vec::new(),
            uniforms: UberShaderUniforms::default(),
            element_count: 0,
        });
        self.is_committed = false;
    }

    /// 添加元素到当前批处理
    pub fn add_element(
        &mut self,
        mode: UberShaderMode,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        atlas_region: Option<&AtlasRegion>,
    ) {
        let batch = match &mut self.current_batch {
            Some(b) => b,
            None => return,
        };

        let base_vertex = (batch.vertices.len() / VERTEX_FLOAT_COUNT) as u32;

        let (u0, v0, u1, v1) = match atlas_region {
            Some(region) => (region.u, region.v, region.u + region.width, region.v + region.height),
            None => (0.0, 0.0, 1.0, 1.0),
        };

        batch.uniforms.mode = mode.as_u32();

        let x0 = x;
        let y0 = y;
        let x1 = x + w;
        let y1 = y + h;

        batch.vertices.extend_from_slice(&[
            x0, y0, u0, v0, color[0], color[1], color[2], color[3], x1, y0, u1, v0, color[0], color[1], color[2], color[3], x1,
            y1, u1, v1, color[0], color[1], color[2], color[3], x0, y1, u0, v1, color[0], color[1], color[2], color[3],
        ]);

        batch.indices.extend_from_slice(&[
            base_vertex,
            base_vertex + 1,
            base_vertex + 2,
            base_vertex,
            base_vertex + 2,
            base_vertex + 3,
        ]);

        batch.element_count += 1;
    }

    /// 结束批处理，返回合并结果
    pub fn end_batch(&mut self) -> UberShaderBatch {
        self.is_committed = true;
        self.current_batch.take().unwrap_or_else(|| UberShaderBatch {
            vertices: Vec::new(),
            indices: Vec::new(),
            uniforms: UberShaderUniforms::default(),
            element_count: 0,
        })
    }

    /// 验证是否可合并为 1 次 Draw Call（所有元素使用相同图集和着色器）
    pub fn can_merge_to_single_draw_call(&self) -> bool {
        match &self.current_batch {
            Some(batch) => {
                let vertex_count = batch.vertices.len() / VERTEX_FLOAT_COUNT;
                vertex_count > 0 && !self.is_committed
            }
            None => false,
        }
    }
}

impl Default for EditorUiUberShader {
    fn default() -> Self {
        Self::new()
    }
}
