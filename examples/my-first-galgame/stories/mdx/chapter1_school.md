---
title: 第一章 - 学生会线
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />
<using galgame::variables::* />
<using galgame::audio::* />

# student_council_entrance

<Audio bgm="audio/bgm/peaceful_theme.mp3" volume="0.5" />

<Scene bg="images/bg/student_council_room.png" transition="fade" duration="1.0" />

<Character name="sakura" sprite="smile" position="center" />

<Dialogue character="sakura">欢迎来到学生会！</Dialogue>

<Character name="yuki" sprite="gentle" position="left" transition="slide_left" />

<Dialogue character="yuki">啊，新同学也来了。欢迎欢迎。</Dialogue>

<Dialogue character="player" expression="curious">白雪同学也在啊。</Dialogue>

<Dialogue character="yuki" expression="smile">我是学生会的书记。樱井是副会长。</Dialogue>

<Dialogue character="sakura" expression="proud">没错！我们学生会可是学校的重要组织！</Dialogue>

<Dialogue character="player" expression="thinking">（学生会...应该能了解到一些学校的事情。）</Dialogue>

<Dialogue character="yuki" expression="serious">对了，新同学，有件事想请你帮忙。</Dialogue>

<Choice>
  <Option text="什么事？" next="ask_for_help" />
  <Option text="我很忙..." next="refuse_help" />
</Choice>

## ask_for_help

<SetVariable name="yuki_affection" value="{yuki_affection + 1}" />
<SetVariable name="trust_points" value="{trust_points + 2}" />

<Dialogue character="yuki" expression="serious">最近学校里发生了一些奇怪的事情。</Dialogue>

<Dialogue character="sakura" expression="worried">嗯...有学生说在旧校舍附近看到了奇怪的光。</Dialogue>

<Dialogue character="player" expression="surprised">奇怪的光？</Dialogue>

<Dialogue character="yuki" expression="thinking">我们想调查一下，但是旧校舍一直锁着...</Dialogue>

<Dialogue character="sakura" expression="determined">如果你能帮忙调查的话，我们会很感激的！</Dialogue>

<Choice>
  <Option text="我愿意帮忙" next="agree_to_help" />
  <Option text="这听起来很危险..." next="worry_about_danger" />
</Choice>

## refuse_help

<SetVariable name="yuki_affection" value="{yuki_affection - 1}" />
<SetVariable name="sakura_affection" value="{sakura_affection - 1}" />

<Dialogue character="sakura" expression="disappointed">这样啊...</Dialogue>

<Dialogue character="yuki" expression="disappointed">没关系，我们理解。</Dialogue>

<Dialogue character="player" expression="thinking">（他们的表情...好像很需要帮助。）</Dialogue>

<Goto next="change_mind" />

## change_mind

<Dialogue character="player" expression="thinking">（也许我应该重新考虑...）</Dialogue>

<Choice>
  <Option text="等等，我改变主意了" next="agree_to_help" />
  <Option text="告辞了" next="leave_student_council" />
</Choice>

## agree_to_help

<SetVariable name="trust_points" value="{trust_points + 2}" />

<Dialogue character="sakura" expression="happy">太好了！谢谢你！</Dialogue>

<Dialogue character="yuki" expression="relieved">有你帮忙就放心多了。</Dialogue>

<Dialogue character="sakura">黑崎同学好像知道怎么进入旧校舍...</Dialogue>

<Dialogue character="player" expression="thinking">（黑崎？他确实说过知道怎么进去...）</Dialogue>

<Goto next="investigation_start" />

## worry_about_danger

<Dialogue character="sakura" expression="reassuring">别担心！我们只是想看看是什么情况。</Dialogue>

<Dialogue character="yuki" expression="gentle">如果你害怕的话，可以和我们一起行动。</Dialogue>

<SetVariable name="sakura_affection" value="{sakura_affection + 1}" />

<Goto next="investigation_start" />

## investigation_start

<Scene bg="images/bg/school_hallway.png" transition="fade" duration="0.8" />

<Dialogue character="player" expression="determined">好，我来帮忙调查。</Dialogue>

<Character name="sakura" sprite="happy" position="center" />

<Dialogue character="sakura">太好了！那我们明天放学后在学校门口集合！</Dialogue>

<Effect type="fade_out" duration="1.0" />

<Call function="load_script" path="chapter2_investigation.galgame" />

## leave_student_council

<Scene bg="images/bg/school_corridor.png" transition="fade" duration="0.8" />

<Dialogue character="player" expression="thinking">（也许我不该卷入这件事...）</Dialogue>

<Effect type="fade_out" duration="1.0" />

<Call function="load_script" path="chapter1_home.galgame" />
