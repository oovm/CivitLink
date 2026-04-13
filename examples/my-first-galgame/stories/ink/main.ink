// Ink 风格 DSL
// 这是 Inkle 工作室开发的叙事脚本语言

// ============================================
// 故事配置
// ============================================

VAR sakura_affection = 0
VAR ren_affection = 0
VAR yuki_affection = 0
VAR trust_points = 0
VAR clues_found = 0
VAR knows_secret = false
VAR ending_type = ""

// ============================================
// 角色列表
// ============================================

LIST characters = player, sakura, ren, yuki

// ============================================
// 包含其他文件
// ============================================

INCLUDE chapter1_school
INCLUDE chapter1_home
INCLUDE chapter1_old_building
INCLUDE endings

// ============================================
// 主入口
// ============================================

-> prologue

// ============================================
// 序章
// ============================================

== prologue ==

~ audio.play("audio/bgm/mystery_theme.mp3", 0.6)
~ scene.change("images/bg/school_gate_sunset.png", "fade", 1.5)

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。

我转学到这里已经一周了，却总觉得有什么不对劲。

~ effect.flash(0.3)

主角：（奇怪...那个旧校舍，为什么总是锁着？）

~ character.show("sakura", "curious", "left", "slide_left")

樱井美咲：新同学！你在看什么呢？

主角：啊，樱井同学...没什么，只是在发呆。

樱井美咲：你的表情可不像在发呆哦。对了，放学后学生会有个活动，要来吗？

*   [好啊，我去看看]
        ~ sakura_affection++
        ~ trust_points++
        -> accept_student_council

*   [抱歉，我还有事]
        ~ sakura_affection--
        -> decline_student_council

*   [学生会有什么活动？]
        -> ask_about_activity

// ============================================
// 分支：接受邀请
// ============================================

== accept_student_council ==

樱井美咲：太好了！那放学后见！

~ character.show("sakura", "happy", "left")

-> after_school_choice

// ============================================
// 分支：拒绝邀请
// ============================================

== decline_student_council ==

樱井美咲：这样啊...那下次有机会再说吧。

~ character.show("sakura", "disappointed", "left")
~ character.hide("left", "slide_left_out")

主角：（她的表情...好像有些失落。）

-> after_school_choice

// ============================================
// 分支：询问活动
// ============================================

== ask_about_activity ==

樱井美咲：嘿嘿，这是秘密哦。来了你就知道了~

主角：（秘密？）

-> after_school_choice

// ============================================
// 放学后选择
// ============================================

== after_school_choice ==

~ scene.change("images/bg/classroom_afternoon.png", "fade", 1.0)

放学后，我面临一个选择。

~ character.show("ren", "cool", "right", "slide_right")

黑崎莲：喂，新来的。要不要跟我去个地方？

主角：黑崎同学？去哪里？

黑崎莲：旧校舍。我知道怎么进去。

*   {sakura_affection >= 0} [跟樱井去学生会]
        ~ school_route = true
        主角：（还是先去学生会看看吧。）
        黑崎莲：切，无聊。
        ~ character.hide("right", "slide_right_out")
        -> chapter1_school.start

*   [跟黑崎去旧校舍]
        ~ school_route = true
        ~ ren_affection += 2
        主角：（旧校舍...也许能发现什么。）
        黑崎莲：明智的选择。
        -> chapter1_old_building.start

*   [回家]
        ~ home_route = true
        主角：（今天还是先回家吧。）
        黑崎莲：随你便。
        -> chapter1_home.start

// ============================================
// 学生会线
// ============================================

== chapter1_school ==

~ audio.play("audio/bgm/peaceful_theme.mp3", 0.5)
~ scene.change("images/bg/student_council_room.png", "fade", 1.0)

~ character.show("sakura", "smile", "center")

樱井美咲：欢迎来到学生会！

~ character.show("yuki", "gentle", "left", "slide_left")

