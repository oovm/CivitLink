# My First Platformer

这是一个使用 GG 游戏引擎创建的平台跳跃游戏示例。

## 项目结构

```
my-first-platformer/
├── assets/           # 资源目录
│   ├── audio/        # 音频资源
│   │   ├── bgm/      # 背景音乐
│   │   └── sfx/      # 音效
│   ├── graphics/     # 图形资源
│   │   ├── sprites/  # 精灵
│   │   ├── backgrounds/ # 背景
│   │   └── ui/       # UI 资源
│   ├── scripts/      # 脚本文件
│   └── scenes/       # 场景配置
├── src/              # 源代码目录
│   ├── components/   # ECS 组件
│   ├── systems/      # ECS 系统
│   ├── resources/    # ECS 资源
│   └── utils/        # 工具函数
├── config/           # 配置文件
├── README.md         # 项目说明
└── game.toml         # 引擎配置
```

## 核心功能

- **角色控制**：支持左右移动和跳跃
- **物理系统**：实现重力和碰撞检测
- **平台系统**：包含静态平台和移动平台
- **收集系统**：收集物品增加分数
- **目标系统**：到达目标完成关卡
- **ECS 架构**：使用 GG 引擎的 ECS 架构实现核心逻辑

## 如何运行

1. 确保你已经安装了 GG 引擎
2. 进入项目目录
3. 运行 `gg run` 命令启动游戏

## 游戏玩法

- **左右箭头键**：控制角色左右移动
- **空格键**：跳跃
- **收集物品**：收集场景中的物品增加分数
- **到达目标**：到达场景右侧的目标完成关卡

## 扩展指南

### 添加新关卡

1. 在 `assets/scripts/start.valkyrie` 文件中，找到 `LevelData` 资源
2. 修改 `platforms`、`collectibles` 和 `goal` 数据
3. 调整平台位置、大小和移动属性
4. 调整收集品位置和价值
5. 调整目标位置

### 添加新平台类型

1. 在 `src/components/platformer.rs` 文件中，扩展 `Platform` 组件
2. 在 `src/systems/platformer_system.rs` 文件中，添加新的平台行为逻辑
3. 在脚本中创建平台实体时设置相应的属性

### 添加新收集品类型

1. 在 `src/components/platformer.rs` 文件中，扩展 `Collectible` 组件
2. 在 `src/systems/platformer_system.rs` 文件中，添加新的收集品行为逻辑
3. 在脚本中创建收集品实体时设置相应的属性

## 技术说明

- **ECS 架构**：使用实体-组件-系统设计模式
- **Valkyrie 脚本**：使用内置脚本系统实现游戏逻辑
- **物理系统**：实现基本的重力和碰撞检测
- **渲染系统**：使用 GG 引擎的渲染系统显示游戏元素

## 注意事项

- 本示例使用占位资源，实际开发中需要替换为真实资源
- 本示例只实现了基础功能，实际游戏可能需要更多功能扩展