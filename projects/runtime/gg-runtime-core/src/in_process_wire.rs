//! 进程内调试通信实现
//!
//! 使用 `std::sync::mpsc` 通道实现 DebugWire trait，
//! 适用于编辑器和运行时在同一进程内的通信场景。

use crate::debug_wire::{DebugWire, DebugWireError, RuntimeCommand, WireMessage};
use crate::hmr::HmrEvent;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// 进程内调试通信通道
///
/// 基于 `std::sync::mpsc` 实现的 DebugWire，
/// 适用于编辑器和运行时在同一进程内运行时的通信。
pub struct InProcessDebugWire {
    /// 发送端，用于向运行时发送消息
    tx: Sender<WireMessage>,
    /// 接收端，用于从编辑器接收消息
    rx: Receiver<WireMessage>,
}

impl InProcessDebugWire {
    /// 创建新的进程内调试通信通道
    ///
    /// 返回一对通信端点：(编辑器端, 运行时端)。
    /// 编辑器端持有发送通道，运行时端持有接收通道。
    pub fn new_pair() -> (InProcessDebugWireEditor, InProcessDebugWire) {
        let (editor_tx, runtime_rx) = mpsc::channel();
        let runtime_wire = InProcessDebugWire { tx: editor_tx.clone(), rx: runtime_rx };
        let editor_wire = InProcessDebugWireEditor { tx: editor_tx };
        (editor_wire, runtime_wire)
    }
}

impl DebugWire for InProcessDebugWire {
    fn send_hmr_event(&self, event: HmrEvent) -> Result<(), DebugWireError> {
        self.tx.send(WireMessage::HmrEvent(event)).map_err(|_| DebugWireError::Disconnected)
    }

    fn send_command(&self, command: RuntimeCommand) -> Result<(), DebugWireError> {
        self.tx.send(WireMessage::Command(command)).map_err(|_| DebugWireError::Disconnected)
    }

    fn try_recv(&self) -> Result<Option<WireMessage>, DebugWireError> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(DebugWireError::Disconnected),
        }
    }

    fn is_connected(&self) -> bool {
        !self.tx.is_closed()
    }
}

/// 编辑器端的进程内调试通信通道
///
/// 仅提供发送功能，编辑器通过此通道向运行时发送
/// HMR 事件和控制命令。
pub struct InProcessDebugWireEditor {
    /// 发送端
    tx: Sender<WireMessage>,
}

impl InProcessDebugWireEditor {
    /// 发送 HMR 事件到运行时
    ///
    /// # 参数
    ///
    /// - `event` - 要发送的 HMR 事件
    pub fn send_hmr_event(&self, event: HmrEvent) -> Result<(), DebugWireError> {
        self.tx.send(WireMessage::HmrEvent(event)).map_err(|_| DebugWireError::Disconnected)
    }

    /// 发送控制命令到运行时
    ///
    /// # 参数
    ///
    /// - `command` - 要发送的控制命令
    pub fn send_command(&self, command: RuntimeCommand) -> Result<(), DebugWireError> {
        self.tx.send(WireMessage::Command(command)).map_err(|_| DebugWireError::Disconnected)
    }

    /// 检查通信通道是否仍然连接
    pub fn is_connected(&self) -> bool {
        !self.tx.is_closed()
    }
}
