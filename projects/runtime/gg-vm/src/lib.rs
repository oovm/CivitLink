#![warn(missing_docs)]

//! GG 引擎字节码虚拟机模块
//! 基于 gg-bytecode 的字节码解释器，提供脚本执行能力

/// VM 调试器模块
pub mod debugger;

/// 热点检测模块
pub mod hotspot;

/// 指令缓存模块
pub mod instruction_cache;

pub use debugger::{VmDebugger, Watch, WatchId};
pub use gg_bytecode::{BytecodeModule, BytecodeValue, Host, InterpretResult as VmResult, InterpreterFrame as CallFrame};
pub use hotspot::HotspotDetector;
pub use instruction_cache::InstructionCache;

use std::{cell::RefCell, rc::Rc};

use gg_bytecode::{BytecodeInterpreter, BytecodeReader, BytecodeWriter, DebugController};
use gg_ir::IrModule;

/// 字节码虚拟机
///
/// 包装 gg-bytecode 的 BytecodeInterpreter，提供脚本执行能力。
/// 支持从 BytecodeModule 或 IrModule 执行。
pub struct Vm {
    /// 内部字节码解释器
    interpreter: BytecodeInterpreter,
    /// 指令缓存
    instruction_cache: InstructionCache,
    /// 热点检测器
    hotspot_detector: HotspotDetector,
}

impl Vm {
    /// 创建新的虚拟机
    pub fn new() -> Self {
        Self {
            interpreter: BytecodeInterpreter::new(),
            instruction_cache: InstructionCache::new(),
            hotspot_detector: HotspotDetector::new(),
        }
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

    /// 设置调试控制器
    pub fn set_debug_controller(&mut self, controller: Option<Rc<RefCell<dyn DebugController>>>) {
        self.interpreter.set_debug_controller(controller);
    }

    /// 检查虚拟机是否因调试而暂停
    pub fn is_debug_paused(&self) -> bool {
        self.interpreter.is_debug_paused()
    }

    /// 恢复调试暂停的执行
    pub fn resume(&mut self) {
        self.interpreter.resume();
    }

    /// 获取调用栈的克隆副本，用于调试检查
    pub fn debug_call_stack(&self) -> Vec<CallFrame> {
        self.interpreter.debug_call_stack()
    }

    /// 获取指定栈帧的局部变量列表
    pub fn debug_local_variables(&self, frame_index: usize) -> Vec<(String, BytecodeValue)> {
        self.interpreter.debug_local_variables(frame_index)
    }

    /// 获取指令缓存的不可变引用
    pub fn instruction_cache(&self) -> &InstructionCache {
        &self.instruction_cache
    }

    /// 获取指令缓存的可变引用
    pub fn instruction_cache_mut(&mut self) -> &mut InstructionCache {
        &mut self.instruction_cache
    }

    /// 获取热点检测器的不可变引用
    pub fn hotspot_detector(&self) -> &HotspotDetector {
        &self.hotspot_detector
    }

    /// 获取热点检测器的可变引用
    pub fn hotspot_detector_mut(&mut self) -> &mut HotspotDetector {
        &mut self.hotspot_detector
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
