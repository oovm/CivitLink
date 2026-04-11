# gg-compiler-script

**GG Game Engine 统一脚本编译器，负责编译 .v、.vx、.shader 三种脚本类型。**

## 📋 模块简介

gg-compiler-script 是 GG Game Engine 的统一脚本编译器，负责编译三种脚本类型：
- **.v**（Valkyrie 脚本）— 通用游戏逻辑脚本
- **.widget**（ValkyrieX 单文件组件）— GUI 界面组件
- **.shader**（GG Shader）— 着色器脚本

三种脚本共享同一套核心编译管线（源码 → AST → IR → 字节码），借助 VM 互通。

## ✨ 核心功能

- **Valkyrie 脚本编译**：将 .v 文件编译为字节码模块
- **VX 组件编译**：解析 .vx 单文件组件，编译脚本部分为字节码，保留模板和样式
- **Shader 编译**：解析 .shader 文件，提取 Valkyrie 兼容代码编译为字节码，保留 GPU 着色器描述
- **Transformer 集成**：所有编译器均实现 `gg-compiler-core::Transformer` trait，可接入编译流水线

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-compiler-script = { workspace = true }
```

### Valkyrie 脚本编译

```rust
use gg_compiler_script::prelude::*;

let transformer = ValkyrieScriptTransformer::new();
```

### VX 组件编译

```rust
use gg_compiler_script::prelude::*;

let transformer = VxTransformer::new();
```

### Shader 编译

```rust
use gg_compiler_script::prelude::*;

let transformer = ShaderTransformer::new();
```

## 📦 依赖关系

- **gg-compiler-core**：编译器核心功能（Transformer trait、Pipeline、ArtifactSet）
- **gg-core**：核心错误类型
- **gg-script**：Valkyrie 脚本编译器（源码 → IR → 字节码）
- **gg-bytecode**：字节码写入器
