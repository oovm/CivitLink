# GG Engines & Plugins 开发计划

## 项目概览

GG Engines 是 GG 游戏引擎的具体游戏引擎实现，采用元引擎架构，通过插件组合生成特定类型的游戏引擎。Plugins 是官方提供的领域专用插件模块，为引擎提供可复用的功能组件。

### 核心设计原则

1. **元引擎架构**：引擎是插件的静态组合，编译时确定
2. **插件化设计**：功能模块独立，通过 ECS 世界通信
3. **Schema 内聚**：类型定义与引擎代码同模块，避免过度分离
4. **ECS 驱动**：所有游戏逻辑通过 ECS 系统执行

### 现有引擎

| 引擎 | 路径 | 职责 |
|------|------|------|
| gg-engine-stg | `projects/engines/gg-engine-stg` | 射击游戏引擎（优先级第一） |
| gg-galgame | `projects/engines/galgame` | 视觉小说引擎（含 Schema） |
| gg-engine-platformer | `projects/engines/gg-engine-platformer` | 平台跳跃引擎 |

### 现有插件

| 插件 | 路径 | 职责 |
|------|------|------|
| gg-plugin-dialogue | `projects/engines/gg-plugin-dialogue` | 对话系统 |
| gg-plugin-portrait | `projects/engines/gg-plugin-portrait` | 立绘系统 |
| gg-plugin-save | `projects/engines/gg-plugin-save` | 存档系统 |
| gg-plugin-scene-transition | `projects/engines/gg-plugin-scene-transition` | 场景转场 |
| gg-plugin-spine | `projects/engines/gg-plugin-spine` | Spine 动画 |
| gg-plugin-tilemap | `projects/engines/gg-plugin-tilemap` | 瓦片地图 |

### 示例项目（游戏）

| 示例 | 路径 | 关联引擎 | 状态 |
|------|------|---------|------|
| my-first-stg | `examples/my-first-stg` | STG | 活跃（优先级第一） |
| my-first-galgame | `examples/my-first-galgame` | Galgame | 活跃 |
| my-first-platformer | `examples/my-first-platformer` | Platformer | 活跃 |
| my-first-rpg | `examples/my-first-rpg` | RPG（规划中） | 规划 |



---

# STG 引擎组

- **负责模块**：gg-engine-stg
- **管辖示例**：my-first-stg
- **优先级**：**第一优先** - 当前重点开发
- **大致进度**：65%
- **本月工作重点**：
  - 实现关卡系统
  - 添加音效和特效
  - 优化渲染性能
  - 完善敌人 AI 行为
  - 实现游戏状态管理
- **长期目标**：
  - 完整的射击游戏引擎
  - 支持多种弹幕模式
  - 支持 Boss 战设计
  - 支持关卡编辑器
- **相关文件**：
  - gg-engine-stg: `projects/engines/gg-engine-stg/src/lib.rs`
  - my-first-stg: `examples/my-first-stg/`

---

# Galgame 引擎组

- **负责模块**：gg-galgame（含 Schema）
- **管辖示例**：my-first-galgame
- **大致进度**：75%
- **当前任务**：**编译器完善** - 完善对话脚本编译器和增量编译
- **本月工作重点**：
  - 完善对话脚本编译器
  - 实现增量编译支持
  - 完善打字机效果
  - 优化 UI 布局系统
  - 实现变量和条件分支
- **长期目标**：
  - 完整的视觉小说引擎
  - 支持分支剧情和变量系统
  - 集成编辑器支持
  - 支持多语言本地化
- **相关文件**：
  - gg-galgame: `projects/engines/galgame/src/lib.rs`
  - my-first-galgame: `examples/my-first-galgame/`

---

# Platformer 引擎组

- **负责模块**：gg-engine-platformer
- **管辖示例**：my-first-platformer
- **大致进度**：40%
- **本月工作重点**：
  - 完善物理系统
  - 实现碰撞检测优化
  - 添加更多平台类型
  - 实现敌人 AI
  - 完善输入系统
