//! 帧性能分析器模块
//! 追踪各阶段耗时和内存分配，提供帧时间稳定性报告

use std::{collections::VecDeque, time::Instant};

/// 阶段性能记录
#[derive(Debug, Clone)]
pub struct StageProfile {
    /// 阶段名称
    pub name: String,
    /// 开始时间（纳秒）
    pub start_ns: u64,
    /// 结束时间（纳秒）
    pub end_ns: u64,
    /// 耗时（纳秒）
    pub elapsed_ns: u64,
}

/// 帧性能记录
#[derive(Debug, Clone)]
pub struct FrameProfile {
    /// 帧索引
    pub frame_index: u64,
    /// 各阶段性能记录
    pub stages: Vec<StageProfile>,
    /// 帧内堆分配次数
    pub alloc_count: u32,
    /// 帧内堆分配总字节数
    pub alloc_bytes: u64,
    /// 帧总耗时（纳秒）
    pub total_ns: u64,
}

/// 帧性能分析报告
#[derive(Debug, Clone)]
pub struct FrameProfileReport {
    /// 分析帧数
    pub frame_count: usize,
    /// 平均帧时间（纳秒）
    pub avg_frame_ns: u64,
    /// 最大帧时间（纳秒）
    pub max_frame_ns: u64,
    /// 最小帧时间（纳秒）
    pub min_frame_ns: u64,
    /// 帧时间标准差（纳秒）
    pub std_dev_ns: u64,
    /// 各阶段平均耗时（纳秒）
    pub stage_avg_ns: Vec<(String, u64)>,
}

/// 帧性能分析器
pub struct FrameProfiler {
    /// 历史帧记录
    profiles: VecDeque<FrameProfile>,
    /// 最大保留帧数
    max_frames: usize,
    /// 当前正在记录的帧
    current: Option<FrameProfile>,
    /// 帧计数器
    frame_counter: u64,
    /// 帧开始时间
    frame_start: Option<Instant>,
    /// 阶段开始时间
    stage_start: Option<Instant>,
    /// 当前阶段名称
    current_stage_name: Option<String>,
    /// 是否启用
    enabled: bool,
}

impl FrameProfiler {
    /// 创建新的帧性能分析器
    pub fn new() -> Self {
        Self {
            profiles: VecDeque::new(),
            max_frames: 300,
            current: None,
            frame_counter: 0,
            frame_start: None,
            stage_start: None,
            current_stage_name: None,
            enabled: false,
        }
    }

    /// 创建已启用的帧性能分析器
    pub fn enabled() -> Self {
        let mut profiler = Self::new();
        profiler.enabled = true;
        profiler
    }

    /// 是否已启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 启用/禁用分析器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.current = None;
            self.frame_start = None;
            self.stage_start = None;
        }
    }

    /// 开始记录一帧
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }
        self.frame_start = Some(Instant::now());
        self.current = Some(FrameProfile {
            frame_index: self.frame_counter,
            stages: Vec::new(),
            alloc_count: 0,
            alloc_bytes: 0,
            total_ns: 0,
        });
    }

    /// 结束记录一帧
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }
        if let Some(start) = self.frame_start.take() {
            let elapsed = start.elapsed().as_nanos() as u64;
            if let Some(mut profile) = self.current.take() {
                profile.total_ns = elapsed;
                if self.profiles.len() >= self.max_frames {
                    self.profiles.pop_front();
                }
                self.profiles.push_back(profile);
            }
        }
        self.frame_counter += 1;
    }

    /// 开始记录一个阶段
    pub fn begin_stage(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        self.stage_start = Some(Instant::now());
        self.current_stage_name = Some(name.to_string());
    }

    /// 结束记录当前阶段
    pub fn end_stage(&mut self) {
        if !self.enabled {
            return;
        }
        if let (Some(start), Some(name)) = (self.stage_start.take(), self.current_stage_name.take()) {
            let elapsed = start.elapsed().as_nanos() as u64;
            let stage = StageProfile { name, start_ns: 0, end_ns: 0, elapsed_ns: elapsed };
            if let Some(ref mut profile) = self.current {
                profile.stages.push(stage);
            }
        }
    }

    /// 记录一次堆分配
    pub fn record_alloc(&mut self, bytes: u64) {
        if !self.enabled {
            return;
        }
        if let Some(ref mut profile) = self.current {
            profile.alloc_count += 1;
            profile.alloc_bytes += bytes;
        }
    }

    /// 生成性能报告
    pub fn report(&self) -> FrameProfileReport {
        let n = self.profiles.len();
        if n == 0 {
            return FrameProfileReport {
                frame_count: 0,
                avg_frame_ns: 0,
                max_frame_ns: 0,
                min_frame_ns: 0,
                std_dev_ns: 0,
                stage_avg_ns: Vec::new(),
            };
        }

        let total_ns: u64 = self.profiles.iter().map(|p| p.total_ns).sum();
        let avg_frame_ns = total_ns / n as u64;
        let max_frame_ns = self.profiles.iter().map(|p| p.total_ns).max().unwrap_or(0);
        let min_frame_ns = self.profiles.iter().map(|p| p.total_ns).min().unwrap_or(0);

        let variance: f64 = self
            .profiles
            .iter()
            .map(|p| {
                let diff = p.total_ns as f64 - avg_frame_ns as f64;
                diff * diff
            })
            .sum::<f64>()
            / n as f64;
        let std_dev_ns = variance.sqrt() as u64;

        let mut stage_map: std::collections::HashMap<String, (u64, usize)> = std::collections::HashMap::new();
        for profile in &self.profiles {
            for stage in &profile.stages {
                let entry = stage_map.entry(stage.name.clone()).or_insert((0, 0));
                entry.0 += stage.elapsed_ns;
                entry.1 += 1;
            }
        }

        let mut stage_avg_ns: Vec<(String, u64)> =
            stage_map.into_iter().map(|(name, (total, count))| (name, total / count as u64)).collect();
        stage_avg_ns.sort_by(|a, b| b.1.cmp(&a.1));

        FrameProfileReport { frame_count: n, avg_frame_ns, max_frame_ns, min_frame_ns, std_dev_ns, stage_avg_ns }
    }
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self::new()
    }
}
