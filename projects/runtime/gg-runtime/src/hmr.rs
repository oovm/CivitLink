//! HMR（热模块替换）模块
//! 提供脚本和资源的运行时热替换能力

use crate::debug_wire::{DebugWire, WireMessage};
use gg_bytecode::format::BytecodeModule;
use std::{collections::HashMap, time::Instant};

/// HMR 事件
#[derive(Debug, Clone)]
pub enum HmrEvent {
    /// 脚本文件变更
    ScriptChanged {
        /// 模块名称
        module_name: String,
        /// 新的字节码模块
        new_module: BytecodeModule,
    },
    /// 资源文件变更
    AssetChanged {
        /// 资源路径
        asset_path: String,
        /// 新的资源数据
        new_data: Vec<u8>,
    },
}

/// HMR 状态迁移结果
#[derive(Debug, Clone)]
pub enum HmrMigrationResult {
    /// 迁移成功
    Success,
    /// 迁移失败，已回滚
    Rollback(String),
}

/// HMR 迁移策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmrMigrationStrategy {
    /// 完整重置，清除所有旧状态并重新初始化
    FullReset,
    /// 增量迁移，仅更新变更部分，保留未变更状态
    Incremental,
}

/// 可序列化状态，用于组件热更新时的状态保存
pub type SerializableState = HashMap<String, gg_bytecode::BytecodeValue>;

/// 组件热更新钩子 trait
///
/// 允许 ECS 组件在热更新前后执行自定义逻辑，
/// 保存和恢复组件状态。
pub trait ComponentHmrHook: Send + Sync {
    /// 热更新前调用，保存组件状态
    fn before_hot_reload(&self, component_type: &str) -> SerializableState;

    /// 热更新后调用，恢复组件状态
    fn after_hot_reload(&self, component_type: &str, state: SerializableState);
}

/// HMR 管理器
pub struct HmrManager {
    /// 待处理的事件队列
    pending_events: Vec<HmrEvent>,
    /// 是否有脚本变更待处理
    script_dirty: bool,
    /// 是否有资源变更待处理
    asset_dirty: bool,
    /// 上一次成功加载的脚本模块（用于回滚）
    last_stable_module: Option<BytecodeModule>,
    /// 已变更的资源路径列表
    changed_assets: Vec<String>,
    /// 迁移策略
    migration_strategy: HmrMigrationStrategy,
    /// 组件热更新钩子
    component_hooks: HashMap<String, Box<dyn ComponentHmrHook>>,
    /// HMR 单帧处理时间预算（纳秒），默认 1ms
    hmr_budget_ns: u64,
}

impl HmrManager {
    /// 创建新的 HMR 管理器
    pub fn new() -> Self {
        Self {
            pending_events: Vec::new(),
            script_dirty: false,
            asset_dirty: false,
            last_stable_module: None,
            changed_assets: Vec::new(),
            migration_strategy: HmrMigrationStrategy::FullReset,
            component_hooks: HashMap::new(),
            hmr_budget_ns: 1_000_000,
        }
    }

    /// 推送 HMR 事件
    pub fn push_event(&mut self, event: HmrEvent) {
        match &event {
            HmrEvent::ScriptChanged { .. } => {
                self.script_dirty = true;
            }
            HmrEvent::AssetChanged { .. } => {
                self.asset_dirty = true;
            }
        }
        self.pending_events.push(event);
    }

    /// 检查是否有待处理的事件
    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    /// 处理脚本热替换
    /// 返回新的 BytecodeModule（如果脚本有变更）
    pub fn process_script_reload(&mut self) -> Option<BytecodeModule> {
        if !self.script_dirty {
            return None;
        }

        let mut result = None;
        let mut remaining = Vec::new();

        for event in self.pending_events.drain(..) {
            match event {
                HmrEvent::ScriptChanged { new_module, .. } => {
                    result = Some(new_module);
                }
                event @ HmrEvent::AssetChanged { .. } => {
                    remaining.push(event);
                }
            }
        }

        self.pending_events = remaining;
        self.script_dirty = false;
        result
    }

    /// 处理资源热替换
    /// 返回变更的资源路径列表
    pub fn process_asset_reload(&mut self) -> Vec<String> {
        if !self.asset_dirty {
            return Vec::new();
        }

        let mut changed = Vec::new();
        let mut remaining = Vec::new();

        for event in self.pending_events.drain(..) {
            match event {
                HmrEvent::AssetChanged { asset_path, .. } => {
                    changed.push(asset_path);
                }
                event @ HmrEvent::ScriptChanged { .. } => {
                    remaining.push(event);
                }
            }
        }

        changed.extend(self.changed_assets.drain(..));
        self.pending_events = remaining;
        self.asset_dirty = false;
        changed
    }

    /// 确认脚本热替换成功
    pub fn confirm_script_reload(&mut self, module: BytecodeModule) {
        self.last_stable_module = Some(module);
    }

    /// 回滚脚本热替换
    pub fn rollback_script_reload(&mut self) -> Option<BytecodeModule> {
        self.last_stable_module.clone()
    }

    /// 设置迁移策略
    pub fn set_migration_strategy(&mut self, strategy: HmrMigrationStrategy) {
        self.migration_strategy = strategy;
    }

    /// 获取当前迁移策略
    pub fn migration_strategy(&self) -> HmrMigrationStrategy {
        self.migration_strategy
    }

    /// 注册组件热更新钩子
    pub fn register_component_hook(&mut self, component_type: &str, hook: Box<dyn ComponentHmrHook>) {
        self.component_hooks.insert(component_type.to_string(), hook);
    }

    /// 处理组件热更新
    ///
    /// 遍历已注册的组件钩子，调用 before/after 方法。
    /// 返回受影响的组件类型列表。
    pub fn process_component_reload(&mut self) -> Vec<String> {
        let mut affected = Vec::new();
        for (component_type, hook) in &self.component_hooks {
            let state = hook.before_hot_reload(component_type);
            hook.after_hot_reload(component_type, state);
            affected.push(component_type.clone());
        }
        affected
    }

    /// 检查 HMR 处理是否超出时间预算
    pub fn has_exceeded_budget(&self, start: Instant) -> bool {
        start.elapsed().as_nanos() as u64 > self.hmr_budget_ns
    }

    /// 清除已处理的事件
    pub fn clear_processed(&mut self) {
        self.pending_events.clear();
        self.changed_assets.clear();
        self.script_dirty = false;
        self.asset_dirty = false;
    }

    /// 从调试通信通道接收并处理 HMR 事件
    ///
    /// 非阻塞地从 DebugWire 通道中读取所有 HmrEvent 消息，
    /// 将其推入内部事件队列等待后续处理。
    /// 忽略非 HmrEvent 类型的消息。
    ///
    /// # 参数
    ///
    /// - `wire` - 调试通信通道引用
    pub fn process_wire_events(&mut self, wire: &dyn DebugWire) {
        let start = Instant::now();
        loop {
            if start.elapsed().as_nanos() as u64 > self.hmr_budget_ns {
                break;
            }
            match wire.try_recv() {
                Ok(Some(WireMessage::HmrEvent(event))) => {
                    self.push_event(event);
                }
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_) => break,
            }
        }
    }
}

impl Default for HmrManager {
    fn default() -> Self {
        Self::new()
    }
}
