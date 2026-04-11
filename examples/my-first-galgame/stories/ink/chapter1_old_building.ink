// ============================================
// 第一章 - 旧校舍线 (Ink 格式)
// ============================================

== chapter1_old_building ==

~ audio.play("audio/bgm/suspense_theme.mp3", 0.6)
~ scene.change("images/bg/old_building_exterior.png", "fade", 1.0)

~ character.show("ren", "smirk", "right")

黑崎莲：到了。这就是旧校舍。

主角：看起来已经废弃很久了...

黑崎莲：表面上是这样。

黑崎莲：但实际上，这里藏着很多秘密。

主角：秘密？

黑崎莲：跟我来。

~ scene.change("images/bg/old_building_side.png", "fade", 0.8)

~ audio.play_se("audio/se/metal_gate.wav")
~ effect.shake(0.3)

黑崎带我来到旧校舍的侧面，那里有一扇生锈的铁门。

黑崎莲：这扇门的锁早就坏了。

~ audio.play_se("audio/se/door_open_creak.wav")
~ scene.change("images/bg/old_building_corridor.png", "fade", 1.0)

我们进入了旧校舍内部。

~ effect.flash(0.5)

主角：好暗...

黑崎莲：小心脚下。

*   [打开手电筒]
        -> use_flashlight

*   [跟着黑崎走]
        -> follow_ren

*   [询问黑崎为什么知道这里]
        -> ask_ren_knowledge

// ============================================
// 分支：使用手电筒
// ============================================

== use_flashlight ==

~ clues_found++

~ audio.play_se("audio/se/flashlight_on.wav")
~ effect.flash(0.5)

主角：（手电筒的光照亮了走廊...）

主角：（墙上好像有什么字...）

---

墙上写着：
"不要相信他们"
"记忆是假的"
"救救我"

---

主角：这些字...是什么意思？

黑崎莲：看来有人在这里待过很久。

-> explore_corridor

// ============================================
// 分支：跟着黑崎
// ============================================

== follow_ren ==

~ ren_affection++

主角：（相信黑崎同学吧。）

黑崎莲：...聪明。

主角：（他好像对我有点改观？）

-> explore_corridor

// ============================================
// 分支：询问黑崎
// ============================================

== ask_ren_knowledge ==

~ clues_found += 2

主角：黑崎同学，你怎么知道这个地方？

黑崎莲：...

黑崎莲：我...曾经在这里待过。

主角：什么意思？

黑崎莲：我是...实验的幸存者之一。

~ knows_secret = true
~ knows_ren_secret = true

主角：实验...？

黑崎莲：以后再解释。先往前走。

-> explore_corridor

// ============================================
// 探索走廊
// ============================================

== explore_corridor ==

~ scene.change("images/bg/old_building_hallway.png", "fade", 0.8)

我们沿着走廊前进，两边是废弃的教室。

~ effect.flash(0.2)
~ audio.play_se("audio/se/strange_sound.wav")

突然，一阵奇怪的声音从远处传来！

主角：什么声音？

黑崎莲：有人在这里。

*   [躲起来观察]
        -> hide_and_observe

*   [直接去查看]
        -> go_check

*   [让黑崎去查看]
        -> send_ren

// ============================================
// 分支：躲起来观察
// ============================================

== hide_and_observe ==

~ clues_found += 2
~ trust_points++

主角：（先躲起来看看情况。）

~ scene.change("images/bg/old_building_shadow.png", "fade", 0.5)

我们躲在一扇破旧的门后。

~ character.show("unknown", "hooded", "center", "fade_in")

???：（低声自语）数据已经转移完毕...

???：下一个目标...新来的转学生。

主角：（他们...在说我？！）

黑崎莲：（小声）看来你被盯上了。

~ character.hide("center", "fade_out")

主角：（这到底是怎么回事...）

-> discover_room

// ============================================
// 分支：直接去查看
// ============================================

== go_check ==

~ ren_affection++

主角：（去看看是什么。）

黑崎莲：...有胆量。

~ scene.change("images/bg/old_building_intersection.png", "fade", 0.5)

我们走向声音传来的方向，但那里已经没有人了。

主角：跑了...

黑崎莲：但是留下了这个。

~ clues_found++

主角：这是...

-> discover_room

// ============================================
// 分支：让黑崎去
// ============================================

== send_ren ==

主角：黑崎同学，你能去看看吗？

黑崎莲：...切。

黑崎莲：在这里等着。

~ character.hide("right", "slide_right_out")

~ effect.pause(2.0)

