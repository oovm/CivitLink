# *.vx Editor UI Toolkit 文件格式规范

## 概述

*.vx 文件是 GG Editor UI Toolkit 使用的文件格式，仅用于编辑器界面开发。类似于 Vue 的单文件组件格式，包含 `<template>`、`<script>` 和 `<style>` 三个主要部分。

> **重要声明**：*.vx 文件仅用于编辑器界面开发，不可用于游戏运行时。游戏运行时 UI 请使用 *.gameui 文件格式（基于 ECS + Canvas 体系）。

## 文件结构

一个完整的 *.vx 文件结构如下：

```vue
<template>
    <Layout style="flex-1">
        <Text class="title" style="text-xl font-bold">GG Editor</Text>
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

`<template>` 部分使用 ValkyrieX 语法，用于定义编辑器界面的结构。

#### 语法规则

- 使用 ValkyrieX 语法，支持 XML 表达式
- 支持基础组件和自定义组件
- 支持属性传递和事件绑定
- 支持条件渲染和列表渲染

#### 基础组件

| 组件名称 | 描述 | 属性 | 编辑器专用 |
|---------|------|------|-----------|
| Layout | 布局容器，支持 flex 布局 | style, class, id | 否 |
| Stack | 栈式布局容器 | style, class, id, orientation | 否 |
| Button | 按钮 | style, class, id, text, onClick | 否 |
| Text | 文本显示 | style, class, id, value | 否 |
| Input | 输入框 | style, class, id, value, onChange | 否 |
| Icon | 图标显示 | style, class, id, name | 否 |
| Panel | 面板容器 | style, class, id | 否 |
| ScrollView | 滚动视图 | style, class, id, scroll_direction | 否 |
| InspectorPanel | 属性检查器面板 | style, class, id, target | 是 |
| HierarchyView | 层级视图 | style, class, id, root | 是 |
| AssetBrowser | 资源浏览器 | style, class, id, path | 是 |
| SceneView | 场景编辑视图 | style, class, id, scene | 是 |
| Toolbar | 工具栏 | style, class, id, items | 是 |
| MenuBar | 菜单栏 | style, class, id, menus | 是 |
| TabContainer | 标签页容器 | style, class, id, active_tab | 是 |
| SplitView | 分割视图 | style, class, id, orientation, ratio | 是 |
| TreeView | 树形视图 | style, class, id, data, on_select | 是 |
| PropertyField | 属性字段 | style, class, id, label, type, value, onChange | 是 |

#### 示例

```vx
<template>
    <Layout style="flex-1 bg-[#1e1e1e]">
        <Stack orientation="vertical" style="gap-8">
            <MenuBar menus={editorMenus} />
            <Toolbar items={toolItems} />
            <SplitView orientation="horizontal" ratio="0.25" style="flex-1">
                <HierarchyView root={sceneRoot} style="flex-1" />
                <SplitView orientation="horizontal" ratio="0.6" style="flex-1">
                    <SceneView scene={currentScene} style="flex-1" />
                    <InspectorPanel target={selectedObject} style="flex-1" />
                </SplitView>
            </SplitView>
            <AssetBrowser path="/assets" style="height-[200px]" />
        </Stack>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Stack, MenuBar, Toolbar, SplitView, HierarchyView, SceneView, InspectorPanel, AssetBrowser};
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

let selectedObject = signal(null);
let currentScene = signal("Main");
```

#### 生命周期钩子

- `onMount`：组件挂载时执行
- `onUpdate`：组件更新时执行
- `onCleanup`：组件卸载时执行

#### 示例

```vx
<template>
    <Layout style="flex-1 padding-16 bg-[#1e1e1e]">
        <Stack orientation="vertical" style="gap-8">
            <Text style="font-14 font-bold color-[#ccc]">属性检查器</Text>
            <if (selectedObject())>
                <InspectorPanel target={selectedObject()} style="flex-1" />
            <else/>
                <Text style="color-[#666]">未选中任何对象</Text>
            </if>
            <ScrollView scroll_direction="vertical" style="flex-1 border-[1px solid #333]">
                <Stack orientation="vertical" style="padding-8 gap-4">
                    <loop (prop, index) in properties()>
                        <PropertyField key={index} label={prop.label} type={prop.type} value={prop.value} onChange={handlePropertyChange} />
                    </loop>
                </Stack>
            </ScrollView>
        </Stack>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Stack, Text, InspectorPanel, ScrollView, PropertyField};
    using gg_editor::ui::signal;
    using gg_editor::ui::{on_mount, on_cleanup, on_update};

    let selectedObject = signal(null);
    let properties = signal([]);

    let handlePropertyChange = micro(prop_name, new_value) {
        let obj = selectedObject();
        obj.set_property(prop_name, new_value);
        selectedObject(obj);
    };

    on_mount(micro() {
        console::log("Inspector panel mounted");
    });

    on_update(micro() {
        console::log("Inspector panel updated");
    });

    on_cleanup(micro() {
        console::log("Inspector panel unmounted");
    });
