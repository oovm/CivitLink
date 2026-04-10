# CI/CD 流水线与多平台构建指南

本指南基于 gg 元引擎框架架构（whitebook2.md 和 whitebook3.md），提供完整的 CI/CD 流水线配置、多平台构建流程以及分发打包方案。

## 目录

1. [多平台构建流程](#多平台构建流程)
2. [自动化构建配置](#自动化构建配置)
3. [分发打包步骤](#分发打包步骤)
4. [引擎插件构建与发布](#引擎插件构建与发布)
5. [游戏项目打包与分发](#游戏项目打包与分发)

---

## 多平台构建流程

gg 引擎支持 Windows、macOS、Linux、iOS、Android、WebAssembly（H5）以及微信小游戏等主要平台。以下是各平台的构建流程：

### 1.1 桌面平台（Windows / macOS / Linux）

#### Windows
```bash
# 构建目标
cargo build --package galgame --target x86_64-pc-windows-msvc --release
```

#### macOS
```bash
# 构建目标
cargo build --package galgame --target x86_64-apple-darwin --release
# 或 Apple Silicon
cargo build --package galgame --target aarch64-apple-darwin --release
```

#### Linux
```bash
# 构建目标
cargo build --package galgame --target x86_64-unknown-linux-gnu --release
```

### 1.2 移动平台（iOS / Android）

#### iOS
```bash
# 构建 iOS 目标
cargo build --package galgame --target aarch64-apple-ios --release
# 使用 xcodebuild 打包
xcodebuild -scheme galgame -archivePath ./build/galgame.xcarchive archive
xcodebuild -exportArchive -archivePath ./build/galgame.xcarchive -exportPath ./build -exportOptionsPlist exportOptions.plist
```

#### Android
```bash
# 构建 Android 目标
cargo build --package galgame --target aarch64-linux-android --release
# 使用 Gradle 打包
./gradlew assembleRelease
```

### 1.3 WebAssembly 平台（H5 / 微信小游戏）

#### WebAssembly (H5)
```bash
# 构建 WASM
cargo build --package galgame --target wasm32-unknown-unknown --release
# 生成绑定
wasm-bindgen target/wasm32-unknown-unknown/release/galgame.wasm --out-dir ./web --target web
```

#### 微信小游戏
```bash
# 构建微信小游戏适配版
cargo build --package galgame --features "platform/wechat" --target wasm32-unknown-unknown --release
# 使用 wasm-bindgen
wasm-bindgen target/wasm32-unknown-unknown/release/galgame.wasm --out-dir ./wechat-game --target wechat
```

---

## 自动化构建配置

### GitHub Actions 完整配置

以下是适用于 gg 引擎的 GitHub Actions CI/CD 配置示例：

```yaml
name: gg Engine CI/CD

on:
    push:
        branches: [main, master, develop]
        tags:
            - 'v*'
    pull_request:
        branches: [main, master, develop]
    release:
        types: [created]

env:
    CARGO_TERM_COLOR: always
    RUST_BACKTRACE: 1
    RUSTFLAGS: "-D warnings"

jobs:
    # Rust 代码检查
    rust-check:
        name: Rust Code Check
        runs-on: ubuntu-latest
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@stable
              with:
                  components: rustfmt, clippy

            - name: Cache cargo registry
              uses: actions/cache@v4
              with:
                  path: |
                      ~/.cargo/registry
                      ~/.cargo/git
                      target
                  key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
                  restore-keys: |
                      ${{ runner.os }}-cargo-

            - name: Check formatting
              run: cargo fmt --all -- --check

            - name: Run clippy
              run: cargo clippy --all-targets --all-features -- -D warnings

            - name: Check compilation
              run: cargo check --all-targets --all-features

    # Rust 测试
    rust-test:
        name: Rust Test
        needs: rust-check
        strategy:
            fail-fast: false
            matrix:
                os: [ubuntu-latest, windows-latest, macos-latest]
        runs-on: ${{ matrix.os }}
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@stable

            - name: Cache cargo registry
              uses: actions/cache@v4
              with:
                  path: |
                      ~/.cargo/registry
                      ~/.cargo/git
                      target
                  key: ${{ runner.os }}-cargo-test-${{ hashFiles('**/Cargo.lock') }}
                  restore-keys: |
                      ${{ runner.os }}-cargo-test-

            - name: Run tests
              run: cargo test --workspace --all-features

    # 构建桌面平台
    build-desktop:
        name: Build Desktop Platforms
        needs: rust-test
        if: startsWith(github.ref, 'refs/tags/v') || github.event_name == 'release'
        strategy:
            fail-fast: false
            matrix:
                include:
                    - os: windows-latest
                      target: x86_64-pc-windows-msvc
                      artifact-name: windows
                    - os: macos-latest
                      target: x86_64-apple-darwin
                      artifact-name: macos-x64
                    - os: macos-latest
                      target: aarch64-apple-darwin
                      artifact-name: macos-arm64
                    - os: ubuntu-latest
                      target: x86_64-unknown-linux-gnu
                      artifact-name: linux
        runs-on: ${{ matrix.os }}
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@stable
              with:
                  targets: ${{ matrix.target }}

            - name: Cache cargo registry
              uses: actions/cache@v4
              with:
                  path: |
                      ~/.cargo/registry
                      ~/.cargo/git
                      target
                  key: ${{ runner.os }}-cargo-build-${{ matrix.target }}-${{ hashFiles('**/Cargo.lock') }}
                  restore-keys: |
                      ${{ runner.os }}-cargo-build-${{ matrix.target }}-

            - name: Build for ${{ matrix.target }}
              run: cargo build --package galgame --target ${{ matrix.target }} --release

            - name: Upload artifact
              uses: actions/upload-artifact@v4
              with:
                  name: gg-engine-${{ matrix.artifact-name }}
                  path: target/${{ matrix.target }}/release/

    # 构建 WebAssembly
    build-wasm:
        name: Build WebAssembly
        needs: rust-test
        if: startsWith(github.ref, 'refs/tags/v') || github.event_name == 'release'
        runs-on: ubuntu-latest
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@stable
              with:
                  targets: wasm32-unknown-unknown

            - name: Install wasm-bindgen-cli
              run: cargo install wasm-bindgen-cli

            - name: Cache cargo registry
              uses: actions/cache@v4
              with:
                  path: |
                      ~/.cargo/registry
                      ~/.cargo/git
                      target
                  key: wasm-cargo-build-${{ hashFiles('**/Cargo.lock') }}
                  restore-keys: |
                      wasm-cargo-build-

            - name: Build WASM (H5)
              run: cargo build --package galgame --target wasm32-unknown-unknown --release

            - name: Generate bindings (H5)
              run: wasm-bindgen target/wasm32-unknown-unknown/release/galgame.wasm --out-dir ./web-h5 --target web

            - name: Build WASM (WeChat Mini Game)
              run: cargo build --package galgame --features "platform/wechat" --target wasm32-unknown-unknown --release

            - name: Generate bindings (WeChat Mini Game)
              run: wasm-bindgen target/wasm32-unknown-unknown/release/galgame.wasm --out-dir ./wechat-game --target wechat

            - name: Upload H5 artifacts
              uses: actions/upload-artifact@v4
              with:
                  name: gg-engine-h5
                  path: ./web-h5/

            - name: Upload WeChat Mini Game artifacts
              uses: actions/upload-artifact@v4
              with:
                  name: gg-engine-wechat-game
                  path: ./wechat-game/

    # 构建 Android
    build-android:
        name: Build Android
        needs: rust-test
        if: startsWith(github.ref, 'refs/tags/v') || github.event_name == 'release'
        runs-on: ubuntu-latest
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@stable
              with:
                  targets: aarch64-linux-android, armv7-linux-androideabi, i686-linux-android, x86_64-linux-android

            - name: Set up JDK
              uses: actions/setup-java@v4
              with:
                  java-version: '17'
                  distribution: 'temurin'

            - name: Setup Android SDK
              uses: android-actions/setup-android@v3
              with:
                  api-level: 33
                  ndk: 25.2.9519653

            - name: Install cargo-ndk
              run: cargo install cargo-ndk

            - name: Build Android AAR
              run: cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o ./jniLibs build --release

            - name: Build APK with Gradle
              run: cd android && ./gradlew assembleRelease

            - name: Upload Android artifacts
              uses: actions/upload-artifact@v4
              with:
                  name: gg-engine-android
                  path: android/app/build/outputs/apk/release/
```

---

## 分发打包步骤

### 3.1 桌面平台打包

#### Windows 安装程序
```bash
# 使用 NSIS 或 Inno Setup 创建安装程序
# 示例：创建便携版 ZIP
powershell Compress-Archive -Path target/x86_64-pc-windows-msvc/release/*.exe -DestinationPath gg-engine-windows.zip
```

#### macOS .app 和 .dmg
```bash
# 创建 .app 包
mkdir -p GalgameEngine.app/Contents/MacOS
mkdir -p GalgameEngine.app/Contents/Resources
cp target/x86_64-apple-darwin/release/galgame GalgameEngine.app/Contents/MacOS/
cp Info.plist GalgameEngine.app/Contents/

# 创建 .dmg
hdiutil create -volname "Galgame Engine" -srcfolder GalgameEngine.app -ov -format UDZO galgame-engine-macos.dmg
```

#### Linux
```bash
# 创建 AppImage
# 使用 linuxdeploy 工具
# 或创建 deb/rpm 包
cargo install cargo-deb
cargo deb --package galgame
```

### 3.2 移动平台打包

#### iOS .ipa
```bash
# 归档
xcodebuild -workspace galgame.xcworkspace -scheme galgame -archivePath build/galgame.xcarchive archive

# 导出 IPA
xcodebuild -exportArchive -archivePath build/galgame.xcarchive -exportPath build -exportOptionsPlist ExportOptions.plist
```

#### Android .apk / .aab
```bash
# 构建 APK
./gradlew assembleRelease

# 构建 AAB (Android App Bundle)
./gradlew bundleRelease

# 签名
jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 -keystore my-release-key.keystore app/build/outputs/apk/release/app-release-unsigned.apk alias_name

# 对齐
zipalign -v 4 app/build/outputs/apk/release/app-release-unsigned.apk galgame-engine.apk
```

### 3.3 Web 平台打包

#### H5 (WebAssembly)
```bash
# 创建完整的 Web 部署包
mkdir -p deploy-h5
cp -r web-h5/* deploy-h5/
cp index.html deploy-h5/

# 压缩
cd deploy-h5 && zip -r ../gg-engine-h5.zip .
```

#### 微信小游戏
```bash
# 按照微信小游戏规范组织文件
mkdir -p wechat-game-deploy
cp -r wechat-game/* wechat-game-deploy/
cp game.json wechat-game-deploy/
cp project.config.json wechat-game-deploy/

# 注意：主包大小限制 4MB，超过需分包
```

---

## 引擎插件构建与发布

### 4.1 插件开发结构

gg 引擎插件采用 Rust crate 形式开发，典型结构如下：

```
gg-plugin-example/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── components.rs
│   ├── systems.rs
│   └── resources.rs
└── README.md
```

### 4.2 插件发布流程

#### 1. 准备插件
```bash
# 更新版本号
# Cargo.toml
version = "0.1.0"

# 运行检查
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

#### 2. 发布到 crates.io
```bash
# 登录
cargo login <your-token>

# 发布（dry-run 先检查）
cargo publish --dry-run

# 正式发布
cargo publish
```

#### 3. GitHub 发布
```yaml
# 在 GitHub Actions 中添加插件发布 job
publish-plugin:
    name: Publish Plugin
    needs: rust-test
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    steps:
        - name: Checkout repository
          uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Login to crates.io
          run: cargo login ${{ secrets.CRATES_IO_TOKEN }}

        - name: Publish plugin
          run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }} --manifest-path crates/modules/galgame/Cargo.toml
```

---

## 游戏项目打包与分发

### 5.1 游戏项目结构

```
my_galgame/
├── game.toml                    # 游戏配置
├── assets/                       # 资源文件
│   ├── images/
│   ├── audio/
│   └── fonts/
├── scripts/                      # 剧本文件
├── dlc/                          # DLC目录
└── mods/                         # Mod脚本
```

### 5.2 游戏项目打包

#### 桌面平台
```bash
# 创建游戏项目 ZIP 包
zip -r my-galgame-game.zip my_galgame/
```

#### 移动平台
```bash
# iOS：通过 iTunes 文件共享或 App 内下载
# Android：放置在外部存储或应用内下载
```

#### H5
```bash
# 小型游戏：预先打包进 WASM
# 大型游戏：通过 HTTP 分块加载
```

#### 微信小游戏
```bash
# 随小游戏代码包提交（4MB 内）
# 或通过 CDN 分包下载
```

### 5.3 DLC 与 Mod 分发

#### DLC 分发
- 作为独立 ZIP 包发布
- 玩家下载后解压到游戏项目的 `dlc/` 目录
- 引擎自动合并 DLC 内容

#### Mod 分发
- 独立 `.gg` 脚本文件
- 玩家放置到 `mods/` 目录
- 虚拟机自动加载并执行
