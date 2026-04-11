---
title: 星光学院的秘密 - 主线
author: 灵之镜工作室
version: 1.0.0
characters:
  - id: player
    name: 主角
    default_expression: normal
  - id: sakura
    name: 樱井美咲
    default_expression: smile
  - id: ren
    name: 黑崎莲
    default_expression: cool
  - id: yuki
    name: 白雪悠希
    default_expression: gentle
variables:
  sakura_affection: 0
  ren_affection: 0
  yuki_affection: 0
  trust_points: 0
  clues_found: 0
  school_route: false
  home_route: false
  knows_secret: false
---

<using galgame::components::* />
<using galgame::variables::* />
<using galgame::audio::* />

# prologue

<Audio bgm="audio/bgm/mystery_theme.mp3" volume="0.6" />

<Scene bg="images/bg/school_gate_sunset.png" transition="fade" duration="1.5" />

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。

我转学到这里已经一周了，却总觉得有什么不对劲。

<Effect type="flash" duration="0.3" />

<Dialogue character="player" expression="thinking">（奇怪...那个旧校舍，为什么总是锁着？）</Dialogue>

<Character name="sakura" sprite="curious" position="left" transition="slide_left" />

<Dialogue character="sakura">新同学！你在看什么呢？</Dialogue>

<Dialogue character="player" expression="surprised">啊，樱井同学...没什么，只是在发呆。</Dialogue>

<Dialogue character="sakura" expression="smile">你的表情可不像在发呆哦。对了，放学后学生会有个活动，要来吗？</Dialogue>

<Choice>
  <Option text="好啊，我去看看" next="accept_student_council" />
  <Option text="抱歉，我还有事" next="decline_student_council" />
  <Option text="学生会有什么活动？" next="ask_about_activity" />
</Choice>

## accept_student_council

<SetVariable name="sakura_affection" value="{sakura_affection + 1}" />
<SetVariable name="trust_points" value="{trust_points + 1}" />

<Dialogue character="sakura" expression="happy">太好了！那放学后见！</Dialogue>

<Character name="sakura" sprite="happy" position="left" />

<Goto next="after_school_choice" />

## decline_student_council

<SetVariable name="sakura_affection" value="{sakura_affection - 1}" />

<Dialogue character="sakura" expression="disappointed">这样啊...那下次有机会再说吧。</Dialogue>

<Character name="sakura" sprite="disappointed" position="left" transition="slide_left_out" />

<Dialogue character="player" expression="thinking">（她的表情...好像有些失落。）</Dialogue>

<Goto next="after_school_choice" />

## ask_about_activity

<Dialogue character="sakura" expression="mysterious">嘿嘿，这是秘密哦。来了你就知道了~</Dialogue>

<Dialogue character="player" expression="suspicious">（秘密？）</Dialogue>

<Goto next="after_school_choice" />

## after_school_choice

<Scene bg="images/bg/classroom_afternoon.png" transition="fade" duration="1.0" />

放学后，我面临一个选择。

<Character name="ren" sprite="cool" position="right" transition="slide_right" />

<Dialogue character="ren">喂，新来的。要不要跟我去个地方？</Dialogue>

<Dialogue character="player" expression="surprised">黑崎同学？去哪里？</Dialogue>

<Dialogue character="ren" expression="smirk">旧校舍。我知道怎么进去。</Dialogue>

<Choice>
  <Option text="跟樱井去学生会" next="route_student_council" condition="sakura_affection >= 0" />
  <Option text="跟黑崎去旧校舍" next="route_old_building" />
  <Option text="回家" next="route_home" />
</Choice>

## route_student_council

<SetVariable name="school_route" value="true" />

<Dialogue character="player" expression="thinking">（还是先去学生会看看吧。）</Dialogue>

<Dialogue character="ren" expression="disappointed">切，无聊。</Dialogue>

<Character name="ren" sprite="disappointed" position="right" transition="slide_right_out" />

<Call function="load_script" path="chapter1_school.galgame" />

## route_old_building

<SetVariable name="school_route" value="true" />
<SetVariable name="ren_affection" value="{ren_affection + 2}" />

<Dialogue character="player" expression="curious">（旧校舍...也许能发现什么。）</Dialogue>

<Dialogue character="ren" expression="smirk">明智的选择。</Dialogue>

<Call function="load_script" path="chapter1_old_building.galgame" />

## route_home

<SetVariable name="home_route" value="true" />

<Dialogue character="player" expression="thinking">（今天还是先回家吧。）</Dialogue>

<Dialogue character="ren" expression="shrug">随你便。</Dialogue>

<Call function="load_script" path="chapter1_home.galgame" />