</script>
```

### 3. `<style>` 部分

`<style>` 部分使用 USS（UI Style Sheet）样式系统，由 Editor UI 独立渲染器解析和渲染。USS 基于 SCSS 语法子集，但存在与标准 CSS 的关键差异。

#### 选择器优先级规则

USS 选择器优先级从高到低排列：

| 优先级 | 选择器类型 | 示例 | 权重 |
|-------|-----------|------|------|
| 1（最高） | 内联样式 | `style="color: red"` | 1000 |
| 2 | ID 选择器 | `#my-panel` | 100 |
| 3 | 类选择器 | `.title` | 10 |
| 4（最低） | 标签选择器 | `Text` | 1 |

优先级计算规则：
- 同一元素上多个选择器的权重累加比较
- 权重相同时，后定义的样式覆盖先定义的样式
- 内联样式始终具有最高优先级
- `!important` 声明可覆盖内联样式以外的所有规则

#### 主题变量系统

USS 内置主题变量，支持亮色主题和暗色主题自动切换：

**基础色变量：**

| 变量名 | 暗色主题值 | 亮色主题值 | 描述 |
|-------|-----------|-----------|------|
| `$primary-color` | `#007ACC` | `#0066BF` | 主色调 |
| `$secondary-color` | `#3A3D41` | `#E8E8E8` | 辅助色 |
| `$bg-color` | `#1E1E1E` | `#FFFFFF` | 背景色 |
| `$bg-secondary-color` | `#252526` | `#F3F3F3` | 次要背景色 |
| `$text-color` | `#CCCCCC` | `#333333` | 文本色 |
| `$text-secondary-color` | `#858585` | `#666666` | 次要文本色 |
| `$border-color` | `#3C3C3C` | `#D4D4D4` | 边框色 |
| `$accent-color` | `#0098FF` | `#0066BF` | 强调色 |
| `$error-color` | `#F44747` | `#D32F2F` | 错误色 |
| `$warning-color` | `#CCA700` | `#F9A825` | 警告色 |
| `$success-color` | `#89D185` | `#388E3C` | 成功色 |

**间距变量：**

| 变量名 | 值 | 描述 |
|-------|-----|------|
| `$spacing-xs` | `4px` | 极小间距 |
| `$spacing-sm` | `8px` | 小间距 |
| `$spacing-md` | `12px` | 中间距 |
| `$spacing-lg` | `16px` | 大间距 |
| `$spacing-xl` | `24px` | 超大间距 |

**字体变量：**

| 变量名 | 值 | 描述 |
|-------|-----|------|
| `$font-size-xs` | `10px` | 极小字号 |
| `$font-size-sm` | `12px` | 小字号 |
| `$font-size-md` | `14px` | 中字号 |
| `$font-size-lg` | `16px` | 大字号 |
| `$font-size-xl` | `20px` | 超大字号 |
| `$font-family` | `"Segoe UI", sans-serif` | 字体族 |

**圆角变量：**

| 变量名 | 值 | 描述 |
|-------|-----|------|
| `$radius-sm` | `2px` | 小圆角 |
| `$radius-md` | `4px` | 中圆角 |
| `$radius-lg` | `8px` | 大圆角 |

#### 布局模式详解

**Flex 布局属性：**

