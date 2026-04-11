# Galgame 格式设计

## 1. 格式概述

Galgame 格式是 GG Game Engine 为 Galgame 引擎专门设计的一种标记语言格式，用于编写游戏中的对话、剧情和交互内容。Galgame 格式使用 XML 语法，增加了 Galgame 特有的功能，如角色对话、立绘切换、场景切换、选择项等。

## 2. 设计目标

- **简洁易读**：基于 XML 语法，保持简洁易读的特点
- **功能丰富**：支持 Galgame 所需的各种功能，如角色对话、立绘切换、场景切换等
- **易于解析**：设计清晰的语法规则，便于引擎解析和处理
- **可扩展性**：支持自定义扩展，满足不同游戏的需求

## 3. 基本语法

### 3.1 角色对话

```xml
<dialogue character="角色名称" expression="表情">对话内容</dialogue>
```

**示例**：
```xml
<dialogue character="女主角" expression="微笑">早上好！今天天气真好啊。</dialogue>
<dialogue character="男主角" expression="普通">是啊，很适合出去走走。</dialogue>
```

### 3.2 场景切换

```xml
<scene name="场景名称" bg="背景图片路径" />
```

**示例**：
```xml
<scene name="学校操场" bg="images/bg/school_playground.png" />
```

### 3.3 立绘切换

```xml
<character name="角色名称" sprite="立绘路径" position="位置" />
```

**示例**：
```xml
<character name="女主角" sprite="images/char/heroine_smile.png" position="left" />
```

### 3.4 选择项

```xml
<choice>
  <option next="标签1">选项1</option>
  <option next="标签2">选项2</option>
  <option next="标签3">选项3</option>
</choice>
```

**示例**：
```xml
<choice>
  <option next="greet">打招呼</option>
  <option next="leave">离开</option>
  <option next="ask">询问</option>
</choice>
```

### 3.5 标签

```xml
<tag name="标签名称" />
```

**示例**：
```xml
<tag name="start" />
<dialogue character="旁白" expression="">这是游戏的开始。</dialogue>

<tag name="greet" />
<dialogue character="男主角" expression="微笑">你好！</dialogue>

<tag name="leave" />
<dialogue character="男主角" expression="普通">我先走了。</dialogue>

<tag name="ask" />
<dialogue character="男主角" expression="疑惑">请问你是？</dialogue>
```

### 3.6 音频控制

```xml
<audio bgm="背景音乐路径" />
<audio se="音效路径" />
```

**示例**：
```xml
<audio bgm="audio/bgm/school.mp3" />
<dialogue character="女主角" expression="微笑">早上好！</dialogue>
<audio se="audio/se/hello.wav" />
```

### 3.7 特效

```xml
<effect type="特效类型" duration="持续时间" />
```

**示例**：
```xml
<effect type="fade_in" duration="1.0" />
<dialogue character="旁白" expression="">画面逐渐淡入。</dialogue>
<effect type="fade_out" duration="1.0" />
```

### 3.8 注释

```xml
<!-- 这是注释内容 -->
```

**示例**：
```xml
<!-- 对话场景开始 -->
<dialogue character="女主角" expression="微笑">早上好！</dialogue>
<dialogue character="男主角" expression="普通">早上好！</dialogue>
<!-- 对话场景结束 -->
```

## 4. 高级功能

### 4.1 变量和表达式

```xml
<dialogue character="角色名称" expression="表情">变量值：${变量名}</dialogue>
```

**示例**：
```xml
<dialogue character="女主角" expression="微笑">你的名字是 ${player_name} 吧？</dialogue>
```

### 4.2 条件语句

```xml
<if condition="条件表达式">
  <then>
    条件为真时的内容
  </then>
  <else>
    条件为假时的内容
  </else>
</if>
```

**示例**：
```xml
<if condition="flag == true">
  <then>
    <dialogue character="女主角" expression="微笑">我们之前见过面！</dialogue>
  </then>
  <else>
    <dialogue character="女主角" expression="普通">你好，初次见面。</dialogue>
  </else>
</if>
```

### 4.3 循环语句

```xml
<loop times="循环次数">
  循环内容
</loop>
```

**示例**：
```xml
<loop times="3">
  <dialogue character="男主角" expression="疲劳">我已经走了很久了...</dialogue>
</loop>
```

### 4.4 函数调用

```xml
<call function="函数名" param1="参数1" param2="参数2" />
```

**示例**：
```xml
<call function="show_message" message="欢迎来到游戏世界！" />
```

## 5. 文件结构

一个 Galgame 文件通常包含以下部分：

1. **元数据**：文件顶部的元数据，如标题、作者、版本等
2. **内容**：主要的对话、剧情和交互内容
3. **标签**：用于导航和分支的标签

**示例文件结构**：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<galgame>
  <metadata>
    <title>第一章：相遇</title>
    <author>灵之镜工作室</author>
    <version>1.0.0</version>
  </metadata>
  
  <content>
    <tag name="start" />
    <scene name="学校门口" bg="images/bg/school_gate.png" />
    <character name="女主角" sprite="images/char/heroine_normal.png" position="left" />
    
    <dialogue character="女主角" expression="微笑">早上好！</dialogue>
    <dialogue character="男主角" expression="普通">早上好！</dialogue>
    
    <choice>
      <option next="greet">打招呼</option>
      <option next="leave">离开</option>
    </choice>
    
    <tag name="greet" />
    <dialogue character="男主角" expression="微笑">你好，我是新来的转学生。</dialogue>
    <dialogue character="女主角" expression="惊讶">转学生？那我们以后就是同学了！</dialogue>
    
    <tag name="leave" />
    <dialogue character="男主角" expression="普通">我先走了，再见。</dialogue>
    <dialogue character="女主角" expression="失落">再见...</dialogue>
  </content>
