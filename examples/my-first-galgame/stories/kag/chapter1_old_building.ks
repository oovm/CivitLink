; ============================================
; 第一章 - 旧校舍线 (KAG 格式)
; ============================================

*chapter1_old_building_start

@bgm storage="audio/bgm/suspense_theme.mp3" volume=60

@bg storage="images/bg/old_building_exterior.png" time=1000 method="crossfade"

@show_char name="ren" sprite="smirk" x=500 y=0

黑崎莲「到了。这就是旧校舍。」[l][r]

主角「看起来已经废弃很久了...」[l][r]

黑崎莲「表面上是这样。」[l][r]

黑崎莲「但实际上，这里藏着很多秘密。」[l][r]

主角「秘密？」[l][r]

黑崎莲「跟我来。」[l][r]

@bg storage="images/bg/old_building_side.png" time=800 method="crossfade"

@playse storage="audio/se/metal_gate.wav"
@quake layer=0 time=300 max=10

黑崎带我来到旧校舍的侧面，那里有一扇生锈的铁门。[l][r]

黑崎莲「这扇门的锁早就坏了。」[l][r]

@playse storage="audio/se/door_open_creak.wav"

@bg storage="images/bg/old_building_corridor.png" time=1000 method="crossfade"

我们进入了旧校舍内部。[l][r]

@flash time=500

主角「好暗...」[l][r]

黑崎莲「小心脚下。」[l][r]

*branch_explore

@link target="use_flashlight"「打开手电筒」[r]
@link target="follow_ren"「跟着黑崎走」[r]
@link target="ask_ren_knowledge"「询问黑崎为什么知道这里」[r]
@s

; ============================================
; 分支：使用手电筒
; ============================================

*use_flashlight

@eval exp="f.clues_found += 1"

@playse storage="audio/se/flashlight_on.wav"
@flash time=500

主角「（手电筒的光照亮了走廊...）」[l][r]

主角「（墙上好像有什么字...）」[l][r]

---

墙上写着：
"不要相信他们"
"记忆是假的"
"救救我"

---

主角「这些字...是什么意思？」[l][r]

黑崎莲「看来有人在这里待过很久。」[l][r]

@jump target="explore_corridor"

; ============================================
; 分支：跟着黑崎
; ============================================

*follow_ren

@eval exp="f.ren_affection += 1"

主角「（相信黑崎同学吧。）」[l][r]

黑崎莲「...聪明。」[l][r]

主角「（他好像对我有点改观？）」[l][r]

@jump target="explore_corridor"

; ============================================
; 分支：询问黑崎
; ============================================

*ask_ren_knowledge

@eval exp="f.clues_found += 2"

主角「黑崎同学，你怎么知道这个地方？」[l][r]

黑崎莲「...」[l][r]

黑崎莲「我...曾经在这里待过。」[l][r]

主角「什么意思？」[l][r]

黑崎莲「我是...实验的幸存者之一。」[l][r]

@eval exp="f.knows_secret = true"
@eval exp="f.knows_ren_secret = true"

主角「实验...？」[l][r]

黑崎莲「以后再解释。先往前走。」[l][r]

@jump target="explore_corridor"

; ============================================
; 探索走廊
; ============================================

*explore_corridor

@bg storage="images/bg/old_building_hallway.png" time=800 method="crossfade"

我们沿着走廊前进，两边是废弃的教室。[l][r]

@flash time=200
@playse storage="audio/se/strange_sound.wav"

突然，一阵奇怪的声音从远处传来！[l][r]

主角「什么声音？」[l][r]

黑崎莲「有人在这里。」[l][r]

*branch_sound

@link target="hide_and_observe"「躲起来观察」[r]
@link target="go_check"「直接去查看」[r]
@link target="send_ren"「让黑崎去查看」[r]
@s

; ============================================
; 分支：躲起来观察
; ============================================

*hide_and_observe

@eval exp="f.clues_found += 2"
@eval exp="f.trust_points += 1"

主角「（先躲起来看看情况。）」[l][r]

@bg storage="images/bg/old_building_shadow.png" time=500 method="crossfade"

我们躲在一扇破旧的门后。[l][r]

@show_char name="unknown" sprite="hooded" x=300 y=0

???「（低声自语）数据已经转移完毕...」[l][r]

???「下一个目标...新来的转学生。」[l][r]

主角「（他们...在说我？！）」[l][r]

黑崎莲「（小声）看来你被盯上了。」[l][r]

@move layer=0 path="(,300,,opacity=0)" time=500

主角「（这到底是怎么回事...）」[l][r]

@jump target="discover_room"

; ============================================
; 分支：直接去查看
; ============================================

*go_check

@eval exp="f.ren_affection += 1"

主角「（去看看是什么。）」[l][r]

黑崎莲「...有胆量。」[l][r]

@bg storage="images/bg/old_building_intersection.png" time=500 method="crossfade"

我们走向声音传来的方向，但那里已经没有人了。[l][r]

主角「跑了...」[l][r]

黑崎莲「但是留下了这个。」[l][r]

@eval exp="f.clues_found += 1"

主角「这是...」[l][r]

@jump target="discover_room"

; ============================================
; 分支：让黑崎去
; ============================================

*send_ren

主角「黑崎同学，你能去看看吗？」[l][r]

黑崎莲「...切。」[l][r]

黑崎莲「在这里等着。」[l][r]

@move layer=0 path="(,500,,opacity=0)" time=500

@wait time=2000

@show_char name="ren" sprite="serious" x=500 y=0

