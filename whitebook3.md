## 技术栈与Monorepo架构设计

基于之前确定的元游戏引擎架构和多平台支持需求，现在需要选择合适的技术栈并设计清晰的代码组织方式。我将先分析主流引擎的功能模块，然后映射到具体技术选型，最后给出两层monorepo结构。

### 1. 主流引擎功能综合分析

#### 1.1 核心功能模块对比

| 功能领域 | Unity | Unreal Engine | Cocos Creator | Godot |
|---------|-------|---------------|---------------|-------|
| **游戏对象模型** | GameObject + Component (ECS可选) | Actor + Component (UObject体系) | Node + Component | Node + Scene (树形结构) |
| **脚本系统** | C# (Mono/IL2CPP) | C++ + Blueprint (可视化) | TypeScript/JavaScript | GDScript/C#/VisualScript/GDExtension |
| **渲染架构** | SRP可定制管线, Shader Graph | PBR, 全局光照, 材质编辑器 | 2D/3D混合, 自定义材质 | 2D独立管线, Vulkan/OpenGL |
| **物理引擎** | PhysX/Bullet (内置) | PhysX/Chaos | 内置2D/3D物理 | GodotPhysics/Bullet |
| **音频系统** | Audio Mixer | 音频组件 | 音频引擎 | AudioServer |
| **UI系统** | UGUI/UI Toolkit | UMG/ Slate UI | Canvas + 组件 | Control节点体系 |
| **资源管理** | Asset Pipeline, Addressables | 资源引用系统 | 动态加载/预加载 | Resource系统 |
| **编辑器扩展** | Editor Scripting, Custom Inspectors | 编辑器插件, Slate | 插件机制 | 编辑器插件(GDScript) |
| **跨平台支持** | Windows/macOS/iOS/Android/Web等 | 主流平台 + 主机 | 全平台 + 小游戏 | 全平台 + WebAssembly |
| **热更新方案** | ILRuntime/HybridCLR | 补丁包 | 资源热更 | 无官方方案 |

#### 1.2 核心架构设计哲学

- **Unity**：组件化设计，强调可视化编辑与代码的结合，ECS/DOTS作为高性能补充
- **Unreal**：面向对象+数据驱动，UObject作为统一基类，强类型系统
- **Cocos**：轻量级节点树，TypeScript优先，适合小游戏和H5
- **Godot**：场景树+信号系统，完全开源，多语言支持

### 2. 技术栈选择

基于上述分析，结合元游戏引擎的架构需求（ECS核心、虚拟机支持、多平台分发），选择以下技术栈：

#### 2.1 核心层 (Core)

| 模块 | 技术选型 | 选型理由 |
|------|---------|---------|
| **ECS框架** | `bevy_ecs` (独立库) | 成熟稳定的Rust ECS实现，无运行时依赖，支持并行调度 |
| **异步运行时** | `tokio` 或 `async-std` | 资源加载、网络请求需要异步支持 |
| **序列化** | `serde` + `ron`/`json`/`yaml` | 配置文件和资源序列化标准 |
| **日志** | `tracing` / `log` | 结构化日志，支持多平台输出 |
| **错误处理** | `anyhow` + `thiserror` | 灵活的错误处理机制 |
| **反射系统** | `bevy_reflect` 或 自定义derive | 编辑器属性编辑、动态组件创建需要 |

#### 2.2 平台抽象层 (Platform Abstraction)

| 模块 | 技术选型 | 选型理由 |
|------|---------|---------|
| **窗口/输入** | `winit` | 跨平台窗口创建和输入事件，支持所有桌面平台 |
| **图形渲染** | `wgpu` | 基于WebGPU规范，支持Vulkan/Metal/DX12/WebGL2，真正的跨平台 |
| **音频** | `kira` / `rodio` | 轻量级音频库，支持常见格式 |
| **文件系统** | 自定义 `AssetIo` trait | 适配各平台存储差异（沙盒、Web、小游戏） |
| **时间管理** | `instant` / `web-time` | 跨平台时间获取 |
| **线程/异步** | `wasm-bindgen-futures` (Web) | WASM环境异步支持 |

#### 2.3 虚拟机层 (VM)

| 模块 | 技术选型 | 选型理由 |
|------|---------|---------|
| **Wasm运行时** | `wasmtime` (桌面) + `wasmi` (轻量) | 桌面用JIT提升性能，嵌入式用解释器 |
| **WASI支持** | 自定义适配 | 提供安全的系统接口访问 |
| **脚本绑定** | 通过wasm导入函数暴露ECS API | 沙箱隔离，安全可控 |

#### 2.4 工具链层 (Tooling)

