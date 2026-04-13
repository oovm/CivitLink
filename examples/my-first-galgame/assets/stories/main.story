# ============================================
# 星光学院的秘密 - 主线剧本
# ============================================
# 作者：灵之镜工作室
# 版本：1.0.0
# 类型：多线多结局 Galgame
# ============================================

# ============================================
# 角色定义
# ============================================

% let player_name = "主角"
% let sakura_name = "樱井美咲"
% let ren_name = "黑崎莲"
% let yuki_name = "白雪悠希"

# ============================================
# 好感度变量
# ============================================

% let sakura_affection = 0
% let ren_affection = 0
% let yuki_affection = 0

# ============================================
# 信任与进度变量
# ============================================

% let trust_points = 0
% let clues_found = 0
% let investigation_progress = 0

# ============================================
# 剧情标志变量
# ============================================

% let knows_secret = false
% let knows_ren_secret = false
% let has_lab_key = false
% let has_lab_access = false
% let has_mysterious_letter = false
% let has_mysterious_document = false
% let met_mystery_person = false

# ============================================
# 路线变量
# ============================================

% let school_route = false
% let home_route = false
% let old_building_route = false

# ============================================
# 结局变量
# ============================================

% let ending_type = ""
% let true_ending_unlocked = false

# ============================================
# 包含其他章节
# ============================================

% include chapter1_school.story
% include chapter1_home.story
% include chapter1_old_building.story
% include chapter2_investigation.story
% include endings.story

# ============================================
# 序章：转学生的第一天
# ============================================

== prologue ==

%audio::play("bgm/mystery_theme.mp3", 0.6)
%scene::change("bg/school_gate_sunset.png", "fade", 1.5)

星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。

我转学到这里已经一周了，却总觉得有什么不对劲。

%effect::flash(0.3)

主角：（奇怪...那个旧校舍，为什么总是锁着？）

%character::show("sakura", "curious", "left", "slide_left")

樱井美咲：新同学！你在看什么呢？

主角：啊，樱井同学...没什么，只是在发呆。

樱井美咲：你的表情可不像在发呆哦。对了，放学后学生会有个活动，要来吗？

* [好啊，我去看看]
    主角：好啊，我去看看！
    ~ %{sakura_affection++}
    ~ %{trust_points++}
    -> accept_student_council

* [抱歉，我还有事]
    主角：抱歉，我还有事。
    ~ %{sakura_affection--}
    -> decline_student_council

* [学生会有什么活动？]
    主角：学生会有什么活动？
    -> ask_about_activity

# ============================================
# 分支：接受邀请
# ============================================

== accept_student_council ==

樱井美咲：太好了！那放学后见！

%character::show("sakura", "happy", "left")

主角：（她看起来很开心...）

-> after_school_choice

# ============================================
# 分支：拒绝邀请
# ============================================

== decline_student_council ==

樱井美咲：这样啊...那下次有机会再说吧。

%character::hide("left", "slide_left_out")

主角：（她的表情...好像有些失落。）

-> after_school_choice

# ============================================
# 分支：询问活动
# ============================================

== ask_about_activity ==

樱井美咲：嘿嘿，这是秘密哦。来了你就知道了~

主角：（秘密？）

-> after_school_choice

# ============================================
# 放学后的选择
# ============================================

== after_school_choice ==

%scene::change("bg/classroom_afternoon.png", "fade", 1.0)

放学后，我面临一个选择。

%character::show("ren", "cool", "right", "slide_right")

黑崎莲：喂，新来的。要不要跟我去个地方？

主角：黑崎同学？去哪里？

黑崎莲：旧校舍。我知道怎么进去。

* [跟樱井去学生会]
    主角：（还是先去学生会看看吧。）
    -> route_student_council

* [跟黑崎去旧校舍]
    主角：（旧校舍...也许能发现什么。）
    -> route_old_building

* [回家]
    主角：（今天还是先回家吧。）
    -> route_home

# ============================================
# 路线：学生会线
# ============================================

== route_student_council ==

~ %{school_route = true}

黑崎莲：切，无聊。

%character::hide("right", "slide_right_out")

主角：（学生会...应该能了解到一些学校的事情。）

-> chapter1_school.start

# ============================================
# 路线：旧校舍线
# ============================================

== route_old_building ==

~ %{school_route = true}
~ %{old_building_route = true}
~ %{ren_affection += 2}

黑崎莲：明智的选择。

主角：（黑崎同学...他知道什么？）

-> chapter1_old_building.start

# ============================================
# 路线：家庭线
# ============================================

== route_home ==

~ %{home_route = true}

黑崎莲：随你便。

%character::hide("right", "slide_right_out")

主角：（今天还是先回家吧。）

-> chapter1_home.start

# ============================================
# 游戏入口
# ============================================

== start ==

-> prologue
