use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use gg_error::GResult;

use super::{
    manager::{BatchingManager, CacheItem, CacheManager, MemoryPool, MultiThreadManager, RenderCommand},
    strategy::{EditorUiOptimization, GameUiOptimization, PerformanceStats},
};

/// Editor UI 优化器
///
/// 专门针对编辑器 UI 场景的优化器，管理 Editor UI 专用优化策略的启用和执行
pub struct EditorUiOptimizer {
    /// 启用的策略列表
    enabled_strategies: Vec<EditorUiOptimization>,
    /// 性能统计
    stats: PerformanceStats,
}

impl EditorUiOptimizer {
    /// 创建新的 Editor UI 优化器，默认启用所有 Editor UI 策略
    pub fn new() -> Self {
        Self {
            enabled_strategies: vec![
                EditorUiOptimization::PreallocatedBuffers,
                EditorUiOptimization::DirtyFlagPartialUpdate,
                EditorUiOptimization::UberShaderBatching,
                EditorUiOptimization::UsageHintsGpuTransform,
                EditorUiOptimization::RetainedModeRendering,
            ],
            stats: PerformanceStats::default(),
        }
    }

    /// 启用指定的 Editor UI 优化策略
    pub fn enable(&mut self, strategy: EditorUiOptimization) {
        if !self.enabled_strategies.contains(&strategy) {
            self.enabled_strategies.push(strategy);
        }
    }

    /// 禁用指定的 Editor UI 优化策略
    pub fn disable(&mut self, strategy: EditorUiOptimization) {
        self.enabled_strategies.retain(|&s| s != strategy);
    }

    /// 执行 Editor UI 优化
    pub fn optimize(&mut self) -> GResult<()> {
        if self.enabled_strategies.contains(&EditorUiOptimization::PreallocatedBuffers) {}
        if self.enabled_strategies.contains(&EditorUiOptimization::DirtyFlagPartialUpdate) {}
        if self.enabled_strategies.contains(&EditorUiOptimization::UberShaderBatching) {}
        if self.enabled_strategies.contains(&EditorUiOptimization::UsageHintsGpuTransform) {}
        if self.enabled_strategies.contains(&EditorUiOptimization::RetainedModeRendering) {}
        Ok(())
    }

    /// 更新性能统计
    pub fn update_stats(&mut self, stats: PerformanceStats) {
        self.stats = stats;
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> &PerformanceStats {
        &self.stats
    }
}

impl Default for EditorUiOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Game UI 优化器
///
/// 专门针对游戏 UI 场景的优化器，管理 Game UI 专用优化策略的启用和执行
pub struct GameUiOptimizer {
    /// 启用的策略列表
    enabled_strategies: Vec<GameUiOptimization>,
    /// 性能统计
    stats: PerformanceStats,
}

impl GameUiOptimizer {
    /// 创建新的 Game UI 优化器，默认启用所有 Game UI 策略
    pub fn new() -> Self {
        Self {
            enabled_strategies: vec![
                GameUiOptimization::CanvasStaticDynamicSeparation,
                GameUiOptimization::DirtyFlagRebuild,
                GameUiOptimization::DynamicBatching,
                GameUiOptimization::MemoryPoolGcReduction,
                GameUiOptimization::MergeRebuildMode,
            ],
            stats: PerformanceStats::default(),
        }
    }

    /// 启用指定的 Game UI 优化策略
    pub fn enable(&mut self, strategy: GameUiOptimization) {
        if !self.enabled_strategies.contains(&strategy) {
            self.enabled_strategies.push(strategy);
        }
    }

    /// 禁用指定的 Game UI 优化策略
    pub fn disable(&mut self, strategy: GameUiOptimization) {
        self.enabled_strategies.retain(|&s| s != strategy);
    }

