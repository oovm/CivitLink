//! 渲染性能分析工具模块
//!
//! 提供渲染性能数据收集和报告功能，帮助开发者识别渲染瓶颈。

use std::collections::VecDeque;

use gg_render::TextureId;

/// 单帧渲染性能数据
#[derive(Debug, Clone)]
pub struct FrameProfile {
    /// 帧索引
    pub frame_index: u64,
    /// Draw Call 数量
    pub draw_call_count: u32,
    /// 三角形数量
    pub triangle_count: u32,
    /// 纹理切换次数
    pub texture_switch_count: u32,
    /// 批次数
    pub batch_count: u32,
    /// 理论最少批次数（按纹理种类计算）
    pub min_batch_count: u32,
}

/// 渲染性能报告
#[derive(Debug, Clone)]
pub struct RenderProfileReport {
    /// 平均 Draw Call 数量
    pub avg_draw_calls: f32,
    /// 最大 Draw Call 数量
    pub max_draw_calls: u32,
    /// 最小 Draw Call 数量
    pub min_draw_calls: u32,
    /// 平均三角形数量
    pub avg_triangles: f32,
    /// 平均纹理切换次数
    pub avg_texture_switches: f32,
    /// 平均批次数
    pub avg_batches: f32,
    /// 平均合批率（实际批次 / 理论最少批次，越小越好）
    pub avg_batch_efficiency: f32,
    /// 采样帧数
    pub sample_count: usize,
}

/// 渲染性能分析器
///
/// 收集和报告每帧渲染性能数据，帮助开发者识别渲染瓶颈。
pub struct RenderProfiler {
    /// 是否启用
    enabled: bool,
    /// 最近 N 帧的性能数据
    frame_data: VecDeque<FrameProfile>,
    /// 最大保留帧数
    max_frames: usize,
    /// 当前帧索引
    current_frame: u64,
    /// 当前帧累计 Draw Call 数量
    current_draw_calls: u32,
    /// 当前帧累计三角形数量
    current_triangles: u32,
    /// 当前帧累计纹理切换次数
    current_texture_switches: u32,
    /// 当前帧累计批次数
    current_batch_count: u32,
    /// 当前帧理论最少批次数
    current_min_batches: u32,
    /// 上一次使用的纹理标识符
    last_texture_id: Option<TextureId>,
}

impl RenderProfiler {
    /// 创建新的渲染性能分析器
    ///
    /// # 参数
    ///
    /// - `max_frames` - 最大保留帧历史数量
    pub fn new(max_frames: usize) -> Self {
        Self {
            enabled: false,
            frame_data: VecDeque::with_capacity(max_frames),
            max_frames,
            current_frame: 0,
            current_draw_calls: 0,
            current_triangles: 0,
            current_texture_switches: 0,
            current_batch_count: 0,
            current_min_batches: 0,
            last_texture_id: None,
        }
    }

    /// 启用或禁用性能分析
    ///
    /// # 参数
    ///
    /// - `enabled` - 是否启用
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查性能分析是否已启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 开始新的一帧，重置每帧累计计数器
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }
        self.current_draw_calls = 0;
        self.current_triangles = 0;
        self.current_texture_switches = 0;
        self.current_batch_count = 0;
        self.current_min_batches = 0;
        self.last_texture_id = None;
    }

    /// 记录一次 Draw Call
    ///
    /// # 参数
    ///
    /// - `triangle_count` - 该 Draw Call 绘制的三角形数量
    pub fn record_draw_call(&mut self, triangle_count: u32) {
        if !self.enabled {
            return;
        }
        self.current_draw_calls += 1;
        self.current_triangles += triangle_count;
    }

    /// 记录纹理切换
    ///
    /// 当纹理标识符与上一帧不同时，纹理切换计数加一。
    ///
    /// # 参数
    ///
    /// - `texture_id` - 当前使用的纹理标识符
    pub fn record_texture_switch(&mut self, texture_id: TextureId) {
        if !self.enabled {
            return;
        }
        match self.last_texture_id {
            Some(last) if last == texture_id => {}
            _ => {
                self.current_texture_switches += 1;
                self.last_texture_id = Some(texture_id);
            }
        }
    }

    /// 记录一个批次
    pub fn record_batch(&mut self) {
        if !self.enabled {
            return;
        }
        self.current_batch_count += 1;
    }

    /// 记录理论最少批次数
    ///
    /// # 参数
    ///
    /// - `count` - 理论最少批次数（按纹理种类计算）
    pub fn record_min_batches(&mut self, count: u32) {
        if !self.enabled {
            return;
        }
        self.current_min_batches = count;
    }

    /// 结束当前帧，将帧数据添加到历史记录
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }
        let profile = FrameProfile {
            frame_index: self.current_frame,
            draw_call_count: self.current_draw_calls,
            triangle_count: self.current_triangles,
            texture_switch_count: self.current_texture_switches,
            batch_count: self.current_batch_count,
            min_batch_count: self.current_min_batches,
        };

        if self.frame_data.len() >= self.max_frames {
            self.frame_data.pop_front();
        }
        self.frame_data.push_back(profile);
        self.current_frame += 1;
    }

    /// 生成性能统计报告
    ///
    /// 基于历史帧数据计算各项平均值、最大值和最小值。
    /// 如果没有历史数据，返回零值报告。
    pub fn generate_report(&self) -> RenderProfileReport {
        if self.frame_data.is_empty() {
            return RenderProfileReport {
                avg_draw_calls: 0.0,
                max_draw_calls: 0,
                min_draw_calls: 0,
                avg_triangles: 0.0,
                avg_texture_switches: 0.0,
                avg_batches: 0.0,
                avg_batch_efficiency: 0.0,
                sample_count: 0,
            };
        }

        let sample_count = self.frame_data.len();
        let mut total_draw_calls: u64 = 0;
        let mut max_draw_calls: u32 = 0;
        let mut min_draw_calls: u32 = u32::MAX;
        let mut total_triangles: u64 = 0;
        let mut total_texture_switches: u64 = 0;
        let mut total_batches: u64 = 0;
        let mut total_batch_efficiency: f64 = 0.0;
        let mut efficiency_samples: usize = 0;

        for frame in &self.frame_data {
            total_draw_calls += frame.draw_call_count as u64;
            if frame.draw_call_count > max_draw_calls {
                max_draw_calls = frame.draw_call_count;
            }
            if frame.draw_call_count < min_draw_calls {
                min_draw_calls = frame.draw_call_count;
            }
            total_triangles += frame.triangle_count as u64;
            total_texture_switches += frame.texture_switch_count as u64;
            total_batches += frame.batch_count as u64;

            if frame.min_batch_count > 0 {
                total_batch_efficiency += frame.batch_count as f64 / frame.min_batch_count as f64;
                efficiency_samples += 1;
            }
        }

        let avg_batch_efficiency =
            if efficiency_samples > 0 { (total_batch_efficiency / efficiency_samples as f64) as f32 } else { 0.0 };

        RenderProfileReport {
            avg_draw_calls: total_draw_calls as f32 / sample_count as f32,
            max_draw_calls,
            min_draw_calls,
            avg_triangles: total_triangles as f32 / sample_count as f32,
            avg_texture_switches: total_texture_switches as f32 / sample_count as f32,
            avg_batches: total_batches as f32 / sample_count as f32,
            avg_batch_efficiency,
            sample_count,
        }
    }

    /// 获取最后一帧的性能数据
    pub fn last_frame(&self) -> Option<&FrameProfile> {
        self.frame_data.back()
    }
}
