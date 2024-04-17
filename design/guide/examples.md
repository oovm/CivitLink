# 示例设计

GG 引擎提供了一系列示例，展示如何使用引擎的各个功能和特性。每个示例都是一个独立的 Cargo 二进制或库项目，演示一个具体的引擎定制与使用场景。

## 1. `define-my-rpg` – 如何定义一个游戏引擎

**目的**：展示通过**引擎清单**（Engine Manifest）和**GOM Schema** 定义来声明一个领域专用引擎（以 2D RPG 为例）。

**演示内容**：

1. 编写一个 `MyRpg.toml` 引擎清单文件，指定：
   - 名称、版本。
   - 包含的插件：`gg-plugin-tilemap`、`gg-plugin-ui`、`gg-plugin-dialogue`。
   - GOM 组件定义（如 `Health`、`Inventory`、`Skill`）。
   - 目标平台：`desktop`、`web`。
2. 使用 `gg-factory` 命令行工具（由 `examples/define-my-rpg` 调用）读取清单，生成：
   - 编辑器专用入口 `myrpg-editor`（二进制）。
   - 运行时引导库 `myrpg-runtime`。
3. 示例代码展示如何**编程方式**调用工厂 API（而非 CLI），以在更复杂场景下定制生成逻辑。

**关键文件**：

```
examples/define-my-rpg/
├── Cargo.toml                 # 依赖 gg-factory 等
├── MyRpg.toml                 # 引擎清单
├── schemas/                   # 自定义 GOM Schema 定义
│   └── components.gom
└── src/main.rs                # 调用 gg_factory::build(manifest_path)
```

## 2. `generate-editor-and-game` – 生成编辑器与可运行游戏

**目的**：在上一步基础上，实际生成出一个可启动的**编辑器**和一个可运行的**游戏二进制**。

**演示内容**：

1. 基于上一示例的 RPG 引擎定义，使用工厂生成完整的编辑器项目和游戏项目。
2. 生成的编辑器将包含：
   - 瓦片地图编辑视图（由 `gg-plugin-tilemap` 提供）。
   - 属性检查器显示 NPC 对话树、物品属性。
   - 一个"运行游戏"按钮，启动附着式运行时进行即时预览。
3. 生成的游戏运行时将包含：
   - 基于 ECS 的游戏循环。
   - 脚本 VM（加载并执行地图上的 Lua 触发器）。
   - 资源打包与加载逻辑。
4. 示例通过一个简单的"村庄地图"展示编辑器->运行时的完整工作流。

**设计要点**：

- 该示例实际上运行了一个**代码生成步骤**，输出的编辑器项目位于 `target/generated/myrpg-editor`，但为了演示便利，可以在示例中直接启动生成的编辑器（通过 `cargo run` 间接调用）。

## 3. `debugging-game-engine` – 调试引擎与游戏

**目的**：展示 gg 引擎提供的调试基础设施，包括**引擎代码调试**和**游戏逻辑源码级调试**。

**演示内容**：

1. **引擎调试**：
   - 使用 Rust 的 `tracing` 和 `log` 基础设施，展示如何通过环境变量开启详细日志。
   - 展示如何在编辑器壳程序中**附加调试器**（利用 Rust 的 `rust-gdb` / `rust-lldb` 或 IDE）。
2. **游戏逻辑调试**：
   - 在编辑器中打开一个包含 Lua 脚本的地图事件。
   - 通过**调试线缆**（基于 JSON-RPC over TCP）将编辑器与运行时连接。
   - 演示在 Lua 脚本中**设置断点**、单步执行、查看变量（利用 `gg-debug-adapter` 模块）。
3. **HMR 状态迁移调试**：展示当脚本热替换时，如何观察运行时打印的状态迁移日志。

**关键依赖**：

- `gg-editor-shell` 内置的调试服务器。
- 示例中会包含一个**预设的断点配置**，用户打开编辑器即可复现。

## 4. `make-a-plugin` – 制作一个引擎插件

