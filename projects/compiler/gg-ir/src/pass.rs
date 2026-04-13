//! IR 优化 Pass 基础设施模块
//! 定义优化 Pass 的标准接口和优化器执行引擎

use std::time::Instant;

use crate::IrModule;
use gg_core::GResult;

/// 优化 Pass 执行统计
#[derive(Debug, Clone)]
pub struct PassStats {
    /// Pass 名称
    pub name: String,
    /// 执行时间（微秒）
    pub execution_time_us: u64,
    /// 修改次数
    pub modifications: usize,
}

/// IR 优化 Pass trait，定义中间表示优化的标准接口
pub trait IrPass {
    /// 获取 Pass 名称
    fn name(&self) -> &str;

    /// 执行优化 Pass
    /// 返回 Ok(true) 表示做了修改，Ok(false) 表示无修改
    fn run(&self, module: &mut IrModule) -> GResult<bool>;
}

/// IR 优化器，按顺序执行多个优化 Pass
pub struct IrOptimizer {
    /// 已注册的优化 Pass 列表
    passes: Vec<Box<dyn IrPass>>,
    /// 各 Pass 执行统计记录
    pass_stats: Vec<PassStats>,
}

impl IrOptimizer {
    /// 创建空的 IR 优化器
    pub fn new() -> Self {
        Self { passes: Vec::new(), pass_stats: Vec::new() }
    }

    /// 添加一个优化 Pass 到优化器
    pub fn add_pass(&mut self, pass: Box<dyn IrPass>) {
        self.passes.push(pass);
    }

    /// 按顺序执行所有优化 Pass
    /// 返回 Ok(true) 表示至少有一个 Pass 做了修改
    pub fn optimize(&mut self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for pass in &self.passes {
            let start = Instant::now();
            let changed = pass.run(module)?;
            let elapsed_us = start.elapsed().as_micros() as u64;
            let modifications = if changed { 1 } else { 0 };
            if changed {
                any_changed = true;
            }
            self.pass_stats.push(PassStats { name: pass.name().to_string(), execution_time_us: elapsed_us, modifications });
        }
        Ok(any_changed)
    }

    /// 反复执行所有优化 Pass 直到不再产生修改或达到最大迭代次数
    /// 返回 Ok(true) 表示至少有一次迭代做了修改
    pub fn optimize_until_fixed_point(&mut self, module: &mut IrModule, max_iterations: usize) -> GResult<bool> {
        let mut any_changed = false;
        for _ in 0..max_iterations {
            let changed = self.optimize(module)?;
            if changed {
                any_changed = true;
            }
            else {
                break;
            }
        }
        Ok(any_changed)
    }

    /// 获取所有 Pass 执行统计记录
    pub fn get_pass_stats(&self) -> &[PassStats] {
        &self.pass_stats
    }

    /// 清空所有 Pass 执行统计记录
    pub fn reset_stats(&mut self) {
        self.pass_stats.clear();
    }
}

impl Default for IrOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoOpPass {
        name: String,
    }

    impl NoOpPass {
        fn new(name: &str) -> Self {
            Self { name: name.to_string() }
        }
    }

    impl IrPass for NoOpPass {
        fn name(&self) -> &str {
            &self.name
        }

        fn run(&self, _module: &mut IrModule) -> GResult<bool> {
            Ok(false)
        }
    }

    struct ModifyPass {
        name: String,
    }

    impl ModifyPass {
        fn new(name: &str) -> Self {
            Self { name: name.to_string() }
        }
    }

    impl IrPass for ModifyPass {
        fn name(&self) -> &str {
            &self.name
        }

        fn run(&self, _module: &mut IrModule) -> GResult<bool> {
            Ok(true)
        }
    }

    #[test]
    fn test_optimize_records_stats_for_each_pass() {
        let mut optimizer = IrOptimizer::new();
        optimizer.add_pass(Box::new(NoOpPass::new("noop1")));
        optimizer.add_pass(Box::new(NoOpPass::new("noop2")));
        let mut module = IrModule::new("test");
        optimizer.optimize(&mut module).unwrap();
        let stats = optimizer.get_pass_stats();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].name, "noop1");
        assert_eq!(stats[1].name, "noop2");
    }

    #[test]
    fn test_modifications_count_correct() {
        let mut optimizer = IrOptimizer::new();
        optimizer.add_pass(Box::new(NoOpPass::new("noop")));
        optimizer.add_pass(Box::new(ModifyPass::new("modify")));
        let mut module = IrModule::new("test");
        optimizer.optimize(&mut module).unwrap();
        let stats = optimizer.get_pass_stats();
        assert_eq!(stats[0].modifications, 0);
        assert_eq!(stats[1].modifications, 1);
    }

    #[test]
    fn test_get_pass_stats_returns_correct_data() {
        let mut optimizer = IrOptimizer::new();
        optimizer.add_pass(Box::new(NoOpPass::new("pass_a")));
        let mut module = IrModule::new("test");
        optimizer.optimize(&mut module).unwrap();
        let stats = optimizer.get_pass_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "pass_a");
        assert_eq!(stats[0].modifications, 0);
    }

    #[test]
    fn test_reset_stats_clears_stats() {
        let mut optimizer = IrOptimizer::new();
        optimizer.add_pass(Box::new(NoOpPass::new("noop")));
        let mut module = IrModule::new("test");
        optimizer.optimize(&mut module).unwrap();
        assert!(!optimizer.get_pass_stats().is_empty());
        optimizer.reset_stats();
        assert!(optimizer.get_pass_stats().is_empty());
    }

    #[test]
    fn test_stats_accumulate_across_optimize_calls() {
        let mut optimizer = IrOptimizer::new();
        optimizer.add_pass(Box::new(NoOpPass::new("noop")));
        let mut module = IrModule::new("test");
        optimizer.optimize(&mut module).unwrap();
        optimizer.optimize(&mut module).unwrap();
        optimizer.optimize(&mut module).unwrap();
        let stats = optimizer.get_pass_stats();
        assert_eq!(stats.len(), 3);
        for s in stats {
            assert_eq!(s.name, "noop");
            assert_eq!(s.modifications, 0);
        }
    }
}
