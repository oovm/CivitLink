# Galgame DSL 语法对比

本文档对比了多种常用的 Galgame DSL 语法风格，帮助选择最适合的脚本格式。

## 对比概览

| 特性    | MDX    | TeX    | Ren'Py | KAG   | Ink  | YAML  |
| ----- | ------ | ------ | ------ | ----- | ---- | ----- |
| 学习曲线  | 低      | 中      | 低      | 中     | 低    | 低     |
| 可读性   | ⭐⭐⭐⭐⭐  | ⭐⭐⭐⭐   | ⭐⭐⭐⭐   | ⭐⭐⭐   | ⭐⭐⭐⭐ | ⭐⭐⭐   |
| 灵活性   | ⭐⭐⭐⭐⭐  | ⭐⭐⭐⭐   | ⭐⭐⭐⭐⭐  | ⭐⭐⭐⭐  | ⭐⭐⭐⭐ | ⭐⭐⭐   |
| 工具支持  | 中      | 低      | ⭐⭐⭐⭐⭐  | ⭐⭐⭐⭐  | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 行业采用  | 新兴     | 新兴     | 主流     | 日本主流  | 叙事游戏 | 数据驱动  |
| 编辑器支持 | VSCode | VSCode | 专用IDE  | 专用编辑器 | Inky | 通用    |

***

## 1. MDX 风格

**特点**：Markdown + JSX，现代化、易读易写

```mdx
---
title: 星光学院的秘密
---

<using galgame::components::* />

# prologue

<Scene bg="images/bg/school.png" transition="fade" />

星光学院，这所历史悠久的名校...

<Dialogue character="sakura">早上好！</Dialogue>

<Choice>
  <Option text="打招呼" next="greet" />
  <Option text="离开" next="leave" />
</Choice>
```

**优点**：

- ✅ Markdown 基础，学习成本低
- ✅ 组件化，可扩展性强
- ✅ 与现代前端生态兼容
- ✅ 支持条件渲染、循环等高级特性

**缺点**：

- ❌ 新兴格式，工具支持较少
- ❌ 需要自定义解析器

**适用场景**：现代游戏引擎、Web 端 Galgame

***

## 2. TeX 风格

**特点**：借鉴 LaTeX 语法，结构化、专业感强

```tex
\documentclass{galgame}

\usepackage{galgame/components}

\begin{document}

\chapter{prologue}

\scene{bg="images/bg/school.png", transition="fade"}

星光学院，这所历史悠久的名校...

\dialogue{character=sakura}{早上好！}

\begin{choice}
  \option{text="打招呼", next="greet"}
  \option{text="离开", next="leave"}
\end{choice}

\end{document}
```

**优点**：

- ✅ 结构清晰，文档感强
- ✅ 宏定义支持，可扩展
- ✅ 适合复杂文档结构
- ✅ 学术/专业风格

**缺点**：

- ❌ 语法相对复杂
- ❌ 学习曲线较陡
- ❌ 需要专门的解析器

**适用场景**：专业级视觉小说、需要复杂文档结构的游戏

***

## 3. Ren'Py 风格

**特点**：Python 风格，最流行的视觉小说引擎

```python
label prologue:
    scene bg school_gate with fade(1.5)
    
    "星光学院，这所历史悠久的名校..."
    
    show sakura smile at left with slide_left
    
    sakura "早上好！"
    
    menu:
        "打招呼":
            $ sakura_affection += 1
            jump greet
        "离开":
            jump leave
```

**优点**：

- ✅ 行业标准，社区庞大
- ✅ 完整的 IDE 支持
- ✅ Python 语法，功能强大
- ✅ 丰富的文档和教程

**缺点**：

- ❌ 需要学习 Ren'Py 特定语法
- ❌ 与 Ren'Py 引擎绑定
- ❌ Python 环境依赖

**适用场景**：独立游戏开发、视觉小说制作

***

## 4. KAG/Kirikiri 风格

**特点**：日本主流，标签式语法

```
*prologue

@bg storage="images/bg/school.png" time=1500 method="crossfade"

星光学院，这所历史悠久的名校。[l][r]

@show_char name="sakura" sprite="smile" x=100 y=0

樱井美咲「早上好！」[l][r]

*branch
@link target="greet"「打招呼」[r]
@link target="leave"「离开」[r]
@s
```

**优点**：

- ✅ 日本行业标准
- ✅ 标签式语法，结构清晰
- ✅ 成熟的工具链
- ✅ 丰富的商业案例

**缺点**：

- ❌ 语法相对老旧
- ❌ 英文文档较少
- ❌ 需要专用引擎

**适用场景**：日式视觉小说、商业 Galgame

***

## 5. Ink 风格

**特点**：叙事导向，简洁优雅

```
== prologue ==

~ scene.change("images/bg/school.png", "fade", 1.5)

星光学院，这所历史悠久的名校。

樱井美咲：早上好！

*   [打招呼]
        ~ sakura_affection++
        -> greet
*   [离开]
        -> leave
```

**优点**：

- ✅ 叙事优先设计
- ✅ 语法简洁优雅
- ✅ 条件分支自然
- ✅ Inky 编辑器支持

**缺点**：

- ❌ 主要面向叙事游戏
- ❌ Galgame 特性支持有限
- ❌ 需要适配层

