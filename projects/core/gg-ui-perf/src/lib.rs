//! GG 引擎 UI 性能优化模块
//! 
//! 提供编辑器UI和游戏UI的性能优化策略，包括批处理、缓存和多线程处理

#![warn(missing_docs)]

use std::collections::HashMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use gg_error::GResult;

/// 性能优化策略
///
/// 已弃用，请使用 [`EditorUiOptimization`] 或 [`GameUiOptimization`] 替代
#[deprecated(
    since = "0.2.0",
    note = "请使用 EditorUiOptimization 或 GameUiOptimization 替代"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationStrategy {
    /// 批处理
    Batching,
    /// 缓存
    Caching,
    /// 多线程
    MultiThreading,
    /// 内存池
    MemoryPooling,
    /// 延迟加载
    LazyLoading,
    /// 资源压缩
    ResourceCompression,
}

/// Editor UI 专用优化策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorUiOptimization {
    /// 预分配 GPU 缓冲区
    PreallocatedBuffers,
    /// 脏标记局部更新
    DirtyFlagPartialUpdate,
    /// Uber-Shader 合批
    UberShaderBatching,
    /// UsageHints GPU 变换
    UsageHintsGpuTransform,
    /// 保留模式渲染
    RetainedModeRendering,
}

/// Game UI 专用优化策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameUiOptimization {
    /// Canvas 动静分离
    CanvasStaticDynamicSeparation,
    /// 脏标记重建
    DirtyFlagRebuild,
    /// 动态批处理
    DynamicBatching,
    /// 内存池减少 GC
    MemoryPoolGcReduction,
    /// 合并-重建模式
    MergeRebuildMode,
}

/// 性能统计
#[derive(Debug, Default, Clone)]
pub struct PerformanceStats {
    /// 绘制调用次数
    pub draw_calls: u32,
    /// 三角形数量
    pub triangles: u32,
    /// 顶点数量
    pub vertices: u32,
    /// 渲染时间 (ms)
    pub render_time: f32,
    /// 布局计算时间 (ms)
    pub layout_time: f32,
    /// 事件处理时间 (ms)
    pub event_time: f32,
    /// 内存使用 (bytes)
    pub memory_usage: usize,
    /// 帧率
    pub frame_rate: f32,
}

/// 批处理管理器
pub struct BatchingManager {
    /// 批处理组
    batches: Vec<Batch>,
    /// 最大批处理大小
    max_batch_size: usize,
    /// 当前批处理索引
    current_batch: usize,
}

/// 批处理
#[derive(Debug, Default)]
pub struct Batch {
    /// 绘制调用次数
    draw_calls: u32,
    /// 三角形数量
    triangles: u32,
    /// 顶点数量
    vertices: u32,
    /// 材质ID
    material_id: u64,
    /// 渲染命令
    render_commands: Vec<RenderCommand>,
}

/// 渲染命令
#[derive(Debug, Clone)]
pub struct RenderCommand {
    /// 顶点数据
    pub vertex_data: Vec<f32>,
    /// 索引数据
    pub index_data: Vec<u32>,
    /// 材质ID
    pub material_id: u64,
    /// 变换矩阵
    pub transform: [f32; 16],
}

