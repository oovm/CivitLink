# *.vx 文件格式规范

## 概述

*.vx 文件是 GG Editor 跨平台 GUI 开发框架使用的文件格式，类似于 Vue 的单文件组件格式，包含 `<template>`、`<script>` 和 `<style>` 三个主要部分。

## 文件结构

一个完整的 *.vx 文件结构如下：

```vue
<template>
    <Layout style="flex-1">
        <Text class="title" style="text-xl font-bold">Hello GG Editor</Text>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Text};
</script>

<style>
    .title {
        color: $primary-color;
    }
</style>
```

## 各部分详细规范

### 1. `<template>` 部分

`<template>` 部分使用 ValkyrieX 语法，用于定义 GUI 界面的结构。

#### 语法规则

- 使用 ValkyrieX 语法，支持 XML 表达式
- 支持基础组件和自定义组件
- 支持属性传递和事件绑定
- 支持条件渲染和列表渲染

#### 基础组件

| 组件名称 | 描述 | 属性 |
|---------|------|------|
| Layout | 布局容器，支持 flex 布局 | style, class, id |
| Stack | 栈式布局容器 | style, class, id, orientation |
| Button | 按钮 | style, class, id, text, onClick |
| Text | 文本显示 | style, class, id, value |
| Input | 输入框 | style, class, id, value, onChange |
| Image | 图片显示 | style, class, id, src |
| Panel | 面板容器 | style, class, id |
| ScrollView | 滚动视图 | style, class, id, direction |

#### 示例

```vx
<template>
    <Layout style="flex-1 bg-[#f0f0f0]">
        <Stack orientation="vertical" style="gap-12">
            <Text style="text-xl font-bold">Hello GG Editor</Text>
            <Button text="Click Me" onClick={() => console.log("Button clicked!")} />
            <Input value={inputValue} onChange={(e) => setInputValue(e.target.value)} />
            <Image src="assets/logo.png" style="width-[200px] height-[200px]" />
            <ScrollView direction="vertical" style="flex-1 border-[1px solid #ccc]">
                <Stack orientation="vertical" style="padding-16 gap-8">
                    <loop (item, index) in items>
                        <Panel key={index} style="padding-8 bg-[#fff] border-r-4">
                            <Text>{item}</Text>
                        </Panel>
                    </loop>
                </Stack>
            </ScrollView>
        </Stack>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Stack, Button, Text, Input, Image, Panel, ScrollView};
</script>
```

### 2. `<script>` 部分

`<script>` 部分使用 Valkyrie 脚本语法，类似 SolidJS 的响应式编程模型。

#### 语法规则

- 使用 Valkyrie 脚本语法
- 使用 `using` 导入依赖
- 使用 `let` 声明变量
- 支持响应式状态管理
- 支持生命周期钩子
- 支持组件通信

#### 响应式状态

使用 `signal` 创建响应式状态：

```valkyrie
using gg_editor::ui::signal;

let counter = signal(0);
let inputValue = signal("");
```

#### 生命周期钩子

- `onMount`：组件挂载时执行
- `onUpdate`：组件更新时执行
- `onCleanup`：组件卸载时执行

#### 示例

```vx
<template>
    <Layout style="flex-1 padding-16 bg-[#f0f0f0]">
        <Stack orientation="vertical" style="gap-12">
            <Text style="font-24 font-bold">Counter: {counter()}</Text>
            <Button text="Increment" onClick={handleClick} />
            <Input value={inputValue()} onChange={(e) => inputValue(e.target.value)} />
            <Text>You entered: {inputValue()}</Text>
            <ScrollView direction="vertical" style="flex-1 border-[1px solid #ccc]">
                <Stack orientation="vertical" style="padding-16 gap-8">
                    <loop (item, index) in items>
                        <Panel key={index} style="padding-8 bg-[#fff] border-r-4">
                            <Text>{item}</Text>
                        </Panel>
                    </loop>
                </Stack>
            </ScrollView>
        </Stack>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Stack, Button, Text, Input, Panel, ScrollView};
    using gg_editor::ui::signal;
    using gg_editor::ui::{on_mount, on_cleanup, on_update};

    let counter = signal(0);
    let inputValue = signal("");
    let items = signal(["Item 1", "Item 2", "Item 3"]);

    let handleClick = micro() {
        counter(counter() + 1);
        items([...items(), "Item " + (items().length + 1)]);
    };

    on_mount(micro() {
        console.log("Component mounted");
    });

    on_update(micro() {
        console.log("Component updated");
    });

    on_cleanup(micro() {
        console.log("Component unmounted");
    });
</script>
```

### 3. `<style>` 部分

`<style>` 部分使用 SCSS 语法，但由 GG Renderer 渲染，不需要支持所有 SCSS 特性。

#### 支持的 SCSS 特性

- 基本选择器（类选择器、ID 选择器、标签选择器）
- 嵌套选择器
- 变量
- 简单的混合（mixins）
- 基本的运算

#### 不支持的 SCSS 特性

