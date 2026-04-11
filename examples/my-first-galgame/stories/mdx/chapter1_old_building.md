---
title: 第一章 - 旧校舍线
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />
<using galgame::variables::* />
<using galgame::audio::* />

# old_building_entrance

<audio bgm="audio/bgm/suspense_theme.mp3" volume="0.6" />

<scene bg="images/bg/old_building_exterior.png" transition="fade" duration="1.0" />

<character name="ren" sprite="smirk" position="right" />

<dialogue character="ren">到了。这就是旧校舍。</dialogue>

<dialogue character="player" expression="curious">看起来已经废弃很久了...</dialogue>

<dialogue character="ren" expression="serious">表面上是这样。</dialogue>

<dialogue character="ren">但实际上，这里藏着很多秘密。</dialogue>

<dialogue character="player" expression="surprised">秘密？</dialogue>

<dialogue character="ren">跟我来。</dialogue>

<scene bg="images/bg/old_building_side.png" transition="fade" duration="0.8" />

<audio se="audio/se/metal_gate.wav" />

<effect type="shake" duration="0.3" />

黑崎带我来到旧校舍的侧面，那里有一扇生锈的铁门。

<dialogue character="ren" expression="smirk">这扇门的锁早就坏了。</dialogue>

<audio se="audio/se/door_open_creak.wav" />

<scene bg="images/bg/old_building_corridor.png" transition="fade" duration="1.0" />

我们进入了旧校舍内部。

<effect type="dust_particles" duration="1.0" />

<dialogue character="player" expression="nervous">好暗...</dialogue>

<dialogue character="ren" expression="calm">小心脚下。</dialogue>

<choice>
  <option text="打开手电筒" next="use_flashlight" />
  <option text="跟着黑崎走" next="follow_ren" />
  <option text="询问黑崎为什么知道这里" next="ask_ren_knowledge" />
</choice>

## use_flashlight

<set name="clues_found" value="{clues_found + 1}" />

<audio se="audio/se/flashlight_on.wav" />

<effect type="light_cone" duration="0.5" />

<dialogue character="player" expression="curious">（手电筒的光照亮了走廊...）</dialogue>

<dialogue character="player">（墙上好像有什么字...）</dialogue>

<showtext>
墙上写着：
"不要相信他们"
"记忆是假的"
"救救我"
</showtext>

<dialogue character="player" expression="shocked">这些字...是什么意思？</dialogue>

<dialogue character="ren" expression="serious">看来有人在这里待过很久。</dialogue>

<goto next="explore_corridor" />

## follow_ren

<set name="ren_affection" value="{ren_affection + 1}" />

<dialogue character="player" expression="trusting">（相信黑崎同学吧。）</dialogue>

<dialogue character="ren" expression="slight_smile">...聪明。</dialogue>

<dialogue character="player">（他好像对我有点改观？）</dialogue>

<goto next="explore_corridor" />

## ask_ren_knowledge

<set name="clues_found" value="{clues_found + 2}" />

<dialogue character="player" expression="suspicious">黑崎同学，你怎么知道这个地方？</dialogue>

<dialogue character="ren" expression="hesitant">...</dialogue>

<dialogue character="ren">我...曾经在这里待过。</dialogue>

<dialogue character="player" expression="shocked">什么意思？</dialogue>

<dialogue character="ren" expression="serious">我是...实验的幸存者之一。</dialogue>

<set name="knows_secret" value="true" />

<dialogue character="player" expression="horrified">实验...？</dialogue>

<dialogue character="ren">以后再解释。先往前走。</dialogue>

<goto next="explore_corridor" />

## explore_corridor

<scene bg="images/bg/old_building_hallway.png" transition="fade" duration="0.8" />

我们沿着走廊前进，两边是废弃的教室。

<effect type="flash" duration="0.2" />

<audio se="audio/se/strange_sound.wav" />

突然，一阵奇怪的声音从远处传来！

<dialogue character="player" expression="alert">什么声音？</dialogue>

<dialogue character="ren" expression="alert">有人在这里。</dialogue>

<choice>
  <option text="躲起来观察" next="hide_and_observe" />
  <option text="直接去查看" next="go_check" />
  <option text="让黑崎去查看" next="send_ren" />