- **长期目标**：
  - 完整的平台跳跃引擎
  - 支持关卡编辑器
  - 支持多种敌人类型
  - 支持道具系统
- **相关文件**：
  - gg-engine-platformer: `projects/engines/gg-engine-platformer/src/lib.rs`
  - my-first-platformer: `examples/my-first-platformer/`

---

# 对话系统插件组

- **负责模块**：gg-plugin-dialogue
- **大致进度**：75%
- **本月工作重点**：
  - 完善对话节点执行器
  - 实现表达式解析器
  - 完善命令系统
  - 优化历史记录管理
  - 实现对话跳转和条件分支
- **长期目标**：
  - 完整的对话系统
  - 支持变量和条件表达式
  - 支持对话预览编辑器
  - 支持多语言文本
- **相关文件**：
  - gg-plugin-dialogue: `projects/engines/gg-plugin-dialogue/src/lib.rs`

---

# 立绘系统插件组

- **负责模块**：gg-plugin-portrait
- **大致进度**：65%
- **本月工作重点**：
  - 完善立绘布局系统
  - 实现立绘动画效果
  - 优化 Z 排序算法
  - 支持立绘特效
  - 实现立绘切换过渡
- **长期目标**：
  - 完整的立绘系统
  - 支持多种立绘格式
  - 支持立绘编辑器
  - 支持动态表情
- **相关文件**：
  - gg-plugin-portrait: `projects/engines/gg-plugin-portrait/src/lib.rs`

---

# 存档系统插件组

- **负责模块**：gg-plugin-save
- **大致进度**：55%
- **本月工作重点**：
  - 完善存档管理器
  - 实现自动存档功能
  - 完善存档 UI
  - 支持存档加密
  - 实现云存档接口
- **长期目标**：
  - 完整的存档系统
  - 支持多存档槽位
  - 支持云存档
  - 支持存档迁移
- **相关文件**：
  - gg-plugin-save: `projects/engines/gg-plugin-save/src/lib.rs`

---

# 场景转场插件组

- **负责模块**：gg-plugin-scene-transition
- **大致进度**：50%
- **本月工作重点**：
  - 完善转场效果库
  - 实现氛围滤镜
  - 优化转场性能
  - 支持自定义转场
  - 实现场景管理器
- **长期目标**：
  - 丰富的转场效果
  - 支持自定义着色器转场
  - 支持转场预览
  - 支持转场链
- **相关文件**：
  - gg-plugin-scene-transition: `projects/engines/gg-plugin-scene-transition/src/lib.rs`

---

# Spine 动画插件组

- **负责模块**：gg-plugin-spine
- **大致进度**：45%
- **本月工作重点**：
  - 完善 Spine 数据解析
  - 实现骨骼动画播放
  - 支持皮肤切换
  - 实现事件系统
  - 优化渲染性能
- **长期目标**：
  - 完整的 Spine 支持
  - 支持 Spine 4.x 格式
  - 支持动画混合
  - 支持 IK 约束
- **相关文件**：
  - gg-plugin-spine: `projects/engines/gg-plugin-spine/src/lib.rs`

---

# 瓦片地图插件组

- **负责模块**：gg-plugin-tilemap
- **大致进度**：50%
- **本月工作重点**：
  - 完善瓦片渲染系统
  - 实现地图加载器
  - 支持瓦片动画
  - 实现相机跟随
  - 完善碰撞检测
- **长期目标**：
  - 完整的瓦片地图系统
  - 支持 Tiled 地图格式
  - 支持地图编辑器
  - 支持多图层
- **相关文件**：
  - gg-plugin-tilemap: `projects/engines/gg-plugin-tilemap/src/lib.rs`

---

## 模块完成度详细评估

### gg-galgame

- **完成度**：75%
- **已完成功能**：
  - 引擎主循环
  - 插件系统集成
  - 对话 UI 系统
  - 资源加载
  - 渲染管线
  - Schema 组件定义
  - Schema 模块合并
  - 编译器框架（解析器、IR、代码生成）
