# My First Galgame

这是一个使用 GG 游戏引擎创建的视觉小说游戏示例。

## 项目结构

```
my-first-galgame/
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

- **对话系统**：支持显示对话文本、角色立绘和表情
- **选择系统**：支持对话中的选择项，影响剧情发展
- **场景系统**：支持场景切换和背景设置
- **ECS 架构**：使用 GG 引擎的 ECS 架构实现核心逻辑

## 如何运行

1. 确保你已经安装了 GG 引擎
2. 进入项目目录
3. 运行 `gg run` 命令启动游戏

## 游戏玩法

- 阅读对话文本
- 当出现选择项时，使用鼠标点击选择
- 剧情会根据你的选择分支发展

## 扩展指南

### 添加新对话

1. 在 `assets/scripts/start.valkyrie` 文件中，找到 `DialogueData` 资源
2. 在 `nodes` 对象中添加新的对话节点
3. 每个节点包含以下属性：
   - `text`：对话文本
   - `speaker`：说话者名称
   - `portrait`：角色立绘名称
   - `expression`：角色表情
   - `position`：立绘位置
   - `choices`：选择项数组
   - `next`：下一个节点 ID

### 添加新角色

1. 在 `assets/graphics/sprites/` 目录中添加角色立绘
2. 在对话节点中引用这些立绘

### 添加新场景

1. 在 `assets/graphics/backgrounds/` 目录中添加背景图片
2. 在 `Scene` 组件中引用这些背景

## 技术说明

- **ECS 架构**：使用实体-组件-系统设计模式
- **Valkyrie 脚本**：使用内置脚本系统实现游戏逻辑
- **UI 系统**：使用 GG 引擎的 UI 系统显示对话和选择项

## 注意事项

- 本示例使用占位资源，实际开发中需要替换为真实资源
- 本示例只实现了基础功能，实际游戏可能需要更多功能扩展