# Locale 格式规范

## 概述

Locale 文件（`.locale`）是 GG 游戏引擎用于存储多语言翻译数据的格式。采用 VON 格式，将所有语言的翻译合并在一个文件中，便于管理和大模型处理。

## 设计原则

**原始文件只存内容**：`.locale` 文件专注于存储翻译条目数组，元信息（源语言、支持语言、版本、统计、作者等）存储在对应的 `.locale.meta` 文件中。

## 基本结构

```von
[
    LocaleEntry { ... },
    LocaleEntry { ... },
    LocaleEntry { ... },
]
```

`.locale` 文件就是一个 `LocaleEntry` 数组，不包含任何元信息字段。

## 字段说明

### LocaleEntry 字段

| 字段 | 类型 | 必填 | 说明 |
|-----|------|------|------|
| id | string | 是 | 唯一标识符 |
| context | string | 否 | 翻译上下文（角色名等） |
| translations | object | 是 | 各语言的翻译文本 |
| status | object | 否 | 各语言的翻译状态 |
| comments | array | 否 | 注释列表 |

### Comment 字段

| 字段 | 类型 | 说明 |
|-----|------|------|
| author | string | 注释作者 |
| content | string | 注释内容 |
| created_at | string | 创建时间（ISO 8601） |

## 翻译状态

| 状态 | 说明 |
|-----|------|
| translated | 已翻译 |
| fuzzy | 模糊匹配，需人工确认 |
| untranslated | 未翻译 |
| obsolete | 已废弃（源文本已删除） |

## 完整示例

```von
[
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
                content: "保持神秘感，开头要吸引人",
                created_at: "2026-04-13T14:00:00Z",
            },
        ],
    },
    
    LocaleEntry {
        id: "main.story:scene[0].dialogue[2]#e4f5g6h7",
        context: "sakura",
        translations: {
            zh-CN: "新同学！你在看什么呢？",
            en-US: "New student! What are you looking at?",
            ja-JP: "新入生さん！何を見ているの？",
        },
        status: {
            en-US: "translated",
            ja-JP: "translated",
        },
        comments: [
            Comment {
                author: "translator1",
                content: "樱井美咲的语气要友好活泼",
                created_at: "2026-04-13T14:00:00Z",
            },
        ],
    },
    
    LocaleEntry {
        id: "main.story:scene[0].choice[0].option[0]#i8j9k0l1",
        context: "",
        translations: {
            zh-CN: "好啊，我去看看",
            en-US: "Sure, I'll check it out",
            ja-JP: "うん、見てみるよ",
        },
        status: {
            en-US: "translated",
            ja-JP: "translated",
        },
        comments: [],
    },
    
    LocaleEntry {
        id: "main.story:scene[1].dialogue[5]#fuzzy123",
        context: "ren",
        translations: {
            zh-CN: "早上好",
            en-US: "Mornin'",
            ja-JP: "よう",
        },
        status: {
            en-US: "fuzzy",
            ja-JP: "translated",
        },
        comments: [
            Comment {
                author: "translator1",
                content: "黑崎更随意的语气，英文 Mornin' 需确认",
                created_at: "2026-04-13T14:30:00Z",
            },
        ],
    },
]
```

## 节点 ID 格式

### ID 结构

```
<文件路径>:<结构路径>#<内容指纹>
```

### 组成部分

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

## 语言代码

使用 BCP 47 语言标签：

| 代码 | 语言 |
|-----|------|
| zh-CN | 简体中文 |
| zh-TW | 繁体中文 |
| en-US | 美式英语 |
| en-GB | 英式英语 |
| ja-JP | 日语 |
| ko-KR | 韩语 |

## 元信息存储

元信息（源语言、支持语言、版本、统计、作者等）存储在对应的 `.locale.meta` 文件中，详见 [meta.md](./meta.md)。

## 文件扩展名

Locale 文件使用 `.locale` 扩展名：

```
main.locale
main.locale.meta
chapter1_school.locale
chapter1_school.locale.meta
```

## 设计优势

### 1. 多语言对比

所有语言的翻译合并在一个条目中，便于：
- 对比不同语言的翻译风格
- 通过其他语言理解原文含义
- 大模型一次性处理所有语言

### 2. 与配表结构一致

```
配表结构：                     Locale 结构：
├── 字段 ID                    ├── 条目 ID
├── 字段描述                   ├── 上下文
└── 各语言文本                 └── 各语言翻译
    ├── zh-CN                      ├── zh-CN
    ├── en-US                      ├── en-US
    └── ja-JP                      └── ja-JP
```

### 3. 内容与元信息分离

原始文件只存内容（纯条目数组），元信息存 `.meta` 文件：
- 便于版本控制（内容变更与元信息变更分离）
- 便于大模型处理（专注翻译内容）
- 符合引擎整体设计原则

## 最佳实践

1. **保持同步**：剧本修改后及时更新 locale 文件
2. **添加注释**：对有歧义的文本添加翻译注释
3. **定期清理**：删除 obsolete 状态的废弃条目
4. **版本控制**：将 locale 和 meta 文件都纳入版本控制
5. **翻译校对**：完成翻译后进行校对确认
