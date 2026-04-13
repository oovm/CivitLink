# Galgame 国际化工作流

## 设计理念

**核心原则：编剧不需要关心翻译，直接用源语言写剧本。**

---

## 工作流程

### 1. 编剧阶段

编剧用源语言（如中文）直接写剧本，不需要关心翻译：

```tex
\begin{document}

\scene{bg="school_gate.png"}

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。

\char{sakura}[smile, left]

\s 新同学！你在看什么呢？

\p.surprised 啊，樱井同学...没什么，只是在发呆。

\choice
  \opt[+s]{好啊，我去看看} -> accept
  \opt[-s]{抱歉，我还有事} -> decline
\endchoice

\end{document}
```

### 2. 提取阶段

运行提取工具，自动生成翻译模板：

```bash
# 提取所有可翻译文本
galgame extract scripts/*.tex -o locales/zh-CN/

# 生成 POT 模板（用于创建新语言）
galgame extract scripts/*.tex -o locales/template.pot
```

生成的翻译文件：

```po
# locales/zh-CN/main.po

#. ID: main.tex:scene[0].narration[0]#a3b2c1d0
msgid "星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。"
msgstr ""

#. ID: main.tex:scene[0].dialogue[2]#e4f5g6h7
msgctxt "sakura"
msgid "新同学！你在看什么呢？"
msgstr ""

#. ID: main.tex:scene[0].choice[0].option[0]#i8j9k0l1
msgid "好啊，我去看看"
msgstr ""

#. ID: main.tex:scene[0].choice[0].option[1]#m2n3o4p5
msgid "抱歉，我还有事"
msgstr ""
```

### 3. 翻译阶段

翻译团队编辑翻译文件，不需要触碰剧本：

```po
# locales/en-US/main.po

#. ID: main.tex:scene[0].narration[0]#a3b2c1d0
msgid "星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。"
msgstr "Star Academy, a historic school, hides secrets unknown to all."

#. ID: main.tex:scene[0].dialogue[2]#e4f5g6h7
msgctxt "sakura"
msgid "新同学！你在看什么呢？"
msgstr "New student! What are you looking at?"

#. ID: main.tex:scene[0].choice[0].option[0]#i8j9k0l1
msgid "好啊，我去看看"
msgstr "Sure, I'll check it out"

#. ID: main.tex:scene[0].choice[0].option[1]#m2n3o4p5
msgid "抱歉，我还有事"
msgstr "Sorry, I have something else"
```

### 4. 构建阶段

构建时指定语言，自动合并翻译：

```bash
# 构建中文版本
galgame build --locale zh-CN

# 构建英文版本
galgame build --locale en-US

# 构建日文版本
galgame build --locale ja-JP
```

---

## 节点 ID 设计

### 为什么不用行号？

行号在剧本修改后会变化，导致翻译引用失效：

```
❌ 行号方式：
#: main.tex:15        # 在第 15 行插入内容后，这里变成 16 行
msgid "星光学院..."
msgstr "..."

✅ 节点 ID 方式：
#. ID: main.tex:scene[0].narration[0]#a3b2c1d0
msgid "星光学院..."
msgstr "..."
```

### ID 格式

```
<文件路径>:<结构路径>#<内容指纹>
```

| 组成部分 | 说明 | 示例 |
|---------|------|------|
| 文件路径 | 源文件相对路径 | `main.tex` |
| 结构路径 | 节点在 AST 中的位置 | `scene[0].dialogue[2]` |
| 内容指纹 | 文本内容的短哈希（前 8 位） | `#a3b2c1d0` |

### 结构路径规则

```
scene[0]              第 1 个场景
scene[0].narration[0] 第 1 个场景的第 1 段叙述
scene[0].dialogue[2]  第 1 个场景的第 3 段对话
scene[0].choice[0]    第 1 个场景的第 1 个选择
scene[0].choice[0].option[1]  第 1 个选择的第 2 个选项
```

### 内容指纹

使用 SHA-256 哈希的前 8 位：

```
文本: "新同学！你在看什么呢？"
上下文: "sakura"
指纹: sha256("sakura:新同学！你在看什么呢？")[:8] = "e4f5g6h7"
```

---

## 增量更新策略

当剧本修改后，使用智能匹配保留已有翻译：

### 匹配优先级

```
1. 精确匹配：ID 完全相同
   → 直接保留翻译

2. 指纹匹配：指纹相同，路径变化
   → 更新路径，保留翻译
   → 示例：scene[0].dialogue[2] → scene[0].dialogue[3]

3. 模糊匹配：路径相似，文本相似度 > 80%
   → 标记为 fuzzy，保留翻译供参考

4. 新增条目：无法匹配
   → msgstr 为空，等待翻译

5. 废弃条目：旧 ID 不在新提取中
   → 标记为 obsolete，可手动清理
```

### 更新示例

```bash
# 提取新文本，保留已有翻译
galgame extract --update scripts/*.tex -o locales/en-US/

# 输出：
# ✓ 精确匹配: 45 条
# ↻ 路径更新: 3 条
# ? 模糊匹配: 2 条（需人工确认）
# + 新增: 5 条
# - 废弃: 1 条
```