    /// 执行 Game UI 优化
    pub fn optimize(&mut self) -> GResult<()> {
        if self.enabled_strategies.contains(&GameUiOptimization::CanvasStaticDynamicSeparation) {}
        if self.enabled_strategies.contains(&GameUiOptimization::DirtyFlagRebuild) {}
        if self.enabled_strategies.contains(&GameUiOptimization::DynamicBatching) {}
        if self.enabled_strategies.contains(&GameUiOptimization::MemoryPoolGcReduction) {}
        if self.enabled_strategies.contains(&GameUiOptimization::MergeRebuildMode) {}
        Ok(())
    }

    /// 更新性能统计
    pub fn update_stats(&mut self, stats: PerformanceStats) {
        self.stats = stats;
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> &PerformanceStats {
        &self.stats
    }
}

impl Default for GameUiOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能优化器
pub struct PerformanceOptimizer {
    /// 批处理管理器
    batching_manager: BatchingManager,
    /// 缓存管理器
    cache_manager: CacheManager<Vec<u8>>,
    /// 多线程管理器
    multi_thread_manager: MultiThreadManager,
    /// 内存池
    memory_pool: MemoryPool<Vec<f32>>,
    /// 性能统计
    performance_stats: PerformanceStats,
    /// Editor UI 优化器
    editor_optimizer: EditorUiOptimizer,
    /// Game UI 优化器
    game_optimizer: GameUiOptimizer,
}

impl PerformanceOptimizer {
    /// 创建新的性能优化器
    pub fn new() -> Self {
        Self {
            batching_manager: BatchingManager::new(100),
            cache_manager: CacheManager::new(1024 * 1024 * 100),
            multi_thread_manager: MultiThreadManager::new(num_cpus::get()),
            memory_pool: MemoryPool::new(1000, || Vec::with_capacity(1024)),
            performance_stats: PerformanceStats::default(),
            editor_optimizer: EditorUiOptimizer::new(),
            game_optimizer: GameUiOptimizer::new(),
        }
    }

    /// 执行批处理优化
    pub fn optimize_batching(&mut self, commands: Vec<RenderCommand>) -> GResult<()> {
        for command in commands {
            self.batching_manager.add_render_command(command);
        }
        self.batching_manager.execute_batches()
    }

    /// 执行缓存优化
    pub fn optimize_caching(&mut self, key: &str, data: Vec<u8>) -> GResult<Arc<Mutex<CacheItem<Vec<u8>>>>> {
        if let Some(item) = self.cache_manager.get(key) {
            Ok(item)
        }
        else {
            let size = data.len();
            self.cache_manager.put(key.to_string(), data, size)
        }
    }

    /// 执行多线程优化
    pub fn optimize_multi_threading(&self, tasks: Vec<Box<dyn FnOnce() + Send + 'static>>) {
        for task in tasks {
            self.multi_thread_manager.add_task(task);
        }
    }

    /// 执行内存池优化
    pub fn optimize_memory_pooling(&mut self) -> Vec<f32> {
        self.memory_pool.acquire()
    }

    /// 释放内存池对象
    pub fn release_memory_pool_object(&mut self, obj: Vec<f32>) {
        self.memory_pool.release(obj);
    }

