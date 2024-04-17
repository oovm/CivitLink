# Galgame 格式设计

## 1. 格式概述

Galgame 格式是 GG Game Engine 为 Galgame 引擎专门设计的一种标记语言格式，基于 **MDX（Markdown + JSX）** 语法，用于编写游戏中的对话、剧情和交互内容。Galgame 格式结合了 Markdown 的简洁性和 JSX 组件的可扩展性，提供了一种现代化的游戏剧本编写方式。

## 2. 设计目标

- **简洁易读**：基于 MDX 语法，保持 Markdown 的简洁易读特点
- **组件化**：使用 JSX 组件语法，支持自定义游戏组件
- **易于解析**：由 oak-markdown 框架提供解析支持
- **可扩展性**：支持自定义组件和扩展，满足不同游戏的需求

## 3. 基本语法

### 3.1 Front Matter

文件开头使用 YAML 格式的 front matter 定义元数据：

```yaml
---
title: 章节标题
author: 作者
version: 1.0.0
---
```

### 3.2 导入语句

使用 `<using />` 组件导入模块：

```mdx
<using galgame::components::* />
<using galgame::audio::* />
```

### 3.3 角色对话

使用 `<Dialogue />` 组件：

```mdx
<Dialogue character="女主角" expression="微笑">早上好！今天天气真好啊。</Dialogue>
<Dialogue character="男主角" expression="普通">是啊，很适合出去走走。</Dialogue>
```

### 3.4 场景切换

使用 `<Scene />` 组件：

```mdx
<Scene name="学校操场" bg="images/bg/school_playground.png" />
```

### 3.5 立绘切换

使用 `<Character />` 组件：

```mdx
<Character name="女主角" sprite="images/char/heroine_smile.png" position="left" />
```

### 3.6 选择项

使用 `<Choice />` 和 `<Option />` 组件：

```mdx
<Choice>
  <Option text="打招呼" next="greet" />
  <Option text="离开" next="leave" />
  <Option text="询问" next="ask" />
</Choice>
```

### 3.7 标签

使用 Markdown 标题作为标签：

```mdx
# start

这是游戏的开始。

## greet

你好！

## leave

我先走了。
```

### 3.8 音频控制

使用 `<Audio />` 组件：

```mdx
<Audio bgm="audio/bgm/school.mp3" />
<Audio se="audio/se/hello.wav" />
```

### 3.9 特效

使用 `<Effect />` 组件：

```mdx
<Effect type="fade_in" duration="1.0" />
画面逐渐淡入。
<Effect type="fade_out" duration="1.0" />
```

### 3.10 注释

使用标准 Markdown 注释或 JSX 注释：

```mdx
<!-- 这是注释内容 -->
{/* 这也是注释内容 */}
```

## 4. 高级功能

### 4.1 变量和表达式

使用 JSX 表达式语法：

```mdx
<Dialogue character="女主角">你的名字是 {player_name} 吧？</Dialogue>
```

### 4.2 条件语句

使用 `<If />` 组件：

```mdx
<If condition="flag == true">
  <Dialogue character="女主角">我们之前见过面！</Dialogue>
<Else />
  <Dialogue character="女主角">你好，初次见面。</Dialogue>
</If>
```

### 4.3 循环语句

使用 `<Loop />` 组件：

```mdx
<Loop times="3">
  我已经走了很久了...
</Loop>
```

### 4.4 函数调用

使用 `<Call />` 组件：

```mdx
<Call function="show_message" message="欢迎来到游戏世界！" />
```

## 5. 文件结构

一个 Galgame 文件通常包含以下部分：

1. **Front Matter**：文件顶部的 YAML 元数据
2. **导入语句**：导入所需的组件模块
3. **内容**：主要的对话、剧情和交互内容
4. **标签**：使用 Markdown 标题定义的导航标签

**示例文件结构**：

```mdx
---
title: 第一章：相遇
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />

# start

<Scene name="学校门口" bg="images/bg/school_gate.png" />
<Character name="女主角" sprite="images/char/heroine_normal.png" position="left" />

<Dialogue character="女主角" expression="微笑">早上好！</Dialogue>
<Dialogue character="男主角" expression="普通">早上好！</Dialogue>

<Choice>
  <Option text="打招呼" next="greet" />
  <Option text="离开" next="leave" />
</Choice>

## greet

<Dialogue character="男主角" expression="微笑">你好，我是新来的转学生。</Dialogue>
<Dialogue character="女主角" expression="惊讶">转学生？那我们以后就是同学了！</Dialogue>

## leave

<Dialogue character="男主角" expression="普通">我先走了，再见。</Dialogue>
<Dialogue character="女主角" expression="失落">再见...</Dialogue>
```

