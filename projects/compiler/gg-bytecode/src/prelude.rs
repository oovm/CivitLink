//! GG 字节码 prelude 模块
//! 导出最常用的字节码类型

pub use crate::{
    debug_info::{DebugEntry, DebugInfo, SourceLocation},
    format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeOpCode, BytecodeValue, MAGIC, VERSION},
    host::Host,
    interpreter::{BytecodeInterpreter, InterpretResult, InterpreterFrame},
    reader::BytecodeReader,
    writer::BytecodeWriter,
};