- **未完成功能**：
  - 增量编译
  - 编辑器集成
  - 多语言支持
  - Schema 类型验证

### gg-platformer

- **完成度**：40%
- **已完成功能**：
  - 引擎框架
  - 基础物理
  - 玩家控制
  - 碰撞检测
- **未完成功能**：
  - 完整物理系统
  - 敌人 AI
  - 关卡系统
  - 道具系统

### gg-engine-stg（第一优先）

- **完成度**：65%
- **状态**：活跃开发
- **已完成功能**：
  - 引擎框架
  - 玩家移动
  - 基础射击
  - 弹幕系统（6种模式）
  - Boss AI（阶段切换、移动模式）
  - 道具系统（5种道具类型）
  - 碰撞检测优化（空间哈希网格）
  - 敌人 AI（4种行为模式）
- **未完成功能**：
  - 关卡系统
  - 音效和特效
  - 游戏状态管理
  - 渲染性能优化

### gg-plugin-dialogue

- **完成度**：70%
- **已完成功能**：
  - 对话节点执行
  - 打字机效果
  - 选项系统
  - 历史记录
- **未完成功能**：
  - 表达式解析
  - 条件分支
  - 变量系统

### gg-plugin-portrait

- **完成度**：60%
- **已完成功能**：
  - 立绘渲染
  - 基础布局
  - Z 排序
- **未完成功能**：
  - 立绘动画
  - 特效系统
  - 表情切换

### gg-plugin-save

- **完成度**：55%
- **已完成功能**：
  - 存档保存/加载
  - 存档管理器
  - 基础 UI
- **未完成功能**：
  - 自动存档
  - 存档加密
  - 云存档

### gg-plugin-scene-transition

- **完成度**：50%
- **已完成功能**：
  - 基础转场效果
  - 场景管理器
- **未完成功能**：
  - 更多转场效果
  - 氛围滤镜
  - 自定义转场

### gg-plugin-spine

- **完成度**：45%
- **已完成功能**：
  - Spine 数据解析
  - 骨骼动画播放
  - 组件定义
- **未完成功能**：
  - 皮肤系统
  - 事件系统
  - 动画混合

### gg-plugin-tilemap

- **完成度**：50%
- **已完成功能**：
  - 瓦片渲染
  - 地图加载
  - 相机系统
- **未完成功能**：
  - 瓦片动画
  - 碰撞优化
  - Tiled 格式支持

---

## 架构图

```mermaid
graph TB
    subgraph Engines[游戏引擎]
        STG[gg-engine-stg<br/>射击游戏（第一优先）]
        Galgame[Galgame 引擎<br/>视觉小说]
        Platformer[gg-engine-platformer<br/>平台跳跃]
    end

    subgraph Schema[Schema 模块]
        GalgameSchema[gg-galgame-schema<br/>类型定义]
    end

    subgraph Plugins[插件系统]
        Dialogue[对话系统<br/>gg-plugin-dialogue]
        Portrait[立绘系统<br/>gg-plugin-portrait]
        Save[存档系统<br/>gg-plugin-save]
        Transition[场景转场<br/>gg-plugin-scene-transition]
        Spine[Spine 动画<br/>gg-plugin-spine]
        Tilemap[瓦片地图<br/>gg-plugin-tilemap]
    end

    subgraph Core[核心依赖]
        ECS[ECS 核心<br/>gg-ecs]
        Render[渲染系统<br/>gg-render]
        UI[UI 系统<br/>gg-ui]
        Asset[资源系统<br/>gg-asset]
    end

    Galgame --> GalgameSchema
    Galgame --> Dialogue
    Galgame --> Portrait
    Galgame --> Save
    Galgame --> Transition

    Platformer --> Tilemap
    Platformer --> Spine

    Dialogue --> ECS
    Portrait --> ECS
    Portrait --> Render
    Save --> ECS
    Transition --> ECS
    Transition --> Render
    Spine --> ECS
    Spine --> Render
    Tilemap --> ECS
    Tilemap --> Render

    Engines --> Core
    Plugins --> Core
```

