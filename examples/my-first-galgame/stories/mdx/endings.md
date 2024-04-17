---
title: 结局 - 星光学院的秘密
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />
<using galgame::variables::* />
<using galgame::audio::* />

# endings_start

<if condition="ending_type == 'public'">
  <goto next="public_truth_ending" />
</if>

<if condition="ending_type == 'restore'">
  <goto next="restore_memory_ending" />
</if>

<if condition="ending_type == 'investigate'">
  <goto next="true_ending" />
</if>

<goto next="normal_ending" />

## public_truth_ending

<audio bgm="audio/bgm/dramatic_theme.mp3" volume="0.7" />

<scene bg="images/bg/school_assembly.png" transition="fade" duration="1.5" />

<effect type="flash" duration="0.3" />

我们决定公开真相。

<dialogue character="player" expression="determined">（这是正确的选择。）</dialogue>

<character name="sakura" sprite="determined" position="center" />
<character name="yuki" sprite="serious" position="left" />
<character name="ren" sprite="serious" position="right" />

<dialogue character="sakura">各位同学，我们有重要的事情要宣布...</dialogue>

<effect type="crowd_murmur" duration="1.0" />

<showtext>
几天后...

学校的秘密实验被媒体曝光，
校长被撤职，
所有参与实验的学生都接受了心理辅导。
</showtext>

<scene bg="images/bg/school_rooftop.png" transition="fade" duration="1.0" />

<audio bgm="audio/bgm/bittersweet.mp3" volume="0.5" />

<dialogue character="sakura" expression="sad">虽然真相大白了...</dialogue>

<dialogue character="sakura">但是学校现在一片混乱。</dialogue>

<dialogue character="player" expression="serious">这是必要的代价。</dialogue>

<dialogue character="yuki" expression="gentle">至少，大家现在都知道真相了。</dialogue>

<dialogue character="ren" expression="neutral">哼，总算结束了。</dialogue>

<if condition="sakura_affection >= 5">
  <goto next="sakura_good_ending" />
<else />
  <if condition="ren_affection >= 5">
    <goto next="ren_good_ending" />
  <else />
    <if condition="yuki_affection >= 5">
      <goto next="yuki_good_ending" />
    <else />
      <goto next="normal_public_ending" />
    </if>
  </if>
</if>

## restore_memory_ending

<audio bgm="audio/bgm/hopeful_theme.mp3" volume="0.6" />

<scene bg="images/bg/old_lab.png" transition="fade" duration="1.0" />

我们决定寻找恢复记忆的方法。

<dialogue character="player" expression="determined">（一定要找到实验数据。）</dialogue>

<character name="sakura" sprite="hopeful" position="center" />

<dialogue character="sakura">我找到了！这里有备份数据！</dialogue>

<effect type="glow" duration="0.5" />

<showtext>
经过几天的努力，
我们成功恢复了所有被改写的记忆。

那些曾经失去记忆的学生，
终于找回了属于自己的过去。
</showtext>

<scene bg="images/bg/cherry_blossom.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/happy_ending.mp3" volume="0.6" />

<dialogue character="sakura" expression="happy">谢谢你...帮我找回了记忆。</dialogue>

<dialogue character="sakura">我现在...终于完整了。</dialogue>

<dialogue character="player" expression="smile">不用谢，这是我应该做的。</dialogue>

<if condition="sakura_affection >= 6">
  <dialogue character="sakura" expression="blush">那个...以后...我们可以...</dialogue>
  <dialogue character="player" expression="curious">可以什么？</dialogue>
  <dialogue character="sakura" expression="shy">一起...一起走下去吗？</dialogue>
  <goto next="true_love_ending" />
<else />
  <goto next="good_ending" />
</if>

## true_ending

<audio bgm="audio/bgm/mystery_deep.mp3" volume="0.7" />

<scene bg="images/bg/old_lab.png" transition="fade" duration="1.0" />

我们决定继续调查。

<dialogue character="player" expression="thinking">（还有太多谜团没有解开。）</dialogue>

<character name="ren" sprite="serious" position="right" />

<dialogue character="ren">等等，看这个。</dialogue>

<effect type="glow" duration="0.5" />

<dialogue character="ren">这里有一个隐藏的文件...</dialogue>

<showtext>
文件内容：
"实验并未结束。
负责人已转移至新校区。
实验代号：星之子计划"
</showtext>

<dialogue character="player" expression="shocked">实验还在继续？！</dialogue>

<dialogue character="sakura" expression="horrified">新校区...那是刚建成的...</dialogue>

<dialogue character="yuki" expression="determined">我们必须阻止他们。</dialogue>

<effect type="dramatic_pause" duration="1.0" />

<scene bg="images/bg/sunset.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/epic_theme.mp3" volume="0.7" />

<showtext>
我们发现了更深层的阴谋。
实验并未结束，
只是转移到了新的地点。

我们的战斗，才刚刚开始...
</showtext>

<effect type="fade_out" duration="2.0" />

<showtext>
**真结局：星之子**

你发现了隐藏在幕后的真相。
故事将在续作中继续...
</showtext>

<call function="show_credits" />

<goto next="ending_summary" />

## normal_ending

<audio bgm="audio/bgm/peaceful_ending.mp3" volume="0.5" />

<scene bg="images/bg/school_gate.png" transition="fade" duration="1.0" />

一切结束后，学校恢复了平静。

<dialogue character="player" expression="thinking">（虽然还有很多谜团...）</dialogue>

