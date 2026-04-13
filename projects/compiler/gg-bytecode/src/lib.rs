#![warn(missing_docs)]

//! GG 引擎字节码模块
//! 提供可移植的字节码格式、序列化/反序列化、调试信息和字节码解释器

/// 调试信息
pub mod debug_info;
/// 调试协议
pub mod debug_protocol;
/// 字节码格式定义
pub mod format;
/// 宿主接口
pub mod host;
/// 字节码解释器
pub mod interpreter;
/// prelude
pub mod prelude;
/// 性能分析器
pub mod profiler;
/// 字节码读取器
pub mod reader;
/// 字节码写入器
pub mod writer;

pub use debug_info::{DebugEntry, DebugInfo, SourceLocation};
pub use debug_protocol::{
    BasicDebugController, BasicDebugProtocol, Breakpoint, BreakpointId, DebugController, DebugEvent, DebugProtocol,
    DebugTermination, DebugValue, StackFrameInfo, StepMode,
};
pub use format::{BytecodeEntryPoint, BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeOpCode, BytecodeValue};
pub use host::Host;
pub use interpreter::{BytecodeInterpreter, InterpretResult, InterpreterFrame};
pub use profiler::{BytecodeProfiler, FunctionProfile};
pub use reader::BytecodeReader;
pub use writer::BytecodeWriter;
