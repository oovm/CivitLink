# GG Platformer 引擎

GG Platformer 引擎是基于 GG 元引擎框架构建的平台跳跃游戏引擎，专注于 2D 平台跳跃游戏的开发。

## 功能特性

- **ECS 架构**：使用实体组件系统管理游戏实体
- **物理系统**：实现了重力、碰撞检测等物理效果
- **输入系统**：支持键盘、鼠标和游戏手柄输入
- **平台系统**：支持静态平台、移动平台、易碎平台等
- **收集系统**：支持收集物品和道具
- **渲染系统**：使用 WGPU 进行图形渲染
- **资源管理**：支持图像、音频等资源的加载和管理

## 架构设计

Platformer 引擎采用以下架构：

1. **核心层**：基于 GG 元引擎的 ECS 核心、资源系统、渲染系统等
2. **引擎层**：Platformer 专用的组件、系统和插件，特别是物理系统
3. **游戏层**：具体的游戏内容和逻辑

## 核心组件

### 实体组件

- **Transform**：位置和旋转信息
- **Velocity**：速度和加速度信息
- **Sprite**：精灵渲染信息
- **Collider**：碰撞检测信息
- **PhysicsBody**：物理属性信息
- **Health**：生命值信息
- **Player**：玩家控制标记
- **Platform**：平台标记
- **Collectible**：可收集物品标记
- **AI**：敌人 AI 信息

### 系统

- **InputSystem**：处理玩家输入
- **PhysicsSystem**：处理物理模拟和碰撞响应
- **MovementSystem**：处理实体移动
- **CollisionSystem**：处理碰撞检测和响应
- **CollectibleSystem**：处理物品收集逻辑
- **AISystem**：处理敌人 AI 行为
- **RenderSystem**：处理渲染逻辑

## 使用方法

### 创建 Platformer 引擎实例

```rust
use gg_engine_platformer::engine::PlatformerEngine;
use gg_engine_platformer::config::PlatformerConfig;

// 创建游戏配置
let config = PlatformerConfig::default();

// 创建 Platformer 引擎实例
let mut engine = PlatformerEngine::new(config, false);

// 初始化引擎
engine.initialize()?;

// 运行游戏
engine.run()?;
```

### 配置选项

Platformer 引擎支持以下配置选项：

- **游戏配置**：游戏名称、版本、作者等
- **显示配置**：窗口宽度、高度、是否全屏等
- **输入配置**：移动速度、跳跃力度、跳跃键等
- **游戏设置**：玩家初始生命值、重力加速度、最大下落速度等
- **物理设置**：碰撞检测精度、物理更新频率、碰撞层配置等

### 扩展功能

Platformer 引擎支持通过插件机制扩展功能：

- **自定义组件**：添加新的实体组件
- **自定义系统**：添加新的游戏系统
- **自定义插件**：添加新的功能插件

## 示例游戏

Platformer 引擎提供了一个基本示例游戏，展示了引擎的核心功能：

```bash
cargo run --example platformer-basic
```

## 后续扩展

- 添加更多平台类型，如移动平台、易碎平台等
- 实现敌人和障碍物
- 添加更多物品和道具
- 实现关卡系统
- 添加音效和背景音乐
- 实现得分系统和排行榜
- 支持多人游戏

## 技术要求

- Rust 1.60+
- Cargo
- WGPU 兼容的图形设备

## 许可证

MIT
