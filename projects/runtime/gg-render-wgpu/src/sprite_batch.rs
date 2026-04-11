use bytemuck::{Pod, Zeroable};
use gg_render::TextureId;

/// 批渲染精灵实例数据
///
/// 每个精灵实例的 per-instance 数据，通过顶点属性传递给着色器。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BatchedSpriteInstance {
    /// MVP 变换矩阵第一行
    pub mvp_row0: [f32; 4],
    /// MVP 变换矩阵第二行
    pub mvp_row1: [f32; 4],
    /// MVP 变换矩阵第三行
    pub mvp_row2: [f32; 4],
    /// MVP 变换矩阵第四行
    pub mvp_row3: [f32; 4],
    /// 着色颜色 RGBA
    pub tint: [f32; 4],
    /// UV 偏移和缩放 `[u_offset, v_offset, u_scale, v_scale]`
    pub uv_transform: [f32; 4],
}

/// 精灵批次
///
/// 同一纹理下的所有精灵实例集合，可合并为一次绘制调用。
pub struct SpriteBatch {
    /// 批次纹理标识符
    pub texture_id: TextureId,
    /// 批次内的精灵实例列表
    pub instances: Vec<BatchedSpriteInstance>,
}

/// 精灵批渲染器
///
/// 收集使用相同纹理的精灵，按纹理分组后合并为单次绘制调用，
/// 减少 GPU 状态切换和绘制调用次数。
///
/// # 工作原理
///
/// 1. 通过 [`SpriteBatcher::push`] 添加精灵实例
/// 2. 纹理切换时自动开始新批次
/// 3. 通过 [`SpriteBatcher::flush`] 获取所有已完成的批次
pub struct SpriteBatcher {
    /// 已完成的批次列表
    completed_batches: Vec<SpriteBatch>,
    /// 当前批次的纹理标识符
    current_texture_id: Option<TextureId>,
    /// 当前批次的实例列表
    current_instances: Vec<BatchedSpriteInstance>,
}

impl SpriteBatcher {
    /// 创建新的精灵批渲染器
    pub fn new() -> Self {
        Self { completed_batches: Vec::new(), current_texture_id: None, current_instances: Vec::new() }
    }

    /// 添加一个精灵实例到当前批次
    ///
    /// 如果纹理与当前批次不同，自动完成当前批次并开始新批次。
    ///
    /// # 参数
    ///
    /// - `texture_id` - 精灵纹理标识符
    /// - `mvp` - MVP 变换矩阵
    /// - `tint` - 着色颜色 RGBA
    /// - `uv_transform` - UV 偏移和缩放
    pub fn push(&mut self, texture_id: TextureId, mvp: [[f32; 4]; 4], tint: [f32; 4], uv_transform: [f32; 4]) {
        match self.current_texture_id {
            Some(tid) if tid == texture_id => {}
            Some(_) => {
                self.finish_current_batch();
                self.current_texture_id = Some(texture_id);
            }
            None => {
                self.current_texture_id = Some(texture_id);
            }
        }

        let mvp_t = [
            [mvp[0][0], mvp[1][0], mvp[2][0], mvp[3][0]],
            [mvp[0][1], mvp[1][1], mvp[2][1], mvp[3][1]],
            [mvp[0][2], mvp[1][2], mvp[2][2], mvp[3][2]],
            [mvp[0][3], mvp[1][3], mvp[2][3], mvp[3][3]],
        ];

        self.current_instances.push(BatchedSpriteInstance {
            mvp_row0: mvp_t[0],
            mvp_row1: mvp_t[1],
            mvp_row2: mvp_t[2],
            mvp_row3: mvp_t[3],
            tint,
            uv_transform,
        });
    }

    /// 完成当前批次，将其移入已完成列表
    fn finish_current_batch(&mut self) {
        if self.current_instances.is_empty() {
            return;
        }
        let texture_id = self.current_texture_id.unwrap_or(TextureId::INVALID);
        let instances = std::mem::take(&mut self.current_instances);
        self.completed_batches.push(SpriteBatch { texture_id, instances });
    }

    /// 刷新所有批次并返回批次列表
    ///
    /// 完成当前未完成的批次，并返回所有批次。
    /// 调用后批渲染器状态被重置。
    pub fn flush(&mut self) -> Vec<SpriteBatch> {
        self.finish_current_batch();
        self.current_texture_id = None;
        std::mem::take(&mut self.completed_batches)
    }

    /// 检查批渲染器是否为空
    pub fn is_empty(&self) -> bool {
        self.current_instances.is_empty() && self.completed_batches.is_empty()
    }

    /// 重置批渲染器状态
    pub fn reset(&mut self) {
        self.completed_batches.clear();
        self.current_texture_id = None;
        self.current_instances.clear();
    }
}

impl Default for SpriteBatcher {
    fn default() -> Self {
        Self::new()
    }
}
