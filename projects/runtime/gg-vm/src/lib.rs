#![warn(missing_docs)]

//! GG 引擎字节码虚拟机模块
//! 基于 gg-bytecode 的字节码解释器，提供脚本执行能力

pub use gg_bytecode::{BytecodeModule, BytecodeValue, Host, InterpretResult as VmResult, InterpreterFrame as CallFrame};

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
        Self { interpreter: BytecodeInterpreter::new() }
    }

    /// 执行字节码模块中的指定函数
    pub fn execute<H: Host>(&mut self, module: &BytecodeModule, function_name: &str, host: &mut H) -> VmResult {
        self.interpreter.execute(module, function_name, host)
    }

    /// 从 IR 模块执行指定函数（便捷方法）
    ///
    /// 内部将 IrModule 编译为字节码后执行。
    pub fn execute_ir<H: Host>(&mut self, module: &IrModule, function_name: &str, host: &mut H) -> VmResult {
        let bytecode_data = match BytecodeWriter::write(module) {
            Ok(data) => data,
            Err(e) => {
                return VmResult::Error {
                    message: format!("编译 IR 到字节码失败: {}", e.message), source_location: None
                };
            }
        };

        let bytecode_module = match BytecodeReader::read(&bytecode_data) {
            Ok(m) => m,
            Err(e) => {
                return VmResult::Error { message: format!("加载字节码失败: {}", e.message), source_location: None };
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

#[cfg(test)]
mod tests {
    use crate::{BytecodeModule, BytecodeValue, Host, Vm, VmResult};
    use gg_bytecode::format::{BytecodeFunction, BytecodeInstruction};
    use gg_ir::{IrFunction, IrModule, IrValue, OpCode};

    /// 测试用宿主实现
    struct TestHost;

    impl Host for TestHost {
        fn spawn_entity(&mut self) -> u64 {
            0
        }

        fn despawn_entity(&mut self, _entity_id: u64) {}

        fn add_component(&mut self, _entity_id: u64, _component_type: &str, _value: BytecodeValue) {}

        fn get_component_field(
            &mut self,
            _entity_id: u64,
            _component_type: &str,
            _field: &str,
        ) -> Option<BytecodeValue> {
            None
        }

        fn set_component_field(
            &mut self,
            _entity_id: u64,
            _component_type: &str,
            _field: &str,
            _value: BytecodeValue,
        ) {}

        fn call_host_function(
            &mut self,
            _name: &str,
            _args: Vec<BytecodeValue>,
        ) -> Option<BytecodeValue> {
            None
        }
    }

    /// 测试 Vm::new() 初始状态
    #[test]
    fn test_vm_new() {
        let vm = Vm::new();
        assert!(vm.stack().is_empty());
        assert!(vm.call_stack().is_empty());
        assert!(!vm.is_running());
    }

    /// 测试通过 BytecodeModule 执行基本函数
    #[test]
    fn test_vm_execute_basic() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![BytecodeValue::Int(42)],
            string_pool: vec![],
            functions: vec![BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 0 },
                    BytecodeInstruction::Return,
                ],
            }],
            debug_info: None,
        };

        let mut vm = Vm::new();
        let mut host = TestHost;
        let result = vm.execute(&module, "main", &mut host);

        match result {
            VmResult::Return(BytecodeValue::Int(42)) => {}
            other => panic!("期望返回 Int(42)，实际: {:?}", other),
        }
    }

    /// 测试通过 IrModule 执行函数
    #[test]
    fn test_vm_execute_ir() {
        let module = IrModule {
            name: "test".to_string(),
            constants: vec![IrValue::Int(42)],
            functions: vec![IrFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![OpCode::LoadConst(0), OpCode::Return],
            }],
        };

        let mut vm = Vm::new();
        let mut host = TestHost;
        let result = vm.execute_ir(&module, "main", &mut host);

        match result {
            VmResult::Return(BytecodeValue::Int(42)) => {}
            other => panic!("期望返回 Int(42)，实际: {:?}", other),
        }
    }

    /// 测试执行栈访问
    #[test]
    fn test_vm_stack_access() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![BytecodeValue::Int(42)],
            string_pool: vec![],
            functions: vec![BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 0 },
                    BytecodeInstruction::Dup,
                    BytecodeInstruction::Return,
                ],
            }],
            debug_info: None,
        };

        let mut vm = Vm::new();
        let mut host = TestHost;
        let result = vm.execute(&module, "main", &mut host);

        match result {
            VmResult::Return(BytecodeValue::Int(42)) => {}
            other => panic!("期望返回 Int(42)，实际: {:?}", other),
        }

        assert_eq!(vm.stack().len(), 1);
        assert_eq!(vm.stack()[0], BytecodeValue::Int(42));
    }

    /// 测试调用栈访问
    #[test]
    fn test_vm_call_stack() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![
                BytecodeValue::Int(10),
                BytecodeValue::String("helper".to_string()),
            ],
            string_pool: vec![],
            functions: vec![
                BytecodeFunction {
                    name: "helper".to_string(),
                    param_count: 0,
                    local_count: 0,
                    instructions: vec![
                        BytecodeInstruction::LoadConst { index: 0 },
                        BytecodeInstruction::Return,
                    ],
                },
                BytecodeFunction {
                    name: "main".to_string(),
                    param_count: 0,
                    local_count: 0,
                    instructions: vec![
                        BytecodeInstruction::LoadConst { index: 1 },
                        BytecodeInstruction::Call { arg_count: 0 },
                        BytecodeInstruction::Return,
                    ],
                },
            ],
            debug_info: None,
        };

        let mut vm = Vm::new();
        let mut host = TestHost;
        let result = vm.execute(&module, "main", &mut host);

        match result {
            VmResult::Return(BytecodeValue::Int(10)) => {}
            other => panic!("期望返回 Int(10)，实际: {:?}", other),
        }

        assert!(vm.call_stack().is_empty());
    }
}
