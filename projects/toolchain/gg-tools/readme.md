# gg-cli

**GG Game Engine 的命令行工具，提供项目管理和开发工具。**

## 📋 模块简介

gg-cli 是 GG Game Engine 的命令行工具，提供项目管理、构建和开发相关的命令行功能，简化游戏开发流程。

## ✨ 核心功能

- **项目管理**：创建、初始化和管理 GG 项目
- **构建命令**：构建游戏和项目
- **运行命令**：运行游戏和测试
- **插件管理**：管理项目的插件
- **资产处理**：处理游戏资产
- **开发工具**：提供开发辅助工具

## 🚀 使用方法

### 安装

```bash
cargo install --path projects/toolchain/gg-cli
```

### 基础命令

```bash
# 创建新项目
gg new my-game

# 构建项目
gg build

# 运行项目
gg run

# 管理插件
gg plugin add gg-plugin-dialogue
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [工具链设计](../../../design/architecture/overview.md) - 了解工具链系统的设计理念