黑崎莲「没人。但是发现了这个。」[l][r]

@eval exp="f.has_lab_access = true"

主角「一张门禁卡？」[l][r]

黑崎莲「旧校舍的地下实验室用的。」[l][r]

@jump target="discover_room"

; ============================================
; 发现密室
; ============================================

*discover_room

@bg storage="images/bg/old_building_door.png" time=800 method="crossfade"

我们来到一扇厚重的金属门前。[l][r]

黑崎莲「这就是目的地。」[l][r]

@if exp="f.has_lab_access"
@jump target="enter_with_card"
@else
@jump target="locked_door"
@endif

; ============================================
; 锁住的门
; ============================================

*locked_door

主角「门锁着...」[l][r]

黑崎莲「我们需要找到开门的方法。」[l][r]

*branch_door

@link target="find_entrance"「寻找其他入口」[r]
@link target="try_lockpick"「尝试撬锁」[r]
@link target="get_sakura_help"「回去找樱井帮忙」[r]
@s

; ============================================
; 使用门禁卡
; ============================================

*enter_with_card

@playse storage="audio/se/card_beep.wav"
@flash time=500

主角「门开了！」[l][r]

黑崎莲「走吧。」[l][r]

@jump target="enter_lab"

; ============================================
; 寻找入口
; ============================================

*find_entrance

@eval exp="f.clues_found += 1"

主角「（一定有其他入口。）」[l][r]

@bg storage="images/bg/old_building_vent.png" time=800 method="crossfade"

黑崎莲「通风管道...可以试试。」[l][r]

主角「这...真的能进去吗？」[l][r]

黑崎莲「跟着我。」[l][r]

@jump target="enter_lab"

; ============================================
; 尝试撬锁
; ============================================

*try_lockpick

主角「让我试试...」[l][r]

@playse storage="audio/se/lockpick_fail.wav"

主角「不行，锁太复杂了。」[l][r]

黑崎莲「我就说没用。」[l][r]

@jump target="find_entrance"

; ============================================
; 找樱井帮忙
; ============================================

*get_sakura_help

@eval exp="f.sakura_affection += 1"
@eval exp="f.trust_points += 1"

主角「（樱井同学可能有办法...）」[l][r]

黑崎莲「还要找那个女人？」[l][r]

主角「她可能有钥匙。」[l][r]

@flash time=500

樱井美咲「喂？怎么了？」[l][r]

主角「我们在旧校舍...门锁着。」[l][r]

樱井美咲「我马上来！」[l][r]

@wait time=1500

@show_char name="sakura" sprite="worried" x=100 y=0

樱井美咲「给你们钥匙。」[l][r]

@playse storage="audio/se/key_unlock.wav"

樱井美咲「但是...请小心。」[l][r]

@jump target="enter_lab"

; ============================================
; 进入实验室
; ============================================

*enter_lab

@bg storage="images/bg/old_lab.png" time=1500 method="crossfade"

@bgm storage="audio/bgm/revelation.mp3" volume=70

@lightning time=200

我们进入了地下实验室。[l][r]

主角「这是...」[l][r]

黑崎莲「记忆操纵实验室。」[l][r]

---

房间中央是一台巨大的机器，
周围散落着各种文件和设备。

---

主角「记忆操纵...真的存在？」[l][r]

黑崎莲「是的。而且...」[l][r]

黑崎莲「我是实验对象之一。」[l][r]

主角「黑崎同学...」[l][r]

黑崎莲「我失去了一部分记忆。」[l][r]

黑崎莲「但是我知道...这所学校还在继续实验。」[l][r]

@eval exp="f.knows_secret = true"
@eval exp="f.clues_found += 3"

*branch_lab

@link target="promise_together"「我们一起阻止他们」[r]
@link target="more_details"「告诉我更多细节」[r]
@s

; ============================================
; 承诺一起行动
; ============================================

*promise_together

@eval exp="f.ren_affection += 3"
@eval exp="f.trust_points += 2"

主角「我们一起阻止他们。」[l][r]

黑崎莲「...」[l][r]

黑崎莲「...谢了。」[l][r]

黑崎莲「第一次有人说要帮我。」[l][r]

@jump target="chapter1_old_building_end"

; ============================================
; 更多细节
; ============================================

*more_details

主角「告诉我更多细节。」[l][r]

黑崎莲「实验代号"星之子"。」[l][r]

黑崎莲「目的是创造没有痛苦记忆的人。」[l][r]

黑崎莲「但是副作用...是人格分裂。」[l][r]

主角「人格分裂？」[l][r]

黑崎莲「我有时候会变成另一个人。」[l][r]

黑崎莲「那个我...更冷酷，更危险。」[l][r]

@eval exp="f.knows_ren_secret = true"

@jump target="chapter1_old_building_end"

; ============================================
; 章节结束
; ============================================

*chapter1_old_building_end

@bg storage="images/bg/school_sunset.png" time=1500 method="crossfade"

@bgm storage="audio/bgm/bittersweet.mp3" volume=50

离开旧校舍后，夕阳已经西沉。[l][r]

主角「（今天发现了太多事情...）」[l][r]

@show_char name="ren" sprite="serious" x=500 y=0

黑崎莲「明天放学后，我们继续调查。」[l][r]

主角「好。」[l][r]

黑崎莲「...你比我想象的可靠。」[l][r]

主角「谢谢。」[l][r]

@fadeout time=1500

@call storage="chapter2_investigation.ks" target="*start"
