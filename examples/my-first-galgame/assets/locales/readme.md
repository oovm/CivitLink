# Galgame 国际化工作流

## 设计理念

**核心原则：编剧不需要关心翻译，直接用源语言写剧本。**

---

## 工作流程

### 1. 编剧阶段

编剧用源语言（如中文）直接写剧本，不需要关心翻译：

```story
# 使用 Story 脚本语法

%audio::play("bgm/mystery_theme.mp3", 0.6)
%scene::change("bg/school_gate_sunset.png", "fade", 1.5)

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。

%character::show("sakura", "smile", "left")

樱井美咲：新同学！你在看什么呢？

* [好啊，我去看看]
    主角：好啊，我去看看！
    ~ %{sakura_affection++}
    -> accept

* [抱歉，我还有事]
    主角：抱歉，我还有事。
    -> decline
```

### 2. 提取阶段

运行提取工具，自动生成翻译文件：

```bash
# 提取所有可翻译文本，生成 .locale 文件
galgame extract assets/stories/*.story -o assets/locales/
```

生成的翻译文件：

```von
# assets/locales/main.locale

LocaleFile {
    version: "1.0",
    source_file: "main.story",
    source_locale: "zh-CN",
    metadata: LocaleMetadata {
        project: "星光学院的秘密",
        created_at: "2026-04-13T12:00:00Z",
        updated_at: "2026-04-13T15:30:00Z",
        total_entries: 100,
    },
    entries: [
        LocaleEntry {
            id: "main.story:scene[0].narration[0]#a3b2c1d0",
            context: "",
            translations: {
                zh-CN: "星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。",
                en-US: "",
                ja-JP: "",
            },
            status: {
                en-US: "untranslated",
                ja-JP: "untranslated",
            },
            comments: [],
        },
    ],
}
```

### 3. 翻译阶段

翻译团队编辑翻译文件，所有语言在一起，便于对比理解：

```von
# assets/locales/main.locale

LocaleFile {
    version: "1.0",
    source_file: "main.story",
    source_locale: "zh-CN",
    metadata: LocaleMetadata {
        project: "星光学院的秘密",
        created_at: "2026-04-13T12:00:00Z",
        updated_at: "2026-04-13T15:30:00Z",
        total_entries: 100,
    },
    entries: [
        LocaleEntry {
            id: "main.story:scene[0].narration[0]#a3b2c1d0",
            context: "",
            translations: {
                zh-CN: "星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。",
                en-US: "Star Academy, a historic school, hides secrets unknown to all.",
                ja-JP: "星光学院、この歴史ある名校は、人知れぬ秘密を秘めている。",
            },
            status: {
                en-US: "translated",
                ja-JP: "translated",
            },
            comments: [
                Comment {
                    author: "translator1",
                    content: "保持神秘感",
                    created_at: "2026-04-13T14:00:00Z",
                },
            ],
        },
    ],
}
```

---

## 节点 ID 设计

### 为什么不用行号？

行号在剧本修改后会变化，导致翻译引用失效：

```
❌ 行号方式：
id: "main.story:15"        # 在第 15 行插入内容后，这里变成 16 行

✅ 节点 ID 方式：
id: "main.story:scene[0].narration[0]#a3b2c1d0"
```

### ID 格式

```
<文件路径>:<结构路径>#<内容指纹>
```

| 组成部分 | 说明 | 示例 |
|---------|------|------|
| 文件路径 | 源文件相对路径 | `main.story` |
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
   → 直接保留所有语言翻译

2. 指纹匹配：指纹相同，路径变化
   → 更新路径，保留所有语言翻译

3. 模糊匹配：路径相似，文本相似度 > 80%
   → 标记为 fuzzy，保留翻译供参考

4. 新增条目：无法匹配
   → 翻译为空，等待翻译

5. 废弃条目：旧 ID 不在新提取中
   → 标记为 obsolete，可手动清理