| 属性 | 可选值 | 描述 |
|------|-------|------|
| `display` | `flex` | 启用 Flex 布局 |
| `flex-direction` | `row`, `column`, `row-reverse`, `column-reverse` | 主轴方向 |
| `flex-wrap` | `nowrap`, `wrap`, `wrap-reverse` | 换行方式 |
| `justify-content` | `flex-start`, `flex-end`, `center`, `space-between`, `space-around`, `space-evenly` | 主轴对齐 |
| `align-items` | `flex-start`, `flex-end`, `center`, `stretch`, `baseline` | 交叉轴对齐 |
| `align-content` | `flex-start`, `flex-end`, `center`, `stretch`, `space-between`, `space-around` | 多行对齐 |
| `flex-grow` | `<number>` | 放大比例 |
| `flex-shrink` | `<number>` | 缩小比例 |
| `flex-basis` | `<length>`, `auto` | 初始大小 |
| `gap` | `<length>` | 子元素间距 |
| `order` | `<integer>` | 排列顺序 |

**Grid 布局属性：**

| 属性 | 可选值 | 描述 |
|------|-------|------|
| `display` | `grid` | 启用 Grid 布局 |
| `grid-template-columns` | `<track-size>...` | 列轨道定义 |
| `grid-template-rows` | `<track-size>...` | 行轨道定义 |
| `grid-template-areas` | `<string>...` | 区域定义 |
| `grid-column-gap` | `<length>` | 列间距 |
| `grid-row-gap` | `<length>` | 行间距 |
| `grid-auto-columns` | `<track-size>` | 隐式列轨道大小 |
| `grid-auto-rows` | `<track-size>` | 隐式行轨道大小 |
| `grid-auto-flow` | `row`, `column`, `dense` | 自动放置方向 |
| `justify-items` | `start`, `end`, `center`, `stretch` | 单元格水平对齐 |
| `align-items` | `start`, `end`, `center`, `stretch` | 单元格垂直对齐 |
| `grid-column` | `<line> / <line>` | 列起止位置 |
| `grid-row` | `<line> / <line>` | 行起止位置 |
| `grid-area` | `<name>` 或 `<line> / <line> / <line> / <line>` | 区域定位 |

#### USS 与 CSS 的差异说明

| 特性 | CSS | USS |
|------|-----|-----|
| 盒模型 | `content-box` / `border-box` | 仅 `border-box` |
| 定位模式 | `static`, `relative`, `absolute`, `fixed`, `sticky` | `relative`, `absolute` |
| 浮动 | 支持 `float` | 不支持 |
| 媒体查询 | 支持 `@media` | 不支持（编辑器窗口尺寸由布局系统管理） |
| 伪元素 | `::before`, `::after` 等 | 不支持 |
| 伪类 | `:hover`, `:focus`, `:active` 等 | 仅支持 `:hover`, `:focus`, `:active`, `:disabled` |
| 动画 | `@keyframes`, `transition` | 仅支持 `transition`（不支持 `@keyframes`） |
| `calc()` | 完整支持 | 仅支持 `+`, `-` 运算 |
| 单位 | `px`, `em`, `rem`, `%`, `vh`, `vw` 等 | 仅 `px` 和 `%` |
| `z-index` | 任意整数值 | 仅在同一层级内生效 |
| `overflow` | `visible`, `hidden`, `scroll`, `auto` | `hidden`, `scroll` |
| `cursor` | 完整支持 | 仅支持 `default`, `pointer`, `text`, `move`, `resize-*` |

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
    .inspector-panel {
        display: flex;
        flex-direction: column;
        gap: $spacing-sm;
        padding: $spacing-md;
        background-color: $bg-secondary-color;
        border-left: 1px solid $border-color;

        .section-title {
            font-size: $font-size-sm;
            font-weight: bold;
            color: $text-secondary-color;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }

        .property-row {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: $spacing-sm;

            .property-label {
                font-size: $font-size-sm;
                color: $text-color;
                min-width: 80px;
            }

            .property-input {
                flex: 1;
                padding: $spacing-xs $spacing-sm;
                background-color: $bg-color;
                border: 1px solid $border-color;
                border-radius: $radius-sm;
                color: $text-color;

                &:focus {
                    border-color: $accent-color;
                }

                &:disabled {
                    opacity: 0.5;
                }
            }
        }
    }

    .hierarchy-item {
        display: flex;
        align-items: center;
        padding: $spacing-xs $spacing-sm;
        cursor: pointer;
        border-radius: $radius-sm;

        &:hover {
            background-color: $secondary-color;
        }

        &.selected {
            background-color: $primary-color;
            color: white;
        }
    }
