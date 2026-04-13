//! VM 热点检测模块
//! 统计函数调用频率和执行时间，识别性能热点

use std::{collections::HashMap, time::Instant};

/// 函数性能记录
#[derive(Debug, Clone)]
pub struct FunctionProfile {
    /// 函数名称
    pub name: String,
    /// 调用次数
    pub call_count: u64,
    /// 总执行时间（纳秒）
    pub total_ns: u64,
    /// 平均执行时间（纳秒）
    pub avg_ns: u64,
}

/// 热点检测器
pub struct HotspotDetector {
    /// 函数性能记录
    profiles: HashMap<String, FunctionProfile>,
    /// 调用栈，用于追踪嵌套调用
    call_stack: Vec<(String, Instant)>,
    /// 是否启用
    enabled: bool,
}

impl HotspotDetector {
    /// 创建新的热点检测器
    pub fn new() -> Self {
        Self { profiles: HashMap::new(), call_stack: Vec::new(), enabled: false }
    }

    /// 创建已启用的热点检测器
    pub fn enabled() -> Self {
        let mut detector = Self::new();
        detector.enabled = true;
        detector
    }

    /// 是否已启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 启用/禁用热点检测器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 函数进入时调用
    pub fn on_function_enter(&mut self, function_name: &str) {
        if !self.enabled {
            return;
        }
        self.call_stack.push((function_name.to_string(), Instant::now()));
    }

    /// 函数退出时调用
    pub fn on_function_exit(&mut self, function_name: &str) {
        if !self.enabled {
            return;
        }
        if let Some((name, start)) = self.call_stack.pop() {
            if name == function_name {
                let elapsed = start.elapsed().as_nanos() as u64;
                let entry = self.profiles.entry(function_name.to_string()).or_insert(FunctionProfile {
                    name: function_name.to_string(),
                    call_count: 0,
                    total_ns: 0,
                    avg_ns: 0,
                });
                entry.call_count += 1;
                entry.total_ns += elapsed;
                entry.avg_ns = entry.total_ns / entry.call_count;
            }
        }
    }

    /// 生成热点报告
    pub fn report(&self) -> Vec<FunctionProfile> {
        let mut profiles: Vec<FunctionProfile> = self.profiles.values().cloned().collect();
        profiles.sort_by(|a, b| b.total_ns.cmp(&a.total_ns));
        profiles
    }

    /// 清除所有记录
    pub fn clear(&mut self) {
        self.profiles.clear();
        self.call_stack.clear();
    }

    /// 获取已追踪的函数数量
    pub fn tracked_count(&self) -> usize {
        self.profiles.len()
    }
}

impl Default for HotspotDetector {
    fn default() -> Self {
        Self::new()
    }
}
