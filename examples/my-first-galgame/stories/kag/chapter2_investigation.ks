; ============================================
; 第二章 - 调查 (KAG 格式)
; ============================================

*chapter2_investigation_start

@bgm storage="audio/bgm/school_morning.mp3" volume=50

@bg storage="images/bg/school_classroom.png" time=1000 method="crossfade"

第二天早上。[l][r]

主角「（今天要继续调查...）」[l][r]

@show_char name="sakura" sprite="smile" x=100 y=0
@show_char name="yuki" sprite="gentle" x=500 y=0

樱井美咲「早上好！准备好了吗？」[l][r]

白雪悠希「我们今天放学后去旧校舍。」[l][r]

@if exp="f.knows_secret"
主角「（昨晚那个人说樱井同学是知情者...）」[l][r]
@else
主角「（希望能找到一些线索。）」[l][r]
@endif

*branch_ready

@link target="ready_to_investigate"「我准备好了」[r]
@link target="ask_about_others"「还有其他人要来吗？」[r]
@s

; ============================================
; 询问其他人
; ============================================

*ask_about_others

樱井美咲「嗯...黑崎同学说他也要来。」[l][r]

@show_char name="ren" sprite="cool" x=300 y=0

黑崎莲「当然要来。旧校舍可是我的地盘。」[l][r]

白雪悠希「黑崎同学...请不要太冲动。」[l][r]

黑崎莲「放心，我知道分寸。」[l][r]

@jump target="ready_to_investigate"

; ============================================
; 准备调查
; ============================================

*ready_to_investigate

@wait time=1500

@bgm storage="audio/bgm/mystery_theme.mp3" volume=60

@bg storage="images/bg/old_building_entrance.png" time=1000 method="crossfade"

放学后，我们来到了旧校舍前。[l][r]

樱井美咲「这里...真的好旧了。」[l][r]

@show_char name="ren" sprite="cool" x=500 y=0

黑崎莲「入口在这边。」[l][r]

@bg storage="images/bg/old_building_interior.png" time=1000 method="crossfade"

@playse storage="audio/se/door_creak.wav"
@quake layer=0 time=300 max=10

门发出刺耳的声音，缓缓打开。[l][r]

白雪悠希「好黑...」[l][r]

@flash time=200
@playse storage="audio/se/strange_light.wav"

突然，一道奇异的光从走廊深处闪过！[l][r]

主角「那是...！」[l][r]

黑崎莲「在那边！追！」[l][r]

*branch_chase

@link target="chase_light"「追上去」[r]
@link target="careful_advance"「小心前进」[r]
@link target="observe_first"「先观察一下」[r]
@s

; ============================================
; 分支：追上去
; ============================================

*chase_light

@eval exp="f.clues_found += 1"
@eval exp="f.ren_affection += 1"

主角「（不能让它跑了！）」[l][r]

@bg storage="images/bg/old_building_hallway.png" time=500 method="crossfade"

@playse storage="audio/se/running_footsteps.wav"

我们追着光跑过走廊，最后来到了一扇门前。[l][r]

黑崎莲「这是...实验室？」[l][r]

@jump target="discover_lab"

; ============================================
; 分支：小心前进
; ============================================

*careful_advance

@eval exp="f.trust_points += 1"
@eval exp="f.yuki_affection += 1"

主角「（太危险了，还是小心点。）」[l][r]

白雪悠希「对，我们还是小心一点比较好。」[l][r]

@bg storage="images/bg/old_building_hallway.png" time=1000 method="crossfade"

我们小心翼翼地前进，发现地上有一些奇怪的痕迹。[l][r]

@eval exp="f.has_lab_key = true"

主角「这是...一把钥匙？」[l][r]

@jump target="discover_lab"

; ============================================
; 分支：先观察
; ============================================

*observe_first

@eval exp="f.clues_found += 2"
@eval exp="f.sakura_affection += 1"

主角「（先观察一下情况。）」[l][r]

樱井美咲「看，光是从那边那个房间传来的。」[l][r]

主角「（樱井同学好像对这里很熟悉...）」[l][r]

樱井美咲「怎么了？」[l][r]

主角「没什么，我们过去看看吧。」[l][r]