| 模块 | 技术选型 | 选型理由 |
|------|---------|---------|
| **编辑器UI** | `egui` + `eframe` | 纯Rust实现的即时模式GUI，易于嵌入 |
| **场景编辑** | 基于ECS的编辑器系统 | 与运行时共享数据结构 |
| **资源处理** | `image` + `hound` + `mp3` 等 | 常见资源格式解码 |
| **构建工具** | `cargo` + 自定义构建脚本 | 多目标交叉编译支持 |

#### 2.5 平台特定适配 (Platform Specific)

| 平台 | 技术选型 |
|------|---------|
| **Windows/macOS/Linux** | 直接使用上述核心栈 |
| **iOS/Android** | `winit` + `wgpu` + 移动适配层 |
| **WebAssembly (H5)** | `wasm-bindgen` + `web-sys` |
| **微信小游戏** | 自定义JS胶水层 + WASM核心 |

### 3. Monorepo组织结构（两层设计）

采用`gwg-engine/crates/xxxx/yyy`的两层结构，上层按功能领域划分，下层按具体模块划分，避免目录过深。

```
gwg-engine/
├── Cargo.toml                    # 工作空间根配置
├── README.md
├── docs/                          # 架构文档、API文档
├── tools/                         # 构建工具、脚手架
│   └── meta-build/                # 引擎构建工具
│
├── crates/                         # 第一层：功能领域
│   ├── core/                       # 核心框架层
│   │   ├── ecs/                    # ECS核心封装 (基于bevy_ecs)
│   │   ├── asset/                  # 资源管理系统
│   │   ├── schedule/               # 系统调度器扩展
│   │   ├── world/                  # 世界管理
│   │   └── reflection/             # 反射系统
│   │
│   ├── platform/                    # 平台抽象层
│   │   ├── window/                  # 窗口管理 (winit封装)
│   │   ├── input/                   # 输入抽象
│   │   ├── graphics/                # 图形抽象 (wgpu封装)
│   │   ├── audio/                   # 音频抽象
│   │   ├── filesystem/              # 文件系统抽象
│   │   └── time/                    # 时间抽象
│   │
│   ├── runtime/                      # 运行时层
│   │   ├── app/                      # 应用生命周期
│   │   ├── scene/                    # 场景管理
│   │   ├── prefab/                   # 预制体系统
│   │   └── serialization/            # 序列化
│   │
│   ├── vm/                            # 虚拟机层
│   │   ├── core/                      # 虚拟机核心接口
│   │   ├── wasmtime/                  # wasmtime后端
│   │   ├── wasmi/                      # 轻量级解释器后端
│   │   ├── api/                        # 暴露给脚本的Rust API
│   │   └── bindings/                   # 语言绑定生成
│   │
│   ├── engine/                         # 引擎插件框架
│   │   ├── plugin/                      # 插件trait定义
│   │   ├── registry/                    # 插件注册表
│   │   ├── builder/                      # 引擎构建器
│   │   └── manifest/                     # 引擎清单处理
│   │
│   ├── editor/                          # 编辑器框架
│   │   ├── ui/                           # 编辑器UI组件 (egui)
│   │   ├── inspector/                     # 属性编辑器
│   │   ├── scene-view/                     # 场景视图
│   │   ├── asset-browser/                   # 资源浏览器
│   │   └── plugin/                          # 编辑器插件系统
│   │
│   ├── modules/                          # 内置功能模块
│   │   ├── rendering/                      # 渲染模块
│   │   │   ├── core/                         # 渲染核心
│   │   │   ├── 2d/                            # 2D渲染
│   │   │   ├── 3d/                            # 3D渲染
│   │   │   ├── sprite/                         # 精灵系统
│   │   │   ├── text/                           # 文本渲染
│   │   │   └── camera/                         # 相机系统
│   │   │
│   │   ├── physics/                        # 物理模块
│   │   │   ├── 2d/                            # 2D物理
│   │   │   ├── 3d/                            # 3D物理
│   │   │   ├── collision/                      # 碰撞检测
│   │   │   └── rapier/                         # Rapier物理后端
│   │   │
│   │   ├── animation/                       # 动画模块
│   │   │   ├── core/                          # 动画核心
│   │   │   ├── sprite/                        # 精灵动画
│   │   │   ├── transform/                      # 变换动画
│   │   │   └── state-machine/                  # 状态机
│   │   │
│   │   ├── audio/                           # 音频模块
│   │   │   ├── player/                        # 播放器
│   │   │   ├── mixer/                         # 混音器
│   │   │   └── spatial/                       # 空间音频
│   │   │
│   │   ├── ui/                              # UI模块
│   │   │   ├── core/                          # UI核心
│   │   │   ├── widgets/                       # 基础控件
│   │   │   ├── layout/                        # 布局系统
│   │   │   └── interaction/                    # 交互处理
│   │   │
│   │   ├── input/                           # 输入模块
│   │   │   ├── keyboard/                      # 键盘
│   │   │   ├── mouse/                         # 鼠标
│   │   │   ├── touch/                         # 触摸
│   │   │   ├── gamepad/                       # 手柄
│   │   │   └── mapping/                       # 输入映射
│   │   │
│   │   └── network/                         # 网络模块（可选）
│   │       ├── core/                          # 网络核心
│   │       ├── client/                        # 客户端
│   │       ├── server/                        # 服务器
│   │       └── sync/                          # 同步机制
│   │
│   ├── platforms/                          # 平台特定实现
│   │   ├── desktop/                          # 桌面通用
│   │   ├── windows/                          # Windows特定
│   │   ├── macos/                            # macOS特定
│   │   ├── linux/                            # Linux特定
│   │   ├── ios/                              # iOS特定
│   │   ├── android/                          # Android特定
│   │   ├── web/                              # WebAssembly通用
│   │   │   ├── h5/                             # 普通H5
│   │   │   └── wechat/                         # 微信小游戏适配
│   │   └── common/                           # 平台公共代码
│   │
│   └── examples/                            # 示例引擎
│       ├── galgame/                           # Galgame引擎示例
│       ├── stg/                                # 飞机大战引擎示例
│       ├── platformer/                         # 横版闯关示例
│       └── minimal/                            # 最小引擎示例
│
└── bins/                                    # 最终生成的可执行文件（构建输出）
    ├── galgame-editor.exe
    ├── galgame-runner.exe
    ├── stg-editor.exe
    └── ...
```