## 6. 解析和处理

Galgame 引擎使用 oak-markdown 框架解析 MDX 文件：

1. **词法分析**：oak-markdown lexer 将源码分解为词法单元
2. **语法分析**：构建 Markdown AST
3. **组件处理**：识别 JSX 组件并转换为游戏指令
4. **代码生成**：生成对话系统可执行的指令

## 7. 与其他格式的关系

- **与 Markdown 的关系**：Galgame 格式基于 MDX，完全兼容 Markdown 语法
- **与 XML 格式的关系**：旧版 XML 格式已废弃，建议迁移到 MDX 格式
- **与脚本语言的关系**：Galgame 格式可以与 Valkyrie 脚本结合使用，实现更复杂的游戏逻辑

## 8. 工具支持

Galgame 引擎提供以下 Galgame 工具：

- **Galgame 编辑器**：内置的 Galgame 编辑器，支持语法高亮和实时预览
- **Galgame 解析器**：基于 oak-markdown 的 MDX 解析器
- **Galgame 验证器**：验证 Galgame 文件的语法正确性
- **Galgame 编译器**：将 Galgame 文件编译为优化的格式，提高运行时性能

## 9. 最佳实践

- **使用 Front Matter**：在文件开头定义元数据
- **组件化开发**：使用 `<using />` 导入组件模块
- **合理使用标签**：使用 Markdown 标题组织剧情分支
- **使用注释**：添加注释说明复杂的逻辑和剧情设计
- **测试和验证**：使用 Galgame 验证器检查语法错误
- **版本控制**：将 Galgame 文件纳入版本控制系统，跟踪变更

## 10. 示例

### 基本对话示例

```mdx
---
title: 基本对话示例
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />

# start

这是一个晴朗的早晨，男主角来到了新学校。

终于到了，这就是我的新学校吗？

<Scene name="学校门口" bg="images/bg/school_gate.png" />
<Character name="女主角" sprite="images/char/heroine_smile.png" position="left" />

<Dialogue character="女主角" expression="微笑">你好，你是新来的转学生吗？</Dialogue>
<Dialogue character="男主角" expression="惊讶">是的，你怎么知道？</Dialogue>
<Dialogue character="女主角" expression="微笑">因为我没见过你呀！我是班长，带你去教室吧。</Dialogue>

<Choice>
  <Option text="好的，谢谢你" next="follow" />
  <Option text="不用了，我自己找" next="alone" />
</Choice>

## follow

<Dialogue character="男主角" expression="微笑">好的，谢谢你。</Dialogue>
<Dialogue character="女主角" expression="微笑">不客气，跟我来。</Dialogue>

## alone

<Dialogue character="男主角" expression="普通">不用了，我自己找吧。</Dialogue>
<Dialogue character="女主角" expression="失落">这样啊，那好吧，祝你好运！</Dialogue>
```

### 高级功能示例

```mdx
---
title: 第二章：冒险
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />

# start

<Scene name="森林" bg="images/bg/forest.png" />
男主角进入了神秘的森林。

<If condition="has_key == true">
  <Dialogue character="男主角" expression="兴奋">我有钥匙，可以打开那个门了！</Dialogue>
<Else />
  <Dialogue character="男主角" expression="疑惑">前面有一扇门，但是我没有钥匙。</Dialogue>
</If>

<Loop times="2">
  我已经走了很久了...
</Loop>

前面有一个宝箱！

<Call function="open_chest" chest_id="1" />

<Choice>
  <Option text="打开宝箱" next="open_chest" />
  <Option text="继续前进" next="continue" />
</Choice>

## open_chest

<Dialogue character="男主角" expression="兴奋">宝箱里有一把钥匙！</Dialogue>
<Call function="set_variable" name="has_key" value="true" />

## continue

<Dialogue character="男主角" expression="普通">先继续前进吧，宝箱下次再说。</Dialogue>
```

## 结论

Galgame 格式基于 MDX（Markdown + JSX），结合了 Markdown 的简洁性和 JSX 组件的可扩展性。通过 oak-markdown 框架提供的解析支持，开发者可以更高效地创建和管理游戏内容，为玩家提供更丰富的游戏体验。
