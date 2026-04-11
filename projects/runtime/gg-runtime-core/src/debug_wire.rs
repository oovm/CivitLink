//! 调试通信协议模块
//!
//! 定义编辑器与附着式运行时之间的通信协议，
//! 支持控制命令下发和 HMR 事件传递。

use crate::hmr::HmrEvent;

/// 调试通信错误
#[derive(Debug, Clone)]
pub enum DebugWireError {
    /// 通道已断开
    Disconnected,
    /// 通道已满
    ChannelFull,
    /// 序列化错误
    SerializationError(String),
}

/// 编辑器发送到运行时的控制命令
#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    /// 暂停运行时
    Pause,
    /// 恢复运行时
    Resume,
    /// 停止运行时
    Stop,
    /// 单步执行（用于调试）
    Step,
}

/// 通过调试通道传递的消息
#[derive(Debug, Clone)]
pub enum WireMessage {
    /// HMR 热更新事件
    HmrEvent(HmrEvent),
    /// 运行时控制命令
    Command(RuntimeCommand),
}

/// 运行时状态报告
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已停止
    Stopped,
}

/// 调试通信协议 trait
///
/// 定义编辑器与附着式运行时之间的通信接口，
/// 支持 HMR 事件传递、控制命令下发和消息接收。
pub trait DebugWire: Send + Sync {
    /// 发送 HMR 事件到运行时
    ///
    /// # 参数
    ///
    /// - `event` - 要发送的 HMR 事件
    fn send_hmr_event(&self, event: HmrEvent) -> Result<(), DebugWireError>;

    /// 发送控制命令到运行时
    ///
    /// # 参数
    ///
    /// - `command` - 要发送的控制命令
    fn send_command(&self, command: RuntimeCommand) -> Result<(), DebugWireError>;

    /// 尝试接收消息（非阻塞）
    ///
    /// 返回 `Ok(Some(message))` 表示成功接收到消息，
    /// 返回 `Ok(None)` 表示没有可用消息，
    /// 返回 `Err` 表示通信错误。
    fn try_recv(&self) -> Result<Option<WireMessage>, DebugWireError>;

    /// 检查通信通道是否仍然连接
    fn is_connected(&self) -> bool;
}
