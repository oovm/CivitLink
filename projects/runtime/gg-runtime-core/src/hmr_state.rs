//! HMR 状态迁移模块
//! 提供热替换过程中的状态快照和恢复能力

use gg_ir::IrValue;
use std::collections::HashMap;

/// 状态快照，用于 HMR 状态迁移
#[derive(Debug, Clone, Default)]
pub struct StateSnapshot {
    /// 全局变量状态
    pub globals: HashMap<String, IrValue>,
}

/// 状态迁移器
pub struct StateMigrator {
    /// 当前状态快照
    snapshot: Option<StateSnapshot>,
}

impl StateMigrator {
    /// 创建新的状态迁移器
    pub fn new() -> Self {
        Self { snapshot: None }
    }

    /// 序列化当前状态到快照
    pub fn capture(&mut self, globals: HashMap<String, IrValue>) {
        self.snapshot = Some(StateSnapshot { globals });
    }

    /// 从快照恢复状态
    pub fn restore(&self) -> Option<&StateSnapshot> {
        self.snapshot.as_ref()
    }

    /// 清除快照
    pub fn clear(&mut self) {
        self.snapshot = None;
    }
}

impl Default for StateMigrator {
    fn default() -> Self {
        Self::new()
    }
}
