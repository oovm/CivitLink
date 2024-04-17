; ============================================
; 第一章 - 学生会线 (KAG 格式)
; ============================================

*chapter1_school_start

@bgm storage="audio/bgm/peaceful_theme.mp3" volume=50

@bg storage="images/bg/student_council_room.png" time=1000 method="crossfade"

@show_char name="sakura" sprite="smile" x=300 y=0

樱井美咲「欢迎来到学生会！」[l][r]

@show_char name="yuki" sprite="gentle" x=100 y=0

白雪悠希「啊，新同学也来了。欢迎欢迎。」[l][r]

主角「白雪同学也在啊。」[l][r]

白雪悠希「我是学生会的书记。樱井是副会长。」[l][r]

樱井美咲「没错！我们学生会可是学校的重要组织！」[l][r]

主角「（学生会...应该能了解到一些学校的事情。）」[l][r]

白雪悠希「对了，新同学，有件事想请你帮忙。」[l][r]

*branch_help

@link target="ask_for_help"「什么事？」[r]
@link target="refuse_help"「我很忙...」[r]
@s

; ============================================
; 分支：帮忙
; ============================================

*ask_for_help

@eval exp="f.yuki_affection += 1"
@eval exp="f.trust_points += 2"

白雪悠希「最近学校里发生了一些奇怪的事情。」[l][r]

樱井美咲「嗯...有学生说在旧校舍附近看到了奇怪的光。」[l][r]

主角「奇怪的光？」[l][r]

白雪悠希「我们想调查一下，但是旧校舍一直锁着...」[l][r]

樱井美咲「如果你能帮忙调查的话，我们会很感激的！」[l][r]

*branch_investigate

@link target="agree_to_help"「我愿意帮忙」[r]
@link target="worry_about_danger"「这听起来很危险...」[r]
@s

; ============================================
; 分支：拒绝帮忙
; ============================================

*refuse_help

@eval exp="f.yuki_affection -= 1"
@eval exp="f.sakura_affection -= 1"

樱井美咲「这样啊...」[l][r]

白雪悠希「没关系，我们理解。」[l][r]

主角「（他们的表情...好像很需要帮助。）」[l][r]

*branch_change_mind

@link target="ask_for_help"「等等，我改变主意了」[r]
@link target="leave_student_council"「告辞了」[r]
@s

; ============================================
; 同意帮忙
; ============================================

*agree_to_help

@eval exp="f.trust_points += 2"

樱井美咲「太好了！谢谢你！」[l][r]

白雪悠希「有你帮忙就放心多了。」[l][r]

樱井美咲「黑崎同学好像知道怎么进入旧校舍...」[l][r]

主角「（黑崎？他确实说过知道怎么进去...）」[l][r]

@jump target="investigation_start"

; ============================================
; 担心危险
; ============================================

*worry_about_danger

樱井美咲「别担心！我们只是想看看是什么情况。」[l][r]

白雪悠希「如果你害怕的话，可以和我们一起行动。」[l][r]

@eval exp="f.sakura_affection += 1"

@jump target="investigation_start"

; ============================================
; 调查开始
; ============================================

*investigation_start

@bg storage="images/bg/school_hallway.png" time=800 method="crossfade"

主角「好，我来帮忙调查。」[l][r]

@show_char name="sakura" sprite="happy" x=300 y=0

樱井美咲「太好了！那我们明天放学后在学校门口集合！」[l][r]

@fadeout time=1000

@call storage="chapter2_investigation.ks" target="*start"

; ============================================
; 离开学生会
; ============================================

*leave_student_council

@bg storage="images/bg/school_corridor.png" time=800 method="crossfade"

主角「（也许我不该卷入这件事...）」[l][r]

@fadeout time=1000

@call storage="chapter1_home.ks" target="*start"

; ============================================
; 额外场景：深入了解
; ============================================

*sakura_secret_talk

@bgm storage="audio/bgm/mystery_theme.mp3" volume=50

樱井美咲「其实...我有件事想告诉你。」[l][r]

主角「什么事？」[l][r]

樱井美咲「这所学校...隐藏着一个秘密。」[l][r]

樱井美咲「关于"记忆"的秘密...」[l][r]

主角「记忆？」[l][r]

樱井美咲「我不能说太多...但是请小心黑崎同学。」[l][r]

樱井美咲「他...不是普通人。」[l][r]

主角「（黑崎同学？他到底是谁？）」[l][r]

@eval exp="f.clues_found += 1"

@jump target="investigation_start"

; ============================================
; 额外场景：白雪的情报
; ============================================

*yuki_information

白雪悠希「我查到了一些资料...」[l][r]

白雪悠希「十年前，这所学校曾经进行过一项秘密实验。」[l][r]

主角「秘密实验？」[l][r]

白雪悠希「代号"星之子计划"。」[l][r]

白雪悠希「目的是...操纵人类的记忆。」[l][r]

主角「操纵记忆？！」[l][r]

白雪悠希「实验后来被终止了，但是...」[l][r]

白雪悠希「据说有些实验对象还在学校里。」[l][r]

@eval exp="f.clues_found += 2"
@eval exp="f.knows_secret = true"

主角「（实验对象...还在学校里？）」[l][r]

@jump target="investigation_start"
