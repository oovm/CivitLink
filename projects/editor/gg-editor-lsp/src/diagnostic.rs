use std::collections::HashMap;

use crate::types::{Diagnostic, DiagnosticSeverity};

/// 诊断信息收集器，按文档 URI 管理诊断信息
pub struct DiagnosticCollector {
    /// 按文档 URI 存储的诊断信息
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
}

impl DiagnosticCollector {
    /// 创建新的诊断信息收集器
    pub fn new() -> Self {
        Self { diagnostics: HashMap::new() }
    }

    /// 添加诊断信息，替换指定 URI 的所有诊断
    ///
    /// # 参数
    /// - `uri`: 文档 URI
    /// - `diagnostics`: 新的诊断信息列表
    pub fn add_diagnostics(&mut self, uri: &str, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.insert(uri.to_string(), diagnostics);
    }

    /// 清除指定 URI 的所有诊断信息
    ///
    /// # 参数
    /// - `uri`: 文档 URI
    pub fn clear_diagnostics(&mut self, uri: &str) {
        self.diagnostics.remove(uri);
    }

    /// 获取指定 URI 的诊断信息
    ///
    /// 如果指定 URI 不存在，返回空切片
    ///
    /// # 参数
    /// - `uri`: 文档 URI
    pub fn get_diagnostics(&self, uri: &str) -> &[Diagnostic] {
        self.diagnostics.get(uri).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 获取所有诊断信息
    pub fn get_all_diagnostics(&self) -> &HashMap<String, Vec<Diagnostic>> {
        &self.diagnostics
    }

    /// 统计所有错误级别的诊断数量
    pub fn error_count(&self) -> usize {
        self.diagnostics.values().flat_map(|v| v.iter()).filter(|d| d.severity == Some(DiagnosticSeverity::Error)).count()
    }

    /// 统计所有警告级别的诊断数量
    pub fn warning_count(&self) -> usize {
        self.diagnostics.values().flat_map(|v| v.iter()).filter(|d| d.severity == Some(DiagnosticSeverity::Warning)).count()
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}
