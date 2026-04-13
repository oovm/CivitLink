use gg_core::GResult;
use gg_render::TextureId;

use crate::{renderer::RenderTarget, texture_cache::TextureCache};

/// 后处理效果描述
///
/// 定义一个后处理效果的着色器和参数。
pub struct PostProcessEffect {
    /// 着色器标识符（.shader 文件路径或已注册的着色器名称）
    pub shader_id: String,
    /// Uniform 数据（键值对，键为 uniform 名称，值为 f32 数组）
    pub uniforms: Vec<(String, Vec<f32>)>,
}

/// 后处理 Pass
///
/// 封装一次全屏后处理操作，包含输入纹理和输出目标。
pub struct PostProcessPass {
    /// 输入纹理标识符
    pub input_texture: TextureId,
    /// 后处理效果列表，按顺序执行
    pub effects: Vec<PostProcessEffect>,
}

/// 后处理管线
///
/// 管理后处理效果链的执行顺序。
/// 先将场景渲染到离屏 RenderTarget，
/// 然后依次执行后处理着色器（全屏四边形 + 离屏纹理采样），
/// 最终输出到屏幕或另一个 RenderTarget。
pub struct PostProcessPipeline {
    /// 后处理 Pass 列表
    passes: Vec<PostProcessPass>,
    /// 中间渲染目标（用于 Ping-Pong 渲染）
    intermediate_targets: Vec<RenderTarget>,
}

impl PostProcessPipeline {
    /// 创建空的后处理管线
    pub fn new() -> Self {
        Self { passes: Vec::new(), intermediate_targets: Vec::new() }
    }

    /// 添加后处理 Pass
    ///
    /// Pass 按添加顺序依次执行。
    ///
    /// # 参数
    ///
    /// - `pass` - 后处理 Pass
    pub fn add_pass(&mut self, pass: PostProcessPass) {
        self.passes.push(pass);
    }

    /// 获取后处理 Pass 列表的引用
    pub fn passes(&self) -> &[PostProcessPass] {
        &self.passes
    }

    /// 获取后处理 Pass 数量
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// 清空所有后处理 Pass
    pub fn clear(&mut self) {
        self.passes.clear();
    }

    /// 确保有足够的中间渲染目标
    ///
    /// 后处理需要 Ping-Pong 渲染，至少需要一个中间目标。
    ///
    /// # 参数
    ///
    /// - `width` - 渲染目标宽度
    /// - `height` - 渲染目标高度
    /// - `device` - wgpu 设备
    /// - `texture_cache` - 纹理缓存
    pub fn ensure_intermediate_targets(
        &mut self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        texture_cache: &mut TextureCache,
    ) -> GResult<()> {
        let needed = if self.passes.len() > 1 { 2 } else { 1 };

        while self.intermediate_targets.len() < needed {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("post_process_intermediate_{}", self.intermediate_targets.len())),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let texture_id = texture_cache.register_texture(texture.clone());

            self.intermediate_targets.push(RenderTarget::new(texture, view, texture_id, width, height));
        }

        Ok(())
    }

    /// 获取中间渲染目标的纹理标识符
    ///
    /// # 参数
    ///
    /// - `index` - 中间目标索引
    pub fn intermediate_texture_id(&self, index: usize) -> Option<TextureId> {
        self.intermediate_targets.get(index).map(|t| t.texture_id())
    }
}

impl Default for PostProcessPipeline {
    fn default() -> Self {
        Self::new()
    }
}
