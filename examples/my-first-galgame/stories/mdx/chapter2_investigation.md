---
title: 第二章 - 调查
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />
<using galgame::variables::* />
<using galgame::audio::* />

# chapter2_start

<Audio bgm="audio/bgm/school_morning.mp3" volume="0.5" />

<Scene bg="images/bg/school_classroom.png" transition="fade" duration="1.0" />

第二天早上。

<Dialogue character="player" expression="thinking">（今天要继续调查...）</Dialogue>

<Character name="sakura" sprite="smile" position="left" transition="slide_left" />
<Character name="yuki" sprite="gentle" position="right" transition="slide_right" />

<Dialogue character="sakura">早上好！准备好了吗？</Dialogue>

<Dialogue character="yuki">我们今天放学后去旧校舍。</Dialogue>

<If condition="knows_secret == true">
  <Dialogue character="player" expression="serious">（昨晚那个人说樱井同学是知情者...）</Dialogue>
<Else />
  <Dialogue character="player" expression="curious">（希望能找到一些线索。）</Dialogue>
</If>

<Choice>
  <Option text="我准备好了" next="ready_to_investigate" />
  <Option text="还有其他人要来吗？" next="ask_about_others" />
</Choice>

## ask_about_others

<Dialogue character="sakura" expression="thinking">嗯...黑崎同学说他也要来。</Dialogue>

<Character name="ren" sprite="cool" position="center" transition="fade_in" />

<Dialogue character="ren">当然要来。旧校舍可是我的地盘。</Dialogue>

<Dialogue character="yuki" expression="worried">黑崎同学...请不要太冲动。</Dialogue>

<Dialogue character="ren" expression="smirk">放心，我知道分寸。</Dialogue>

<Goto next="ready_to_investigate" />

## ready_to_investigate

<Effect type="clock_transition" duration="1.5" />

<Audio bgm="audio/bgm/mystery_theme.mp3" volume="0.6" />

<Scene bg="images/bg/old_building_entrance.png" transition="fade" duration="1.0" />

放学后，我们来到了旧校舍前。

<Dialogue character="sakura" expression="nervous">这里...真的好旧了。</Dialogue>

<Character name="ren" sprite="cool" position="right" />

<Dialogue character="ren">入口在这边。</Dialogue>

<Scene bg="images/bg/old_building_interior.png" transition="fade" duration="1.0" />

<Audio se="audio/se/door_creak.wav" />

<Effect type="shake" duration="0.3" />

门发出刺耳的声音，缓缓打开。

<Dialogue character="yuki" expression="scared">好黑...</Dialogue>

<Effect type="flash" duration="0.2" />

<Audio se="audio/se/strange_light.wav" />

突然，一道奇异的光从走廊深处闪过！

<Dialogue character="player" expression="shocked">那是...！</Dialogue>

<Dialogue character="ren" expression="serious">在那边！追！</Dialogue>

<Choice>
  <Option text="追上去" next="chase_light" />
  <Option text="小心前进" next="careful_advance" />
  <Option text="先观察一下" next="observe_first" />
</Choice>

## chase_light

<SetVariable name="clues_found" value="{clues_found + 1}" />
<SetVariable name="ren_affection" value="{ren_affection + 1}" />

<Dialogue character="player" expression="determined">（不能让它跑了！）</Dialogue>

<Scene bg="images/bg/old_building_hallway.png" transition="fade" duration="0.5" />

<Audio se="audio/se/running_footsteps.wav" />

我们追着光跑过走廊，最后来到了一扇门前。

<Dialogue character="ren" expression="surprised">这是...实验室？</Dialogue>

<Goto next="discover_lab" />

## careful_advance

<SetVariable name="trust_points" value="{trust_points + 1}" />
<SetVariable name="yuki_affection" value="{yuki_affection + 1}" />

<Dialogue character="player" expression="cautious">（太危险了，还是小心点。）</Dialogue>

<Dialogue character="yuki" expression="relieved">对，我们还是小心一点比较好。</Dialogue>

<Scene bg="images/bg/old_building_hallway.png" transition="fade" duration="1.0" />

我们小心翼翼地前进，发现地上有一些奇怪的痕迹。

<ItemReceive item="strange_key" />

<Dialogue character="player" expression="curious">这是...一把钥匙？</Dialogue>

<SetVariable name="has_lab_key" value="true" />

<Goto next="discover_lab" />

## observe_first

<SetVariable name="clues_found" value="{clues_found + 2}" />
<SetVariable name="sakura_affection" value="{sakura_affection + 1}" />

<Dialogue character="player" expression="thinking">（先观察一下情况。）</Dialogue>

<Dialogue character="sakura" expression="curious">看，光是从那边那个房间传来的。</Dialogue>

<Dialogue character="player">（樱井同学好像对这里很熟悉...）</Dialogue>

<Dialogue character="sakura" expression="nervous">怎么了？</Dialogue>

