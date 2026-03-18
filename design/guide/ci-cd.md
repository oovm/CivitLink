# CI/CD 流水线模板

本页面提供完整的 CI/CD 流水线模板，用于 WAE 项目的自动化构建、测试和部署。

## GitHub Actions

### 完整 CI 流水线

```yaml
name: WAE CI/CD

on:
    push:
        branches: [main, master, dev]
        tags:
            - 'v*'
    pull_request:
        branches: [main, master, dev]
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
              uses: dtolnay/rust-toolchain@nightly
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
                feature: [no-default, default, all]
        runs-on: ${{ matrix.os }}
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@nightly

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

            - name: Run tests (no-default features)
              if: matrix.feature == 'no-default'
              run: cargo test --workspace --no-default-features

            - name: Run tests (default features)
              if: matrix.feature == 'default'
              run: cargo test --workspace

            - name: Run tests (all features)
              if: matrix.feature == 'all'
              run: cargo test --workspace --all-features

    # JavaScript/TypeScript 检查
    js-check:
        name: JavaScript/TypeScript Check
        runs-on: ubuntu-latest
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Setup Node.js
              uses: actions/setup-node@v4
              with:
                  node-version: "20"

            - name: Install pnpm
              uses: pnpm/action-setup@v4
              with:
                  version: 9

            - name: Get pnpm store directory
              shell: bash
              run: |
                  echo "STORE_PATH=$(pnpm store path --silent)" >> $GITHUB_OUTPUT
              id: pnpm-cache

            - name: Setup pnpm cache
              uses: actions/cache@v4
              with:
                  path: ${{ steps.pnpm-cache.outputs.STORE_PATH }}
                  key: ${{ runner.os }}-pnpm-store-${{ hashFiles('**/pnpm-lock.yaml') }}
                  restore-keys: |
                      ${{ runner.os }}-pnpm-store-

            - name: Install dependencies
              run: pnpm install --frozen-lockfile

            - name: Run Biome CI (format + lint)
              run: npx biome ci .

            - name: Run TypeScript type check
              run: pnpm run typecheck:frontend

            - name: Build frontend packages
              run: pnpm run build:frontend

    # 构建发布（仅在 tag 或 release 时触发）
    build-release:
        name: Build Release
        needs: [rust-test, js-check]
        if: startsWith(github.ref, 'refs/tags/v') || github.event_name == 'release'
        runs-on: ubuntu-latest
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@nightly
              with:
                  targets: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin

            - name: Cache cargo registry
              uses: actions/cache@v4
              with:
                  path: |
                      ~/.cargo/registry
                      ~/.cargo/git
                      target
                  key: ${{ runner.os }}-cargo-release-${{ hashFiles('**/Cargo.lock') }}

            - name: Build for Linux
              run: cargo build --release --workspace --all-features

            - name: Upload artifacts
              uses: actions/upload-artifact@v4
              with:
                  name: wae-release
                  path: target/release/

    # 发布到 crates.io（仅在 tag 时触发）
    publish-crates:
        name: Publish to crates.io
        needs: build-release
        if: startsWith(github.ref, 'refs/tags/v')
        runs-on: ubuntu-latest
        environment: crates.io
        steps:
            - name: Checkout repository
              uses: actions/checkout@v4

            - name: Install Rust toolchain
              uses: dtolnay/rust-toolchain@nightly

            - name: Login to crates.io
              run: cargo login ${{ secrets.CRATES_IO_TOKEN }}

            - name: Publish wae-types
              run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }} --manifest-path backends/wae-types/Cargo.toml

            - name: Publish wae-config
              run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }} --manifest-path backends/wae-config/Cargo.toml

            - name: Publish other crates
              run: |
                  for crate in $(find backends -name "Cargo.toml" -not -path "backends/wae-types/Cargo.toml" -not -path "backends/wae-config/Cargo.toml"); do
                      cargo publish --token ${{ secrets.CRATES_IO_TOKEN }} --manifest-path "$crate" || true
                  done
```

