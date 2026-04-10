# MDX 格式设计

## 1. 格式概述

MDX（Markdown eXtended for Galgame）是 GG Game Engine 为 Galgame 引擎专门设计的一种标记语言格式，用于编写游戏中的对话、剧情和交互内容。MDX 基于 Markdown 语法扩展，增加了 Galgame 特有的功能，如角色对话、立绘切换、场景切换、选择项等。

## 2. 设计目标

- **简洁易读**：基于 Markdown 语法，保持简洁易读的特点
- **功能丰富**：支持 Galgame 所需的各种功能，如角色对话、立绘切换、场景切换等
- **易于解析**：设计清晰的语法规则，便于引擎解析和处理
- **可扩展性**：支持自定义扩展，满足不同游戏的需求

## 3. 基本语法

### 3.1 角色对话

```mdx
[角色名称]{表情} 对话内容
```

**示例**：
```mdx
[女主角]{微笑} 早上好！今天天气真好啊。
[男主角]{普通} 是啊，很适合出去走走。
```

### 3.2 场景切换

```mdx
::scene{name="场景名称" bg="背景图片路径"}
```

**示例**：
```mdx
::scene{name="学校操场" bg="images/bg/school_playground.png"}
```

### 3.3 立绘切换

```mdx
::character{name="角色名称" sprite="立绘路径" position="位置"}
```

**示例**：
```mdx
::character{name="女主角" sprite="images/char/heroine_smile.png" position="left"}
```

### 3.4 选择项

```mdx
::choice{
  option="选项1" next="标签1"
  option="选项2" next="标签2"
  option="选项3" next="标签3"
}
```

**示例**：
```mdx
::choice{
  option="打招呼" next="greet"
  option="离开" next="leave"
  option="询问" next="ask"
}
```

### 3.5 标签

```mdx
#tag{标签名称}
```

**示例**：
```mdx
#tag{start}
[旁白] 这是游戏的开始。

#tag{greet}
[男主角]{微笑} 你好！

#tag{leave}
[男主角]{普通} 我先走了。

#tag{ask}
[男主角]{疑惑} 请问你是？
```

### 3.6 音频控制

```mdx
::audio{bgm="背景音乐路径"}
::audio{se="音效路径"}
```

**示例**：
```mdx
::audio{bgm="audio/bgm/school.mp3"}
[女主角]{微笑} 早上好！
::audio{se="audio/se/hello.wav"}
```

### 3.7 特效

```mdx
::effect{type="特效类型" duration="持续时间"}
```

**示例**：
```mdx
::effect{type="fade_in" duration="1.0"}
[旁白] 画面逐渐淡入。
::effect{type="fade_out" duration="1.0"}
```

### 3.8 注释

```mdx
%% 这是注释内容 %%
```

**示例**：
```mdx
%% 对话场景开始 %%
[女主角]{微笑} 早上好！
[男主角]{普通} 早上好！
%% 对话场景结束 %%
```

## 4. 高级功能

### 4.1 变量和表达式

```mdx
[角色名称]{表情} 变量值：${变量名}
```

**示例**：
```mdx
[女主角]{微笑} 你的名字是 ${player_name} 吧？
```

### 4.2 条件语句

```mdx
::if{condition="条件表达式"}
  条件为真时的内容
::else
  条件为假时的内容
::endif
```

**示例**：
```mdx
::if{condition="flag == true"}
  [女主角]{微笑} 我们之前见过面！
::else
  [女主角]{普通} 你好，初次见面。
::endif
```

### 4.3 循环语句

```mdx
::loop{times="循环次数"}
  循环内容
::endloop
```

**示例**：
```mdx
::loop{times="3"}
  [男主角]{疲劳} 我已经走了很久了...
::endloop
```

### 4.4 函数调用

```mdx
::call{function="函数名" param1="参数1" param2="参数2"}
```

**示例**：
```mdx
::call{function="show_message" message="欢迎来到游戏世界！"}
```

## 5. 文件结构

一个 MDX 文件通常包含以下部分：

1. **元数据**：文件顶部的元数据，如标题、作者、版本等
2. **内容**：主要的对话、剧情和交互内容
3. **标签**：用于导航和分支的标签

**示例文件结构**：