@jump target="discover_lab"

; ============================================
; 发现实验室
; ============================================

*discover_lab

@bg storage="images/bg/old_lab.png" time=1500 method="crossfade"

@lightning time=200

@bgm storage="audio/bgm/revelation.mp3" volume=70

我们来到了一个废弃的实验室。[l][r]

樱井美咲「这...这是...」[l][r]

白雪悠希「不可能...为什么会有这种东西...」[l][r]

黑崎莲「看来传说是真的。」[l][r]

主角「（这些设备...看起来像是用来做某种实验的...）」[l][r]

---

桌上散落着文件：
"记忆操纵实验 - 第七次测试报告
实验对象：学生志愿者
结果：部分记忆成功被改写
副作用：轻微头痛，偶尔出现幻觉
..."

---

主角「记忆操纵...？」[l][r]

@show_char name="sakura" sprite="serious" x=100 y=0

樱井美咲「...你们想知道真相吗？」[l][r]

*branch_truth

@link target="reveal_truth"「告诉我真相」[r]
@link target="confront_sakura"「你早就知道了？」[r]
@s

; ============================================
; 揭示真相
; ============================================

*reveal_truth

樱井美咲「是的...我知道。」[l][r]

樱井美咲「这所学校...曾经进行过记忆操纵的实验。」[l][r]

樱井美咲「而我...是实验的参与者之一。」[l][r]

主角「什么？！」[l][r]

白雪悠希「樱井同学...」[l][r]

樱井美咲「我...我不记得那段时间发生了什么。」[l][r]

樱井美咲「但是我知道，我的记忆...被改写过。」[l][r]

@eval exp="f.knows_secret = true"

@jump target="final_choice"

; ============================================
; 质问樱井
; ============================================

*confront_sakura

@eval exp="f.sakura_affection -= 2"
@eval exp="f.trust_points -= 1"

主角「你早就知道了？为什么要瞒着我们？」[l][r]

樱井美咲「对不起...我害怕...」[l][r]

樱井美咲「害怕你们知道真相后会讨厌我...」[l][r]

*branch_forgive

@link target="forgive_sakura"「我理解你的顾虑」[r]
@link target="need_time"「我需要时间消化」[r]
@s

; ============================================
; 原谅樱井
; ============================================

*forgive_sakura

@eval exp="f.sakura_affection += 3"
@eval exp="f.trust_points += 2"

主角「没关系，我理解。」[l][r]

樱井美咲「谢谢你...」[l][r]

@jump target="reveal_truth"

; ============================================
; 需要时间
; ============================================

*need_time

主角「我需要时间消化这些信息。」[l][r]

樱井美咲「我明白...」[l][r]

@jump target="reveal_truth"

; ============================================
; 最终选择
; ============================================

*final_choice

樱井美咲「现在，我们有一个选择。」[l][r]

樱井美咲「我们可以把这一切公之于众，或者...」[l][r]

黑崎莲「或者什么？」[l][r]

樱井美咲「或者...找到当初的实验数据，恢复所有人的记忆。」[l][r]

*branch_ending

@link target="ending_public"「公布真相」[r]
@link target="ending_restore"「恢复记忆」[r]
@link target="ending_investigate"「先找到更多证据」[r]
@s

; ============================================
; 结局分支：公布真相
; ============================================

*ending_public

@eval exp="f.ending_type = 'public'"

主角「真相应该被公开。」[l][r]

樱井美咲「我同意。大家有权知道发生了什么。」[l][r]

@call storage="endings.ks" target="*start"

; ============================================
; 结局分支：恢复记忆
; ============================================

*ending_restore

@eval exp="f.ending_type = 'restore'"

主角「如果能恢复记忆...那才是最好的结果。」[l][r]

白雪悠希「是的，这样大家都能找回自己失去的记忆。」[l][r]

@call storage="endings.ks" target="*start"

; ============================================
; 结局分支：继续调查
; ============================================

*ending_investigate

@eval exp="f.ending_type = 'investigate'"
@eval exp="f.clues_found += 3"

主角「我们需要更多证据才能做决定。」[l][r]

黑崎莲「说得对，不能草率行事。」[l][r]

@call storage="endings.ks" target="*start"
