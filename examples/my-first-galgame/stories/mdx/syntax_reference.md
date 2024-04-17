# Galgame MDX 语法参考

## 文档结构

---
title: 剧本标题
author: 作者
version: 1.0.0
characters:
  - id: player
    name: 主角
    default_expression: normal
variables:
  affection: 0
---

# label_name

内容...

## 组件列表

### 场景控制

属性写法：

<scene background="images/bg/school.png" transition="fade" duration="1.5" />

DSL 写法：

<scene>images/bg/school.png,audios/school.mp3</scene>

| 属性 | 类型 | 说明 |
|-----|------|------|
| background | string | 背景图片路径 |
| transition | string | 过渡效果：fade, slide_left, slide_right |
| duration | number | 过渡时长（秒） |

### 角色显示

属性写法：

<character name="sakura" sprite="smile" position="left" transition="slide_left" />

DSL 写法：

<character>sakura,smile,left</character>

| 属性 | 类型 | 说明 |
|-----|------|------|
| name | string | 角色ID |
| sprite | string | 表情/姿势 |
| position | string | 位置：left, center, right |
| transition | string | 出场动画 |

### 对话

`<dialogue>` 是切换说话角色，内部不是内容，下面的文本才是对话内容：

属性写法：

<dialogue character="sakura" expression="smile" />

对话内容写在这里。

DSL 写法：

<dialogue>sakura,smile</dialogue>

对话内容写在这里。

| 属性 | 类型 | 说明 |
|-----|------|------|
| character | string | 说话角色ID |
| expression | string | 表情（可选） |

### 选择分支

<choice>
  <option text="选项文本" next="label_name" />
  <option text="条件选项" next="label_name" condition="affection >= 5" />
</choice>

| 属性 | 类型 | 说明 |
|-----|------|------|
| text | string | 选项显示文本 |
| next | string | 跳转标签 |
| condition | string | 显示条件（可选） |

### 变量操作

属性写法：

<set name="affection" value="{affection + 1}" />

DSL 写法：

<set>affection += 1</set>

| 属性 | 类型 | 说明 |
|-----|------|------|
| name | string | 变量名 |
| value | string | 新值（支持表达式） |

### 条件判断

属性写法：

<if condition="{sakura_affection >= 5}">
  <dialogue character="sakura" />
  好感度高
<else />
  <dialogue character="sakura" />
  好感度低
</if>

DSL 写法：

<if>affection >= 5</if>
  <dialogue>sakura</dialogue>
  好感度高
<else />
  <dialogue>sakura</dialogue>
  好感度低
<end />

### 跳转

属性写法：

<goto next="label_name" />

DSL 写法：

<goto>label_name</goto>

### 音频控制

属性写法：

<audio bgm="audio/bgm/theme.mp3" volume="0.6" />
<audio se="audio/se/door.wav" />

DSL 写法：

<audio>bgm:audio/bgm/theme.mp3,volume:0.6</audio>
<audio>se:audio/se/door.wav</audio>

| 属性 | 类型 | 说明 |
|-----|------|------|
| bgm | string | 背景音乐路径 |
| se | string | 音效路径 |
| volume | number | 音量（0-1） |

### 特效

属性写法：

<effect type="flash" duration="0.3" />

DSL 写法：

<effect>flash,0.3</effect>

| type | 说明 |
|------|------|
| flash | 闪光 |
| shake | 震动 |
| fade_out | 淡出 |
| fade_in | 淡入 |
| pause | 暂停 |

### 文本显示

<showtext>
多行文本内容

支持 **Markdown** 格式
</showtext>

### 物品

属性写法：

<item item="mysterious_letter" />

DSL 写法：

<item>mysterious_letter</item>

### 函数调用

属性写法：

<call function="load_script" path="chapter2.galgame" />
<call function="show_credits" />

DSL 写法：

<call>load_script(chapter2.galgame)</call>
<call>show_credits</call>

## 标签

使用 Markdown 标题定义标签：

# chapter1_start

内容...

## branch_a

分支A内容...

## branch_b

分支B内容...

## 文本规则

### 叙述文本

顶级文本直接写，空行分隔为多个文本框：

这是第一个文本框。

这是第二个文本框。

### 变量插值

使用 `{}` 插入变量：

好感度：{sakura_affection}

## 完整示例

---
title: 示例剧本
author: 灵之镜工作室
version: 1.0.0
characters:
  - id: player
    name: 主角
  - id: sakura
    name: 樱井美咲
variables:
  sakura_affection: 0
---

import { Scene, Character, Dialogue, Choice, Option, Set, If, Else, End, Goto, Audio, Effect, ShowText, Item, Call } from 'galgame/components'

# start

<audio>bgm:audio/bgm/theme.mp3,volume:0.6</audio>

<scene>images/bg/school.png</scene>

星光学院，一所神秘的学校。

<character>sakura,smile,left</character>

<dialogue>sakura</dialogue>

欢迎来到星光学院！

<dialogue>player,surprised</dialogue>

这里是...？

<dialogue>sakura,happy</dialogue>

这里是你的新学校！

<choice>
  <option text="谢谢你的介绍" next="thank" />
  <option text="你是谁？" next="ask_who" />
</choice>

## thank

<set>sakura_affection += 1</set>

<dialogue>player,smile</dialogue>

谢谢你的介绍。

<dialogue>sakura,giggle</dialogue>

不客气！

<goto>continue</goto>

## ask_who

<dialogue>player,curious</dialogue>

请问你是谁？

<dialogue>sakura</dialogue>

我是樱井美咲，学生会副会长！

<goto>continue</goto>

## continue

<dialogue>sakura</dialogue>

那么跟我来吧~

<effect>fade_out@1.0</effect>

<call>load_script,chapter2.galgame</call>