    /// 更新性能统计
    pub fn update_performance_stats(&mut self, stats: PerformanceStats) {
        self.performance_stats = stats;
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    /// 优化编辑器UI性能，委托给 Editor UI 优化器执行
    pub fn optimize_editor_ui(&mut self) -> GResult<()> {
        self.editor_optimizer.optimize()
    }

    /// 优化游戏UI性能，委托给 Game UI 优化器执行
    pub fn optimize_game_ui(&mut self) -> GResult<()> {
        self.game_optimizer.optimize()
    }

    /// 获取 Editor UI 优化器的不可变引用
    pub fn editor_optimizer(&self) -> &EditorUiOptimizer {
        &self.editor_optimizer
    }

    /// 获取 Game UI 优化器的不可变引用
    pub fn game_optimizer(&self) -> &GameUiOptimizer {
        &self.game_optimizer
    }

    /// 获取 Editor UI 优化器的可变引用
    pub fn editor_optimizer_mut(&mut self) -> &mut EditorUiOptimizer {
        &mut self.editor_optimizer
    }

    /// 获取 Game UI 优化器的可变引用
    pub fn game_optimizer_mut(&mut self) -> &mut GameUiOptimizer {
        &mut self.game_optimizer
    }
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能分析工具
pub struct PerformanceAnalyzer {
    /// 性能统计
    performance_stats: PerformanceStats,
    /// 分析状态
    is_analyzing: bool,
    /// 开始时间
    start_time: Instant,
}

impl PerformanceAnalyzer {
    /// 创建新的性能分析工具
    pub fn new() -> Self {
        Self { performance_stats: PerformanceStats::default(), is_analyzing: false, start_time: Instant::now() }
    }

    /// 开始分析
    pub fn start_analyzing(&mut self) {
        self.is_analyzing = true;
        self.start_time = Instant::now();
        self.performance_stats = PerformanceStats::default();
    }

    /// 停止分析
    pub fn stop_analyzing(&mut self) {
        self.is_analyzing = false;
    }

    /// 记录绘制调用
    pub fn record_draw_call(&mut self) {
        if self.is_analyzing {
            self.performance_stats.draw_calls += 1;
        }
    }

    /// 记录三角形数量
    pub fn record_triangles(&mut self, count: u32) {
        if self.is_analyzing {
            self.performance_stats.triangles += count;
        }
    }

    /// 记录顶点数量
    pub fn record_vertices(&mut self, count: u32) {
        if self.is_analyzing {
            self.performance_stats.vertices += count;
        }
    }

    /// 记录渲染时间
    pub fn record_render_time(&mut self, time: f32) {
        if self.is_analyzing {
            self.performance_stats.render_time += time;
        }
    }

    /// 记录布局计算时间
    pub fn record_layout_time(&mut self, time: f32) {
        if self.is_analyzing {
            self.performance_stats.layout_time += time;
        }
    }

    /// 记录事件处理时间
    pub fn record_event_time(&mut self, time: f32) {
        if self.is_analyzing {
            self.performance_stats.event_time += time;
        }
    }

    /// 记录内存使用
    pub fn record_memory_usage(&mut self, usage: usize) {
        if self.is_analyzing {
            self.performance_stats.memory_usage = usage;
        }
    }

    /// 记录帧率
    pub fn record_frame_rate(&mut self, frame_rate: f32) {
        if self.is_analyzing {
            self.performance_stats.frame_rate = frame_rate;
        }
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> String {
        format!(
            concat!(
                "Performance Report:\n",
                "Draw Calls: {}\n",
                "Triangles: {}\n",
                "Vertices: {}\n",
                "Render Time: {:.2}ms\n",
                "Layout Time: {:.2}ms\n",
                "Event Time: {:.2}ms\n",
                "Memory Usage: {}KB\n",
                "Frame Rate: {:.2}fps\n",
            ),
            self.performance_stats.draw_calls,
            self.performance_stats.triangles,
            self.performance_stats.vertices,
            self.performance_stats.render_time,
            self.performance_stats.layout_time,
            self.performance_stats.event_time,
            self.performance_stats.memory_usage / 1024,
            self.performance_stats.frame_rate
        )
    }

    /// 分析性能瓶颈
    pub fn analyze_bottlenecks(&self) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        if self.performance_stats.draw_calls > 100 {
            bottlenecks.push(format!("High draw calls: {}", self.performance_stats.draw_calls));
        }

        if self.performance_stats.triangles > 10000 {
            bottlenecks.push(format!("High triangle count: {}", self.performance_stats.triangles));
        }

        if self.performance_stats.render_time > 1.0 {
            bottlenecks.push(format!("High render time: {:.2}ms", self.performance_stats.render_time));
        }

        if self.performance_stats.layout_time > 0.5 {
            bottlenecks.push(format!("High layout time: {:.2}ms", self.performance_stats.layout_time));
        }

        if self.performance_stats.event_time > 0.5 {
            bottlenecks.push(format!("High event time: {:.2}ms", self.performance_stats.event_time));
        }

        if self.performance_stats.frame_rate < 30.0 {
            bottlenecks.push(format!("Low frame rate: {:.2}fps", self.performance_stats.frame_rate));
        }

        bottlenecks
    }
}
