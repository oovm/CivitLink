# GG Editor 基础示例

本示例展示了 GG Editor 跨平台 GUI 开发框架的基本使用方法，包括：

- 响应式状态管理
- 事件处理
- 组件布局
- 样式定义

## 示例功能

1. **计数器**：展示响应式状态和事件处理
2. **文本输入**：展示双向数据绑定
3. **项目列表**：展示列表渲染和动态添加项目

## 如何运行

1. 使用 GG Editor 打开 `basic.vx` 文件
2. 点击预览按钮查看效果
3. 尝试与界面交互：
   - 点击"增加"和"减少"按钮调整计数器
   - 在输入框中输入文本
   - 点击"添加项目"按钮向列表添加新项目

## 代码结构

```
basic.vx
├── <template>：使用 TSX 语法定义界面结构
├── <script>：使用 Valkyrie 脚本定义逻辑
└── <style>：使用 SCSS 语法定义样式
```

## 核心概念

### 响应式状态

使用 `createSignal` 创建响应式状态：

```javascript
const [count, setCount] = createSignal(0);
```

### 事件处理

定义事件处理函数并绑定到组件：

```javascript
const handleIncrement = () => {
  setCount(count() + 1);
};

// 在模板中绑定
<Button text="增加" onClick={handleIncrement} />
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

### 样式定义

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

## 更多资源

- [GG Editor 文档](../../../design/modules/vx-format.md)
- [*.vx 文件格式规范](../../../design/modules/vx-format.md)
- [GG Editor 架构设计](../../../design/architecture/overview.md)