**适用场景**：互动叙事、文字冒险游戏

***

## 6. YAML 风格

**特点**：数据驱动，结构化配置

```yaml
nodes:
  prologue:
    type: scene
    background: images/bg/school.png
    transition:
      type: fade
      duration: 1.5
    next: prologue_text

  prologue_text:
    type: text
    content: 星光学院，这所历史悠久的名校...
    next: sakura_greeting

  sakura_greeting:
    type: dialogue
    character: sakura
    content: 早上好！
    next: first_choice

  first_choice:
    type: choice
    options:
      - text: 打招呼
        next: greet
        effects:
          - variable: sakura_affection
            operation: add
            value: 1
```

**优点**：

- ✅ 数据驱动，易于解析
- ✅ 与任何编程语言兼容
- ✅ 版本控制友好
- ✅ 易于生成和处理

**缺点**：

- ❌ 可读性较差
- ❌ 编写效率低
- ❌ 不适合手写

**适用场景**：工具生成、数据迁移、后端处理

***

## 语法对比：相同场景

### 对话

| DSL    | 语法                                                       |
| ------ | -------------------------------------------------------- |
| MDX    | `<Dialogue character="sakura">早上好！</Dialogue>`           |
| TeX    | `\dialogue{character=sakura}{早上好！}`                      |
| Ren'Py | `sakura "早上好！"`                                          |
| KAG    | `樱井美咲「早上好！」`                                             |
| Ink    | `樱井美咲：早上好！`                                              |
| YAML   | `type: dialogue` + `character: sakura` + `content: 早上好！` |

### 选择分支

| DSL    | 语法                                                  |
| ------ | --------------------------------------------------- |
| MDX    | `<Choice><Option text="..." next="..." /></Choice>` |
| TeX    | `\begin{choice}\option{...}\end{choice}`            |
| Ren'Py | `menu:` + `"...": jump ...`                         |
| KAG    | `@link target="..."「...」`                           |
| Ink    | `* [选项] -> 跳转`                                      |
| YAML   | `type: choice` + `options: [...]`                   |

### 变量操作

| DSL    | 语法                                                   |
| ------ | ---------------------------------------------------- |
| MDX    | `<SetVariable name="x" value="{x + 1}" />`           |
| TeX    | `\set{x}{x + 1}}`                                    |
| Ren'Py | `$ x += 1`                                           |
| KAG    | `@eval exp="f.x += 1"`                               |
| Ink    | `~ x++`                                              |
| YAML   | `effects: [{variable: x, operation: add, value: 1}]` |

### 条件判断

| DSL    | 语法                                                   |
| ------ | ---------------------------------------------------- |
| MDX    | `<If condition="x >= 5">...</If>`                    |
| TeX    | `\ifthenelse{\expr{x >= 5}}{...}{}`                  |
| Ren'Py | `if x >= 5:`                                         |
| KAG    | `@if exp="f.x >= 5"`                                 |
| Ink    | `{x >= 5: ...}`                                      |
| YAML   | `condition: {variable: x, operator: ">=", value: 5}` |

***

## 推荐选择

### 对于新手

推荐 **MDX** 或 **Ren'Py**：

- MDX：Markdown 基础，学习成本最低
- Ren'Py：社区庞大，教程丰富

### 对于专业团队

推荐 **Ren'Py** 或 **KAG**：

- Ren'Py：行业标准，工具完善
- KAG：日本商业游戏首选

### 对于技术团队

推荐 **MDX** 或 **TeX**：

- MDX：现代技术栈，可扩展
- TeX：结构化，适合复杂项目

### 对于数据驱动项目

推荐 **YAML**：

- 易于解析和处理
- 与后端系统集成方便

***

## 国际化支持对比

| 特性 | MDX | TeX | Ren'Py | KAG | Ink | YAML |
|-----|-----|-----|--------|-----|-----|------|
| 语言文件分离 | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅ |
| 内联多语言 | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ | ❌ |
| 文本键引用 | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ |
| 角色名国际化 | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ✅ |
| 变量名国际化 | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ |

### TeX 国际化语法

```tex
% 方式一：文本键引用（推荐）
\t{prologue.intro}

% 方式二：内联多语言
\s \t{zh:早上好！, en:Good morning!, ja:おはよう！}

% 方式三：语言环境块
\begin{lang}{zh-CN}
  \s 早上好！
\end{lang}
```

### 语言文件结构

```
locales/
  zh-CN/
    main.json      # 中文文本
    characters.json # 角色名称
  en-US/
    main.json      # 英文文本
    characters.json
  ja-JP/
    main.json      # 日文文本
    characters.json
```

---

## 结论

| 使用场景       | 推荐格式      |
| ---------- | --------- |
| 新手入门       | MDX       |
| 独立游戏开发     | Ren'Py    |
| 商业 Galgame | KAG       |
| 现代游戏引擎     | MDX / TeX |
| 互动叙事       | Ink       |
| 数据驱动       | YAML      |

**最终建议**：GG Game Engine 采用 **MDX** 作为主要格式，同时支持 **TeX** 作为备选，因为：

1. MDX 学习成本低，易于上手
2. 组件化设计，扩展性强
3. 与现代前端生态兼容
4. TeX 风格适合需要更严格结构的项目

