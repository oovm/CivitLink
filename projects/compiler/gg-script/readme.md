# gg-script

GG Game Engine 的脚本编译器，负责将脚本代码编译为 IR 格式。

## 功能

- 脚本代码解析
- 语法检查
- 生成 IR 代码

## 依赖

- gg-core
- gg-ir
- oak-valkyrie

## 使用

```rust
use gg_script::compile;

// 编译脚本代码
let ir = compile("function main() { console.log('Hello, world!'); }")?;

// 使用生成的 IR
println!("Generated IR: {:?}", ir);
```