**目的**：教会用户如何将自定义功能封装为 **gg 引擎插件**，以便在引擎清单中引用。

**演示内容**：制作一个简单的 **"计分板"插件**。

1. 插件结构：
   - `Cargo.toml` 声明对 `gg-core`、`gg-ecs` 的依赖。
   - 实现 `Plugin` trait，包含：
     - `register_editor_views(&mut EditorShell)` – 在编辑器中添加一个计分板配置面板。
     - `register_compiler_transforms(&mut CompilerGraph)` – 添加将计分板配置编译为资产的处理步骤。
     - `register_runtime_systems(&mut Scheduler)` – 在游戏运行时添加一个更新计分板的 ECS 系统。
2. 演示如何将编译出的动态库（`libscoreboard_plugin.so` / `.dylib` / `.dll`）放置在引擎插件目录，并在引擎清单中通过 `plugin = "scoreboard"` 引用。
3. 最终在编辑器中可以看到新面板，在游戏运行时可看到计分板效果。

**文件结构**：

```
examples/make-a-plugin/
├── scoreboard-plugin/
│   ├── Cargo.toml          # [lib] crate-type = ["cdylib"]
│   └── src/lib.rs          # 实现 Plugin trait
├── MyGameWithPlugin.toml   # 引擎清单，包含此插件
└── README.md               # 说明如何编译和加载插件
```

## 5. `write-a-dlc` – 制作 DLC / Mod

**目的**：演示最终用户（玩家或 Mod 作者）如何为基于 gg 引擎的游戏制作 DLC 或 Mod。

**演示内容**：

1. **基础游戏**：一个简单的视觉小说章节（使用 `gg-plugin-dialogue`），打包为 `base.game` 资产包和 `base.bin` 可执行文件。
2. **DLC 制作流程**：
   - 提供一个命令行工具 `gg-mod-pack`，输入一个包含覆盖脚本与资产的文件夹，输出一个 `.dlc` 包（实质为 zip + 元数据）。
   - Mod 包含：
     - `override/` 目录下替换角色立绘的图片。
     - `scripts/` 目录下**增量脚本**（例如新增一个可选的额外对话分支）。
3. **游戏加载 DLC**：
   - 游戏启动时扫描 `mods/` 文件夹，按优先级挂载 DLC。
   - 运行时脚本 VM 会优先从 DLC 中查找脚本块，实现逻辑覆盖。
   - 演示安全沙盒：DLC 脚本无法访问文件系统，只能调用游戏暴露的 API。

**示例重点**：清晰展示从"制作 DLC"到"游戏中看到效果"的完整链条，并强调**无需安装 Rust 工具链**（只需用现成的 gg 游戏与打包工具）。

## 6. `hot-reload-in-action` – 热更新演示

**目的**：最直观地展示 gg 引擎的**HMR（热模块替换）** 能力，这是元引擎的重要卖点。

**演示内容**：

1. 启动一个包含实时预览的游戏编辑器（如示例 3.2 生成的 RPG 编辑器）。
2. 同时运行一个游戏窗口（附着式运行时）。
3. **资源热更新**：
   - 在外部图像编辑器中修改一张角色行走图并保存。
   - 编辑器检测到文件变化，自动调用纹理压缩转换器，并将新纹理推送到运行时，游戏窗口中的角色立绘**立即变化**。
4. **脚本热更新**：
   - 打开一个 NPC 的对话脚本（Lua），修改某句台词。
   - 保存后，编辑器将修改后的字节码推送到运行时。
   - 运行时执行状态迁移，再次与 NPC 对话时将显示**新台词**，而其他游戏状态（玩家位置、任务进度）保持不变。
5. 演示 HMR 失败时的**优雅降级**：当新脚本有语法错误时，编辑器会提示错误，而运行时仍使用旧版本继续运行。

**技术支撑**：该示例大量依赖 `gg-editor-shell` 中的文件监视器、`gg-compiler-core` 的增量构建图，以及 `gg-runtime-core` 的状态迁移钩子。