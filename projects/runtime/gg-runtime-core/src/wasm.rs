use gg_core::GResult;
use gg_bytecode::BytecodeValue;
use std::path::Path;

/// WASM 模块标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WasmModuleId(
    /// 模块唯一标识
    pub u64,
);

/// WASM 实例标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WasmInstanceId(
    /// 实例唯一标识
    pub u64,
);

/// WASM 沙箱配置
#[derive(Debug, Clone)]
pub struct WasmSandboxConfig {
    /// 最大内存页数
    pub max_memory_pages: u32,
    /// 执行时间限制（毫秒），0 表示无限制
    pub execution_time_limit_ms: u64,
    /// 允许的导入函数列表
    pub allowed_imports: Vec<String>,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 256,
            execution_time_limit_ms: 0,
            allowed_imports: Vec::new(),
        }
    }
}

impl WasmSandboxConfig {
    /// 创建新的沙箱配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大内存页数
    pub fn with_max_memory_pages(mut self, pages: u32) -> Self {
        self.max_memory_pages = pages;
        self
    }

    /// 设置执行时间限制
    pub fn with_execution_time_limit(mut self, ms: u64) -> Self {
        self.execution_time_limit_ms = ms;
        self
    }

    /// 添加允许的导入函数
    pub fn with_allowed_import(mut self, import: &str) -> Self {
        self.allowed_imports.push(import.to_string());
        self
    }
}

/// WASM 运行时 trait
///
/// 定义 WASM 模块加载、实例化和函数调用的标准接口。
/// 与具体 WASM 引擎（wasmtime/wasmer）解耦。
pub trait WasmRuntime {
    /// 从文件加载 WASM 模块
    fn load_module(&mut self, path: &Path) -> GResult<WasmModuleId>;

    /// 从字节加载 WASM 模块
    fn load_module_from_bytes(&mut self, bytes: &[u8], name: &str) -> GResult<WasmModuleId>;

    /// 实例化 WASM 模块
    fn instantiate(
        &mut self,
        module_id: WasmModuleId,
        config: &WasmSandboxConfig,
    ) -> GResult<WasmInstanceId>;

    /// 调用 WASM 函数
    fn call_function(
        &mut self,
        instance_id: WasmInstanceId,
        name: &str,
        args: &[BytecodeValue],
    ) -> GResult<Option<BytecodeValue>>;

    /// 获取 WASM 实例的线性内存
    fn get_memory(&self, instance_id: WasmInstanceId) -> Option<&[u8]>;

    /// 销毁 WASM 实例
    fn drop_instance(&mut self, instance_id: WasmInstanceId);

    /// 销毁 WASM 模块
    fn drop_module(&mut self, module_id: WasmModuleId);
}

/// WASM 宿主函数 trait
///
/// 定义 WASM 模块可导入的宿主函数集合。
/// 沙箱限制脚本只能通过此 trait 暴露的函数访问引擎功能。
pub trait WasmHostFunctions {
    /// 创建实体
    fn spawn_entity(&mut self) -> u64;

    /// 销毁实体
    fn despawn_entity(&mut self, entity_id: u64);

    /// 添加组件
    fn add_component(&mut self, entity_id: u64, component_type: &str, value: BytecodeValue);

    /// 获取组件字段
    fn get_component_field(
        &mut self,
        entity_id: u64,
        component_type: &str,
        field: &str,
    ) -> Option<BytecodeValue>;

    /// 设置组件字段
    fn set_component_field(
        &mut self,
        entity_id: u64,
        component_type: &str,
        field: &str,
        value: BytecodeValue,
    );

    /// 调用宿主函数
    fn call_host_function(&mut self, name: &str, args: Vec<BytecodeValue>) -> Option<BytecodeValue>;
}

/// WASM 运行时错误类型
#[derive(Debug, Clone)]
pub enum WasmError {
    /// 模块加载失败
    LoadFailed(String),
    /// 实例化失败
    InstantiationFailed(String),
    /// 函数调用失败
    CallFailed(String),
    /// 函数未找到
    FunctionNotFound(String),
    /// 沙箱违规
    SandboxViolation(String),
    /// 内存不足
    OutOfMemory,
    /// 执行超时
    Timeout,
}