~ character.show("ren", "serious", "right", "slide_right")

黑崎莲：没人。但是发现了这个。

~ has_lab_access = true

主角：一张门禁卡？

黑崎莲：旧校舍的地下实验室用的。

-> discover_room

// ============================================
// 发现密室
// ============================================

== discover_room ==

~ scene.change("images/bg/old_building_door.png", "fade", 0.8)

我们来到一扇厚重的金属门前。

黑崎莲：这就是目的地。

{
    - has_lab_access:
        -> enter_with_card
    - else:
        -> locked_door
}

// ============================================
// 锁住的门
// ============================================

== locked_door ==

主角：门锁着...

黑崎莲：我们需要找到开门的方法。

*   [寻找其他入口]
        -> find_entrance

*   [尝试撬锁]
        -> try_lockpick

*   [回去找樱井帮忙]
        -> get_sakura_help

// ============================================
// 使用门禁卡
// ============================================

== enter_with_card ==

~ audio.play_se("audio/se/card_beep.wav")
~ effect.flash(0.5)

主角：门开了！

黑崎莲：走吧。

-> enter_lab

// ============================================
// 寻找入口
// ============================================

== find_entrance ==

~ clues_found++

主角：（一定有其他入口。）

~ scene.change("images/bg/old_building_vent.png", "fade", 0.8)

黑崎莲：通风管道...可以试试。

主角：这...真的能进去吗？

黑崎莲：跟着我。

-> enter_lab

// ============================================
// 尝试撬锁
// ============================================

== try_lockpick ==

主角：让我试试...

~ audio.play_se("audio/se/lockpick_fail.wav")

主角：不行，锁太复杂了。

黑崎莲：我就说没用。

-> find_entrance

// ============================================
// 找樱井帮忙
// ============================================

== get_sakura_help ==

~ sakura_affection++
~ trust_points++

主角：（樱井同学可能有办法...）

黑崎莲：还要找那个女人？

主角：她可能有钥匙。

~ effect.flash(0.5)

樱井美咲：喂？怎么了？

主角：我们在旧校舍...门锁着。

樱井美咲：我马上来！

~ effect.clock_transition(1.5)

~ character.show("sakura", "worried", "left", "slide_left")

樱井美咲：给你们钥匙。

~ audio.play_se("audio/se/key_unlock.wav")

樱井美咲：但是...请小心。

-> enter_lab

// ============================================
// 进入实验室
// ============================================

== enter_lab ==

~ scene.change("images/bg/old_lab.png", "fade", 1.5)

~ audio.play("audio/bgm/revelation.mp3", 0.7)
~ effect.lightning(0.2)

我们进入了地下实验室。

主角：这是...

黑崎莲：记忆操纵实验室。

---

房间中央是一台巨大的机器，
周围散落着各种文件和设备。

---

主角：记忆操纵...真的存在？

黑崎莲：是的。而且...

黑崎莲：我是实验对象之一。

主角：黑崎同学...

黑崎莲：我失去了一部分记忆。

黑崎莲：但是我知道...这所学校还在继续实验。

~ knows_secret = true
~ clues_found += 3

*   [我们一起阻止他们]
        -> promise_together

*   [告诉我更多细节]
        -> more_details

// ============================================
// 承诺一起行动
// ============================================

== promise_together ==

~ ren_affection += 3
~ trust_points += 2

主角：我们一起阻止他们。

黑崎莲：...

黑崎莲：...谢了。

黑崎莲：第一次有人说要帮我。

-> chapter1_old_building_end

// ============================================
// 更多细节
// ============================================

== more_details ==

主角：告诉我更多细节。

黑崎莲：实验代号"星之子"。

黑崎莲：目的是创造没有痛苦记忆的人。

黑崎莲：但是副作用...是人格分裂。

主角：人格分裂？

黑崎莲：我有时候会变成另一个人。

黑崎莲：那个我...更冷酷，更危险。

~ knows_ren_secret = true

-> chapter1_old_building_end

// ============================================
// 章节结束
// ============================================

== chapter1_old_building_end ==

~ scene.change("images/bg/school_sunset.png", "fade", 1.5)

~ audio.play("audio/bgm/bittersweet.mp3", 0.5)

离开旧校舍后，夕阳已经西沉。

主角：（今天发现了太多事情...）

~ character.show("ren", "serious", "right")

黑崎莲：明天放学后，我们继续调查。

主角：好。

黑崎莲：...你比我想象的可靠。

主角：谢谢。

~ effect.fade_out(1.5)

-> chapter2_investigation.start