impl BatchingManager {
    /// 创建新的批处理管理器
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            batches: vec![Batch::default()],
            max_batch_size,
            current_batch: 0,
        }
    }

    /// 添加渲染命令
    pub fn add_render_command(&mut self, command: RenderCommand) {
        let current_batch = &mut self.batches[self.current_batch];

        // 检查是否可以添加到当前批处理
        if current_batch.material_id == 0 || current_batch.material_id == command.material_id {
            if current_batch.render_commands.len() < self.max_batch_size {
                current_batch.render_commands.push(command);
                current_batch.draw_calls += 1;
                current_batch.triangles += (command.index_data.len() / 3) as u32;
                current_batch.vertices += (command.vertex_data.len() / 3) as u32;
                if current_batch.material_id == 0 {
                    current_batch.material_id = command.material_id;
                }
                return;
            }
        }

        // 创建新的批处理
        let mut new_batch = Batch::default();
        new_batch.render_commands.push(command);
        new_batch.draw_calls += 1;
        new_batch.triangles += (command.index_data.len() / 3) as u32;
        new_batch.vertices += (command.vertex_data.len() / 3) as u32;
        new_batch.material_id = command.material_id;
        self.batches.push(new_batch);
        self.current_batch += 1;
    }

    /// 执行批处理
    pub fn execute_batches(&mut self) -> GResult<()> {
        // 执行所有批处理
        for batch in &self.batches {
            if !batch.render_commands.is_empty() {
                // 这里应该实现实际的批处理渲染逻辑
                // 例如，合并顶点数据，执行单次绘制调用
            }
        }

        // 重置批处理
        self.batches.clear();
        self.batches.push(Batch::default());
        self.current_batch = 0;

        Ok(())
    }

    /// 获取批处理统计信息
    pub fn get_batch_stats(&self) -> (u32, u32) {
        let total_draw_calls = self.batches.iter().map(|b| b.draw_calls).sum::<u32>();
        let total_batches = self.batches.len() as u32;
        (total_draw_calls, total_batches)
    }
}

/// 缓存管理器
pub struct CacheManager<T> {
    /// 缓存存储
    cache: HashMap<String, Arc<Mutex<CacheItem<T>>>>,
    /// 最大缓存大小
    max_cache_size: usize,
    /// 当前缓存大小
    current_cache_size: usize,
    /// 缓存命中次数
    hits: u32,
    /// 缓存未命中次数
    misses: u32,
}

/// 缓存项
#[derive(Debug)]
pub struct CacheItem<T> {
    /// 数据
    data: T,
    /// 大小
    size: usize,
    /// 最后访问时间
    last_accessed: Instant,
    /// 引用计数
    ref_count: usize,
}

impl<T> CacheManager<T> {
    /// 创建新的缓存管理器
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size,
            current_cache_size: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// 获取缓存项
    pub fn get(&mut self, key: &str) -> Option<Arc<Mutex<CacheItem<T>>>> {
        if let Some(item) = self.cache.get(key) {
            // 更新最后访问时间
            if let Ok(mut item) = item.lock() {
                item.last_accessed = Instant::now();
                item.ref_count += 1;
            }
            self.hits += 1;
            Some(Arc::clone(item))
        } else {
            self.misses += 1;
            None
        }
    }

    /// 添加缓存项
    pub fn put(&mut self, key: String, data: T, size: usize) -> GResult<Arc<Mutex<CacheItem<T>>>> {
        // 检查缓存大小
        if self.current_cache_size + size > self.max_cache_size {
            // 清理缓存
            self.evict();
        }

        // 创建缓存项
        let item = Arc::new(Mutex::new(CacheItem {
            data,
            size,
            last_accessed: Instant::now(),
            ref_count: 1,
        }));

        // 添加到缓存
        self.cache.insert(key, Arc::clone(&item));
        self.current_cache_size += size;

        Ok(item)
    }

    /// 释放缓存项
    pub fn release(&mut self, key: &str) {
        if let Some(item) = self.cache.get(key) {
            if let Ok(mut item) = item.lock() {
                item.ref_count -= 1;
                if item.ref_count <= 0 {
                    // 从缓存中移除
                    let size = item.size;
                    self.cache.remove(key);
                    self.current_cache_size -= size;
                }
            }
        }
    }