</style>
```

## Editor UI 独立渲染器架构

### 架构概述

Editor UI 使用独立渲染器，直接在图形设备层上工作，与游戏运行时渲染管线完全隔离。

### 核心特性

- **独立渲染器**：Editor UI 渲染器直接在图形设备层上工作，不依赖游戏渲染管线
- **最上层绘制**：默认绘制在游戏画面最上层，确保编辑器界面始终可见
- **无 3D 交互**：不与 3D 世界交互，不支持 World Space 模式
- **独立输入系统**：编辑器 UI 拥有独立的输入事件处理，不与游戏输入冲突

### 渲染流程

```
DOM 树 → 布局计算 → 样式计算 → 绘制命令 → 独立渲染器
```

1. **DOM 树构建**：解析 `<template>` 中的 ValkyrieX 语法，构建虚拟 DOM 树
2. **布局计算**：根据 Flex / Grid 布局属性，计算每个节点的位置和大小
3. **样式计算**：解析 USS 样式，结合主题变量和选择器优先级，确定最终样式
4. **绘制命令**：将样式化的 DOM 树转换为底层绘制命令（矩形、文本、图标等）
5. **独立渲染器**：独立渲染器接收绘制命令，直接调用图形设备 API 完成渲染

### 与游戏渲染的关系

| 特性 | Editor UI 渲染 | 游戏渲染 |
|------|---------------|---------|
| 渲染管线 | 独立渲染器 | 游戏渲染管线 |
| 绘制层级 | 始终最上层 | 由场景决定 |
| 输入处理 | 独立输入系统 | 游戏输入系统 |
| World Space | 不支持 | 支持 |
| Screen Space | 支持（默认） | 支持 |
| 帧率 | 跟随编辑器刷新率 | 跟随游戏帧率 |

## 组件导入与导出

### 导入组件

```vue
<template>
    <Layout>
        <Button text="Native Button" />
        <CustomInspector message="Hello from custom inspector" />
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Button};
    using package::widgets::CustomInspector;
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
    class CustomInspectorProps {
        message: string
    }
</script>
```

## 特殊语法

### 条件渲染

```vue
<template>
    <Layout>
        <if (selectedObject())>
            <InspectorPanel target={selectedObject()} />
        <else/>
            <Text style="color-[#666]">未选中任何对象</Text>
        </if>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Text, InspectorPanel};
    using gg_editor::ui::signal;

    let selectedObject = signal(null);
</script>
```

### 列表渲染

```vue
<template>
    <Layout>
        <loop (node, index) in hierarchyNodes>
            <TreeView key={index} data={node} on_select={handleSelect} />
        </loop>
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, TreeView};
    using gg_editor::ui::signal;

    let hierarchyNodes = signal([]);
</script>
```

### 事件绑定

```vue
<template>
    <Layout>
        <Button text="刷新资源" onClick={handleRefresh} />
        <Input value={searchQuery()} onChange={(e) => searchQuery(e.target.value)} placeholder="搜索资源..." />
    </Layout>
</template>

<script>
    using gg_editor::ui::widgets::{Layout, Button, Input};
    using gg_editor::ui::signal;

    let searchQuery = signal("");

    let handleRefresh = micro() {
        console::log("Refreshing assets...");
    };
</script>
```

## 最佳实践

1. **组件拆分**：将复杂的编辑器面板拆分为多个小型、可复用的组件
2. **状态管理**：使用 Valkyrie 的响应式 API 管理编辑器状态
3. **样式组织**：使用 USS 的嵌套和主题变量功能组织样式，确保亮色/暗色主题兼容
4. **性能优化**：避免在模板中使用复杂的计算表达式，使用计算属性代替
5. **编辑器规范**：遵循编辑器 UI 设计规范，保持与内置面板一致的视觉风格
6. **属性检查器**：使用 PropertyField 组件而非自行构建属性编辑控件
7. **布局优先**：优先使用 Flex / Grid 布局，避免绝对定位

## 示例完整文件

### 属性检查器面板

```vue
<template>
    <Panel class="inspector-panel" style="flex-1">
        <Stack orientation="vertical" style="gap-8">
            <Text class="section-title">属性检查器</Text>
            <if (selectedObject())>
                <Stack orientation="vertical" style="gap-4">
                    <PropertyField label="名称" type="string" value={selectedObject().name} onChange={(v) => updateProperty("name", v)} />
                    <PropertyField label="位置" type="vec3" value={selectedObject().position} onChange={(v) => updateProperty("position", v)} />
                    <PropertyField label="旋转" type="vec3" value={selectedObject().rotation} onChange={(v) => updateProperty("rotation", v)} />
                    <PropertyField label="缩放" type="vec3" value={selectedObject().scale} onChange={(v) => updateProperty("scale", v)} />
                </Stack>
            <else/>
                <Text class="empty-hint">未选中任何对象</Text>
            </if>
        </Stack>
    </Panel>
