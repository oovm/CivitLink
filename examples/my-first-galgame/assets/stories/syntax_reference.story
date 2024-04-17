# Story 脚本语法参考
# 星光学院的秘密 - Galgame 脚本语言

# ============================================
# 核心语法规则
# ============================================

# 语法规则说明：
# 1. % 开头 + 空格 + 命令 + 空格 + DSL = 命令定义
#    例如：% let x = 0, % include file.story
# 2. % 开头但不加空格 = 命令调用
#    例如：%audio::play("bgm.mp3", 0.6)
# 3. %{ } = 插值表达式
#    例如：%{sakura_affection++}
# 4. \% = 输出 % 字符
# 5. a::b::c = 函数路径（模块调用）
# 6. a.b = 方法调用（对象方法）

# ============================================
# 1. 变量定义
# ============================================
# 格式：% let 变量名 = 初始值

% let sakura_affection = 0
% let ren_affection = 0
% let yuki_affection = 0
% let trust_points = 0
% let clues_found = 0
% let knows_secret = false
% let knows_ren_secret = false
% let has_lab_key = false
% let has_lab_access = false
% let school_route = false
% let home_route = false
% let ending_type = ""

# ============================================
# 2. 列表定义
# ============================================
# 格式：% let 列表名 = [元素1, 元素2, ...]

% let characters = [player, sakura, ren, yuki]
% let inventory = []
% let flags = []

# ============================================
# 3. 包含其他文件
# ============================================
# 格式：% include 文件名

% include chapter1_school.story
% include chapter1_home.story
% include chapter1_old_building.story
% include chapter2_investigation.story
% include endings.story

# ============================================
# 4. 场景定义
# ============================================
# 格式：== 场景名 ==

== prologue ==
这是一个场景的开始。

# ============================================
# 5. 选项分支
# ============================================
# 格式：* [选项文本]
#       缩进内容

* [好啊，我去看看]
    主角：好啊，我去看看！
    -> next_scene

* [抱歉，我还有事]
    主角：抱歉，我还有事。
    -> other_scene

# ============================================
# 6. 条件选项
# ============================================
# 格式：* {条件表达式} [选项文本]

* {sakura_affection >= 5} [特殊选项：深入了解]
    樱井美咲：其实...我有话想对你说。
    -> special_route

* {knows_secret == true} [询问秘密]
    主角：关于那个秘密...
    -> secret_talk

* {clues_found >= 3 && trust_points >= 5} [揭示真相]
    主角：我已经知道一切了。
    -> reveal_truth

# ============================================
# 7. 命令调用
# ============================================
# 格式：%模块::函数(参数)
# 使用 :: 表示函数路径

%audio::play("bgm/theme.mp3", 0.6)
%audio::stop()
%audio::fade_out(1.5)

%scene::change("bg/school.png", "fade", 1.0)
%scene::transition("slide_left", 0.8)

%character::show("sakura", "smile", "left")
%character::hide("left", "fade_out")
%character::move("sakura", "center", "slide")

%effect::flash(0.3)
%effect::shake(0.5, 10)
%effect::heart_float(1.0)
%effect::sparkle(1.5)
%effect::fade_out(2.0)
%effect::lightning(0.2)

# ============================================
# 8. 方法调用
# ============================================
# 格式：对象.方法(参数)
# 使用 . 表示对象方法调用

~ @player.say("你好！")
~ @sakura.smile()
~ @inventory.add("mysterious_letter")
~ @flags.set("met_sakura", true)

# ============================================
# 9. 变量操作（插值表达式）
# ============================================
# 格式：~ %{表达式}

~ %{sakura_affection++}
~ %{ren_affection += 2}
~ %{yuki_affection -= 1}
~ %{trust_points += 3}
~ %{clues_found++}
~ %{knows_secret = true}
~ %{has_lab_key = true}

# 复杂表达式
~ %{trust_points = trust_points + sakura_affection}
~ %{total_score = sakura_affection + ren_affection + yuki_affection}

# ============================================
# 10. 跳转
# ============================================
# 格式：-> 目标

-> next_scene
-> chapter1_school.start
-> DONE

# ============================================
# 11. 条件判断
# ============================================
# 格式：{
#     - 条件1:
#         内容1
#     - 条件2:
#         内容2
#     - else:
#         内容3
# }

{
    - sakura_affection >= 8:
        -> sakura_true_ending
    - sakura_affection >= 5:
        -> sakura_good_ending
    - ren_affection >= 8:
        -> ren_true_ending
    - ren_affection >= 5:
        -> ren_good_ending
    - yuki_affection >= 5:
        -> yuki_good_ending
    - else:
        -> normal_ending
}

# ============================================
# 12. 结局定义
# ============================================
# 格式：=== 结局标题 ===

