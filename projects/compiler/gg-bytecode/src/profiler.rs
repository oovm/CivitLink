#![warn(missing_docs)]

//! 字节码解释器性能分析器
//! 提供函数级别的执行时间统计、热点分析和 JSON 导出功能

use std::{collections::HashMap, time::Instant};

/// 函数性能分析数据
#[derive(Debug, Clone)]
pub struct FunctionProfile {
    /// 函数名称
    pub name: String,
    /// 函数被调用次数
    pub call_count: u64,
    /// 总执行时间（微秒），包含子函数调用时间
    pub total_time_us: u64,
    /// 自身执行时间（微秒），不包含子函数调用时间
    pub self_time_us: u64,
}

/// 调用栈帧条目，包含函数名、函数进入时间和自身时间累计起始点
struct CallStackEntry {
    /// 函数名称
    name: String,
    /// 函数进入时间
    start: Instant,
    /// 自身时间累计起始点（进入时或从子调用返回时重置）
    self_start: Instant,
    /// 已累计的自身时间（微秒）
    accumulated_self_us: u64,
}

/// 字节码解释器性能分析器
///
/// 通过在函数进入和退出时记录时间戳，收集函数调用的性能数据。
/// 分析器默认禁用，启用后才会记录数据，未启用时开销为零。
pub struct BytecodeProfiler {
    /// 函数名称到性能数据的映射
    profiles: HashMap<String, FunctionProfile>,
    /// 当前正在执行的函数调用栈
    call_stack: Vec<CallStackEntry>,
    /// 是否启用性能分析
    enabled: bool,
}

impl BytecodeProfiler {
    /// 创建新的性能分析器（默认禁用）
    pub fn new() -> Self {
        Self { profiles: HashMap::new(), call_stack: Vec::new(), enabled: false }
    }

    /// 启用性能分析
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用性能分析
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 检查性能分析是否已启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 记录函数进入事件
    ///
    /// 将函数名和当前时间戳压入调用栈。
    /// 如果当前有调用者，先累计调用者从上次恢复到现在的自身时间。
    /// 如果分析器未启用，此方法不做任何操作。
    pub fn on_function_enter(&mut self, function_name: &str) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();

        if let Some(caller) = self.call_stack.last_mut() {
            let self_elapsed = now.duration_since(caller.self_start).as_micros() as u64;
            caller.accumulated_self_us += self_elapsed;
            caller.self_start = now;
        }

        self.call_stack.push(CallStackEntry {
            name: function_name.to_string(),
            start: now,
            self_start: now,
            accumulated_self_us: 0,
        });
    }

    /// 记录函数退出事件
    ///
    /// 从调用栈弹出函数条目，计算经过时间并更新性能数据。
    /// total_time 为函数从进入到退出的总时间，
    /// self_time 为总时间减去所有子调用的时间。
    /// 如果分析器未启用，此方法不做任何操作。
    pub fn on_function_exit(&mut self, _function_name: &str) {
        if !self.enabled {
            return;
        }

        if let Some(entry) = self.call_stack.pop() {
            let now = Instant::now();
            let total_elapsed_us = now.duration_since(entry.start).as_micros() as u64;
            let self_elapsed_us = entry.accumulated_self_us + now.duration_since(entry.self_start).as_micros() as u64;

            let profile = self.profiles.entry(entry.name.clone()).or_insert_with(|| FunctionProfile {
                name: entry.name.clone(),
                call_count: 0,
                total_time_us: 0,
                self_time_us: 0,
            });
            profile.call_count += 1;
            profile.total_time_us += total_elapsed_us;
            profile.self_time_us += self_elapsed_us;

            if let Some(caller) = self.call_stack.last_mut() {
                caller.self_start = now;
            }
        }
    }

    /// 获取指定函数的性能数据
    pub fn get_profile(&self, function_name: &str) -> Option<&FunctionProfile> {
        self.profiles.get(function_name)
    }

    /// 返回按自身执行时间降序排列的前 N 个热点函数
    pub fn hotspot_functions(&self, top_n: usize) -> Vec<&FunctionProfile> {
        let mut profiles: Vec<&FunctionProfile> = self.profiles.values().collect();
        profiles.sort_by(|a, b| b.self_time_us.cmp(&a.self_time_us));
        profiles.truncate(top_n);
        profiles
    }

    /// 导出所有性能数据为 JSON 字符串
    ///
    /// 手动构建 JSON，不依赖 serde 序列化。
    /// 每个函数条目包含 name、call_count、total_time_us、self_time_us 和 avg_time_us。
    pub fn export_json(&self) -> String {
        let mut entries: Vec<&FunctionProfile> = self.profiles.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let items: Vec<String> = entries
            .iter()
            .map(|p| {
                let avg = if p.call_count > 0 {
                    p.total_time_us / p.call_count
                } else {
                    0
                };
                format!(
                    "    {{\n      \"name\": \"{}\",\n      \"call_count\": {},\n      \"total_time_us\": {},\n      \"self_time_us\": {},\n      \"avg_time_us\": {}\n    }}",
                    p.name, p.call_count, p.total_time_us, p.self_time_us, avg
                )
            })
            .collect();
        if items.is_empty() {
            format!("{{\n  \"profiles\": [\n  ]\n}}")
        }
        else {
            format!("{{\n  \"profiles\": [\n{}\n  ]\n}}", items.join(",\n"))
        }
    }

    /// 清除所有性能数据
    pub fn reset(&mut self) {
        self.profiles.clear();
        self.call_stack.clear();
    }

    /// 返回所有性能数据的迭代器
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FunctionProfile)> {
        self.profiles.iter()
    }
}

