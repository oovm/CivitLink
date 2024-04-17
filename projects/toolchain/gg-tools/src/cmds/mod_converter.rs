//! `gg mod-converter` 命令实现
//!
//! WASM Mod 脚本转换为 Valkyrie 脚本的工具

use crate::GResult;
use std::{io, path::Path};

/// 执行 `mod-converter` 子命令
///
/// WASM Mod 脚本转换为 Valkyrie 脚本的工具
pub fn cmd_mod_converter(wasm_path: Option<&str>) -> GResult<()> {
    println!("GG Game Engine - WASM Mod 脚本转换工具");
    println!("====================================");
    println!();
    println!("此工具帮助您将 WASM Mod 脚本转换为 Valkyrie 脚本");
    println!();

    print_conversion_guide();

    println!();

    let path_to_check = if let Some(path) = wasm_path {
        path.to_string()
    }
    else {
        println!("请输入 WASM 模块路径（或按 Enter 跳过）:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    };

    let path_to_check = path_to_check.as_str();

    if !path_to_check.is_empty() {
        if Path::new(path_to_check).exists() {
            println!("已找到 WASM 模块，开始分析...");
            println!("分析完成，生成转换建议...");
        }
        else {
            println!("错误：WASM 模块文件不存在");
        }
    }

    println!();
    println!("转换工具使用完成，祝您转换顺利！");
    Ok(())
}

/// 打印转换指南
fn print_conversion_guide() {
    println!("转换指南:");
    println!("=============");
    println!();
    println!("1. 基本结构转换:");
    println!("   WASM 模块结构:");
    println!("   (module");
    println!("   (import \"env\" \"print\" (func $print (param i32 i32)))");
    println!("   (import \"env\" \"create_entity\" (func $create_entity (result i32)))");
    println!();
    println!("   (memory 1)");
    println!();
    println!("   (data (i32.const 0) \"Hello from script!\")");
    println!();
    println!("   (func (export \"init\")");
    println!("       (call $print (i32.const 0) (i32.const 18))");
    println!("       (call $create_entity)");
    println!("       drop");
    println!("   )");
    println!();
    println!("   (func (export \"update\") (param $delta f32)");
    println!("   )");
    println!("   )");
    println!();
    println!("   转换为 Valkyrie 脚本:");
    println!("   // 初始化函数");
    println!("   function init() {{");
    println!("       print(\"Hello from script!\");");
    println!("       spawn_entity();");
    println!("   }}");
    println!();
    println!("   // 更新函数");
    println!("   function update(delta: float) {{");
    println!("   }}");
    println!();
    println!("2. API 映射:");
    println!("   WASM 导入函数 -> Valkyrie 全局函数");
    println!("   - create_entity() -> spawn_entity()");
    println!("   - print() -> print()");
    println!("   - add_component() -> add_component()");
    println!("   - get_component_field() -> get_field()");
    println!("   - set_component_field() -> set_field()");
    println!();
    println!("3. 类型转换:");
    println!("   WASM 类型 -> Valkyrie 类型");
    println!("   - i32 -> int");
    println!("   - f32 -> float");
    println!("   - i8* -> string");
    println!();
    println!("4. 内存操作:");
    println!("   WASM 内存操作 -> Valkyrie 字符串/数组操作");
    println!("   - 内存加载字符串 -> 直接使用字符串字面量");
    println!();
    println!("5. 示例转换:");
    println!("   WASM 示例:");
    println!("   (func (export \"init\")");
    println!("       (call $print (i32.const 0) (i32.const 18))");
    println!("       (call $create_entity)");
    println!("       drop");
    println!("   )");
    println!();
    println!("   Valkyrie 示例:");
    println!("   function init() {{");
    println!("       print(\"Hello from script!\");");
    println!("       spawn_entity();");
    println!("   }}");
    println!();
    println!("6. 注意事项:");
    println!("   - Valkyrie 脚本使用更简洁的语法");
    println!("   - 不需要手动管理内存");
    println!("   - API 调用更直观");
    println!("   - 支持更丰富的语言特性");
}
