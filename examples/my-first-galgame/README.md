# My First Galgame

这是一个使用 GG 游戏引擎创建的视觉小说游戏示例。

## 项目结构

```
my-first-galgame/
├── assets/           # 资源目录
│   ├── backgrounds/  # 背景图片
│   ├── portraits/    # 角色立绘
│   ├── scripts/      # 脚本文件
│   │   └── start.script  # 对话脚本
├── src/              # 源代码目录
│   ├── components/   # ECS 组件
│   ├── resources/    # ECS 资源
│   └── systems/      # ECS 系统
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

1. 在 `assets/scripts/start.script` 文件中，添加新的对话节点
2. 使用以下格式编写对话：
   - `@node node_id`：定义新节点
   - `[speaker:character_id]`：指定说话者
   - 对话文本：直接写在说话者后面
   - `+ 选项文本 -> target_node_id`：添加选择项
   - `-> target_node_id`：指定跳转目标
   - `[command:args]`：添加内联命令

   示例：
   ```script
   @node start
   [speaker:sakura]
   你好，欢迎来到我的游戏！
   + 你好 -> hello
   + 再见 -> goodbye

   @node hello
   [speaker:sakura]
   很高兴见到你！
   -> end

   @node goodbye
   [speaker:sakura]
   再见，希望你玩得开心！
   -> end

   @node end
   [speaker:narrator]
   游戏结束
   ```

### 添加新角色

1. 在 `assets/portraits/` 目录中添加角色立绘
2. 在 `start.script` 文件中使用 `[show_portrait:character_id:expression:position]` 命令显示角色立绘

### 添加新场景

1. 在 `assets/backgrounds/` 目录中添加背景图片
2. 在 `start.script` 文件中使用 `[change_background:background_path]` 命令切换背景

## 技术说明

- **ECS 架构**：使用实体-组件-系统设计模式
- **Valkyrie 脚本**：使用内置脚本系统实现游戏逻辑
- **UI 系统**：使用 GG 引擎的 UI 系统显示对话和选择项

## 注意事项

- 本示例使用占位资源，实际开发中需要替换为真实资源
- 本示例只实现了基础功能，实际游戏可能需要更多功能扩展