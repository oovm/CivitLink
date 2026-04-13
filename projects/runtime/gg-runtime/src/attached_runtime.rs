//! 附着式运行时模块
//!
//! 提供在编辑器进程内运行游戏实例的能力，
//! 通过 DebugWire 通道实现编辑器与运行时的隔离通信，
//! 支持暂停/恢复/停止控制和状态快照导出。

use crate::{
    Runtime,
    debug_wire::{DebugResponse, DebugWire, RuntimeCommand, RuntimeState, WireMessage},
    hmr::HmrManager,
    hmr_state::StateMigrator,
    in_process_wire::InProcessDebugWire,
};
use gg_bytecode::BytecodeValue;
use gg_core::GResult;
use std::{collections::HashMap, time::Duration};

/// 附着式运行时状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachedRuntimeState {
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已停止
    Stopped,
}

/// 附着式运行时
///
/// 在编辑器进程内运行的游戏实例，通过 DebugWire 通道
/// 与编辑器隔离通信。支持暂停/恢复/停止控制，
/// 以及通过 detach 导出当前世界状态快照。
///
/// 编辑器通过 tick() 方法驱动运行时的帧循环，
/// 而非独立运行游戏循环。
pub struct AttachedRuntime {
    /// 内部运行时实例
    runtime: Runtime,
    /// 调试通信通道
    wire: InProcessDebugWire,
    /// 当前运行状态
    state: AttachedRuntimeState,
    /// 状态迁移器，用于 detach 时导出状态
    state_migrator: StateMigrator,
}

impl AttachedRuntime {
    /// 创建新的附着式运行时
    ///
    /// # 参数
    ///
    /// - `runtime` - 内部运行时实例
    /// - `wire` - 调试通信通道
    pub fn new(runtime: Runtime, wire: InProcessDebugWire) -> Self {
        Self { runtime, wire, state: AttachedRuntimeState::Stopped, state_migrator: StateMigrator::new() }
    }

    /// 启动附着式运行时
    ///
    /// 执行脚本 init 函数，将状态设置为 Running。
    pub fn start(&mut self) -> GResult<()> {
        if self.state != AttachedRuntimeState::Stopped {
            return Ok(());
        }

        if self.runtime.script_engine().has_function("init") {
            match self.runtime.call_script_function("init") {
                gg_vm::VmResult::Ok | gg_vm::VmResult::Return(_) => {}
                gg_vm::VmResult::Error { message, .. } => {
                    return Err(gg_core::GError {
                        kind: gg_core::GErrorKind::Runtime,
                        message: format!("Script init error: {}", message),
                    });
                }
            }
        }

        self.state = AttachedRuntimeState::Running;
        Ok(())
    }

    /// 暂停运行时
    ///
    /// 将运行时状态设置为 Paused，tick() 时跳过帧更新。
    pub fn pause(&mut self) {
        if self.state == AttachedRuntimeState::Running {
            self.state = AttachedRuntimeState::Paused;
        }
    }

    /// 恢复运行时
    ///
    /// 将运行时状态从 Paused 恢复为 Running。
    pub fn resume(&mut self) {
        if self.state == AttachedRuntimeState::Paused {
            self.state = AttachedRuntimeState::Running;
        }
    }

    /// 停止运行时
    ///
    /// 将运行时状态设置为 Stopped，并执行 shutdown 流程。
    pub fn stop(&mut self) {
        if self.state == AttachedRuntimeState::Stopped {
            return;
        }

        if self.runtime.script_engine().has_function("shutdown") {
            self.runtime.call_script_function("shutdown");
        }

        self.state = AttachedRuntimeState::Stopped;
    }

    /// 执行一帧
    ///
    /// 由编辑器的渲染循环调用。先处理 DebugWire 通道中的消息，
    /// 然后根据当前状态决定是否执行帧更新。
    /// Running 状态下正常执行帧更新，
    /// Paused 状态下仅处理通道消息不执行帧更新，
    /// Stopped 状态下不做任何操作。
    pub fn tick(&mut self, delta: Duration) -> GResult<()> {
        self.process_wire_messages();

        match self.state {
            AttachedRuntimeState::Running => {
                if let Some(ref mut hmr) = self.runtime.hmr_manager() {
                    hmr.process_wire_events(&self.wire);
                }

                self.runtime.tick(delta)
            }
            AttachedRuntimeState::Paused => Ok(()),
            AttachedRuntimeState::Stopped => Ok(()),
        }
    }