## GitLab CI

### 完整 CI 流水线

```yaml
stages:
  - check
  - test
  - build
  - deploy

variables:
  CARGO_TERM_COLOR: "always"
  RUST_BACKTRACE: "1"
  RUSTFLAGS: "-D warnings"

# Rust 检查
rust-check:
  stage: check
  image: rust:latest
  cache:
    paths:
      - target/
      - ~/.cargo/registry/
      - ~/.cargo/git/
  before_script:
    - rustup component add rustfmt clippy
  script:
    - cargo fmt --all -- --check
    - cargo clippy --all-targets --all-features -- -D warnings
    - cargo check --all-targets --all-features

# Rust 测试 - Linux
rust-test-linux:
  stage: test
  image: rust:latest
  cache:
    paths:
      - target/
      - ~/.cargo/registry/
      - ~/.cargo/git/
  script:
    - cargo test --workspace --no-default-features
    - cargo test --workspace
    - cargo test --workspace --all-features

# Rust 测试 - Windows
rust-test-windows:
  stage: test
  tags:
    - windows
  cache:
    paths:
      - target/
  script:
    - cargo test --workspace --all-features

# JavaScript/TypeScript 检查
js-check:
  stage: check
  image: node:20
  cache:
    paths:
      - node_modules/
      - .pnpm-store/
  before_script:
    - corepack enable
    - corepack prepare pnpm@latest --activate
    - pnpm config set store-dir .pnpm-store
  script:
    - pnpm install --frozen-lockfile
    - npx biome ci .
    - pnpm run typecheck:frontend
    - pnpm run build:frontend

# 构建发布
build-release:
  stage: build
  image: rust:latest
  cache:
    paths:
      - target/
      - ~/.cargo/registry/
      - ~/.cargo/git/
  only:
    - tags
    - /^v\d+\.\d+\.\d+$/
  script:
    - cargo build --release --workspace --all-features
  artifacts:
    paths:
      - target/release/
    expire_in: 1 week

# 发布到 crates.io
publish-crates:
  stage: deploy
  image: rust:latest
  cache:
    paths:
      - target/
      - ~/.cargo/registry/
      - ~/.cargo/git/
  only:
    - tags
    - /^v\d+\.\d+\.\d+$/
  dependencies:
    - build-release
  script:
    - cargo login $CRATES_IO_TOKEN
    - cargo publish --token $CRATES_IO_TOKEN --manifest-path backends/wae-types/Cargo.toml
    - cargo publish --token $CRATES_IO_TOKEN --manifest-path backends/wae-config/Cargo.toml
    - |
      for crate in $(find backends -name "Cargo.toml" -not -path "backends/wae-types/Cargo.toml" -not -path "backends/wae-config/Cargo.toml"); do
          cargo publish --token $CRATES_IO_TOKEN --manifest-path "$crate" || true
      done
```

## 使用说明

### GitHub Actions

1. 将 `.github/workflows/ci.yml` 文件复制到您的项目中
2. 在 GitHub 仓库设置中添加 `CRATES_IO_TOKEN` secret（用于发布到 crates.io）
3. 配置环境 `crates.io` 用于发布

### GitLab CI

1. 将 `.gitlab-ci.yml` 文件复制到您的项目中
2. 在 GitLab CI/CD 设置中添加 `CRATES_IO_TOKEN` 变量
3. 根据需要配置 Runner（特别是 Windows Runner）

## 流水线功能

### 代码检查
- Rust 代码格式化检查
- Rust Clippy  lint 检查
- JavaScript/TypeScript 格式化和 lint 检查
- TypeScript 类型检查

### 测试
- 多平台测试（Linux、Windows、macOS）
- 多 feature 组合测试
- 前端构建测试

### 构建
- Release 模式构建
- 多目标平台构建
- 构建产物上传

### 部署
- 发布到 crates.io
- 版本标签触发
