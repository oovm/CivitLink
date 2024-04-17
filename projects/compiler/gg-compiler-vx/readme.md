# gg-compiler-vx

GG 引擎的 VX 文件编译器，负责将 *.vx 文件编译为平台特定的 GUI 代码。

## 功能

- 解析 *.vx 文件结构
- 编译 `<template>` 部分为平台特定的 GUI 代码
- 编译 `<script>` 部分为 Valkyrie 脚本
- 编译 `<style>` 部分为 GG Renderer 可处理的样式

## 使用

```rust
use gg_compiler_vx::VxCompiler;

let compiler = VxCompiler::new();
let result = compiler.compile("path/to/file.vx");
```

## 依赖

- `gg-core`: 核心类型定义
- `gg-compiler-core`: 编译器核心接口