```

---

## 文件结构

```
project/
├── assets/
│   ├── stories/               # 剧本文件（源语言）
│   │   ├── main.story
│   │   ├── chapter1_school.story
│   │   ├── chapter1_home.story
│   │   ├── chapter1_old_building.story
│   │   ├── chapter2_investigation.story
│   │   ├── endings.story
│   │   └── syntax_reference.story
│   │
│   └── locales/               # 翻译文件（所有语言合并）
│       ├── main.locale        # main.story 的所有语言翻译
│       ├── chapter1_school.locale
│       ├── chapter1_home.locale
│       ├── chapter1_old_building.locale
│       ├── chapter2_investigation.locale
│       └── endings.locale
```

---

## 设计优势

### 1. 多语言对比，便于理解

```von
LocaleEntry {
    id: "main.story:scene[0].dialogue[2]#e4f5g6h7",
    context: "sakura",
    translations: {
        zh-CN: "新同学！你在看什么呢？",
        en-US: "New student! What are you looking at?",
        ja-JP: "新入生さん！何を見ているの？",
    },
}
```

- 可以同时看到所有语言的翻译
- 通过对比其他语言更好地理解原文含义
- 大模型可以一次性处理所有语言

### 2. 与配表结构一致

```
配表结构：
├── 字段 ID
├── 字段描述
└── 各语言文本
    ├── zh-CN
    ├── en-US
    └── ja-JP

Locale 结构：
├── 条目 ID
├── 上下文
└── 各语言翻译
    ├── zh-CN
    ├── en-US
    └── ja-JP
```

### 3. 便于大模型处理

- 单文件包含完整信息
- 无需跨文件查找
- 上下文完整，理解更准确

### 4. 翻译进度一目了然

```von
metadata: LocaleMetadata {
    translation_stats: {
        en-US: TranslationStats {
            total: 100,
            translated: 85,
            fuzzy: 5,
            untranslated: 10,
            progress: 85.0,
        },
    },
}
```

---

## 提取规则

### 自动提取的内容

| 类型 | 示例 | 提取内容 |
|-----|------|---------|
| 叙述文本 | `星光学院...` | ✅ 提取 |
| 对话 | `樱井美咲：你好！` | ✅ 提取 |
| 选择选项 | `* [好啊]` | ✅ 提取 |
| 信件内容 | `--- 信件 ---` | ✅ 提取 |
| 结局文本 | `=== 结局 ===` | ✅ 提取 |

### 不提取的内容

| 类型 | 示例 | 原因 |
|-----|------|------|
| 变量名 | `% let x = 0` | 逻辑代码 |
| 命令调用 | `%audio::play()` | 系统命令 |
| 文件路径 | `"bg/school.png"` | 资源引用 |
| 跳转目标 | `-> next_scene` | 内部标识符 |
| 条件表达式 | `{sakura_affection >= 5}` | 逻辑代码 |

---

## 工具命令

```bash
# 提取文本，生成 .locale 文件
galgame extract <scripts> -o <output-dir> [options]
  --update          增量更新，保留已有翻译
  --locales=<list>  支持的语言列表，如 "zh-CN,en-US,ja-JP"

# 合并多个 .locale 文件
galgame merge <locale-files> -o <output>

# 统计翻译进度
galgame stats <locale-file>
  --by-status       按状态统计
  --by-context      按上下文统计

# 验证翻译文件
galgame validate <locale-file>
  --strict          严格模式

# 添加新语言
galgame add-locale <locale-file> --locale=<lang>
```

---

## 支持的剧本格式

| 格式 | 目录 | 说明 |
|-----|------|------|
| Story | `assets/stories/` | 本引擎原生格式 |
| Ink | `stories/ink/` | Inkle 工作室格式 |
| KAG | `stories/kag/` | Kirikiri 引擎格式 |
| MDX | `stories/mdx/` | Markdown + JSX 格式 |
| Ren'Py | `stories/renpy/` | Ren'Py 引擎格式 |
| TeX | `stories/tex/` | LaTeX 格式 |
| YAML | `stories/yaml/` | 数据驱动格式 |

---

## 总结

**编剧视角**：直接写剧本，不关心翻译
**翻译视角**：编辑 .locale 文件，所有语言在一起，便于对比理解
**工具视角**：自动提取、自动合并、智能匹配
**格式优势**：
- 统一使用 VON 格式，与引擎其他配置文件保持一致
- 所有语言合并在一起，便于大模型处理
- 与配表结构一致，降低学习成本
- 翻译进度一目了然