```mdx
---
title: "第一章：相遇"
author: "灵之镜工作室"
version: "1.0.0"
---

#tag{start}
::scene{name="学校门口" bg="images/bg/school_gate.png"}
::character{name="女主角" sprite="images/char/heroine_normal.png" position="left"}

[女主角]{微笑} 早上好！
[男主角]{普通} 早上好！

::choice{
  option="打招呼" next="greet"
  option="离开" next="leave"
}

#tag{greet}
[男主角]{微笑} 你好，我是新来的转学生。
[女主角]{惊讶} 转学生？那我们以后就是同学了！

#tag{leave}
[男主角]{普通} 我先走了，再见。
[女主角]{失落} 再见...
```

## 6. 解析和处理

Galgame 引擎的 MDX 解析器会将 MDX 文件解析为内部的数据结构，然后由对话系统处理。解析过程包括：

1. **词法分析**：将 MDX 文本分解为词法单元
2. **语法分析**：根据语法规则构建语法树
3. **语义分析**：处理变量、表达式和函数调用
4. **代码生成**：生成对话系统可执行的指令

## 7. 与其他格式的关系

- **与 Markdown 的关系**：MDX 基于 Markdown 语法扩展，保持了 Markdown 的简洁易读性
- **与 JSON/YAML 的关系**：MDX 比 JSON/YAML 更适合编写对话和剧情内容，更易于阅读和编辑
- **与脚本语言的关系**：MDX 可以与 Valkyrie 脚本结合使用，实现更复杂的游戏逻辑

## 8. 工具支持

Galgame 引擎提供以下 MDX 工具：

- **MDX 编辑器**：内置的 MDX 编辑器，支持语法高亮和实时预览
- **MDX 解析器**：将 MDX 文件解析为引擎可处理的数据结构
- **MDX 验证器**：验证 MDX 文件的语法正确性
- **MDX 编译器**：将 MDX 文件编译为优化的格式，提高运行时性能

## 9. 最佳实践

- **保持简洁**：使用简洁的语法，避免过于复杂的结构
- **合理使用标签**：使用标签组织剧情分支，便于导航和管理
- **使用注释**：添加注释说明复杂的逻辑和剧情设计
- **测试和验证**：使用 MDX 验证器检查语法错误
- **版本控制**：将 MDX 文件纳入版本控制系统，跟踪变更

## 10. 示例

### 基本对话示例

```mdx
[旁白] 这是一个晴朗的早晨，男主角来到了新学校。

[男主角]{普通} 终于到了，这就是我的新学校吗？

::scene{name="学校门口" bg="images/bg/school_gate.png"}
::character{name="女主角" sprite="images/char/heroine_smile.png" position="left"}

[女主角]{微笑} 你好，你是新来的转学生吗？
[男主角]{惊讶} 是的，你怎么知道？
[女主角]{微笑} 因为我没见过你呀！我是班长，带你去教室吧。

::choice{
  option="好的，谢谢你" next="follow"
  option="不用了，我自己找" next="alone"
}

#tag{follow}
[男主角]{微笑} 好的，谢谢你。
[女主角]{微笑} 不客气，跟我来。

#tag{alone}
[男主角]{普通} 不用了，我自己找吧。
[女主角]{失落} 这样啊，那好吧，祝你好运！
```

### 高级功能示例

```mdx
---
title: "第二章：冒险"
author: "灵之镜工作室"
version: "1.0.0"
---

#tag{start}
::scene{name="森林" bg="images/bg/forest.png"}
[旁白] 男主角进入了神秘的森林。

::if{condition="has_key == true"}
  [男主角]{兴奋} 我有钥匙，可以打开那个门了！
::else
  [男主角]{疑惑} 前面有一扇门，但是我没有钥匙。
::endif

::loop{times="2"}
  [男主角]{疲劳} 我已经走了很久了...
::endloop

[男主角]{惊讶} 前面有一个宝箱！
::call{function="open_chest" chest_id="1"}

::choice{
  option="打开宝箱" next="open_chest"
  option="继续前进" next="continue"
}

#tag{open_chest}
[男主角]{兴奋} 宝箱里有一把钥匙！
::call{function="set_variable" name="has_key" value="true"}

#tag{continue}
[男主角]{普通} 先继续前进吧，宝箱下次再说。
```

## 11. 结论

MDX 格式是 GG Game Engine 为 Galgame 引擎专门设计的一种标记语言格式，基于 Markdown 语法扩展，增加了 Galgame 特有的功能。MDX 格式简洁易读，功能丰富，易于解析和处理，为 Galgame 游戏的对话、剧情和交互内容提供了一种理想的编写方式。通过 MDX 格式，开发者可以更高效地创建和管理游戏内容，为玩家提供更丰富的游戏体验。