    /// 清理缓存
    fn evict(&mut self) {
        // 按最后访问时间排序，移除最久未使用的项
        let mut items: Vec<(String, Instant)> = self
            .cache
            .iter()
            .filter_map(|(key, item)| {
                if let Ok(item) = item.lock() {
                    if item.ref_count <= 0 {
                        Some((key.clone(), item.last_accessed))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // 按最后访问时间排序
        items.sort_by(|a, b| a.1.cmp(&b.1));

        // 移除最久未使用的项
        for (key, _) in items {
            if let Some(item) = self.cache.get(&key) {
                if let Ok(item) = item.lock() {
                    let size = item.size;
                    self.cache.remove(&key);
                    self.current_cache_size -= size;

                    if self.current_cache_size <= self.max_cache_size * 3 / 4 {
                        break;
                    }
                }
            }
        }
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> (u32, u32, usize) {
        (self.hits, self.misses, self.current_cache_size)
    }

    /// 清除缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_cache_size = 0;
        self.hits = 0;
        self.misses = 0;
    }
}

/// 多线程管理器
pub struct MultiThreadManager {
    /// 线程池大小
    pool_size: usize,
    /// 任务队列
    task_queue: Arc<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>>,
    /// 线程句柄
    threads: Vec<thread::JoinHandle<()>>,
    /// 停止标志
    stop: Arc<Mutex<bool>>,
}

impl MultiThreadManager {
    /// 创建新的多线程管理器
    pub fn new(pool_size: usize) -> Self {
        let task_queue = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(Mutex::new(false));
        let mut threads = Vec::new();

        for _ in 0..pool_size {
            let task_queue_clone = Arc::clone(&task_queue);
            let stop_clone = Arc::clone(&stop);

            let thread = thread::spawn(move || {
                while !*stop_clone.lock().unwrap() {
                    let task = {
                        let mut queue = task_queue_clone.lock().unwrap();
                        queue.pop()
                    };

                    if let Some(task) = task {
                        task();
                    } else {
                        // 短暂休眠，避免忙等
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            });

            threads.push(thread);
        }

        Self {
            pool_size,
            task_queue,
            threads,
            stop,
        }
    }

    /// 添加任务
    pub fn add_task<F>(&self, task: F) where F: FnOnce() + Send + 'static {
        self.task_queue.lock().unwrap().push(Box::new(task));
    }

    /// 等待所有任务完成
    pub fn wait_completion(&self) {
        // 等待任务队列为空
        while !self.task_queue.lock().unwrap().is_empty() {
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// 停止线程池
    pub fn stop(&mut self) {
        *self.stop.lock().unwrap() = true;

        // 等待所有线程结束
        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }
}

impl Drop for MultiThreadManager {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 内存池
pub struct MemoryPool<T> {
    /// 空闲对象池
    free_objects: Vec<T>,
    /// 最大池大小
    max_pool_size: usize,
    /// 创建对象的闭包
    create_fn: Box<dyn Fn() -> T>,
}

impl<T> MemoryPool<T> {
    /// 创建新的内存池
    pub fn new(max_pool_size: usize, create_fn: impl Fn() -> T + 'static) -> Self {
        Self {
            free_objects: Vec::new(),
            max_pool_size,
            create_fn: Box::new(create_fn),
        }
    }

    /// 获取对象
    pub fn acquire(&mut self) -> T {
        if let Some(obj) = self.free_objects.pop() {
            obj
        } else {
            (self.create_fn)()
        }
    }

    /// 释放对象
    pub fn release(&mut self, obj: T) {
        if self.free_objects.len() < self.max_pool_size {
            self.free_objects.push(obj);
        }
    }

    /// 清除内存池
    pub fn clear(&mut self) {
        self.free_objects.clear();
    }

    /// 获取内存池大小
    pub fn size(&self) -> usize {
        self.free_objects.len()
    }
}

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
        if self.enabled_strategies.contains(&EditorUiOptimization::PreallocatedBuffers) {
            // 预分配 GPU 缓冲区优化逻辑
        }
        if self.enabled_strategies.contains(&EditorUiOptimization::DirtyFlagPartialUpdate) {
            // 脏标记局部更新优化逻辑
        }
        if self.enabled_strategies.contains(&EditorUiOptimization::UberShaderBatching) {
            // Uber-Shader 合批优化逻辑
        }
        if self.enabled_strategies.contains(&EditorUiOptimization::UsageHintsGpuTransform) {
            // UsageHints GPU 变换优化逻辑
        }
        if self.enabled_strategies.contains(&EditorUiOptimization::RetainedModeRendering) {
            // 保留模式渲染优化逻辑
        }
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
        if self.enabled_strategies.contains(&GameUiOptimization::CanvasStaticDynamicSeparation) {
            // Canvas 动静分离优化逻辑
        }
        if self.enabled_strategies.contains(&GameUiOptimization::DirtyFlagRebuild) {
            // 脏标记重建优化逻辑
        }
        if self.enabled_strategies.contains(&GameUiOptimization::DynamicBatching) {
            // 动态批处理优化逻辑
        }
        if self.enabled_strategies.contains(&GameUiOptimization::MemoryPoolGcReduction) {
            // 内存池减少 GC 优化逻辑
        }
        if self.enabled_strategies.contains(&GameUiOptimization::MergeRebuildMode) {
            // 合并-重建模式优化逻辑
        }
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
        } else {
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
        Self {
            performance_stats: PerformanceStats::default(),
            is_analyzing: false,
            start_time: Instant::now(),
        }
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
            "Performance Report:\n"
            "Draw Calls: {}\n"
            "Triangles: {}\n"
            "Vertices: {}\n"
            "Render Time: {:.2}ms\n"
            "Layout Time: {:.2}ms\n"
            "Event Time: {:.2}ms\n"
            "Memory Usage: {}KB\n"
            "Frame Rate: {:.2}fps\n",
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

        // 分析绘制调用
        if self.performance_stats.draw_calls > 100 {
            bottlenecks.push(format!("High draw calls: {}", self.performance_stats.draw_calls));
        }

        // 分析三角形数量
        if self.performance_stats.triangles > 10000 {
            bottlenecks.push(format!("High triangle count: {}", self.performance_stats.triangles));
        }

        // 分析渲染时间
        if self.performance_stats.render_time > 1.0 {
            bottlenecks.push(format!("High render time: {:.2}ms", self.performance_stats.render_time));
        }

        // 分析布局计算时间
        if self.performance_stats.layout_time > 0.5 {
            bottlenecks.push(format!("High layout time: {:.2}ms", self.performance_stats.layout_time));
        }

        // 分析事件处理时间
        if self.performance_stats.event_time > 0.5 {
            bottlenecks.push(format!("High event time: {:.2}ms", self.performance_stats.event_time));
        }

        // 分析帧率
        if self.performance_stats.frame_rate < 30.0 {
            bottlenecks.push(format!("Low frame rate: {:.2}fps", self.performance_stats.frame_rate));
        }

        bottlenecks
    }
}

/// Canvas 脏标记位标志
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasDirtyFlag(u32);

impl CanvasDirtyFlag {
    /// 顶点数据脏
    pub const VERTICES: CanvasDirtyFlag = CanvasDirtyFlag(1 << 0);
    /// 材质数据脏
    pub const MATERIAL: CanvasDirtyFlag = CanvasDirtyFlag(1 << 1);
    /// 布局数据脏
    pub const LAYOUT: CanvasDirtyFlag = CanvasDirtyFlag(1 << 2);
    /// 所有标记
    pub const ALL: CanvasDirtyFlag = CanvasDirtyFlag(Self::VERTICES.0 | Self::MATERIAL.0 | Self::LAYOUT.0);

    /// 判断是否包含指定脏标记
    pub fn contains(self, other: CanvasDirtyFlag) -> bool {
        self.0 & other.0 != 0
    }

    /// 判断是否没有任何脏标记
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 获取内部位值
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl Default for CanvasDirtyFlag {
    fn default() -> Self {
        CanvasDirtyFlag::ALL
    }
}

impl BitOr for CanvasDirtyFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        CanvasDirtyFlag(self.0 | rhs.0)
    }
}

impl BitOrAssign for CanvasDirtyFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for CanvasDirtyFlag {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        CanvasDirtyFlag(self.0 & rhs.0)
    }
}

impl BitAndAssign for CanvasDirtyFlag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for CanvasDirtyFlag {
    type Output = Self;

    fn not(self) -> Self::Output {
        CanvasDirtyFlag(!self.0 & CanvasDirtyFlag::ALL.0)
    }
}

/// Canvas 渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasRenderMode {
    /// 屏幕空间
    ScreenSpace,
    /// 世界空间
    WorldSpace,
    /// 相机空间
    CameraSpace,
}

/// Canvas 中的 UI 元素
#[derive(Debug, Clone)]
pub struct CanvasElement {
    /// 元素 ID
    pub id: String,
    /// 材质 ID
    pub material_id: u64,
    /// 纹理 ID
    pub texture_id: u64,
    /// 顶点数据
    pub vertex_data: Vec<f32>,
    /// 索引数据
    pub index_data: Vec<u32>,
    /// 脏标记
    pub dirty_flags: CanvasDirtyFlag,
    /// 渲染排序
    pub sort_order: i32,
}

/// 批处理合并结果
#[derive(Debug, Clone)]
pub struct CanvasBatch {
    /// 合并后的材质 ID
    pub material_id: u64,
    /// 合并后的纹理 ID
    pub texture_id: u64,
    /// 合并后的顶点数据
    pub merged_vertex_data: Vec<f32>,
    /// 合并后的索引数据
    pub merged_index_data: Vec<u32>,
    /// 包含的元素数量
    pub element_count: usize,
    /// Draw Call 数量
    pub draw_call_count: u32,
}

/// 负责每帧收集和合并 Canvas 下的 UI 网格
pub struct CanvasRebuilder {
    /// Canvas ID
    pub canvas_id: String,
    /// 渲染模式
    pub render_mode: CanvasRenderMode,
    /// Canvas 中的所有元素
    pub elements: Vec<CanvasElement>,
    /// 当前帧的批处理结果
    pub batches: Vec<CanvasBatch>,
    /// 是否为静态 Canvas（静态 Canvas 跳过每帧重建）
    pub is_static: bool,
    /// 是否有脏元素
    pub has_dirty_elements: bool,
}

impl CanvasRebuilder {
    /// 创建新的 Canvas 重建器
    pub fn new(canvas_id: &str, render_mode: CanvasRenderMode) -> Self {
        Self {
            canvas_id: canvas_id.to_string(),
            render_mode,
            elements: Vec::new(),
            batches: Vec::new(),
            is_static: false,
            has_dirty_elements: true,
        }
    }

    /// 添加 UI 元素
    pub fn add_element(&mut self, element: CanvasElement) {
        if !element.dirty_flags.is_empty() {
            self.has_dirty_elements = true;
        }
        self.elements.push(element);
    }

    /// 标记元素顶点为脏
    pub fn set_vertices_dirty(&mut self, element_id: &str) {
        if let Some(element) = self.elements.iter_mut().find(|e| e.id == element_id) {
            element.dirty_flags = element.dirty_flags | CanvasDirtyFlag::VERTICES;
            self.has_dirty_elements = true;
        }
    }

    /// 标记元素材质为脏
    pub fn set_material_dirty(&mut self, element_id: &str) {
        if let Some(element) = self.elements.iter_mut().find(|e| e.id == element_id) {
            element.dirty_flags = element.dirty_flags | CanvasDirtyFlag::MATERIAL;
            self.has_dirty_elements = true;
        }
    }

    /// 执行合并-重建：收集脏元素，按材质/纹理分组合并网格，生成 Draw Call 队列。
    /// 如果是静态 Canvas 且无脏元素，跳过重建
    pub fn rebuild(&mut self) {
        if self.is_static && !self.has_dirty_elements {
            return;
        }

        self.batches.clear();

        let mut group_map: HashMap<(u64, u64), Vec<&CanvasElement>> = HashMap::new();

        for element in &self.elements {
            let key = (element.material_id, element.texture_id);
            group_map.entry(key).or_default().push(element);
        }

        for ((material_id, texture_id), group_elements) in &group_map {
            let mut merged_vertex_data = Vec::new();
            let mut merged_index_data = Vec::new();
            let mut element_count = 0usize;
            let mut vertex_offset = 0u32;

            let mut sorted_elements: Vec<&&CanvasElement> = group_elements.iter().collect();
            sorted_elements.sort_by_key(|e| e.sort_order);

            for element in sorted_elements {
                merged_vertex_data.extend_from_slice(&element.vertex_data);
                for &index in &element.index_data {
                    merged_index_data.push(index + vertex_offset);
                }
                vertex_offset += (element.vertex_data.len() / 3) as u32;
                element_count += 1;
            }

            self.batches.push(CanvasBatch {
                material_id: *material_id,
                texture_id: *texture_id,
                merged_vertex_data,
                merged_index_data,
                element_count,
                draw_call_count: 1,
            });
        }

        for element in &mut self.elements {
            element.dirty_flags = CanvasDirtyFlag::default() & !CanvasDirtyFlag::ALL;
        }
        self.has_dirty_elements = false;
    }

    /// 获取当前帧的批处理结果
    pub fn get_batches(&self) -> &[CanvasBatch] {
        &self.batches
    }

    /// 设置 Canvas 是否为静态
    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    /// 动静分离：将指定元素分离到新的动态 Canvas，返回新的 CanvasRebuilder
    pub fn separate_dynamic_elements(&mut self, dynamic_element_ids: &[&str]) -> CanvasRebuilder {
        let mut dynamic_rebuilder = CanvasRebuilder::new(
            &format!("{}_dynamic", self.canvas_id),
            self.render_mode,
        );

        let dynamic_id_set: std::collections::HashSet<&str> = dynamic_element_ids.iter().copied().collect();

        let mut remaining = Vec::new();
        for element in self.elements.drain(..) {
            if dynamic_id_set.contains(element.id.as_str()) {
                dynamic_rebuilder.add_element(element);
            } else {
                remaining.push(element);
            }
        }

        self.elements = remaining;
        self.has_dirty_elements = self.elements.iter().any(|e| !e.dirty_flags.is_empty());

        dynamic_rebuilder
    }
}

/// 批处理合并结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMergeResult {
    /// 成功合并
    Merged,
    /// 材质不同，无法合并
    MaterialDifferent,
    /// 纹理不同，无法合并
    TextureDifferent,
    /// 渲染顺序被中断
    SortOrderInterrupted,
}

/// 单个 Canvas 的批处理信息
#[derive(Debug, Clone)]
pub struct CanvasBatchInfo {
    /// Canvas ID
    pub canvas_id: String,
    /// 总 Draw Call 数量
    pub total_draw_calls: u32,
    /// 总批次数
    pub total_batches: u32,
    /// 总元素数量
    pub total_elements: usize,
    /// 每个元素的合并结果
    pub merge_results: Vec<(String, BatchMergeResult)>,
    /// 是否为静态 Canvas
    pub is_static: bool,
}

/// 动静分离建议
#[derive(Debug, Clone)]
pub struct DynamicSeparationSuggestion {
    /// 建议分离的元素 ID
    pub element_id: String,
    /// 当前所在 Canvas ID
    pub parent_canvas_id: String,
    /// 分离原因
    pub reason: String,
    /// 脏标记频率（0.0~1.0）
    pub dirty_frequency: f32,
}

/// 批处理调试器
pub struct CanvasBatchDebugger {
    /// 所有 Canvas 的批处理信息
    pub canvas_infos: Vec<CanvasBatchInfo>,
    /// 动静分离建议
    pub suggestions: Vec<DynamicSeparationSuggestion>,
    /// 元素 ID → 最近 N 帧的脏标记历史
    pub dirty_history: HashMap<String, Vec<bool>>,
    /// 历史窗口大小（默认 60 帧）
    pub history_window: usize,
}

impl CanvasBatchDebugger {
    /// 创建新的批处理调试器
    pub fn new() -> Self {
        Self {
            canvas_infos: Vec::new(),
            suggestions: Vec::new(),
            dirty_history: HashMap::new(),
            history_window: 60,
        }
    }

    /// 记录一帧的批处理信息，接收所有 Canvas 的 CanvasRebuilder 引用
    pub fn record_frame(&mut self, rebuilders: &[&CanvasRebuilder]) {
        self.canvas_infos.clear();

        for rebuilder in rebuilders {
            let total_draw_calls: u32 = rebuilder.batches.iter().map(|b| b.draw_call_count).sum();
            let total_batches = rebuilder.batches.len() as u32;
            let total_elements = rebuilder.elements.len();
            let merge_results = Self::compute_merge_results(rebuilder);

            for element in &rebuilder.elements {
                let is_dirty = !element.dirty_flags.is_empty();
                let history = self.dirty_history.entry(element.id.clone()).or_default();
                history.push(is_dirty);
                if history.len() > self.history_window {
                    history.remove(0);
                }
            }

            self.canvas_infos.push(CanvasBatchInfo {
                canvas_id: rebuilder.canvas_id.clone(),
                total_draw_calls,
                total_batches,
                total_elements,
                merge_results,
                is_static: rebuilder.is_static,
            });
        }
    }

    /// 分析所有 Canvas 的批处理数据，生成动静分离建议。
    /// 规则：如果元素在最近 N 帧中脏标记频率超过 50%，建议分离到独立 Canvas
    pub fn analyze(&self) -> Vec<DynamicSeparationSuggestion> {
        let mut result = Vec::new();

        for canvas_info in &self.canvas_infos {
            for (element_id, _) in &canvas_info.merge_results {
                if let Some(history) = self.dirty_history.get(element_id) {
                    if history.is_empty() {
                        continue;
                    }
                    let dirty_count = history.iter().filter(|&&d| d).count();
                    let frequency = dirty_count as f32 / history.len() as f32;
                    if frequency > 0.5 {
                        result.push(DynamicSeparationSuggestion {
                            element_id: element_id.clone(),
                            parent_canvas_id: canvas_info.canvas_id.clone(),
                            reason: format!(
                                "元素脏标记频率 {:.1}% 超过阈值 50%，建议分离到独立动态 Canvas",
                                frequency * 100.0
                            ),
                            dirty_frequency: frequency,
                        });
                    }
                }
            }
        }

        result
    }

    /// 获取指定 Canvas 的批处理信息
    pub fn get_canvas_info(&self, canvas_id: &str) -> Option<&CanvasBatchInfo> {
        self.canvas_infos.iter().find(|info| info.canvas_id == canvas_id)
    }

    /// 生成调试报告，包含每个 Canvas 的 Draw Call 数量、批处理合并情况、未合并原因、动静分离建议
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Canvas Batch Debug Report ===\n\n");

        for canvas_info in &self.canvas_infos {
            report.push_str(&format!("Canvas: {}\n", canvas_info.canvas_id));
            report.push_str(&format!(
                "  Static: {}\n",
                if canvas_info.is_static { "Yes" } else { "No" }
            ));
            report.push_str(&format!("  Draw Calls: {}\n", canvas_info.total_draw_calls));
            report.push_str(&format!("  Batches: {}\n", canvas_info.total_batches));
            report.push_str(&format!("  Elements: {}\n", canvas_info.total_elements));

            report.push_str("  Merge Results:\n");
            for (element_id, merge_result) in &canvas_info.merge_results {
                let label = match merge_result {
                    BatchMergeResult::Merged => "Merged",
                    BatchMergeResult::MaterialDifferent => "Material Different",
                    BatchMergeResult::TextureDifferent => "Texture Different",
                    BatchMergeResult::SortOrderInterrupted => "Sort Order Interrupted",
                };
                report.push_str(&format!("    {} -> {}\n", element_id, label));
            }

            report.push('\n');
        }

        let suggestions = self.analyze();
        if suggestions.is_empty() {
            report.push_str("No dynamic separation suggestions.\n");
        } else {
            report.push_str("=== Dynamic Separation Suggestions ===\n\n");
            for suggestion in &suggestions {
                report.push_str(&format!("Element: {}\n", suggestion.element_id));
                report.push_str(&format!("  Canvas: {}\n", suggestion.parent_canvas_id));
                report.push_str(&format!("  Reason: {}\n", suggestion.reason));
                report.push_str(&format!(
                    "  Dirty Frequency: {:.1}%\n",
                    suggestion.dirty_frequency * 100.0
                ));
                report.push('\n');
            }
        }

        report
    }

    /// 计算每个元素的合并结果
    fn compute_merge_results(rebuilder: &CanvasRebuilder) -> Vec<(String, BatchMergeResult)> {
        let mut results = Vec::new();
        let elements = &rebuilder.elements;

        for element in elements {
            let element_key = (element.material_id, element.texture_id);
            let same_group_count = elements
                .iter()
                .filter(|e| (e.material_id, e.texture_id) == element_key)
                .count();

            if same_group_count > 1 {
                let group_sort_orders: Vec<i32> = elements
                    .iter()
                    .filter(|e| (e.material_id, e.texture_id) == element_key)
                    .map(|e| e.sort_order)
                    .collect();

                let min_sort = *group_sort_orders.iter().min().unwrap_or(&element.sort_order);
                let max_sort = *group_sort_orders.iter().max().unwrap_or(&element.sort_order);

                let has_interrupt = elements
                    .iter()
                    .filter(|e| (e.material_id, e.texture_id) != element_key)
                    .any(|other| other.sort_order >= min_sort && other.sort_order <= max_sort);

                if has_interrupt {
                    results.push((element.id.clone(), BatchMergeResult::SortOrderInterrupted));
                } else {
                    results.push((element.id.clone(), BatchMergeResult::Merged));
                }
            } else {
                let has_same_material_diff_texture = elements
                    .iter()
                    .any(|e| e.material_id == element.material_id && e.texture_id != element.texture_id);
                let has_diff_material = elements
                    .iter()
                    .any(|e| e.material_id != element.material_id);

                if has_same_material_diff_texture {
                    results.push((element.id.clone(), BatchMergeResult::TextureDifferent));
                } else if has_diff_material {
                    results.push((element.id.clone(), BatchMergeResult::MaterialDifferent));
                } else {
                    results.push((element.id.clone(), BatchMergeResult::Merged));
                }
            }
        }

        results
    }
}

impl Default for CanvasBatchDebugger {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能优化模块
pub mod perf {
    use super::*;