    /// 分离运行时，停止并返回世界状态快照
    ///
    /// 停止运行时，捕获当前全局状态并返回快照。
    /// 调用后运行时进入 Stopped 状态。
    pub fn detach(&mut self) -> HashMap<String, BytecodeValue> {
        self.stop();

        let snapshot = self.state_migrator.restore().map(|s| s.globals.clone()).unwrap_or_default();
        snapshot
    }

    /// 捕获当前状态到迁移器
    ///
    /// 将当前全局变量状态保存到状态迁移器中，
    /// 用于后续 detach 时导出。
    pub fn capture_state(&mut self, globals: HashMap<String, BytecodeValue>) {
        self.state_migrator.capture(globals);
    }

    /// 检查运行时是否正在运行
    pub fn is_running(&self) -> bool {
        self.state == AttachedRuntimeState::Running
    }

    /// 检查运行时是否已暂停
    pub fn is_paused(&self) -> bool {
        self.state == AttachedRuntimeState::Paused
    }

    /// 获取当前运行状态
    pub fn state(&self) -> AttachedRuntimeState {
        self.state
    }

    /// 获取运行时状态报告（用于 DebugWire 通信）
    pub fn runtime_state(&self) -> RuntimeState {
        match self.state {
            AttachedRuntimeState::Running => RuntimeState::Running,
            AttachedRuntimeState::Paused => RuntimeState::Paused,
            AttachedRuntimeState::Stopped => RuntimeState::Stopped,
        }
    }

    /// 获取内部运行时的可变引用
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    /// 处理 DebugWire 通道中的控制命令
    fn process_wire_messages(&mut self) {
        loop {
            match self.wire.try_recv() {
                Ok(Some(WireMessage::Command(cmd))) => match cmd {
                    RuntimeCommand::Pause => self.pause(),
                    RuntimeCommand::Resume => self.resume(),
                    RuntimeCommand::Stop => self.stop(),
                    RuntimeCommand::Step => {
                        if self.state == AttachedRuntimeState::Paused {
                            if let Ok(()) = self.runtime.tick(Duration::ZERO) {}
                        }
                    }
                    RuntimeCommand::SetBreakpoint { file: _, line: _, condition: _ } => {
                        let _ = self.wire.send_response(DebugResponse::BreakpointSet { id: 0 });
                    }
                    RuntimeCommand::RemoveBreakpoint { id: _ } => {
                        let _ = self.wire.send_response(DebugResponse::BreakpointRemoved { success: false });
                    }
                    RuntimeCommand::AddWatch { expression } => {
                        let _ = self.wire.send_response(DebugResponse::WatchResult {
                            id: 0,
                            result_json: format!("{{\"error\": \"Watch not yet supported: {}\"}}", expression),
                        });
                    }
                    RuntimeCommand::RemoveWatch { id: _ } => {}
                    RuntimeCommand::GetCallStack => {
                        let _ = self.wire.send_response(DebugResponse::CallStackInfo { frames_json: "[]".to_string() });
                    }
                    RuntimeCommand::GetVariables { frame_index: _ } => {
                        let _ = self.wire.send_response(DebugResponse::VariablesInfo { variables_json: "[]".to_string() });
                    }
                    RuntimeCommand::Evaluate { expression } => {
                        let _ = self.wire.send_response(DebugResponse::EvaluationResult {
                            result_json: format!("{{\"error\": \"Cannot evaluate while not paused\"}}"),
                        });
                    }
                },
                Ok(Some(WireMessage::HmrEvent(event))) => {
                    if let Some(ref mut hmr) = self.runtime.hmr_manager() {
                        hmr.push_event(event);
                    }
                }
                Ok(Some(WireMessage::DebugResponse(_))) => {}
                Ok(None) => break,
                Err(_) => break,
            }
        }
    }
}
