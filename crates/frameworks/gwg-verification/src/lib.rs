//! 自动化验证系统
//! 
//! 提供自动化的测试执行和验证报告生成功能

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gwg_agent::prelude::*;

/// 验证结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 跳过
    Skipped,
}

/// 验证报告
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// 任务ID
    pub task_id: String,
    /// 验证结果
    pub result: VerificationResult,
    /// 执行时间（秒）
    pub execution_time: f64,
    /// 输出信息
    pub output: String,
    /// 错误信息
    pub error: Option<String>,
}

impl VerificationReport {
    /// 创建新的验证报告
    pub fn new(task_id: &str, result: VerificationResult, execution_time: f64, output: &str, error: Option<&str>) -> Self {
        Self {
            task_id: task_id.to_string(),
            result,
            execution_time,
            output: output.to_string(),
            error: error.map(|s| s.to_string()),
        }
    }

    /// 生成报告字符串
    pub fn to_string(&self) -> String {
        let result_str = match self.result {
            VerificationResult::Success => "SUCCESS",
            VerificationResult::Failure => "FAILURE",
            VerificationResult::Skipped => "SKIPPED",
        };

        format!(
            "Verification Report for Task {}\n" 
            "Result: {}\n" 
            "Execution Time: {:.2}s\n" 
            "Output:\n{}\n" 
            "Error:{}",
            self.task_id,
            result_str,
            self.execution_time,
            self.output,
            self.error.as_ref().map(|e| format!("\n{}", e)).unwrap_or(" None".to_string())
        )
    }
}

/// 验证系统
pub struct VerificationSystem {
    /// 报告存储
    reports: Arc<Mutex<Vec<VerificationReport>>>,
}

impl VerificationSystem {
    /// 创建新的验证系统
    pub fn new() -> Self {
        Self {
            reports: Arc::new(Mutex::new(vec![])),
        }
    }

    /// 执行验证
    pub fn verify(&self, task: &Task) -> VerificationReport {
        let start_time = std::time::Instant::now();

        let result = match task.task_type {
            TaskType::Compile => self.verify_compile(),
            TaskType::Test => self.verify_test(),
            TaskType::Build => self.verify_build(),
            TaskType::Verify => self.verify_verify(),
        };

        let execution_time = start_time.elapsed().as_secs_f64();
        let report = VerificationReport::new(
            &task.id,
            result.0,
            execution_time,
            &result.1,
            result.2.as_deref(),
        );

        let mut reports = self.reports.lock().unwrap();
        reports.push(report.clone());

        report
    }

    /// 验证编译
    fn verify_compile(&self) -> (VerificationResult, String, Option<String>) {
        let output = Command::new("cargo")
            .arg("check")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    (VerificationResult::Success, stdout, None)
                } else {
                    (VerificationResult::Failure, stdout, Some(stderr))
                }
            }
            Err(e) => {
                (VerificationResult::Failure, "".to_string(), Some(e.to_string()))
            }
        }
    }

    /// 验证测试
    fn verify_test(&self) -> (VerificationResult, String, Option<String>) {
        let output = Command::new("cargo")
            .arg("test")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    (VerificationResult::Success, stdout, None)
                } else {
                    (VerificationResult::Failure, stdout, Some(stderr))
                }
            }
            Err(e) => {
                (VerificationResult::Failure, "".to_string(), Some(e.to_string()))
            }
        }
    }

    /// 验证构建
    fn verify_build(&self) -> (VerificationResult, String, Option<String>) {
        let output = Command::new("cargo")
            .arg("build")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    (VerificationResult::Success, stdout, None)
                } else {
                    (VerificationResult::Failure, stdout, Some(stderr))
                }
            }
            Err(e) => {
                (VerificationResult::Failure, "".to_string(), Some(e.to_string()))
            }
        }
    }

    /// 验证验证（预留）
    fn verify_verify(&self) -> (VerificationResult, String, Option<String>) {
        (VerificationResult::Skipped, "Verification skipped".to_string(), None)
    }

    /// 获取所有报告
    pub fn get_reports(&self) -> Vec<VerificationReport> {
        let reports = self.reports.lock().unwrap();
        reports.clone()
    }

    /// 获取任务的报告
    pub fn get_task_report(&self, task_id: &str) -> Option<VerificationReport> {
        let reports = self.reports.lock().unwrap();
        reports
            .iter()
            .find(|r| r.task_id == task_id)
            .cloned()
    }
}

/// 预导入模块
pub mod prelude {
    pub use super::{VerificationReport, VerificationResult, VerificationSystem};
}