    /// 创建性能优化器
    pub fn create_performance_optimizer() -> PerformanceOptimizer {
        PerformanceOptimizer::new()
    }

    /// 创建性能分析工具
    pub fn create_performance_analyzer() -> PerformanceAnalyzer {
        PerformanceAnalyzer::new()
    }

    /// 创建批处理管理器
    pub fn create_batching_manager(max_batch_size: usize) -> BatchingManager {
        BatchingManager::new(max_batch_size)
    }

    /// 创建缓存管理器
    pub fn create_cache_manager<T>(max_cache_size: usize) -> CacheManager<T> {
        CacheManager::new(max_cache_size)
    }

    /// 创建多线程管理器
    pub fn create_multi_thread_manager(pool_size: usize) -> MultiThreadManager {
        MultiThreadManager::new(pool_size)
    }

    /// 创建内存池
    pub fn create_memory_pool<T>(max_pool_size: usize, create_fn: impl Fn() -> T + 'static) -> MemoryPool<T> {
        MemoryPool::new(max_pool_size, create_fn)
    }

    /// 创建 Editor UI 优化器
    pub fn create_editor_ui_optimizer() -> EditorUiOptimizer {
        EditorUiOptimizer::new()
    }

    /// 创建 Game UI 优化器
    pub fn create_game_ui_optimizer() -> GameUiOptimizer {
        GameUiOptimizer::new()
    }
}