### 场景演示

**原始剧本：**
```tex
\scene{bg="school.png"}

星光学院，历史悠久的名校。

\s 你好！
```

**翻译文件：**
```po
#. ID: main.tex:scene[0].narration[0]#abc12345
msgid "星光学院，历史悠久的名校。"
msgstr "Star Academy, a historic school."

#. ID: main.tex:scene[0].dialogue[0]#def67890
msgctxt "sakura"
msgid "你好！"
msgstr "Hello!"
```

**修改后剧本（插入内容）：**
```tex
\scene{bg="school.png"}

阳光明媚的早晨。

星光学院，历史悠久的名校。

\s 你好！
```

**增量更新结果：**
```po
#. ID: main.tex:scene[0].narration[0]#xyz11111  # 新增
msgid "阳光明媚的早晨。"
msgstr ""  # 等待翻译

#. ID: main.tex:scene[0].narration[1]#abc12345  # 路径更新，指纹匹配
msgid "星光学院，历史悠久的名校。"
msgstr "Star Academy, a historic school."  # 保留翻译

#. ID: main.tex:scene[0].dialogue[0]#def67890  # 精确匹配
msgctxt "sakura"
msgid "你好！"
msgstr "Hello!"  # 保留翻译
```

---

## 文件结构

```
project/
├── scripts/
│   ├── main.tex              # 剧本文件（源语言）
│   ├── chapter1_school.tex
│   └── endings.tex
│
├── locales/
│   ├── zh-CN/                # 中文（源语言，可空或用于校对）
│   │   └── main.po
│   ├── en-US/                # 英文翻译
│   │   └── main.po
│   ├── ja-JP/                # 日文翻译
│   │   └── main.po
│   └── template.pot          # 翻译模板
│
└── build/
    ├── zh-CN/                # 中文构建输出
    ├── en-US/                # 英文构建输出
    └── ja-JP/                # 日文构建输出
```

---

## 提取规则

### 自动提取的内容

| 类型 | 示例 | 提取内容 |
|-----|------|---------|
| 叙述文本 | `星光学院...` | ✅ 提取 |
| 对话 | `\s 新同学！` | ✅ 提取 |
| 选择选项 | `\opt{好啊}` | ✅ 提取 |
| 信件内容 | `\begin{letter}...\end{letter}` | ✅ 提取 |
| 角色名 | `\newcharacter{sakura}{樱井美咲}` | ✅ 提取 |
| 变量名 | `\newvariable{sakura_affection}{好感度}` | ✅ 提取 |

### 不提取的内容

| 类型 | 示例 | 原因 |
|-----|------|------|
| 标签名 | `\label{start}` | 内部标识符 |
| 文件路径 | `bg="school.png"` | 资源引用 |
| 变量操作 | `\set{x}{1}` | 逻辑代码 |
| 跳转目标 | `\goto{label}` | 内部标识符 |

---

## 翻译上下文

为避免歧义，自动添加上下文信息：

```po
#. ID: main.tex:scene[0].dialogue[2]#abc12345
msgctxt "sakura"              # 角色上下文
msgid "早上好"
msgstr "Good morning"

#. ID: main.tex:scene[1].dialogue[5]#def67890
msgctxt "ren"                 # 不同角色可能语气不同
msgid "早上好"
msgstr "Mornin'"              # 黑崎更随意的语气
```

---

## 对比：传统方式 vs 提取工具方式

| 方面 | 传统方式（ID） | 提取工具方式 |
|-----|--------------|-----------------|
| 编剧体验 | ❌ 需要记住 ID | ✅ 直接写文本 |
| 翻译体验 | ❌ 需要对照 ID 表 | ✅ 看到原文直接翻译 |
| 维护成本 | ❌ ID 与文本分离，易出错 | ✅ 文本即 ID，自动同步 |
| 重构难度 | ❌ 改文本需要改 ID | ✅ 直接改文本 |
| 上下文 | ❌ 需要手动注释 | ✅ 自动提取位置和角色 |
| 行号变化 | ❌ 插入内容后失效 | ✅ 节点 ID 稳定可靠 |

---

## 工具命令

```bash
# 提取文本
galgame extract <scripts> -o <output-dir> [options]
  --update          增量更新，保留已有翻译
  --format=<fmt>    输出格式：po, json, xlsx
  --context         包含上下文信息

# 构建游戏
galgame build [options]
  --locale=<lang>   指定语言
  --output=<dir>    输出目录

# 合并翻译
galgame merge <locale-dir> -o <output>
  --missing=<action> 处理缺失翻译：error, warn, skip, fallback

# 统计翻译进度
galgame stats <locale-dir>
  --by-file         按文件统计
  --by-context      按上下文统计
```

---

## 总结

**编剧视角**：直接写剧本，不关心翻译
**翻译视角**：只翻译文本文件，不触碰剧本
**工具视角**：自动提取、自动合并、智能匹配
