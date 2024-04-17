#![warn(missing_docs)]

//! GG 引擎字节码模块
//! 提供可移植的字节码格式、序列化/反序列化、调试信息和字节码解释器

/// 调试信息
pub mod debug_info;
/// 字节码格式定义
pub mod format;
/// 宿主接口
pub mod host;
/// 字节码解释器
pub mod interpreter;
/// prelude
pub mod prelude;
/// 字节码读取器
pub mod reader;
/// 字节码写入器
pub mod writer;

pub use debug_info::{DebugInfo, SourceLocation};
pub use format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeOpCode, BytecodeValue};
pub use host::Host;
pub use interpreter::{BytecodeInterpreter, InterpretResult, InterpreterFrame};
pub use reader::BytecodeReader;
pub use writer::BytecodeWriter;