</template>

<script>
    using gg_editor::ui::widgets::{Panel, Stack, Text, PropertyField};
    using gg_editor::ui::signal;
    using gg_editor::ui::on_mount;

    let selectedObject = signal(null);

    let updateProperty = micro(prop_name, new_value) {
        let obj = selectedObject();
        obj.set_property(prop_name, new_value);
        selectedObject(obj);
    };

    on_mount(micro() {
        console::log("Inspector panel mounted");
    });
</script>

<style>
    .inspector-panel {
        padding: $spacing-md;
        background-color: $bg-secondary-color;
    }

    .section-title {
        font-size: $font-size-sm;
        font-weight: bold;
        color: $text-secondary-color;
        text-transform: uppercase;
    }

    .empty-hint {
        color: $text-secondary-color;
        font-size: $font-size-sm;
        padding: $spacing-lg;
        text-align: center;
    }
</style>
```

### 层级视图

```vue
<template>
    <Panel class="hierarchy-panel" style="flex-1">
        <Stack orientation="vertical" style="gap-4">
            <Text class="section-title">层级</Text>
            <Input class="search-input" value={searchQuery()} onChange={(e) => searchQuery(e.target.value)} placeholder="搜索节点..." />
            <ScrollView scroll_direction="vertical" style="flex-1">
                <Stack orientation="vertical" style="gap-0">
                    <loop (node, index) in filteredNodes()>
                        <TreeView key={index} data={node} on_select={handleNodeSelect} />
                    </loop>
                </Stack>
            </ScrollView>
        </Stack>
    </Panel>
</template>

<script>
    using gg_editor::ui::widgets::{Panel, Stack, Text, Input, ScrollView, TreeView};
    using gg_editor::ui::signal;
    using gg_editor::ui::on_mount;

    let searchQuery = signal("");
    let filteredNodes = signal([]);

    let handleNodeSelect = micro(node) {
        console::log("Selected node: " + node.name);
    };

    on_mount(micro() {
        console::log("Hierarchy panel mounted");
    });
</script>

<style>
    .hierarchy-panel {
        padding: $spacing-md;
        background-color: $bg-secondary-color;
    }

    .section-title {
        font-size: $font-size-sm;
        font-weight: bold;
        color: $text-secondary-color;
        text-transform: uppercase;
    }

    .search-input {
        padding: $spacing-xs $spacing-sm;
        background-color: $bg-color;
        border: 1px solid $border-color;
        border-radius: $radius-sm;
        color: $text-color;

        &:focus {
            border-color: $accent-color;
        }
    }
</style>
```

### 资源浏览器

```vue
<template>
    <Panel class="asset-browser" style="flex-1">
        <Stack orientation="vertical" style="gap-8">
            <Stack orientation="horizontal" style="gap-8 align-items-center">
                <Text class="section-title">资源</Text>
                <Input class="search-input" value={searchQuery()} onChange={(e) => searchQuery(e.target.value)} placeholder="搜索资源..." style="flex-1" />
                <Button text="导入" onClick={handleImport} class="btn-sm" />
            </Stack>
            <ScrollView scroll_direction="horizontal" style="flex-1">
                <Stack orientation="horizontal" style="gap-8 padding-8">
                    <loop (asset, index) in filteredAssets()>
                        <Panel key={index} class="asset-item">
                            <Icon name={asset.icon} style="width-[48px] height-[48px]" />
                            <Text class="asset-name">{asset.name}</Text>
                        </Panel>
                    </loop>
                </Stack>
            </ScrollView>
        </Stack>
    </Panel>
</template>

<script>
    using gg_editor::ui::widgets::{Panel, Stack, Text, Input, ScrollView, Button, Icon};
    using gg_editor::ui::signal;
    using gg_editor::ui::on_mount;

    let searchQuery = signal("");
    let filteredAssets = signal([]);

    let handleImport = micro() {
        console::log("Import asset...");
    };

    on_mount(micro() {
        console::log("Asset browser mounted");
    });
</script>

