# GG 引擎 Web 平台实现

为 WebAssembly/Web 环境提供平台抽象的具体实现。

## 模块结构

- **fs**: Web 平台文件系统实现
- **input**: Web 平台输入实现
- **platform**: Web 平台构建时实现
- **render**: Web 平台渲染后端适配
- **runtime**: Web 平台运行时实现
- **services**: Web 平台服务工厂
- **thread**: Web 平台线程实现
- **time**: Web 平台时间实现
- **window**: Web 平台窗口实现

## 渲染后端

Web 平台支持 WebGPU 和 WebGL2 渲染后端，提供自动检测和回退机制。

## Shader 编译管线

本引擎的 shader 始终使用 GG Shader (.shader) 格式编写，绝不直接使用 WGSL。
编译管线如下：

```text
GG Shader (.shader) → GG Shader Compiler → Naga IR → 目标后端格式
  - WebGPU 后端：Naga IR → SPIR-V（内部中间表示）
  - WebGL2 后端：Naga IR → GLSL
```

WGSL 仅在 Naga 内部作为中间表示使用，开发者始终编写 .shader 格式。
渲染后端检测结果会反馈到 shader 编译器，以选择正确的输出格式。