</choice>

## hide_and_observe

<set name="clues_found" value="{clues_found + 2}" />
<set name="trust_points" value="{trust_points + 1}" />

<dialogue character="player" expression="cautious">（先躲起来看看情况。）</dialogue>

<scene bg="images/bg/old_building_shadow.png" transition="fade" duration="0.5" />

我们躲在一扇破旧的门后。

<character name="unknown" sprite="hooded" position="center" transition="fade_in" />

<dialogue character="unknown">（低声自语）数据已经转移完毕...</dialogue>

<dialogue character="unknown">下一个目标...新来的转学生。</dialogue>

<dialogue character="player" expression="shocked">（他们...在说我？！）</dialogue>

<dialogue character="ren" expression="serious">（小声）看来你被盯上了。</dialogue>

<character name="unknown" sprite="hooded" position="center" transition="fade_out" />

<dialogue character="player" expression="worried">（这到底是怎么回事...）</dialogue>

<goto next="discover_room" />

## go_check

<set name="ren_affection" value="{ren_affection + 1}" />

<dialogue character="player" expression="brave">（去看看是什么。）</dialogue>

<dialogue character="ren" expression="impressed">...有胆量。</dialogue>

<scene bg="images/bg/old_building_intersection.png" transition="fade" duration="0.5" />

我们走向声音传来的方向，但那里已经没有人了。

<dialogue character="player" expression="frustrated">跑了...</dialogue>

<dialogue character="ren" expression="serious">但是留下了这个。</dialogue>

<item item="mysterious_document" />

<dialogue character="player" expression="curious">这是...</dialogue>

<set name="clues_found" value="{clues_found + 1}" />

<goto next="discover_room" />

## send_ren

<dialogue character="player" expression="cautious">黑崎同学，你能去看看吗？</dialogue>

<dialogue character="ren" expression="annoyed">...切。</dialogue>

<dialogue character="ren">在这里等着。</dialogue>

<character name="ren" sprite="cool" position="right" transition="slide_right_out" />

<effect type="pause" duration="2.0" />

<character name="ren" sprite="serious" position="right" transition="slide_right" />

<dialogue character="ren">没人。但是发现了这个。</dialogue>

<item item="access_card" />

<dialogue character="player" expression="curious">一张门禁卡？</dialogue>

<dialogue character="ren">旧校舍的地下实验室用的。</dialogue>

<set name="has_lab_access" value="true" />

<goto next="discover_room" />

## discover_room

<scene bg="images/bg/old_building_door.png" transition="fade" duration="0.8" />

我们来到一扇厚重的金属门前。

<dialogue character="ren">这就是目的地。</dialogue>

<if condition="has_lab_access == true">
  <goto next="enter_with_card" />
<else />
  <goto next="locked_door" />
</if>

## locked_door

<dialogue character="player" expression="frustrated">门锁着...</dialogue>

<dialogue character="ren" expression="thinking">我们需要找到开门的方法。</dialogue>

<choice>
  <option text="寻找其他入口" next="find_entrance" />
  <option text="尝试撬锁" next="try_lockpick" />
  <option text="回去找樱井帮忙" next="get_sakura_help" />
</choice>

## enter_with_card

<audio se="audio/se/card_beep.wav" />

<effect type="door_unlock" duration="0.5" />

<dialogue character="player" expression="surprised">门开了！</dialogue>

<dialogue character="ren" expression="smirk">走吧。</dialogue>

<goto next="enter_lab" />

## find_entrance

<set name="clues_found" value="{clues_found + 1}" />

<dialogue character="player" expression="determined">（一定有其他入口。）</dialogue>

<scene bg="images/bg/old_building_vent.png" transition="fade" duration="0.8" />

<dialogue character="ren" expression="serious">通风管道...可以试试。</dialogue>

<dialogue character="player" expression="nervous">这...真的能进去吗？</dialogue>

<dialogue character="ren">跟着我。</dialogue>

<goto next="enter_lab" />

## try_lockpick

<dialogue character="player" expression="determined">让我试试...</dialogue>

<audio se="audio/se/lockpick_fail.wav" />

<dialogue character="player" expression="frustrated">不行，锁太复杂了。</dialogue>

