# 多平台发布

多平台发布，涉及将游戏高效、安全地分发到不同平台，并对接各平台特有的生态。gg 引擎采用**三层发布流水线**架构，将平台无关的构建产物与平台特定的适配层清晰分离。

## 统一发布流水线：从源码到全平台

整个发布流程的核心是一个统一的构建图（Build Graph），它会将游戏工程编译、打包，并针对不同平台调用相应的后端生成最终产物。

```mermaid
graph TD
    A[游戏工程与引擎清单] --> B[gg 元编译器/工厂];
    B --> C{编译图 (Build Graph)};
    C --> D[平台无关资产包];
    C --> E[平台无关逻辑字节码];
    D --> F{平台发布后端};
    E --> F;
    F --> G[桌面/移动原生平台];
    F --> H[Web/WASI 平台];
    F --> I[小程序平台];
    subgraph G [桌面/移动原生]
        G1[Windows: .exe]
        G2[macOS: .app]
        G3[Linux: .bin]
        G4[Android: .apk]
        G5[iOS: .ipa]
    end
    subgraph H [Web/WASI]
        H1[Web: .wasm + .js]
        H2[WASI: .wasm]
    end
    subgraph I [小程序]
        I1[微信: .wxapkg]
        I2[支付宝: .apk]
        I3[其他...]
    end
```

这个流程将游戏逻辑（如`Player`、`NPC`）与平台细节（如文件系统、支付接口）解耦，从而确保核心玩法在不同平台上表现一致，且只需维护一套核心代码。

## 平台作为插件：核心抽象与实现

在 gg 引擎中，平台本身被设计为一种特殊的插件，遵循核心层定义的`Platform` trait。

```rust
// gg-core/src/platform.rs
pub trait Platform {
    // 平台标识
    fn id(&self) -> PlatformId;
    fn display_name(&self) -> &str;
    // 构建配置
    fn configure_build(&self, config: &mut BuildConfig);
    // 生成平台特定代码
    fn generate_code(&self, ctx: &GenerateContext) -> Result<PathBuf>;
    // 打包最终产物
    fn package(&self, ctx: &PackageContext) -> Result<Vec<PathBuf>>;
    // 可选：本地运行/部署
    fn run(&self, ctx: &RunContext) -> Result<()> { Ok(()) }
}
```

- **核心接口 (`gg-core`)**：提供基础能力，如文件系统、网络请求、支付、广告等。
- **内置平台实现 (`gg-platforms`)**：官方维护的对主流平台的支持，如`desktop`（Windows/Linux/macOS）、`web`（WebAssembly）等。
- **小程序平台实现 (`gg-platform-miniprogram`)**：为微信、支付宝等小程序环境提供通用适配层，封装平台特定API。

## 插件分发新范式：WASI 与 `wit-component`

为了让插件支持多语言开发并简化分发，gg 引擎插件系统采用**WASI (WebAssembly System Interface)** 和 **WIT (WebAssembly Interface Type)** 标准。

### 1. 简化分发逻辑

传统原生插件（如`.dll`/`.so`）依赖具体指令集与操作系统。通过 `wit-component` 工具，可将任意语言的插件代码编译为**跨平台、沙箱化的`.wasm`组件**，实现"一次编译，处处运行"。

### 2. 统一多语言接口

使用WIT接口定义语言（IDL）精确描述插件与引擎宿主之间的契约。gg 引擎会发布一个官方WIT描述文件（如 `gg-plugin.wit`），第三方开发者只需基于此文件，通过 `wit-bindgen` 等工具生成对应语言的绑定代码（Rust, C/C++, Go等）进行插件开发。引擎在加载插件时，也通过此接口进行安全、高效的类型化调用。

### 3. 工具链集成

`wit-component` 等工具链能够自动处理从`*.wit`文件到Wasm组件的生成。gg引擎的编译器流水线会集成这些工具，自动完成插件的组件化处理。

## 实战案例：为gg引擎新增平台支持

### 案例一：官方未支持？为gg引擎新增支付宝小游戏平台

当官方未提供支持时，开发者可通过实现`Platform` trait来新增平台，并可选择用Rust或WIT两种方式实现。

- **传统方式（Rust原生）**
  1. **创建Cargo项目**：`cargo new gg-platform-alipay --lib`
  2. **实现`Platform` Trait**：在 `src/lib.rs` 中实现上述所有接口，适配支付宝API。
  3. **编译为动态库**：配置为`cdylib`，编译出`.dll`/`.so`/`.dylib`。
  4. **在清单中注册**：在 `Engine.toml` 中配置 `[[platform]]` 项，指向编译好的动态库。
  5. **运行发布**：`gg-cli build --platform alipay`。

- **现代方式（基于WIT/WASI）**
  1. **定义WIT接口**：在 `gg-platform.wit` 中定义平台插件的 `world`，包含 `build`, `package` 等函数。
  2. **生成绑定代码**：使用 `wit-bindgen` 为Rust生成trait，或为其他语言生成接口。
  3. **实现WIT定义的World**：在任何支持WASI的语言中，实现该 `world` 导出的所有函数。
  4. **编译为Wasm组件**：使用对应工具链（如`cargo-component`）编译为`.wasm`组件。
  5. **注册与运行**：gg引擎的工厂工具会加载并执行这些Wasm组件，通过WIT接口调度其功能。

### 案例二：从Google Play到小米应用商城，如何编写插件？

上架国内应用商城（如小米商城），通常需要对接其特定的SDK，如支付、登录等。

1. **创建平台插件项目**：`cargo new gg-platform-xiaomi --lib`
2. **添加SDK依赖**：在 `Cargo.toml` 中添加小米SDK的Rust绑定或C库的FFI绑定。
3. **实现关键接口**：重点实现 `Platform` trait 中的 `package` 和 `initialize` 方法，处理SDK初始化、AndroidManifest.xml的合并等。
4. **处理资产和代码注入**：在 `generate_code` 方法中，生成包含SDK调用的Java/Kotlin代码，并注入到Android项目模板。
5. **编译与发布**：`gg-cli build --platform xiaomi`，gg引擎的构建系统将集成插件逻辑，生成包含小米SDK的APK。

## 总结：gg引擎多平台战略的设计优势

- **核心稳定，边界灵活**：清晰的三层架构确保了核心逻辑的稳定性，同时提供了极大的扩展自由度。
- **真正的多语言与跨平台**：采用WASI/WIT标准，打破了语言壁垒，实现了插件的跨平台、跨语言分发，极大扩展了开发者生态。
- **社区驱动与去中心化**：任何人都可以为新平台开发支持插件并贡献社区，无需官方介入，引擎生命力由整个生态共同维系。