=== 樱井美咲结局：樱花之约 ===
你与樱井美咲成为了恋人。
在樱花树下，你们许下了永远的约定。

=== 黑崎莲结局：暗夜的盟约 ===
你获得了黑崎莲的认可。
虽然他表面冷淡，但你知道，他会是你最可靠的伙伴。

=== 真结局：永恒的星光 ===
你不仅揭开了学校的秘密，还找到了真正的爱情。

# ============================================
# 13. 分隔线
# ============================================
# 格式：---

---

游戏统计

---

# ============================================
# 14. 注释
# ============================================
# 格式：# 注释内容

# 这是一行注释
# 可以在任何地方使用

# ============================================
# 15. 输出 % 符号
# ============================================
# 格式：\%

显示百分比：\%
进度：50\%

# ============================================
# 16. 文本显示
# ============================================
# 格式：直接书写文本，或使用角色名：对话

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。

樱井美咲：你好！欢迎来到星光学院！

主角：你好！

# ============================================
# 完整示例场景
# ============================================

== example_scene ==

%audio::play("bgm/peaceful.mp3", 0.5)
%scene::change("bg/classroom.png", "fade", 1.0)

%character::show("sakura", "smile", "center")

樱井美咲：你好！欢迎来到星光学院！

* [你好！]
    主角：你好！
    ~ %{sakura_affection++}
    -> next_part

* [请问你是？]
    主角：请问你是？
    -> introduce

* {knows_secret == true} [关于那个秘密...]
    主角：关于那个秘密...
    -> secret_talk

== introduce ==

樱井美咲：我是樱井美咲，学生会副会长！

-> next_part

== next_part ==

樱井美咲：要不要参观一下学校？

* [好啊]
    主角：好啊！
    -> tour

* [下次吧]
    主角：下次吧，谢谢。
    -> end

== tour ==

%scene::change("bg/school_hallway.png", "slide_right", 0.8)

樱井美咲：这里是教学楼，那边是图书馆...

-> end

== end ==

樱井美咲：再见！

%effect::fade_out(1.0)

-> DONE

# ============================================
# 复杂分支示例
# ============================================

== complex_choice ==

黑崎莲：你想知道真相吗？

* [我想知道一切]
    ~ %{trust_points += 2}
    ~ %{ren_affection += 1}
    -> reveal_all

* [太危险了，算了吧]
    ~ %{trust_points -= 1}
    -> avoid_danger

* {clues_found >= 3} [我已经知道一些了]
    ~ %{ren_affection += 2}
    主角：我已经发现了一些线索...
    -> share_intel

* {knows_ren_secret == true} [你也是实验对象？]
    ~ %{ren_affection += 3}
    主角：黑崎同学，你也是...
    -> ren_confession

# ============================================
# 多条件判断示例
# ============================================

== determine_ending ==

{
    - sakura_affection >= 8 && trust_points >= 10:
        -> sakura_true_ending
    - ren_affection >= 8 && knows_ren_secret == true:
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

# ============================================
# 常用命令参考
# ============================================

# 音频命令
# %audio::play("文件路径", 音量)
# %audio::stop()
# %audio::fade_out(时长)
# %audio::play_se("音效文件路径")

# 场景命令
# %scene::change("背景图片路径", "过渡效果", 时长)
# 过渡效果：fade, slide_left, slide_right, slide_up, slide_down

# 角色命令
# %character::show("角色ID", "表情", "位置", "动画")
# %character::hide("位置", "动画")
# %character::move("角色ID", "目标位置", "动画")
# 位置：left, center, right
# 动画：slide_left, slide_right, fade_in, fade_out

# 特效命令
# %effect::flash(时长)
# %effect::shake(时长, 强度)
# %effect::heart_float(时长)
# %effect::sparkle(时长)
# %effect::fade_out(时长)
# %effect::lightning(时长)
# %effect::glow(时长)

# ============================================
# 语法特点总结
# ============================================

# 1. 缩进敏感：使用缩进来表示代码块
# 2. 简洁明了：语法设计简洁，易于阅读和编写
# 3. 强大的叙事能力：支持分支、条件、变量等功能
# 4. 模块化：可以通过 % include 包含其他文件
# 5. 清晰的调用方式：
#    - % 命令 DSL = 命令定义
#    - %module::func() = 模块函数调用
#    - %{expr} = 插值表达式
#    - object.method() = 对象方法调用

# ============================================
# 注意事项
# ============================================

# 1. 变量名不能包含空格和特殊字符
# 2. 场景名必须唯一
# 3. 跳转目标必须存在
# 4. 条件表达式必须是有效的逻辑表达式
# 5. 命令调用的参数必须正确
# 6. 使用 % 前缀时注意空格规则