- 复杂的混合（mixins）
- 函数
- 继承（@extend）
- 条件语句
- 循环语句

#### 示例

```vue
<style>
    $primary-color: #4CAF50;
    $secondary-color: #2196F3;

    .container {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 16px;
        background-color: #f0f0f0;

        .title {
            font-size: 24px;
            font-weight: bold;
            color: $primary-color;
        }

        .button {
            background-color: $secondary-color;
            color: white;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;

            &:hover {
                background-color: darken($secondary-color, 10%);
            }
        }

        .input {
            padding: 8px;
            border: 1px solid #ccc;
            border-radius: 4px;
        }
    }
</style>
```

## 组件导入与导出

### 导入组件

```vue
<template>
    <Layout>
        <Button text="Native Button" />
        <CustomComponent message="Hello from custom component" />
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Button};
    using package::widgets::CustomComponent;
</script>
```

### 导出组件

```vue
<template>
    <Panel>
        <Text>{props.message}</Text>
    </Panel>
</template>

<script>
    using gg_editor::ui::widgets::{Panel, Text};
    @property("props")
    class CustomComponentProps {
        message: string
    }
</script>
```

## 特殊语法

### 条件渲染

```vue
<template>
    <Layout>
        <if (count > 5)>
            <Text>Count is greater than 5</Text>
        <else/>
            <Text>Count is less than or equal to 5</Text>
        </if>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Text};
    using gg_editor::ui::signal;

    let count = signal(0);
</script>
```

### 列表渲染

```vue
<template>
    <Layout>
        <loop (item, index) in items>
            <Text key={index}>{item}</Text>
        </loop>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Text};
    using gg_editor::ui::signal;

    let items = signal(["Item 1", "Item 2", "Item 3"]);
</script>
```

### 事件绑定

```vue
<template>
    <Layout>
        <Button text="Click Me" onClick={handleClick} />
        <Input value={inputValue()} onChange={(e) => inputValue(e.target.value)} />
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Button, Input};
    using gg_editor::ui::signal;

    let inputValue = signal("");

    let handleClick = micro() {
        console::log("Button clicked!");
    };
</script>
```

## 最佳实践

1. **组件拆分**：将复杂的 UI 拆分为多个小型、可复用的组件
2. **状态管理**：使用 Valkyrie 的响应式 API 管理组件状态
3. **样式组织**：使用 SCSS 的嵌套和变量功能组织样式
4. **性能优化**：避免在模板中使用复杂的计算表达式，使用计算属性代替
5. **代码规范**：遵循一致的代码风格和命名约定

## 示例完整文件

```vue
<template>
    <Layout style="flex-1 p-16 bg-[#f0f0f0]">
        <Stack orientation="vertical" style="gap-12">
            <Text class="title">Counter: {count}</Text>
            <Button class="button" text="Increment" onClick={handleIncrement} />
            <Button class="button" text="Decrement" onClick={handleDecrement} />
            <Input class="input" value={inputValue()} onChange={handleInputChange} placeholder="Enter text" />
            <Text>You entered: {inputValue}</Text>
            <ScrollView direction="vertical" style="flex-1 border-[1px solid #ccc]">
                <Stack orientation="vertical" style="padding-16 gap-8">
                    <loop (item, index) in items>
                        <Panel key={index} class="item-panel">
                            <Text>{item}</Text>
                        </Panel>
                    </loop>
                </Stack>
            </ScrollView>
        </Stack>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Stack, Button, Text, Input, Panel, ScrollView};
    using gg_editor::ui::signal;
    using gg_editor::ui::onMount;

    let count = signal(0);
    let inputValue = signal("");
    let items = signal(["Item 1", "Item 2", "Item 3"]);

    let handleIncrement = micro() {
        count(count.value + 1);
    };

    let handleDecrement = micro() {
        count(count.value - 1);
    };

    let handleInputChange = micro() {
        inputValue(e.target.value);
    };

    on_mount(micro() {
        console::log("Component mounted");
    });
</script>

<style>
    $primary-color: #4CAF50;
    $secondary-color: #2196F3;
    $background-color: #f0f0f0;

    .title {
        font-size: 24px;
        font-weight: bold;
        color: $primary-color;
    }

    .button {
        background-color: $secondary-color;
        color: white;
        padding: 8px 16px;
        border-radius: 4px;
        cursor: pointer;

        &:hover {
            background-color: darken($secondary-color, 10%);
        }
    }

    .input {
        padding: 8px;
        border: 1px solid #ccc;
        border-radius: 4px;
    }

    .item-panel {
        padding: 8px;
        background-color: #fff;
        border-radius: 4px;
        border: 1px solid #eee;
    }
</style>
```

## 总结

*.vx 文件格式为 GG Editor 提供了一种统一、直观的方式来开发跨平台 GUI 界面。通过结合 TSX 模板、Valkyrie 脚本和 SCSS 样式，开发者可以快速构建出美观、响应式的 GUI 界面，同时享受跨平台的便利。