<dialogue character="player">（但至少现在，大家都安全了。）</dialogue>

<character name="sakura" sprite="smile" position="left" transition="slide_left" />

<dialogue character="sakura">谢谢你，新同学。</dialogue>

<dialogue character="sakura">如果没有你，我们可能永远不会知道真相。</dialogue>

<dialogue character="player" expression="smile">这是我应该做的。</dialogue>

<effect type="fade_out" duration="1.5" />

<showtext>
**普通结局：平静的日常**

你帮助学校揭开了秘密，
但真相似乎不止于此...
</showtext>

<goto next="ending_summary" />

## sakura_good_ending

<scene bg="images/bg/cherry_blossom.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/romantic_theme.mp3" volume="0.6" />

<character name="sakura" sprite="blush" position="center" />

<dialogue character="sakura">那个...我有话想对你说。</dialogue>

<dialogue character="player" expression="curious">什么事？</dialogue>

<dialogue character="sakura" expression="shy">这段时间...谢谢你一直陪在我身边。</dialogue>

<dialogue character="sakura">我...我好像...</dialogue>

<dialogue character="player" expression="surprised">？</dialogue>

<dialogue character="sakura" expression="determined">我喜欢你！</dialogue>

<effect type="heart_float" duration="1.0" />

<dialogue character="player" expression="blush">樱井同学...</dialogue>

<dialogue character="sakura" expression="happy">以后...请多多指教了！</dialogue>

<effect type="fade_out" duration="2.0" />

<showtext>
**樱井美咲结局：樱花之约**

你与樱井美咲成为了恋人。
在樱花树下，你们许下了永远的约定。
</showtext>

<goto next="ending_summary" />

## ren_good_ending

<scene bg="images/bg/night_city.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/cool_theme.mp3" volume="0.5" />

<character name="ren" sprite="smile" position="center" />

<dialogue character="ren">喂。</dialogue>

<dialogue character="player" expression="curious">黑崎同学？</dialogue>

<dialogue character="ren" expression="smirk">没想到你还挺有本事的。</dialogue>

<dialogue character="ren">我对你刮目相看了。</dialogue>

<dialogue character="player" expression="smile">谢谢...？</dialogue>

<dialogue character="ren" expression="serious">以后...有什么事可以找我。</dialogue>

<dialogue character="ren">我会帮你的。</dialogue>

<dialogue character="player" expression="surprised">真的？</dialogue>

<dialogue character="ren" expression="smile">别让我说第二遍。</dialogue>

<effect type="fade_out" duration="2.0" />

<showtext>
**黑崎莲结局：暗夜的盟约**

你获得了黑崎莲的认可。
虽然他表面冷淡，但你知道，
他会是你最可靠的伙伴。
</showtext>

<goto next="ending_summary" />

## yuki_good_ending

<scene bg="images/bg/library.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/gentle_theme.mp3" volume="0.5" />

<character name="yuki" sprite="smile" position="center" />

<dialogue character="yuki">那个...谢谢你。</dialogue>

<dialogue character="player" expression="curious">怎么了，白雪同学？</dialogue>

<dialogue character="yuki" expression="gentle">谢谢你让我变得勇敢。</dialogue>

<dialogue character="yuki">以前的我...总是害怕面对真相。</dialogue>

<dialogue character="yuki" expression="determined">但是现在...我想和你一起面对未来。</dialogue>

<dialogue character="player" expression="smile">我也是。</dialogue>

<dialogue character="yuki" expression="blush">那...我们约定好了？</dialogue>

<effect type="sparkle" duration="1.0" />

<effect type="fade_out" duration="2.0" />

<showtext>
**白雪悠希结局：温柔的约定**

你与白雪悠希建立了深厚的羁绊。
在图书馆的角落，你们约定一起面对未来。
</showtext>

<goto next="ending_summary" />

## true_love_ending

<audio bgm="audio/bgm/love_theme.mp3" volume="0.7" />

<scene bg="images/bg/cherry_blossom_sunset.png" transition="fade" duration="1.5" />

<effect type="petal_fall" duration="2.0" />

<character name="sakura" sprite="happy" position="center" />

<dialogue character="sakura">从今以后...我们一起走吧。</dialogue>

<dialogue character="player" expression="happy">嗯，一起走。</dialogue>

<effect type="kiss" duration="1.0" />

<showtext>
**真结局：永恒的星光**

你不仅揭开了学校的秘密，
还找到了真正的爱情。

樱井美咲的笑容，
将是你心中永远的光芒。
</showtext>

<goto next="ending_summary" />

## good_ending

<showtext>
**好结局：新的开始**

你帮助恢复了所有人的记忆。
虽然还有未解的谜团，
但你知道，未来会更好。
</showtext>

<goto next="ending_summary" />

## normal_public_ending

<showtext>
**结局：真相大白**

学校的秘密被公之于众。
虽然付出了代价，
但正义得到了伸张。
</showtext>

<goto next="ending_summary" />

## ending_summary

<effect type="fade_out" duration="2.0" />

<scene bg="images/bg/black.png" transition="fade" duration="1.0" />

<showtext>
---

# 游戏统计

- 樱井美咲好感度: {sakura_affection}
- 黑崎莲好感度: {ren_affection}
- 白雪悠希好感度: {yuki_affection}
- 信任点数: {trust_points}
- 发现线索: {clues_found}

---

感谢游玩《星光学院的秘密》！

如果想要体验其他结局，
请尝试不同的选择路径。
</showtext>

<call function="show_title_screen" />
