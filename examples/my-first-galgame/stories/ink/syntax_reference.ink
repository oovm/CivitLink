// Ink 风格 DSL - 语法参考
// Inkle 工作室开发的叙事脚本语言

// ============================================
// 变量定义
// ============================================

VAR sakura_affection = 0
VAR ren_affection = 0
VAR yuki_affection = 0
VAR trust_points = 0
VAR clues_found = 0
VAR knows_secret = false
VAR knows_ren_secret = false
VAR has_lab_key = false
VAR has_lab_access = false
VAR school_route = false
VAR home_route = false
VAR ending_type = ""

// ============================================
// 列表定义
// ============================================

LIST characters = player, sakura, ren, yuki
LIST inventory = ()

// ============================================
// 包含其他文件
// ============================================

INCLUDE chapter1_school
INCLUDE chapter1_home
INCLUDE chapter1_old_building
INCLUDE chapter2_investigation
INCLUDE endings

// ============================================
// 语法规则说明
// ============================================

/*
1. 变量定义：VAR 变量名 = 初始值
2. 列表定义：LIST 列表名 = (元素1, 元素2)
3. 场景定义：== 场景名 ==
4. 选择分支：* [选项文本]
5. 条件选项：* {条件} [选项文本]
6. 变量操作：~ 变量名++
7. 跳转：-> 目标
8. 条件判断：{ - 条件: 内容 }
9. 函数调用：~ function()
10. 注释：// 单行注释 或 /* 多行注释 */
*/

// ============================================
// 完整示例场景
// ============================================

== example_scene ==

~ audio.play("bgm/peaceful.mp3", 0.5)
~ scene.change("bg/classroom.png", "fade", 1.0)

~ character.show("sakura", "smile", "center")

樱井美咲：你好！欢迎来到星光学院！

*   [你好！]
        主角：你好！
        ~ sakura_affection++
        -> next_part

*   [请问你是？]
        主角：请问你是？
        -> introduce

*   {knows_secret} [关于那个秘密...]
        主角：关于那个秘密...
        -> secret_talk

== introduce ==

樱井美咲：我是樱井美咲，学生会副会长！

-> next_part

== next_part ==

樱井美咲：要不要参观一下学校？

*   [好啊]
        主角：好啊！
        -> tour

*   [下次吧]
        主角：下次吧，谢谢。
        -> end

== tour ==

~ scene.change("bg/school_hallway.png", "slide_right", 0.8)

樱井美咲：这里是教学楼，那边是图书馆...

-> end

== end ==

樱井美咲：再见！

~ effect.fade_out(1.0)

-> DONE

// ============================================
// 复杂分支示例
// ============================================

== complex_choice ==

黑崎莲：你想知道真相吗？

*   [我想知道一切]
        ~ trust_points += 2
        ~ ren_affection += 1
        -> reveal_all

*   [太危险了，算了吧]
        ~ trust_points -= 1
        -> avoid_danger

*   {clues_found >= 3} [我已经知道一些了]
        ~ ren_affection += 2
        主角：我已经发现了一些线索...
        -> share_intel

*   {knows_ren_secret} [你也是实验对象？]
        ~ ren_affection += 3
        主角：黑崎同学，你也是...
        -> ren_confession

// ============================================
// 多条件判断示例
// ============================================

== determine_ending ==

{
    - sakura_affection >= 8 && trust_points >= 10:
        -> sakura_true_ending
    - ren_affection >= 8 && knows_ren_secret:
        -> ren_true_ending
    - yuki_affection >= 6 && clues_found >= 5:
        -> yuki_true_ending
    - sakura_affection >= 5:
        -> sakura_good_ending
    - ren_affection >= 5:
        -> ren_good_ending
    - yuki_affection >= 5:
        -> yuki_good_ending
    - trust_points >= 8:
        -> good_ending
    - else:
        -> normal_ending
}

// ============================================
// 结局定义
// ============================================

=== 樱井美咲结局：樱花之约 ===

你与樱井美咲成为了恋人。
在樱花树下，你们许下了永远的约定。

-> show_stats

=== 黑崎莲结局：暗夜的盟约 ===

你获得了黑崎莲的认可。
虽然他表面冷淡，但你知道，他会是你最可靠的伙伴。

-> show_stats

=== 真结局：永恒的星光 ===

你不仅揭开了学校的秘密，还找到了真正的爱情。

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

-> DONE

// ============================================
// 常用命令参考
// ============================================

// 音频命令
// ~ audio.play("文件路径", 音量)
// ~ audio.stop()
// ~ audio.fade_out(时长)

// 场景命令
// ~ scene.change("背景图片路径", "过渡效果", 时长)
// 过渡效果：fade, slide_left, slide_right, slide_up, slide_down

// 角色命令
// ~ character.show("角色ID", "表情", "位置", "动画")
// ~ character.hide("位置", "动画")
// 位置：left, center, right
// 动画：slide_left, slide_right, fade_in, fade_out

// 特效命令
// ~ effect.flash(时长)
// ~ effect.shake(时长, 强度)
// ~ effect.heart_float(时长)
// ~ effect.sparkle(时长)
// ~ effect.fade_out(时长)
