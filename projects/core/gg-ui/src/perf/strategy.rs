/// 性能优化策略
///
/// 已弃用，请使用 [`EditorUiOptimization`] 或 [`GameUiOptimization`] 替代
#[deprecated(since = "0.2.0", note = "请使用 EditorUiOptimization 或 GameUiOptimization 替代")]
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
