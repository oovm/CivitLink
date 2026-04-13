//! 性能优化模块
//!
//! 提供 UI 性能优化相关的策略、管理器、分析器和 Canvas 批处理功能。

mod analyzer;
mod canvas;
mod manager;
mod strategy;

pub use analyzer::{EditorUiOptimizer, GameUiOptimizer, PerformanceAnalyzer, PerformanceOptimizer};
pub use canvas::{
    BatchMergeResult, CanvasBatch, CanvasBatchDebugger, CanvasBatchInfo, CanvasDirtyFlag, CanvasElement, CanvasRebuilder,
    CanvasRenderMode, DynamicSeparationSuggestion,
};
pub use manager::{Batch, BatchingManager, CacheItem, CacheManager, MemoryPool, MultiThreadManager, RenderCommand};
#[allow(deprecated)]
pub use strategy::OptimizationStrategy;
pub use strategy::{EditorUiOptimization, GameUiOptimization, PerformanceStats};

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