### 4. 模块依赖关系

```
core
├── ecs
├── asset
└── reflection
    ↓
platform (各抽象层)
    ↓
runtime (整合平台与核心)
    ↓
modules (功能模块，相互可组合)
    ├── rendering
    ├── physics
    ├── audio
    ├── ui
    └── ...
    ↓
engine (插件框架，整合模块)
    ↓
editor (编辑器框架，依赖engine和modules)
```

### 5. 构建与分发流程

#### 5.1 引擎插件开发（资深玩家）
```rust
// examples/galgame/src/main.rs
use gwg_engine::prelude::*;

fn main() {
    EngineBuilder::new()
        .add_plugin(rendering::RenderPlugin::new_2d())
        .add_plugin(audio::AudioPlugin::default())
        .add_plugin(ui::UIPlugin::default())
        .add_plugin(galgame::GalgamePlugin::new())
        .with_editor(true)
        .build()
        .run();
}
```

#### 5.2 多平台构建
```bash
# 构建桌面版Galgame引擎
cargo build --package galgame --target x86_64-pc-windows-msvc --release

# 构建WebAssembly版
cargo build --package galgame --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/galgame.wasm --out-dir ./web

# 构建微信小游戏适配版
cargo build --package galgame --features "platform/wechat" --target wasm32-unknown-unknown
```

#### 5.3 游戏项目结构（普通玩家）
```
my_galgame/
├── game.toml                    # 游戏配置（引擎选择、参数）
├── assets/                       # 资源文件
│   ├── images/
│   │   ├── characters/
│   │   └── backgrounds/
│   ├── audio/
│   │   ├── bgm/
│   │   └── se/
│   └── fonts/
├── scripts/                      # 剧本文件
│   ├── chapter1.yarn
│   └── chapter2.yarn
├── dlc/                          # DLC目录
│   └── extra-chapter/
│       ├── assets/
│       └── scripts/
└── mods/                         # Mod脚本
    └── custom-system.gwg
```

### 6. 关键技术决策说明

1. **ECS作为统一核心**：借鉴Unity的DOTS架构，所有游戏逻辑最终都通过ECS系统执行，确保性能和数据驱动。

2. **WASM作为脚本虚拟机**：类似于Godot的GDExtension，但更安全、跨平台。脚本在沙箱中运行，不能直接访问内存。

3. **插件化架构**：参考Unreal的模块系统，每个功能模块独立编译，引擎构建时静态链接，保证运行时性能。

4. **平台抽象层**：借鉴Godot的跨平台设计，通过trait抽象各平台差异，上层代码无需关心底层实现。

5. **两层monorepo结构**：避免目录过深，同时保持模块清晰。`crates/`下按功能领域划分，每个领域内部再按具体模块细分。

### 7. 总结

该技术栈方案基于主流引擎的功能分析，结合Rust生态的优势，构建了一个**高性能、跨平台、可扩展**的元游戏引擎框架。两层monorepo结构既保证了模块清晰，又避免了目录过深。最终实现：
- **资深玩家**：基于`engine`和`modules`开发具体引擎插件
- **高级玩家**：通过WASM编写Mod脚本扩展游戏逻辑
- **普通玩家**：通过数据配置和剧本创作游戏

所有层通过ECS世界紧密耦合，确保从引擎开发到游戏创作的完整链条。