白雪悠希：啊，新同学也来了。欢迎欢迎。

主角：白雪同学也在啊。

白雪悠希：我是学生会的书记。樱井是副会长。

樱井美咲：没错！我们学生会可是学校的重要组织！

主角：（学生会...应该能了解到一些学校的事情。）

白雪悠希：对了，新同学，有件事想请你帮忙。

*   [什么事？]
        ~ yuki_affection++
        ~ trust_points += 2
        -> ask_for_help

*   [我很忙...]
        ~ yuki_affection--
        ~ sakura_affection--
        -> refuse_help

// ============================================
// 分支：帮忙
// ============================================

== ask_for_help ==

白雪悠希：最近学校里发生了一些奇怪的事情。

樱井美咲：嗯...有学生说在旧校舍附近看到了奇怪的光。

主角：奇怪的光？

白雪悠希：我们想调查一下，但是旧校舍一直锁着...

樱井美咲：如果你能帮忙调查的话，我们会很感激的！

*   [我愿意帮忙]
        ~ trust_points += 2
        樱井美咲：太好了！谢谢你！
        白雪悠希：有你帮忙就放心多了。
        -> investigation_start

*   [这听起来很危险...]
        樱井美咲：别担心！我们只是想看看是什么情况。
        白雪悠希：如果你害怕的话，可以和我们一起行动。
        ~ sakura_affection++
        -> investigation_start

// ============================================
// 分支：拒绝帮忙
// ============================================

== refuse_help ==

樱井美咲：这样啊...

白雪悠希：没关系，我们理解。

主角：（他们的表情...好像很需要帮助。）

*   [等等，我改变主意了]
        -> ask_for_help

*   [告辞了]
        -> leave_student_council

// ============================================
// 调查开始
// ============================================

== investigation_start ==

~ scene.change("images/bg/school_hallway.png", "fade", 0.8)

主角：好，我来帮忙调查。

~ character.show("sakura", "happy", "center")

樱井美咲：太好了！那我们明天放学后在学校门口集合！

~ effect.fade_out(1.0)

-> chapter2_investigation.start

// ============================================
// 离开学生会
// ============================================

== leave_student_council ==

~ scene.change("images/bg/school_corridor.png", "fade", 0.8)

主角：（也许我不该卷入这件事...）

~ effect.fade_out(1.0)

-> chapter1_home.start

// ============================================
// 条件判断示例
// ============================================

== check_affection ==

{
    - sakura_affection >= 5:
        -> sakura_good_ending
    - ren_affection >= 5:
        -> ren_good_ending
    - yuki_affection >= 5:
        -> yuki_good_ending
    - else:
        -> normal_ending
}

// ============================================
// 樱井美咲结局
// ============================================

== sakura_good_ending ==

~ scene.change("images/bg/cherry_blossom.png", "fade", 1.5)
~ audio.play("audio/bgm/romantic_theme.mp3", 0.6)

~ character.show("sakura", "blush", "center")

樱井美咲：那个...我有话想对你说。

主角：什么事？

樱井美咲：这段时间...谢谢你一直陪在我身边。

樱井美咲：我...我好像...

主角：？

樱井美咲：我喜欢你！

~ effect.heart_float(1.0)

主角：樱井同学...

樱井美咲：以后...请多多指教了！

~ effect.fade_out(2.0)

=== 樱井美咲结局：樱花之约 ===

你与樱井美咲成为了恋人。
在樱花树下，你们许下了永远的约定。

-> show_stats

// ============================================
// 游戏统计
// ============================================

== show_stats ==

---

游戏统计

* 樱井美咲好感度: {sakura_affection}
* 黑崎莲好感度: {ren_affection}
* 白雪悠希好感度: {yuki_affection}
* 信任点数: {trust_points}
* 发现线索: {clues_found}

---

感谢游玩《星光学院的秘密》！

如果想要体验其他结局，请尝试不同的选择路径。

-> DONE