impl Default for BytecodeProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn test_profiler_disabled_by_default() {
        let profiler = BytecodeProfiler::new();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut profiler = BytecodeProfiler::new();
        assert!(!profiler.is_enabled());
        profiler.enable();
        assert!(profiler.is_enabled());
        profiler.disable();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_no_data_when_disabled() {
        let mut profiler = BytecodeProfiler::new();
        profiler.on_function_enter("foo");
        profiler.on_function_exit("foo");
        assert!(profiler.get_profile("foo").is_none());
    }

    #[test]
    fn test_call_count_tracking() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        for _ in 0..3 {
            profiler.on_function_enter("foo");
            profiler.on_function_exit("foo");
        }
        let profile = profiler.get_profile("foo").unwrap();
        assert_eq!(profile.call_count, 3);
    }

    #[test]
    fn test_self_time_vs_total_time() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("outer");
        profiler.on_function_enter("inner");
        thread::sleep(Duration::from_millis(10));
        profiler.on_function_exit("inner");
        thread::sleep(Duration::from_millis(5));
        profiler.on_function_exit("outer");

        let outer = profiler.get_profile("outer").unwrap();
        let inner = profiler.get_profile("inner").unwrap();

        assert!(inner.total_time_us > 0);
        assert_eq!(inner.self_time_us, inner.total_time_us);
        assert!(outer.total_time_us >= inner.total_time_us);
        assert!(outer.self_time_us < outer.total_time_us);
    }

    #[test]
    fn test_hotspot_functions() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("slow");
        thread::sleep(Duration::from_micros(200));
        profiler.on_function_exit("slow");
        profiler.on_function_enter("fast");
        profiler.on_function_exit("fast");

        let hotspots = profiler.hotspot_functions(2);
        assert_eq!(hotspots.len(), 2);
        assert_eq!(hotspots[0].name, "slow");
        assert!(hotspots[0].self_time_us > hotspots[1].self_time_us);
    }

    #[test]
    fn test_hotspot_functions_top_n() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("a");
        profiler.on_function_exit("a");
        profiler.on_function_enter("b");
        profiler.on_function_exit("b");
        profiler.on_function_enter("c");
        profiler.on_function_exit("c");

        let hotspots = profiler.hotspot_functions(2);
        assert_eq!(hotspots.len(), 2);
    }

    #[test]
    fn test_export_json_valid() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("foo");
        profiler.on_function_exit("foo");
        profiler.on_function_enter("bar");
        profiler.on_function_exit("bar");

        let json = profiler.export_json();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("\"profiles\""));
        assert!(json.contains("\"name\": \"bar\""));
        assert!(json.contains("\"name\": \"foo\""));
        assert!(json.contains("\"call_count\""));
        assert!(json.contains("\"total_time_us\""));
        assert!(json.contains("\"self_time_us\""));
        assert!(json.contains("\"avg_time_us\""));
    }

    #[test]
    fn test_export_json_avg_time() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("fn");
        profiler.on_function_exit("fn");
        profiler.on_function_enter("fn");
        profiler.on_function_exit("fn");

        let profile = profiler.get_profile("fn").unwrap();
        let expected_avg = profile.total_time_us / profile.call_count;
        let json = profiler.export_json();
        assert!(json.contains(&format!("\"avg_time_us\": {}", expected_avg)));
    }

    #[test]
    fn test_export_json_empty() {
        let profiler = BytecodeProfiler::new();
        let json = profiler.export_json();
        assert_eq!(json, "{\n  \"profiles\": [\n  ]\n}");
    }

    #[test]
    fn test_reset_clears_data() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("foo");
        profiler.on_function_exit("foo");
        assert!(profiler.get_profile("foo").is_some());

        profiler.reset();
        assert!(profiler.get_profile("foo").is_none());
    }

    #[test]
    fn test_reset_clears_call_stack() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("foo");
        assert_eq!(profiler.call_stack.len(), 1);

        profiler.reset();
        assert_eq!(profiler.call_stack.len(), 0);
    }

    #[test]
    fn test_default_impl() {
        let profiler = BytecodeProfiler::default();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_get_profile_nonexistent() {
        let profiler = BytecodeProfiler::new();
        assert!(profiler.get_profile("nonexistent").is_none());
    }

    #[test]
    fn test_nested_calls_self_time_subtraction() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("outer");
        profiler.on_function_enter("middle");
        profiler.on_function_enter("inner");
        profiler.on_function_exit("inner");
        profiler.on_function_exit("middle");
        profiler.on_function_exit("outer");

        let outer = profiler.get_profile("outer").unwrap();
        let middle = profiler.get_profile("middle").unwrap();
        let inner = profiler.get_profile("inner").unwrap();

        assert_eq!(inner.self_time_us, inner.total_time_us);
        assert!(middle.self_time_us <= middle.total_time_us);
        assert!(outer.self_time_us <= outer.total_time_us);
    }

    #[test]
    fn test_iter() {
        let mut profiler = BytecodeProfiler::new();
        profiler.enable();
        profiler.on_function_enter("a");
        profiler.on_function_exit("a");
        profiler.on_function_enter("b");
        profiler.on_function_exit("b");

        let entries: Vec<_> = profiler.iter().collect();
        assert_eq!(entries.len(), 2);
    }
}
