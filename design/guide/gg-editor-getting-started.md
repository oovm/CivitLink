# GG Editor 快速入门

GG Editor 是一个跨平台 GUI 开发框架，支持使用类似 Vue 的 *.widget 格式开发原生 GUI 界面。本指南将帮助你快速上手 GG Editor。

## 安装

1. 确保你已经安装了 Rust 开发环境
2. 克隆 GG 游戏引擎仓库
3. 进入仓库目录并构建项目：

```bash
cargo build
```

## 创建第一个 GUI 项目

### 步骤 1：创建 *.widget 文件

创建一个名为 `hello.vx` 的文件，内容如下：

```vue
<template>
  <Layout style="flex: 1; padding: 16px; background-color: #f0f0f0;">
    <Stack orientation="vertical" style="gap: 12px;">
      <Text style="font-size: 24px; font-weight: bold; color: #4CAF50;">Hello GG Editor</Text>
      <Button text="Click Me" onClick={handleClick} style="padding: 8px 16px; background-color: #2196F3; color: white; border-radius: 4px; cursor: pointer;" />
      <Text>Clicked: {{ count() }} times</Text>
    </Stack>
  </Layout>
</template>

<script>
import { createSignal } from 'valkyrie';

// 响应式状态
const [count, setCount] = createSignal(0);

// 事件处理函数
const handleClick = () => {
  setCount(count() + 1);
};
</script>

<style>
/* 全局样式 */
body {
  font-family: Arial, sans-serif;
  margin: 0;
  padding: 0;
  background-color: #f0f0f0;
}
</style>
```

### 步骤 2：使用 GG Editor 打开文件

1. 运行 GG Editor：

```bash
cargo run --bin gg-editor
```

2. 在编辑器中打开 `hello.vx` 文件

### 步骤 3：预览效果

点击编辑器中的预览按钮，查看 GUI 界面效果。你应该能看到一个包含标题、按钮和计数器的界面。

### 步骤 4：交互测试

点击"Click Me"按钮，观察计数器是否增加。

## 核心概念

### 1. *.widget 文件格式

*.widget 文件包含三个主要部分：

- **<template>**：使用 TSX 语法定义界面结构
- **<script>**：使用 Valkyrie 脚本定义逻辑
- **<style>**：使用 SCSS 语法定义样式

### 2. 响应式状态

使用 `createSignal` 创建响应式状态：

```javascript
const [count, setCount] = createSignal(0);
```

### 3. 事件处理

定义事件处理函数并绑定到组件：

```javascript
const handleClick = () => {
  setCount(count() + 1);
};

// 在模板中绑定
<Button text="Click Me" onClick={handleClick} />
```

### 4. 组件

GG Editor 提供了以下基础组件：

- **Layout**：布局容器，支持 flex 布局
- **Stack**：栈式布局容器
- **Button**：按钮
- **Text**：文本显示
- **Input**：输入框
- **Image**：图片显示
- **Panel**：面板容器
- **ScrollView**：滚动视图

## 进阶功能

### 组件嵌套

你可以嵌套组件来创建复杂的界面：

```tsx
<Layout>
  <Stack orientation="vertical">
    <Text>标题</Text>
    <Panel>
      <Input placeholder="输入文本" />
      <Button text="提交" />
    </Panel>
  </Stack>
</Layout>
```

### 条件渲染

使用条件表达式进行条件渲染：

```tsx
{count() > 5 ? (
  <Text>Count is greater than 5</Text>
) : (
  <Text>Count is less than or equal to 5</Text>
)}
```

### 列表渲染

使用 `map` 函数渲染列表：

```tsx
{items().map((item, index) => (
  <Panel key={index}>
    <Text>{item}</Text>
  </Panel>
))}
```

### 样式

使用 SCSS 语法定义样式：

```scss
.button {
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  
  &:hover {
    opacity: 0.9;
  }
}
```

## 常见问题

### Q: 如何添加自定义组件？

A: 创建一个新的 *.widget 文件，定义组件，然后在其他文件中导入使用：

```javascript
import CustomComponent from './CustomComponent.vx';

// 在模板中使用
<CustomComponent message="Hello" />
```

### Q: 如何与游戏引擎集成？

A: GG Editor 生成的 GUI 可以通过 ECS 架构与游戏引擎集成，访问游戏世界的实体和组件。

### Q: 支持哪些平台？

A: GG Editor 支持 Windows、macOS、iOS、Android、H5、微信小游戏等平台。

## 更多资源

- [*.widget 文件格式规范](../modules/vx-format.md)
- [GG Editor 架构设计](../architecture/overview.md)
- [示例项目](../../examples/gui-basic/)