</galgame>
```

## 6. 解析和处理

Galgame 引擎的 Galgame 解析器会将 Galgame 文件解析为内部的数据结构，然后由对话系统处理。解析过程包括：

1. **词法分析**：将 Galgame 文本分解为词法单元
2. **语法分析**：根据语法规则构建语法树
3. **语义分析**：处理变量、表达式和函数调用
4. **代码生成**：生成对话系统可执行的指令

## 7. 与其他格式的关系

- **与 Markdown 的关系**：Galgame 格式不基于 Markdown 语法，而是使用 XML 语法，保持了简洁易读性
- **与 JSON/YAML 的关系**：Galgame 格式比 JSON/YAML 更适合编写对话和剧情内容，更易于阅读和编辑
- **与脚本语言的关系**：Galgame 格式可以与 Valkyrie 脚本结合使用，实现更复杂的游戏逻辑

## 8. 工具支持

Galgame 引擎提供以下 Galgame 工具：

- **Galgame 编辑器**：内置的 Galgame 编辑器，支持语法高亮和实时预览
- **Galgame 解析器**：将 Galgame 文件解析为引擎可处理的数据结构
- **Galgame 验证器**：验证 Galgame 文件的语法正确性
- **Galgame 编译器**：将 Galgame 文件编译为优化的格式，提高运行时性能

## 9. 最佳实践

- **保持简洁**：使用简洁的语法，避免过于复杂的结构
- **合理使用标签**：使用标签组织剧情分支，便于导航和管理
- **使用注释**：添加注释说明复杂的逻辑和剧情设计
- **测试和验证**：使用 Galgame 验证器检查语法错误
- **版本控制**：将 Galgame 文件纳入版本控制系统，跟踪变更

## 10. 示例

### 基本对话示例

```xml
<?xml version="1.0" encoding="UTF-8"?>
<galgame>
  <content>
    <dialogue character="旁白" expression="">这是一个晴朗的早晨，男主角来到了新学校。</dialogue>
    
    <dialogue character="男主角" expression="普通">终于到了，这就是我的新学校吗？</dialogue>
    
    <scene name="学校门口" bg="images/bg/school_gate.png" />
    <character name="女主角" sprite="images/char/heroine_smile.png" position="left" />
    
    <dialogue character="女主角" expression="微笑">你好，你是新来的转学生吗？</dialogue>
    <dialogue character="男主角" expression="惊讶">是的，你怎么知道？</dialogue>
    <dialogue character="女主角" expression="微笑">因为我没见过你呀！我是班长，带你去教室吧。</dialogue>
    
    <choice>
      <option next="follow">好的，谢谢你</option>
      <option next="alone">不用了，我自己找</option>
    </choice>
    
    <tag name="follow" />
    <dialogue character="男主角" expression="微笑">好的，谢谢你。</dialogue>
    <dialogue character="女主角" expression="微笑">不客气，跟我来。</dialogue>
    
    <tag name="alone" />
    <dialogue character="男主角" expression="普通">不用了，我自己找吧。</dialogue>
    <dialogue character="女主角" expression="失落">这样啊，那好吧，祝你好运！</dialogue>
  </content>
</galgame>
```

### 高级功能示例

```xml
<?xml version="1.0" encoding="UTF-8"?>
<galgame>
  <metadata>
    <title>第二章：冒险</title>
    <author>灵之镜工作室</author>
    <version>1.0.0</version>
  </metadata>
  
  <content>
    <tag name="start" />
    <scene name="森林" bg="images/bg/forest.png" />
    <dialogue character="旁白" expression="">男主角进入了神秘的森林。</dialogue>
    
    <if condition="has_key == true">
      <then>
        <dialogue character="男主角" expression="兴奋">我有钥匙，可以打开那个门了！</dialogue>
      </then>
      <else>
        <dialogue character="男主角" expression="疑惑">前面有一扇门，但是我没有钥匙。</dialogue>
      </else>
    </if>
    
    <loop times="2">
      <dialogue character="男主角" expression="疲劳">我已经走了很久了...</dialogue>
    </loop>
    
    <dialogue character="男主角" expression="惊讶">前面有一个宝箱！</dialogue>
    <call function="open_chest" chest_id="1" />
    
    <choice>
      <option next="open_chest">打开宝箱</option>
      <option next="continue">继续前进</option>
    </choice>
    
    <tag name="open_chest" />
    <dialogue character="男主角" expression="兴奋">宝箱里有一把钥匙！</dialogue>
    <call function="set_variable" name="has_key" value="true" />
    
    <tag name="continue" />
    <dialogue character="男主角" expression="普通">先继续前进吧，宝箱下次再说。</dialogue>
  </content>
</galgame>
```

## 11. 结论

Galgame 格式是 GG Game Engine 为 Galgame 引擎专门设计的一种标记语言格式，使用 XML 语法，增加了 Galgame 特有的功能。Galgame 格式简洁易读，功能丰富，易于解析和处理，为 Galgame 游戏的对话、剧情和交互内容提供了一种理想的编写方式。通过 Galgame 格式，开发者可以更高效地创建和管理游戏内容，为玩家提供更丰富的游戏体验。