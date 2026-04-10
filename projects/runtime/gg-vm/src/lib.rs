#![warn(missing_docs)]

//! GG 引擎字节码虚拟机模块
//! 基于 gg-bytecode 的字节码解释器，提供脚本执行能力

pub use gg_bytecode::{
    BytecodeModule,
    BytecodeValue,
    Host,
    InterpretResult as VmResult,
    InterpreterFrame as CallFrame,
};

use gg_bytecode::{BytecodeInterpreter, BytecodeReader, BytecodeWriter};
use gg_ir::IrModule;

/// 字节码虚拟机
///
/// 包装 gg-bytecode 的 BytecodeInterpreter，提供脚本执行能力。
/// 支持从 BytecodeModule 或 IrModule 执行。
pub struct Vm {
    /// 内部字节码解释器
    interpreter: BytecodeInterpreter,
}

impl Vm {
    /// 创建新的虚拟机
    pub fn new() -> Self {
        Self {
            interpreter: BytecodeInterpreter::new(),
        }
    }

    /// 执行字节码模块中的指定函数
    pub fn execute<H: Host>(
        &mut self,
        module: &BytecodeModule,
        function_name: &str,
        host: &mut H,
    ) -> VmResult {
        self.interpreter.execute(module, function_name, host)
    }

    /// 从 IR 模块执行指定函数（便捷方法）
    ///
    /// 内部将 IrModule 编译为字节码后执行。
    pub fn execute_ir<H: Host>(
        &mut self,
        module: &IrModule,
        function_name: &str,
        host: &mut H,
    ) -> VmResult {
        let bytecode_data = match BytecodeWriter::write(module) {
            Ok(data) => data,
            Err(e) => {
                return VmResult::Error(format!(
                    "编译 IR 到字节码失败: {}",
                    e.message
                ));
            }
        };

        let bytecode_module = match BytecodeReader::read(&bytecode_data) {
            Ok(m) => m,
            Err(e) => {
                return VmResult::Error(format!(
                    "加载字节码失败: {}",
                    e.message
                ));
            }
        };

        self.execute(&bytecode_module, function_name, host)
    }

    /// 获取执行栈
    pub fn stack(&self) -> &[BytecodeValue] {
        &self.interpreter.stack
    }

    /// 获取调用栈
    pub fn call_stack(&self) -> &[CallFrame] {
        &self.interpreter.call_stack
    }

    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        self.interpreter.running
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
