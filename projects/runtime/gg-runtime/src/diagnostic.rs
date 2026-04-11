//! 结构化运行时诊断模块
//! 提供替代 eprintln! 的结构化诊断系统，支持错误堆栈和源码映射

use std::collections::VecDeque;

/// 诊断级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
}

/// 诊断来源
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSource {
    /// 脚本引擎
    Script,
    /// HMR 热更新
    Hmr,
    /// 调度器
    Scheduler,
    /// 渲染后端
    Render,
    /// 音频后端
    Audio,
    /// 插件系统
    Plugin,
    /// 虚拟机
    Vm,
    /// 通用运行时
    Runtime,
}

/// 源码位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// 源文件路径
    pub file: String,
    /// 行号（1-based）
    pub line: usize,
    /// 列号（1-based）
    pub column: usize,
}

/// 运行时诊断信息
#[derive(Debug, Clone)]
pub struct RuntimeDiagnostic {
    /// 诊断级别
    pub level: DiagnosticLevel,
    /// 诊断来源
    pub source: DiagnosticSource,
    /// 诊断消息
    pub message: String,
    /// 错误堆栈（可选）
    pub stack_trace: Option<String>,
    /// 源码位置（可选）
    pub source_location: Option<SourceLocation>,
    /// 时间戳（自运行时启动以来的纳秒数）
    pub timestamp_ns: u64,
}

/// 诊断收集器 trait
pub trait DiagnosticCollector: Send + Sync {
    /// 收集一条诊断信息
    fn collect(&mut self, diagnostic: RuntimeDiagnostic);

    /// 取出所有已收集的诊断信息
    fn drain(&mut self) -> Vec<RuntimeDiagnostic>;

    /// 清除所有已收集的诊断信息
    fn clear(&mut self);
}

/// 基于 Vec 的诊断收集器
pub struct VecDiagnosticCollector {
    diagnostics: VecDeque<RuntimeDiagnostic>,
    max_capacity: usize,
}

impl VecDiagnosticCollector {
    /// 创建新的诊断收集器
    pub fn new() -> Self {
        Self { diagnostics: VecDeque::new(), max_capacity: 1024 }
    }

    /// 创建带容量限制的诊断收集器
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self { diagnostics: VecDeque::with_capacity(max_capacity.min(64)), max_capacity }
    }
}

impl Default for VecDiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticCollector for VecDiagnosticCollector {
    fn collect(&mut self, diagnostic: RuntimeDiagnostic) {
        if self.diagnostics.len() >= self.max_capacity {
            self.diagnostics.pop_front();
        }
        self.diagnostics.push_back(diagnostic);
    }

    fn drain(&mut self) -> Vec<RuntimeDiagnostic> {
        self.diagnostics.drain(..).collect()
    }

    fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

/// 诊断构建器，用于便捷构建 RuntimeDiagnostic
pub struct DiagnosticBuilder {
    level: DiagnosticLevel,
    source: DiagnosticSource,
    message: String,
    stack_trace: Option<String>,
    source_location: Option<SourceLocation>,
}

impl DiagnosticBuilder {
    /// 创建新的诊断构建器
    pub fn new(level: DiagnosticLevel, source: DiagnosticSource, message: impl Into<String>) -> Self {
        Self { level, source, message: message.into(), stack_trace: None, source_location: None }
    }

    /// 设置错误堆栈
    pub fn stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// 设置源码位置
    pub fn source_location(mut self, file: impl Into<String>, line: usize, column: usize) -> Self {
        self.source_location = Some(SourceLocation { file: file.into(), line, column });
        self
    }

    /// 构建诊断信息
    pub fn build(self, timestamp_ns: u64) -> RuntimeDiagnostic {
        RuntimeDiagnostic {
            level: self.level,
            source: self.source,
            message: self.message,
            stack_trace: self.stack_trace,
            source_location: self.source_location,
            timestamp_ns,
        }
    }
}
