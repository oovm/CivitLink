; KAG/Kirikiri 风格 DSL
; 这是日本最常用的视觉小说引擎 Kirikiri 的 KAG 脚本语法

; ============================================
; 宏定义
; ============================================

@macro name="dialogue"
@eval exp="f.characters[mp.character].name"
@font size=24
@emb exp="mp.text"
@font size=default
@endmacro

@macro name="show_char"
@image storage="images/char/%(mp.name)_%(mp.sprite).png" layer=0 page=fore visible=true
@move layer=0 path="(,%(mp.x),%(mp.y),opacity=255)" time=500
@endmacro

@macro name="choice"
@link target="%(mp.target)" storage="%(mp.storage)" %(mp.condition)
@emb exp="mp.text"
@endlink
@endmacro

; ============================================
; 变量初始化
; ============================================

@iscript
f.sakura_affection = 0;
f.ren_affection = 0;
f.yuki_affection = 0;
f.trust_points = 0;
f.clues_found = 0;
f.knows_secret = false;

f.characters = {
    player: { name: "主角" },
    sakura: { name: "樱井美咲" },
    ren: { name: "黑崎莲" },
    yuki: { name: "白雪悠希" }
};
@endscript

; ============================================
; 角色定义
; ============================================

*start|开始游戏

@jump target="prologue"

; ============================================
; 序章
; ============================================

*prologue|序章

@bgm storage="audio/bgm/mystery_theme.mp3" volume=60

@bg storage="images/bg/school_gate_sunset.png" time=1500 method="crossfade"

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。[l][r]
我转学到这里已经一周了，却总觉得有什么不对劲。[l][r]

@wait time=300
@flash time=300

主角「（奇怪...那个旧校舍，为什么总是锁着？）」[l][r]

@show_char name="sakura" sprite="curious" x=100 y=0

樱井美咲「新同学！你在看什么呢？」[l][r]

主角「啊，樱井同学...没什么，只是在发呆。」[l][r]

樱井美咲「你的表情可不像在发呆哦。对了，放学后学生会有个活动，要来吗？」[l][r]

@choice_storage="main.ks"
*branch_student_council

@link target="accept_student_council"「好啊，我去看看」[r]
@link target="decline_student_council"「抱歉，我还有事」[r]
@link target="ask_about_activity"「学生会有什么活动？」[r]
@s

; ============================================
; 分支：接受邀请
; ============================================

*accept_student_council

@eval exp="f.sakura_affection += 1"
@eval exp="f.trust_points += 1"

樱井美咲「太好了！那放学后见！」[l][r]

@show_char name="sakura" sprite="happy" x=100 y=0

@jump target="after_school_choice"

; ============================================
; 分支：拒绝邀请
; ============================================

*decline_student_council

@eval exp="f.sakura_affection -= 1"

樱井美咲「这样啊...那下次有机会再说吧。」[l][r]

@show_char name="sakura" sprite="disappointed" x=100 y=0
@move layer=0 path="(,100,,opacity=0)" time=500

主角「（她的表情...好像有些失落。）」[l][r]

@jump target="after_school_choice"

; ============================================
; 分支：询问活动
; ============================================

*ask_about_activity

樱井美咲「嘿嘿，这是秘密哦。来了你就知道了~」[l][r]

主角「（秘密？）」[l][r]

@jump target="after_school_choice"

; ============================================
; 放学后选择
; ============================================

*after_school_choice

@bg storage="images/bg/classroom_afternoon.png" time=1000 method="crossfade"

放学后，我面临一个选择。[l][r]

@show_char name="ren" sprite="cool" x=500 y=0

黑崎莲「喂，新来的。要不要跟我去个地方？」[l][r]

主角「黑崎同学？去哪里？」[l][r]

黑崎莲「旧校舍。我知道怎么进去。」[l][r]

*branch_after_school

@if exp="f.sakura_affection >= 0"
@link target="route_student_council"「跟樱井去学生会」[r]
@endif

@link target="route_old_building"「跟黑崎去旧校舍」[r]
@link target="route_home"「回家」[r]
@s

; ============================================
; 路线：学生会
; ============================================

*route_student_council

@eval exp="f.school_route = true"

主角「（还是先去学生会看看吧。）」[l][r]

黑崎莲「切，无聊。」[l][r]

@move layer=0 path="(,500,,opacity=0)" time=500

@call storage="chapter1_school.ks" target="*start"

; ============================================
; 路线：旧校舍
; ============================================

*route_old_building

@eval exp="f.school_route = true"
@eval exp="f.ren_affection += 2"

主角「（旧校舍...也许能发现什么。）」[l][r]

黑崎莲「明智的选择。」[l][r]

@call storage="chapter1_old_building.ks" target="*start"

; ============================================
; 路线：回家
; ============================================

*route_home

@eval exp="f.home_route = true"

主角「（今天还是先回家吧。）」[l][r]

黑崎莲「随你便。」[l][r]

@call storage="chapter1_home.ks" target="*start"

; ============================================
; 学生会线详细
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
; 结局判定
; ============================================

*endings

@if exp="f.ending_type == 'public'"
@jump target="public_truth_ending"
@elsif exp="f.ending_type == 'restore'"
@jump target="restore_memory_ending"
@elsif exp="f.ending_type == 'investigate'"
@jump target="true_ending"
@else
@jump target="normal_ending"
@endif

; ============================================
; 樱井美咲结局
; ============================================

*sakura_good_ending

@bg storage="images/bg/cherry_blossom.png" time=1500 method="crossfade"

@bgm storage="audio/bgm/romantic_theme.mp3" volume=60

@show_char name="sakura" sprite="blush" x=300 y=0

樱井美咲「那个...我有话想对你说。」[l][r]

主角「什么事？」[l][r]

樱井美咲「这段时间...谢谢你一直陪在我身边。」[l][r]

樱井美咲「我...我好像...」[l][r]

主角「？」[l][r]

樱井美咲「我喜欢你！」[l][r]

@quake layer=0 time=1000 max=10

主角「樱井同学...」[l][r]

樱井美咲「以后...请多多指教了！」[l][r]

@fadeout time=2000

@locate target="*game_end"

; ============================================
; 游戏统计
; ============================================

*show_stats

樱井美咲好感度: [emb exp="f.sakura_affection"][r]
黑崎莲好感度: [emb exp="f.ren_affection"][r]
白雪悠希好感度: [emb exp="f.yuki_affection"][r]
信任点数: [emb exp="f.trust_points"][r]
发现线索: [emb exp="f.clues_found"][r]

感谢游玩《星光学院的秘密》！[l][r]

@title

; ============================================
; 游戏结束
; ============================================

*game_end

@close
