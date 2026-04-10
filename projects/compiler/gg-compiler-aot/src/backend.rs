use gg_bytecode::format::BytecodeModule;
use gg_core::{GError, GErrorKind, GResult};

use crate::error::AotError;
use crate::target::TargetPlatform;

/// AOT 编译后端 trait，定义将字节码编译为目标平台原生代码的标准接口
pub trait AotBackend {
    /// 获取后端名称
    fn name(&self) -> &str;

    /// 获取此后端支持的目标平台列表
    fn supported_targets(&self) -> Vec<TargetPlatform>;

    /// 将字节码模块编译为目标平台的原生代码
    fn compile(&self, module: &BytecodeModule, target: &TargetPlatform) -> GResult<Vec<u8>>;
}

/// AOT 后端注册表，管理可用的编译后端
pub struct AotBackendRegistry {
    /// 已注册的后端列表
    backends: Vec<Box<dyn AotBackend>>,
}

impl AotBackendRegistry {
    /// 创建新的 AOT 后端注册表
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    /// 注册一个编译后端
    pub fn register(&mut self, backend: Box<dyn AotBackend>) {
        self.backends.push(backend);
    }

    /// 查找支持指定目标平台的编译后端
    pub fn find_backend(&self, target: &TargetPlatform) -> Option<&dyn AotBackend> {
        self.backends
            .iter()
            .find(|b| b.supported_targets().contains(target))
            .map(|b| b.as_ref())
    }

    /// 查找支持指定目标平台的可变编译后端
    pub fn find_backend_mut(&mut self, target: &TargetPlatform) -> Option<&mut (dyn AotBackend + '_)> {
        self.backends
            .iter_mut()
            .find(|b| b.supported_targets().contains(target))
            .map(move |b| b.as_mut())
    }

    /// 获取所有已注册后端支持的目标平台列表
    pub fn available_targets(&self) -> Vec<TargetPlatform> {
        let mut targets = Vec::new();
        for backend in &self.backends {
            for target in backend.supported_targets() {
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }
        targets
    }

    /// 便捷方法：查找后端并编译字节码模块
    pub fn compile(
        &self,
        module: &BytecodeModule,
        target: &TargetPlatform,
    ) -> GResult<Vec<u8>> {
        if self.backends.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: AotError::NoBackendRegistered.to_string(),
            });
        }

        match self.find_backend(target) {
            Some(backend) => backend.compile(module, target),
            None => Err(GError {
                kind: GErrorKind::Runtime,
                message: AotError::UnsupportedTarget { target: *target }.to_string(),
            }),
        }
    }
}

impl Default for AotBackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}