<style>
    .asset-browser {
        padding: $spacing-md;
        background-color: $bg-secondary-color;
        border-top: 1px solid $border-color;
    }

    .section-title {
        font-size: $font-size-sm;
        font-weight: bold;
        color: $text-secondary-color;
        text-transform: uppercase;
    }

    .search-input {
        padding: $spacing-xs $spacing-sm;
        background-color: $bg-color;
        border: 1px solid $border-color;
        border-radius: $radius-sm;
        color: $text-color;

        &:focus {
            border-color: $accent-color;
        }
    }

    .btn-sm {
        padding: $spacing-xs $spacing-sm;
        background-color: $primary-color;
        color: white;
        border-radius: $radius-sm;
        font-size: $font-size-sm;
        cursor: pointer;
    }

    .asset-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: $spacing-xs;
        padding: $spacing-sm;
        border-radius: $radius-md;
        cursor: pointer;

        &:hover {
            background-color: $secondary-color;
        }
    }

    .asset-name {
        font-size: $font-size-xs;
        color: $text-color;
        text-align: center;
        max-width: 64px;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
```

### 场景编辑器面板

```vue
<template>
    <Panel class="scene-editor" style="flex-1">
        <Stack orientation="vertical" style="gap-0">
            <Toolbar items={toolItems} style="width-[100%]" />
            <SceneView scene={currentScene()} style="flex-1" />
            <Stack orientation="horizontal" style="gap-8 padding-4 bg-[#2d2d2d]">
                <Text class="status-text">场景: {currentScene()}</Text>
                <Text class="status-text">对象数: {objectCount()}</Text>
                <Text class="status-text">FPS: {fps()}</Text>
            </Stack>
        </Stack>
    </Panel>
</template>

<script>
    using gg_editor::ui::widgets::{Panel, Stack, Text, Toolbar, SceneView};
    using gg_editor::ui::signal;
    using gg_editor::ui::on_mount;

    let currentScene = signal("Main");
    let objectCount = signal(0);
    let fps = signal(60);
    let toolItems = signal([]);

    on_mount(micro() {
        console::log("Scene editor mounted");
    });
</script>

<style>
    .scene-editor {
        background-color: $bg-color;
    }

    .status-text {
        font-size: $font-size-xs;
        color: $text-secondary-color;
    }
</style>
```

## 与 Game UI 的边界

### 核心原则

*.vx 文件仅用于 Editor UI，不可用于游戏运行时。两者使用完全不同的技术体系和渲染架构。

### 格式对照

| 特性 | *.vx（Editor UI） | *.gameui（Game UI） |
|------|-------------------|---------------------|
| 用途 | 编辑器界面开发 | 游戏运行时 UI |
| 技术体系 | DOM + USS + 独立渲染器 | ECS + Canvas |
| 渲染方式 | 独立渲染器，直接在图形设备层工作 | Canvas 渲染，集成到游戏渲染管线 |
| 布局系统 | Flex / Grid | Canvas 坐标系 |
| 样式系统 | USS（主题变量、选择器） | 代码式样式设置 |
| 组件模型 | 声明式组件树 | ECS 实体组件 |
| 交互方式 | 编辑器输入系统 | 游戏输入系统 |
| World Space | 不支持 | 支持 |
| Screen Space | 支持（默认） | 支持 |
| 主题支持 | 亮色/暗色主题自动切换 | 由游戏自行管理 |
| 性能要求 | 跟随编辑器刷新率 | 跟随游戏帧率，需优化至 60fps+ |
| 数据绑定 | 响应式 signal 系统 | ECS 组件数据 |
| 生命周期 | onMount / onUpdate / onCleanup | ECS 系统生命周期 |

### 不可混用规则

1. *.vx 文件中不可引用 *.gameui 组件
2. *.gameui 文件中不可引用 *.vx 组件
3. Editor UI 和 Game UI 不共享渲染上下文
4. Editor UI 的样式变量和主题系统不适用于 Game UI
5. 编辑器专用组件（InspectorPanel、HierarchyView 等）仅在 *.vx 中可用

## 总结

*.vx 文件格式为 GG Editor UI Toolkit 提供了一种统一、直观的方式来开发编辑器界面。通过结合 ValkyrieX 模板、Valkyrie 脚本和 USS 样式系统，开发者可以快速构建出美观、响应式的编辑器界面。*.vx 文件仅用于编辑器界面开发，游戏运行时 UI 请使用 *.gameui 文件格式。
