---
layout: home

hero:
    name: GG
    text: Game Engine
    tagline: 基于 Rust 的游戏引擎框架，ECS 核心、多平台支持、插件化架构，一次编写，到处运行
    actions:
        - theme: brand
          text: 开始使用
          link: /guide/introduction
        - theme: alt
          text: 架构设计
          link: /architecture/overview

features:
    - icon: 🧩
      title: ECS 核心架构
      details: 基于 bevy_ecs 的高性能实体组件系统，支持并行调度，数据驱动设计
    - icon: 🌐
      title: 多平台分发
      details: 支持 Windows、macOS、iOS、Android、H5、微信小游戏，统一平台抽象层
    - icon: 🔌
      title: 插件化架构
      details: 模块化设计，编译时静态链接插件，资深玩家可自由扩展引擎功能
    - icon: 🚀
      title: WASM 虚拟机
      details: 内置 WASM 脚本虚拟机，高级玩家可编写 Mod 脚本，安全沙箱隔离
    - icon: 🔥
      title: HMR 热更新
      details: 支持资源、数据、脚本的实时热重载，加速创作迭代
    - icon: 📦
      title: Monorepo 结构
      details: 两层 monorepo 组织，模块清晰，避免目录过深
---