<dialogue character="ren" expression="sigh">我就说没用。</dialogue>

<goto next="find_entrance" />

## get_sakura_help

<set name="sakura_affection" value="{sakura_affection + 1}" />
<set name="trust_points" value="{trust_points + 1}" />

<dialogue character="player" expression="thinking">（樱井同学可能有办法...）</dialogue>

<dialogue character="ren" expression="annoyed">还要找那个女人？</dialogue>

<dialogue character="player" expression="serious">她可能有钥匙。</dialogue>

<effect type="phone_call" duration="1.0" />

<dialogue character="sakura" expression="worried">喂？怎么了？</dialogue>

<dialogue character="player">我们在旧校舍...门锁着。</dialogue>

<dialogue character="sakura">我马上来！</dialogue>

<effect type="clock_transition" duration="1.5" />

<character name="sakura" sprite="worried" position="left" transition="slide_left" />

<dialogue character="sakura">给你们钥匙。</dialogue>

<audio se="audio/se/key_unlock.wav" />

<dialogue character="sakura">但是...请小心。</dialogue>

<goto next="enter_lab" />

## enter_lab

<scene bg="images/bg/old_lab.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/revelation.mp3" volume="0.7" />

<effect type="lightning" duration="0.2" />

我们进入了地下实验室。

<dialogue character="player" expression="shocked">这是...</dialogue>

<dialogue character="ren" expression="serious">记忆操纵实验室。</dialogue>

<showtext>
房间中央是一台巨大的机器，
周围散落着各种文件和设备。
</showtext>

<dialogue character="player" expression="horrified">记忆操纵...真的存在？</dialogue>

<dialogue character="ren">是的。而且...</dialogue>

<dialogue character="ren" expression="sad">我是实验对象之一。</dialogue>

<dialogue character="player" expression="shocked">黑崎同学...</dialogue>

<dialogue character="ren">我失去了一部分记忆。</dialogue>

<dialogue character="ren">但是我知道...这所学校还在继续实验。</dialogue>

<set name="knows_secret" value="true" />
<set name="clues_found" value="{clues_found + 3}" />

<choice>
  <option text="我们一起阻止他们" next="promise_together" />
  <option text="告诉我更多细节" next="more_details" />
</choice>

## promise_together

<set name="ren_affection" value="{ren_affection + 3}" />
<set name="trust_points" value="{trust_points + 2}" />

<dialogue character="player" expression="determined">我们一起阻止他们。</dialogue>

<dialogue character="ren" expression="surprised">...</dialogue>

<dialogue character="ren" expression="smile">...谢了。</dialogue>

<dialogue character="ren">第一次有人说要帮我。</dialogue>

<goto next="chapter1_old_building_end" />

## more_details

<dialogue character="player" expression="serious">告诉我更多细节。</dialogue>

<dialogue character="ren" expression="serious">实验代号"星之子"。</dialogue>

<dialogue character="ren">目的是创造没有痛苦记忆的人。</dialogue>

<dialogue character="ren">但是副作用...是人格分裂。</dialogue>

<dialogue character="player" expression="horrified">人格分裂？</dialogue>

<dialogue character="ren">我有时候会变成另一个人。</dialogue>

<dialogue character="ren">那个我...更冷酷，更危险。</dialogue>

<set name="knows_ren_secret" value="true" />

<goto next="chapter1_old_building_end" />

## chapter1_old_building_end

<scene bg="images/bg/school_sunset.png" transition="fade" duration="1.5" />

<audio bgm="audio/bgm/bittersweet.mp3" volume="0.5" />

离开旧校舍后，夕阳已经西沉。

<dialogue character="player" expression="thinking">（今天发现了太多事情...）</dialogue>

<character name="ren" sprite="serious" position="right" />

<dialogue character="ren">明天放学后，我们继续调查。</dialogue>

<dialogue character="player" expression="determined">好。</dialogue>

<dialogue character="ren" expression="slight_smile">...你比我想象的可靠。</dialogue>

<dialogue character="player" expression="smile">谢谢。</dialogue>

<effect type="fade_out" duration="1.5" />

<call function="load_script" path="chapter2_investigation.galgame" />
