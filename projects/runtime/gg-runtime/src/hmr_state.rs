//! HMR 状态迁移模块
//! 提供热替换过程中的状态快照和恢复能力

use gg_bytecode::BytecodeValue;
use std::collections::HashMap;

/// 组件状态快照
#[derive(Debug, Clone, Default)]
pub struct ComponentStateSnapshot {
    /// 组件类型名称
    pub component_type: String,
    /// 组件字段状态
    pub fields: HashMap<String, BytecodeValue>,
}

/// 状态快照，用于 HMR 状态迁移
#[derive(Debug, Clone, Default)]
pub struct StateSnapshot {
    /// 全局变量状态
    pub globals: HashMap<String, BytecodeValue>,
    /// 组件状态快照
    pub component_states: HashMap<String, ComponentStateSnapshot>,
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
    pub fn capture(&mut self, globals: HashMap<String, BytecodeValue>) {
        self.snapshot = Some(StateSnapshot { globals, component_states: HashMap::new() });
    }

    /// 序列化当前状态到快照（包含组件状态）
    pub fn capture_with_components(
        &mut self,
        globals: HashMap<String, BytecodeValue>,
        component_states: HashMap<String, ComponentStateSnapshot>,
    ) {
        self.snapshot = Some(StateSnapshot { globals, component_states });
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
