//! 自定义验证器注册表模块
//! 提供自定义验证函数的注册和查找机制

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

/// 自定义验证函数类型
///
/// 接收字段值的字符串表示，返回验证是否通过
pub type ValidatorFn = fn(&str) -> bool;

/// 自定义验证器注册表
///
/// 存储名称到验证函数的映射，供验证时查找调用
#[derive(Debug)]
pub struct CustomValidatorRegistry {
    /// 验证器名称到验证函数的映射
    validators: HashMap<String, ValidatorFn>,
}

impl CustomValidatorRegistry {
    /// 创建空的验证器注册表
    pub fn new() -> Self {
        Self { validators: HashMap::new() }
    }

    /// 注册自定义验证器
    ///
    /// 如果名称已存在，覆盖原有验证函数
    pub fn register(&mut self, name: impl Into<String>, validator: ValidatorFn) {
        self.validators.insert(name.into(), validator);
    }

    /// 查找指定名称的验证器
    ///
    /// 返回 Some 表示找到验证器，None 表示未注册
    pub fn get(&self, name: &str) -> Option<&ValidatorFn> {
        self.validators.get(name)
    }

    /// 检查指定名称的验证器是否已注册
    pub fn contains(&self, name: &str) -> bool {
        self.validators.contains_key(name)
    }

    /// 使用指定验证器验证值
    ///
    /// 返回 true 表示验证通过，false 表示验证失败
    /// 如果验证器未注册，返回 false
    pub fn validate(&self, name: &str, value: &str) -> bool {
        match self.validators.get(name) {
            Some(validator) => validator(value),
            None => false,
        }
    }
}

impl Default for CustomValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局自定义验证器注册表
static GLOBAL_REGISTRY: LazyLock<Mutex<CustomValidatorRegistry>> = LazyLock::new(|| Mutex::new(CustomValidatorRegistry::new()));

/// 向全局注册表注册自定义验证器
pub fn register_validator(name: impl Into<String>, validator: ValidatorFn) {
    GLOBAL_REGISTRY.lock().unwrap().register(name, validator);
}

/// 检查全局注册表中是否存在指定验证器
pub fn has_validator(name: &str) -> bool {
    GLOBAL_REGISTRY.lock().unwrap().contains(name)
}

/// 使用全局注册表验证值
pub fn validate_with_registry(name: &str, value: &str) -> bool {
    GLOBAL_REGISTRY.lock().unwrap().validate(name, value)
}
