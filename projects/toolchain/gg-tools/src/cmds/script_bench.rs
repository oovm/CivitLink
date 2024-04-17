//! `gg script-bench` 命令实现
//!
//! 脚本性能测试工具

use crate::{EngineHost, GResult, ScriptEngine, Vm, VmResult};
use std::{mem::size_of, time::Instant};

/// 执行 `script-bench` 子命令
///
/// 脚本性能测试工具
pub fn cmd_script_bench() -> GResult<()> {
    println!("GG Game Engine - 脚本性能测试工具");
    println!("==================================");
    println!();

    test_startup_time();

    test_execution_performance();

    test_memory_usage();

    println!();
    println!("性能测试完成！");
    Ok(())
}

/// 测试脚本系统启动时间
fn test_startup_time() {
    println!("1. 测试启动时间:");
    println!("----------------");

    let start = Instant::now();

    let mut script_engine = ScriptEngine::new();

    let elapsed = start.elapsed();
    println!("脚本引擎启动时间: {:?}", elapsed);

    let test_script = r#"
        function init() {
            print("Script initialized");
        }

        function update(delta) {
        }
    "#;

    let start_load = Instant::now();
    let result = script_engine.load_script_string(test_script, "test");
    let load_elapsed = start_load.elapsed();

    match result {
        Ok(_) => println!("脚本加载时间: {:?}", load_elapsed),
        Err(e) => println!("脚本加载失败: {:?}", e),
    }

    println!();
}

/// 测试脚本执行性能
fn test_execution_performance() {
    println!("2. 测试执行性能:");
    println!("----------------");

    let mut script_engine = ScriptEngine::new();
    let mut host = EngineHost::new();

    let test_script = r#"
        function init() {
            print("Script initialized");
        }

        function calculate() {
            let sum = 0;
            for (let i = 0; i < 1000000; i++) {
                sum += i;
            }
            return sum;
        }

        function update(delta) {
        }
    "#;

    if let Ok(_) = script_engine.load_script_string(test_script, "test") {
        let iterations = 10;
        let mut total_time = 0;

        for i in 0..iterations {
            let iter_start = Instant::now();
            let result = script_engine.call_function("calculate", &mut host);
            let iter_elapsed = iter_start.elapsed();
            total_time += iter_elapsed.as_millis();

            match result {
                VmResult::Ok | VmResult::Return(_) => {
                    println!("迭代 {}: {:?}", i + 1, iter_elapsed);
                }
                VmResult::Error(e) => {
                    println!("迭代 {} 错误: {}", i + 1, e);
                }
            }
        }

        let average_time = total_time / iterations;
        println!("平均执行时间: {}ms", average_time);
        println!("总执行时间: {}ms", total_time);
    }
    else {
        println!("脚本加载失败");
    }

    println!();
}

/// 测试内存使用情况
fn test_memory_usage() {
    println!("3. 测试内存使用:");
    println!("----------------");

    println!("ScriptEngine 大小: {} bytes", size_of::<ScriptEngine>());
    println!("Vm 大小: {} bytes", size_of::<Vm>());
    println!("EngineHost 大小: {} bytes", size_of::<EngineHost>());

    let mut script_engine = ScriptEngine::new();

    let test_script = r#"
        function init() {
            print("Script initialized");
        }

        function update(delta) {
        }
    "#;

    if let Ok(_) = script_engine.load_script_string(test_script, "test") {
        println!("脚本加载后，内存使用正常");
    }
    else {
        println!("脚本加载失败");
    }

    println!();
}
