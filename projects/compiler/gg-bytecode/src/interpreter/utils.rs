use crate::format::BytecodeModule;

use super::BytecodeInterpreter;

impl BytecodeInterpreter {
    /// 确保函数缓存已构建，从模块的 function_index 或函数列表构建
    pub(crate) fn ensure_function_cache(&mut self, module: &BytecodeModule) {
        if !self.function_cache.is_empty() {
            return;
        }
        if !module.function_index.is_empty() {
            for (name, idx) in &module.function_index {
                self.function_cache.insert(name.clone(), *idx);
            }
        }
        else {
            for (i, func) in module.functions.iter().enumerate() {
                self.function_cache.insert(func.name.clone(), i);
            }
        }
    }

    /// 查找函数索引，优先使用缓存，未命中时回退到线性搜索并缓存结果
    pub(crate) fn find_function_index(&mut self, module: &BytecodeModule, name: &str) -> Option<usize> {
        self.ensure_function_cache(module);
        if let Some(&idx) = self.function_cache.get(name) {
            return Some(idx);
        }
        for (i, func) in module.functions.iter().enumerate() {
            if func.name == name {
                self.function_cache.insert(name.to_string(), i);
                return Some(i);
            }
        }
        None
    }

    /// 创建新的字节码解释器
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(32),
            running: false,
            debug_controller: None,
            debug_paused: false,
            debug_initial_depth: 0,
            debug_skip_check: false,
            function_cache: std::collections::HashMap::new(),
            profiler: None,
        }
    }

    /// 创建带性能分析器的字节码解释器
    pub fn with_profiler(profiler: std::sync::Arc<std::sync::Mutex<crate::profiler::BytecodeProfiler>>) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(32),
            running: false,
            debug_controller: None,
            debug_paused: false,
            debug_initial_depth: 0,
            debug_skip_check: false,
            function_cache: std::collections::HashMap::new(),
            profiler: Some(profiler),
        }
    }

    /// 设置调试控制器
    pub fn set_debug_controller(
        &mut self,
        controller: Option<std::rc::Rc<std::cell::RefCell<dyn crate::debug_protocol::DebugController>>>,
    ) {
        self.debug_controller = controller;
    }

    /// 检查解释器是否因调试而暂停
    pub fn is_debug_paused(&self) -> bool {
        self.debug_paused
    }

    /// 恢复调试暂停的执行
    pub fn resume(&mut self) {
        self.debug_paused = false;
        self.debug_skip_check = true;
    }

    /// 获取调用栈的克隆副本，用于调试检查
    pub fn debug_call_stack(&self) -> Vec<super::InterpreterFrame> {
        self.call_stack.clone()
    }

    /// 获取指定栈帧的局部变量列表
    pub fn debug_local_variables(&self, frame_index: usize) -> Vec<(String, crate::format::BytecodeValue)> {
        match self.call_stack.get(frame_index) {
            Some(frame) => frame.locals.iter().enumerate().map(|(i, v)| (format!("local_{}", i), v.clone())).collect(),
            None => Vec::new(),
        }
    }
}

impl Default for BytecodeInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
