---
title: 第一章 - 家庭线
author: 灵之镜工作室
version: 1.0.0
---

<using galgame::components::* />
<using galgame::variables::* />
<using galgame::audio::* />

# home_route_start

<audio bgm="audio/bgm/home_theme.mp3" volume="0.4" />

<scene bg="images/bg/home_living_room.png" transition="fade" duration="1.5" />

回到家，我疲惫地躺在沙发上。

<dialogue character="player" expression="tired">呼...今天也结束了。</dialogue>

<effect type="pause" duration="1.0" />

<character name="mom" sprite="worried" position="left" transition="slide_left" />

<dialogue character="mom">你回来了。学校怎么样？</dialogue>

<dialogue character="player" expression="normal">还行...就是有点累。</dialogue>

<dialogue character="mom" expression="serious">对了，今天有个奇怪的人来找过你。</dialogue>

<dialogue character="player" expression="surprised">奇怪的人？</dialogue>

<dialogue character="mom">一个戴着帽子的男生，说是你的同学。他留了这个给你。</dialogue>

<effect type="shake" duration="0.3" />

<audio se="audio/se/item_receive.wav" />

<item item="mysterious_letter" />

<dialogue character="player" expression="curious">这是...</dialogue>

<character name="mom" sprite="normal" position="left" transition="slide_left_out" />

<dialogue character="mom">我先去准备晚餐了。</dialogue>

<scene bg="images/bg/home_bedroom.png" transition="fade" duration="0.8" />

回到房间，我打开那封信。

<dialogue character="player" expression="serious">（信上写着...）</dialogue>

<showtext>
"新来的转学生：

如果你对这所学校的秘密感兴趣，
今晚12点到旧校舍后面。

——一个知道真相的人"
</showtext>

<dialogue character="player" expression="shocked">这是...什么意思？</dialogue>

<choice>
  <option text="今晚去看看" next="go_tonight" />
  <option text="太危险了，不去" next="not_go_tonight" />
  <option text="先调查一下这个写信人" next="investigate_sender" />
</choice>

## go_tonight

<set name="home_route" value="true" />
<set name="clues_found" value="{clues_found + 1}" />

<dialogue character="player" expression="determined">（既然有人邀请我，那就去看看。）</dialogue>

<effect type="clock_transition" duration="2.0" />

<audio bgm="audio/bgm/night_mystery.mp3" volume="0.6" />

<scene bg="images/bg/old_building_night.png" transition="fade" duration="1.5" />

<effect type="lightning" duration="0.2" />

深夜，我悄悄来到了旧校舍后面。

<dialogue character="player" expression="nervous">（这里...真的好黑...）</dialogue>

<character name="unknown" sprite="hooded" position="center" transition="fade_in" />

<dialogue character="unknown" expression="mysterious">你来了。</dialogue>

<dialogue character="player" expression="surprised">你是谁？</dialogue>

<dialogue character="unknown">这不重要。重要的是，你想知道真相吗？</dialogue>

<choice>
  <option text="我想知道" next="want_truth" />
  <option text="你到底是谁？" next="demand_identity" />
</choice>

## want_truth

<set name="knows_secret" value="true" />

<dialogue character="unknown">很好。那么，听好了...</dialogue>

<dialogue character="unknown">这所学校，曾经进行过一项秘密实验。</dialogue>

<dialogue character="player" expression="shocked">秘密实验？</dialogue>

<dialogue character="unknown">关于"记忆操纵"的实验。而旧校舍，就是实验的地点。</dialogue>

<dialogue character="unknown">那些奇怪的光...是实验残留的能量。</dialogue>

<dialogue character="player" expression="horrified">记忆操纵...？</dialogue>

<dialogue character="unknown">如果你想知道更多，明天来学生会找樱井美咲。</dialogue>

<dialogue character="unknown">她...也是知情者之一。</dialogue>

<character name="unknown" sprite="hooded" position="center" transition="fade_out" />

<dialogue character="player" expression="confused">等等！</dialogue>

<dialogue character="player" expression="thinking">（樱井同学...她知道什么？）</dialogue>

<goto next="chapter1_home_end" />

## demand_identity

<dialogue character="unknown">我的身份不重要。重要的是，你有权利知道真相。</dialogue>

<dialogue character="unknown">这所学校隐藏着一个巨大的秘密。</dialogue>

<goto next="want_truth" />

## not_go_tonight

<set name="home_route" value="true" />
<set name="trust_points" value="{trust_points - 1}" />

<dialogue character="player" expression="worried">（太危险了...还是不要去为好。）</dialogue>

<effect type="clock_transition" duration="1.0" />

<dialogue character="player" expression="thinking">（但是...这封信到底是谁寄的？）</dialogue>

<dialogue character="player">（也许明天去学校问问看...）</dialogue>

<goto next="chapter1_home_end" />

## investigate_sender

<set name="home_route" value="true" />
<set name="clues_found" value="{clues_found + 2}" />

<dialogue character="player" expression="thinking">（先调查一下这个写信的人。）</dialogue>

<dialogue character="player">（字迹...很工整。用纸是学校的学生会专用纸...）</dialogue>

<dialogue character="player" expression="surprised">（学生会？难道是...）</dialogue>

<dialogue character="player">（樱井同学？还是白雪同学？）</dialogue>

<goto next="chapter1_home_end" />

## chapter1_home_end

<scene bg="images/bg/home_bedroom_night.png" transition="fade" duration="1.0" />

<dialogue character="player" expression="thinking">（今天发生了很多事...）</dialogue>

<dialogue character="player">（明天去学校，一定要弄清楚。）</dialogue>

<effect type="fade_out" duration="1.5" />

<call function="load_script" path="chapter2_investigation.galgame" />
