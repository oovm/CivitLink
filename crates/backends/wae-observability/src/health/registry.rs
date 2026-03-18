//! 健康检查注册管理
//!
//! 提供健康检查器的注册和管理功能

use crate::{HealthChecker, HealthReport};
use std::{collections::HashMap, sync::Arc};

/// 健康检查分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckCategory {
    /// 存活检查
    Liveness,
    /// 就绪检查
    Readiness,
}

/// 健康检查注册器
///
/// 管理多个健康检查器，支持分类为存活检查和就绪检查
pub struct HealthRegistry {
    liveness_checkers: HashMap<String, Arc<dyn HealthChecker>>,
    readiness_checkers: HashMap<String, Arc<dyn HealthChecker>>,
}

impl HealthRegistry {
    /// 创建新的健康检查注册器
    pub fn new() -> Self {
        Self { liveness_checkers: HashMap::new(), readiness_checkers: HashMap::new() }
    }

    /// 添加健康检查器
    ///
    /// # Arguments
    ///
    /// * `name` - 检查器名称
    /// * `checker` - 健康检查器实例
    /// * `category` - 检查分类
    pub fn add_checker(&mut self, name: String, checker: Arc<dyn HealthChecker>, category: CheckCategory) {
        match category {
            CheckCategory::Liveness => {
                self.liveness_checkers.insert(name, checker);
            }
            CheckCategory::Readiness => {
                self.readiness_checkers.insert(name, checker);
            }
        }
    }

    /// 移除健康检查器
    ///
    /// # Arguments
    ///
    /// * `name` - 检查器名称
    /// * `category` - 检查分类
    pub fn remove_checker(&mut self, name: &str, category: CheckCategory) {
        match category {
            CheckCategory::Liveness => {
                self.liveness_checkers.remove(name);
            }
            CheckCategory::Readiness => {
                self.readiness_checkers.remove(name);
            }
        }
    }

    /// 执行所有存活检查
    ///
    /// 返回存活检查报告
    pub async fn check_liveness(&self) -> HealthReport {
        let mut checks = Vec::with_capacity(self.liveness_checkers.len());
        for checker in self.liveness_checkers.values() {
            checks.push(checker.check().await);
        }
        HealthReport::new(checks)
    }

    /// 执行所有就绪检查
    ///
    /// 返回就绪检查报告
    pub async fn check_readiness(&self) -> HealthReport {
        let mut checks = Vec::with_capacity(self.readiness_checkers.len());
        for checker in self.readiness_checkers.values() {
            checks.push(checker.check().await);
        }
        HealthReport::new(checks)
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}
