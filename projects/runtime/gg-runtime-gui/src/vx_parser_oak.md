# VX 文件解析器（基于 oak-voc）

## 概述

本模块使用 oak-voc 作为 VX 文件的解析器，替换了原来的手写解析器。oak-voc 是一个基于 oak_core 的语言解析框架，提供标准化的词法分析和语法解析能力。

## 集成过程

1. **添加依赖**：在 `Cargo.toml` 文件中添加 oak-voc 作为依赖：

   ```toml
   [dependencies]
   oak-voc = { path = "../../../oaks/examples/oak-voc" }
   ```

2. **创建解析器封装**：创建 `vx_parser_oak.rs` 文件，封装 oak-voc 解析器的使用。

3. **移除原有解析器**：删除手写的 `vx_ast.rs`、`vx_lexer.rs` 和 `vx_parser.rs` 文件。

4. **更新类型引用**：将所有对原有类型的引用更新为 oak-voc 的类型，例如：
   - `TemplateNode` → `oak_voc::TemplateNode`
   - `VxDocument` → `oak_voc::VxDocument`

## 使用方法

### 解析 VX 文件

使用 `parse_vx` 函数解析 VX 文件源码：

```rust
use gg_runtime_gui::vx_parser_oak::parse_vx;

let source = r#"
<template>
  <Layout style="flex-1">
    <Text class="title">Hello GG Editor</Text>
  </Layout>
</template>

<script>
  using gg_editor::ui::widgets::{Layout, Text};
</script>

<style>
  .title {
    color: #fff;
  }
</style>
"#;

let doc = parse_vx(source).unwrap();
```

### 创建动态 VX 组件

使用解析后的 `VxDocument` 创建动态 VX 组件：

```rust
use gg_runtime_gui::{DynamicVxComponent, vx_parser_oak::parse_vx};

let source = r#"
<template>
  <Layout style="flex-1">
    <Text>Hello World</Text>
  </Layout>
</template>
"#;

let doc = parse_vx(source).unwrap();
let template = doc.template;
let style_string = doc.style.as_ref().map(|style_ast| {
    style_ast
        .rules
        .iter()
        .map(|rule| {
            let props = rule
                .properties
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            if props.is_empty() {
                format!("{} {{ }}", rule.selector)
            } else {
                format!("{} {{ {}; }}", rule.selector, props)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
});
let script_source = doc.script.as_ref().map(|s| s.raw_source.clone());

let comp = DynamicVxComponent::from_document("test-component", template, style_string, script_source);
```

## 性能对比

### 解析速度

- **原有解析器**：手写解析器，解析速度较快，但维护成本高。
- **oak-voc 解析器**：基于 oak_core 的标准化解析器，解析速度与原有解析器相当，维护成本低。

### 内存使用

- **原有解析器**：内存使用较低。
- **oak-voc 解析器**：内存使用与原有解析器相当。

## 优势

1. **标准化**：使用 oak_core 提供的标准化解析框架，代码结构更清晰。
2. **可维护性**：解析逻辑与业务逻辑分离，更容易维护和扩展。
3. **一致性**：与其他使用 oak_core 的解析器保持一致的设计风格。
4. **功能完备**：支持 VX 文件的 template、script 和 style 三个部分的解析。

## 注意事项

1. **依赖管理**：确保 oak-voc 依赖正确配置。
2. **类型引用**：所有对 VX 相关类型的引用都需要更新为 oak-voc 的类型。
3. **错误处理**：使用 `VxParseError` 处理解析错误。

## 示例

### 基本 VX 文件解析

```rust
let source = "<template><div>Hello</div></template>";
let doc = parse_vx(source).unwrap();
assert!(doc.template.is_some());
assert!(doc.script.is_none());
assert!(doc.style.is_none());
```

### 完整 VX 文件解析

```rust
let source = r#"
<template>
  <Layout style="flex-1">
    <Text class="title">Hello GG Editor</Text>
  </Layout>
</template>

<script>
  using gg_editor::ui::widgets::{Layout, Text};
</script>

<style>
  .title {
    color: #fff;
  }
</style>
"#;
let doc = parse_vx(source).unwrap();
assert!(doc.template.is_some());
assert!(doc.script.is_some());
assert!(doc.style.is_some());
```