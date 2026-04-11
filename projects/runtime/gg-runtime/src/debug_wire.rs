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
    /// 设置断点
    SetBreakpoint {
        /// 文件路径
        file: String,
        /// 行号
        line: usize,
        /// 条件表达式
        condition: Option<String>,
    },
    /// 移除断点
    RemoveBreakpoint {
        /// 断点标识符
        id: u64,
    },
    /// 添加变量监视
    AddWatch {
        /// 监视表达式
        expression: String,
    },
    /// 移除变量监视
    RemoveWatch {
        /// 监视标识符
        id: u64,
    },
    /// 请求调用栈信息
    GetCallStack,
    /// 请求变量信息
    GetVariables {
        /// 栈帧索引
        frame_index: usize,
    },
    /// 求值表达式
    Evaluate {
        /// 表达式
        expression: String,
    },
}

/// 调试响应
#[derive(Debug, Clone)]
pub enum DebugResponse {
    /// 断点已设置
    BreakpointSet {
        /// 断点标识符
        id: u64,
    },
    /// 断点已移除
    BreakpointRemoved {
        /// 是否成功
        success: bool,
    },
    /// 调用栈信息
    CallStackInfo {
        /// 调用栈帧列表（JSON 格式）
        frames_json: String,
    },
    /// 变量信息
    VariablesInfo {
        /// 变量列表（JSON 格式）
        variables_json: String,
    },
    /// 表达式求值结果
    EvaluationResult {
        /// 求值结果（JSON 格式）
        result_json: String,
    },
    /// 变量监视结果
    WatchResult {
        /// 监视标识符
        id: u64,
        /// 求值结果（JSON 格式）
        result_json: String,
    },
}

/// 通过调试通道传递的消息
#[derive(Debug, Clone)]
pub enum WireMessage {
    /// HMR 热更新事件
    HmrEvent(HmrEvent),
    /// 运行时控制命令
    Command(RuntimeCommand),
    /// 调试响应
    DebugResponse(DebugResponse),
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

    /// 发送调试响应到编辑器
    fn send_response(&self, response: DebugResponse) -> Result<(), DebugWireError>;
}
