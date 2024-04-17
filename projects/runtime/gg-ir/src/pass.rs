//! IR 优化 Pass 基础设施模块
//! 定义优化 Pass 的标准接口和优化器执行引擎

use crate::IrModule;
use gg_core::GResult;

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
}

impl IrOptimizer {
    /// 创建空的 IR 优化器
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// 添加一个优化 Pass 到优化器
    pub fn add_pass(&mut self, pass: Box<dyn IrPass>) {
        self.passes.push(pass);
    }

    /// 按顺序执行所有优化 Pass
    /// 返回 Ok(true) 表示至少有一个 Pass 做了修改
    pub fn optimize(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for pass in &self.passes {
            let changed = pass.run(module)?;
            if changed {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }

    /// 反复执行所有优化 Pass 直到不再产生修改或达到最大迭代次数
    /// 返回 Ok(true) 表示至少有一次迭代做了修改
    pub fn optimize_until_fixed_point(&self, module: &mut IrModule, max_iterations: usize) -> GResult<bool> {
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
}

impl Default for IrOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