<Dialogue character="player" expression="normal">没什么，我们过去看看吧。</Dialogue>

<Goto next="discover_lab" />

## discover_lab

<Scene bg="images/bg/old_lab.png" transition="fade" duration="1.5" />

<Effect type="lightning" duration="0.2" />

<Audio bgm="audio/bgm/revelation.mp3" volume="0.7" />

我们来到了一个废弃的实验室。

<Dialogue character="sakura" expression="shocked">这...这是...</Dialogue>

<Dialogue character="yuki" expression="horrified">不可能...为什么会有这种东西...</Dialogue>

<Dialogue character="ren" expression="serious">看来传说是真的。</Dialogue>

<Dialogue character="player" expression="shocked">（这些设备...看起来像是用来做某种实验的...）</Dialogue>

<ShowText>
桌上散落着文件：
"记忆操纵实验 - 第七次测试报告
实验对象：学生志愿者
结果：部分记忆成功被改写
副作用：轻微头痛，偶尔出现幻觉
..."
</ShowText>

<Dialogue character="player" expression="horrified">记忆操纵...？</Dialogue>

<Character name="sakura" sprite="serious" position="left" />

<Dialogue character="sakura">...你们想知道真相吗？</Dialogue>

<Choice>
  <Option text="告诉我真相" next="reveal_truth" />
  <Option text="你早就知道了？" next="confront_sakura" />
</Choice>

## reveal_truth

<Dialogue character="sakura" expression="sad">是的...我知道。</Dialogue>

<Dialogue character="sakura">这所学校...曾经进行过记忆操纵的实验。</Dialogue>

<Dialogue character="sakura">而我...是实验的参与者之一。</Dialogue>

<Dialogue character="player" expression="shocked">什么？！</Dialogue>

<Dialogue character="yuki" expression="worried">樱井同学...</Dialogue>

<Dialogue character="sakura" expression="crying">我...我不记得那段时间发生了什么。</Dialogue>

<Dialogue character="sakura">但是我知道，我的记忆...被改写过。</Dialogue>

<SetVariable name="knows_secret" value="true" />

<Goto next="final_choice" />

## confront_sakura

<SetVariable name="sakura_affection" value="{sakura_affection - 2}" />
<SetVariable name="trust_points" value="{trust_points - 1}" />

<Dialogue character="player" expression="angry">你早就知道了？为什么要瞒着我们？</Dialogue>

<Dialogue character="sakura" expression="guilty">对不起...我害怕...</Dialogue>

<Dialogue character="sakura">害怕你们知道真相后会讨厌我...</Dialogue>

<Choice>
  <Option text="我理解你的顾虑" next="forgive_sakura" />
  <Option text="我需要时间消化" next="need_time" />
</Choice>

## forgive_sakura

<SetVariable name="sakura_affection" value="{sakura_affection + 3}" />
<SetVariable name="trust_points" value="{trust_points + 2}" />

<Dialogue character="player" expression="gentle">没关系，我理解。</Dialogue>

<Dialogue character="sakura" expression="relieved">谢谢你...</Dialogue>

<Goto next="reveal_truth" />

## need_time

<Dialogue character="player" expression="serious">我需要时间消化这些信息。</Dialogue>

<Dialogue character="sakura" expression="sad">我明白...</Dialogue>

<Goto next="reveal_truth" />

## final_choice

<Dialogue character="sakura">现在，我们有一个选择。</Dialogue>

<Dialogue character="sakura">我们可以把这一切公之于众，或者...</Dialogue>

<Dialogue character="ren" expression="serious">或者什么？</Dialogue>

<Dialogue character="sakura">或者...找到当初的实验数据，恢复所有人的记忆。</Dialogue>

<Choice>
  <Option text="公布真相" next="ending_public" />
  <Option text="恢复记忆" next="ending_restore" />
  <Option text="先找到更多证据" next="ending_investigate" />
</Choice>

## ending_public

<SetVariable name="ending_type" value="public" />

<Dialogue character="player" expression="determined">真相应该被公开。</Dialogue>

<Dialogue character="sakura" expression="determined">我同意。大家有权知道发生了什么。</Dialogue>

<Call function="load_script" path="endings.galgame" />

## ending_restore

<SetVariable name="ending_type" value="restore" />

<Dialogue character="player" expression="thinking">如果能恢复记忆...那才是最好的结果。</Dialogue>

<Dialogue character="yuki" expression="hopeful">是的，这样大家都能找回自己失去的记忆。</Dialogue>

<Call function="load_script" path="endings.galgame" />

## ending_investigate

<SetVariable name="ending_type" value="investigate" />
<SetVariable name="clues_found" value="{clues_found + 3}" />

<Dialogue character="player" expression="cautious">我们需要更多证据才能做决定。</Dialogue>

<Dialogue character="ren" expression="nod">说得对，不能草率行事。</Dialogue>

<Call function="load_script" path="endings.galgame" />
