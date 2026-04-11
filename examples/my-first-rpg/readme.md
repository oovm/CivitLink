# My First RPG

这是一个使用 GG 游戏引擎创建的角色扮演游戏示例。

## 项目结构

```
my-first-rpg/
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
├── config/           # 配置文件
├── README.md         # 项目说明
└── game.toml         # 引擎配置
```

## 核心功能

- **角色系统**：包含等级、经验、属性等角色数据
- **对话系统**：与 NPC 进行对话
- **战斗系统**：与敌人进行战斗，包含攻击、技能、物品使用等
- **物品系统**：管理背包中的物品
- **位置系统**：在不同位置之间切换
- **菜单系统**：打开游戏菜单

## 如何运行

1. 确保你已经安装了 GG 引擎
2. 进入项目目录
3. 运行 `gg run` 命令启动游戏

## 游戏玩法

- **方向键**：控制角色移动
- **空格键**：与 NPC 交互或继续对话
- **ESC 键**：打开游戏菜单

## 扩展指南

### 添加新 NPC

1. 在 `assets/scripts/start.valkyrie` 文件中，找到 `game_state.npcs` 数组
2. 添加新的 NPC 对象，包含 id、name、position 和 dialogues 属性
3. NPC 会自动在游戏中创建

### 添加新敌人

1. 在 `assets/scripts/start.valkyrie` 文件中，找到 `enemies` 数组
2. 添加新的敌人对象，包含 id、name、hp、attack、defense、exp 等属性
3. 使用 `start_battle(enemy_id)` 函数开始与该敌人的战斗

### 添加新位置

1. 在 `assets/scripts/start.valkyrie` 文件中，找到 `locations` 对象
2. 添加新的位置对象，包含 name、background 和 spawn_point 属性
3. 使用 `load_location(location_id)` 函数加载该位置

### 添加新物品

1. 在 `assets/scripts/start.valkyrie` 文件中，找到 `game_state.inventory` 数组
2. 添加新的物品对象，包含 id、name、quantity 和 effect 属性
3. 物品会自动添加到玩家背包中

## 技术说明

- **Valkyrie 脚本**：使用内置脚本系统实现游戏逻辑
- **实体系统**：使用 GG 引擎的实体系统创建游戏对象
- **UI 系统**：使用 GG 引擎的 UI 系统显示对话和战斗界面
- **音频系统**：使用 GG 引擎的音频系统播放背景音乐

## 注意事项

- 本示例使用占位资源，实际开发中需要替换为真实资源
- 本示例只实现了基础功能，实际游戏可能需要更多功能扩展