---

## 插件依赖关系

```mermaid
graph LR
    subgraph STGPlugins[gg-engine-stg 插件集]
        BulletPattern[弹幕模式]
        BossAI[Boss AI]
        PowerUp[道具系统]
    end

    subgraph GalgamePlugins[Galgame 插件集]
        Dialogue[对话系统]
        Portrait[立绘系统]
        Save[存档系统]
        Transition[场景转场]
    end

    subgraph PlatformerPlugins[gg-engine-platformer 插件集]
        Tilemap[瓦片地图]
        SpineAnim[Spine 动画]
    end

    Dialogue --> Portrait
    Dialogue --> Save
    Transition --> Portrait
    BulletPattern --> BossAI
```

---

## 开发里程碑

### Phase 0: 插件模块迁移 - ⏳ 进行中

- [ ] 迁移 gg-plugin-dialogue 到 plugins 目录
- [ ] 迁移 gg-plugin-portrait 到 plugins 目录
- [ ] 迁移 gg-plugin-save 到 plugins 目录
- [ ] 迁移 gg-plugin-scene-transition 到 plugins 目录
- [ ] 迁移 gg-plugin-spine 到 plugins 目录
- [ ] 迁移 gg-plugin-tilemap 到 plugins 目录
- [ ] 更新工作区 Cargo.toml 依赖路径
- [ ] 验证迁移后编译通过

### Phase 1: gg-engine-stg 引擎核心 (Week 1-4) - 第一优先

- [x] 完善弹幕系统
- [x] 实现 Boss AI
- [x] 完善敌人弹幕模式
- [x] 实现道具系统
- [x] 优化碰撞检测性能

### Phase 2: gg-engine-stg 引擎完善 (Week 5-8)

- [x] 实现多种弹幕模式
- [x] 完善 Boss 战设计
- [ ] 实现关卡系统
- [ ] 添加音效和特效
- [ ] 优化渲染性能

### Phase 3: Galgame 核心 (Week 9-12)

- [ ] 完善对话脚本编译器
- [ ] 实现增量编译
- [ ] 完善打字机效果
- [ ] 实现变量和条件分支
- [ ] 完善立绘布局系统

### Phase 4: Galgame 完善 (Week 13-16)

- [ ] 实现存档系统加密
- [ ] 完善转场效果库
- [ ] 实现立绘动画
- [ ] 完善对话 UI
- [ ] 集成编辑器支持

### Phase 5: gg-engine-platformer 核心 (Week 17-20)

- [ ] 完善物理系统
- [ ] 实现敌人 AI
- [ ] 完善瓦片地图
- [ ] 实现关卡系统
- [ ] 完善碰撞检测

### Phase 6: 插件完善 (Week 21-24)

- [ ] 完善 Spine 动画插件
- [ ] 实现云存档接口
- [ ] 支持自定义转场
- [ ] 支持 Tiled 地图格式
- [ ] 完善插件文档

---

## 文档计划

### 技术文档

- [ ] 引擎架构文档
- [ ] 插件开发指南
- [ ] API 参考文档
- [ ] 脚本语言规范

### 用户文档

- [ ] 引擎使用指南
- [ ] 插件配置说明
- [ ] 示例项目文档

---

## 发布计划

### 版本规划

- **v0.1.0**：gg-engine-stg 引擎基础功能
- **v0.2.0**：gg-engine-stg 引擎完善
- **v0.3.0**：Galgame 引擎基础功能
- **v0.4.0**：Galgame 引擎完善
- **v0.5.0**：gg-engine-platformer 引擎基础功能
- **v0.6.0**：插件系统完善
- **v1.0.0**：稳定版本

### 发布标准

- 所有测试通过
- 代码覆盖率达到 70% 以上
- API 文档完善
- 性能达到 60fps
