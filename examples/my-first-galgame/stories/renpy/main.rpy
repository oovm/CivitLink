# Ren'Py 风格 DSL
# 这是最流行的视觉小说引擎 Ren'Py 的语法风格

# ============================================
# 角色定义
# ============================================

define player = Character("主角", color="#88c8ff")
define sakura = Character("樱井美咲", color="#ffb7c5")
define ren = Character("黑崎莲", color="#7b68ee")
define yuki = Character("白雪悠希", color="#e6e6fa")

# ============================================
# 变量定义
# ============================================

default sakura_affection = 0
default ren_affection = 0
default yuki_affection = 0
default trust_points = 0
default clues_found = 0
default knows_secret = False

# ============================================
# 图片定义
# ============================================

image bg school_gate = "images/bg/school_gate_sunset.png"
image bg classroom = "images/bg/classroom_afternoon.png"
image bg student_council = "images/bg/student_council_room.png"

image sakura smile = "images/char/sakura_smile.png"
image sakura happy = "images/char/sakura_happy.png"
image sakura worried = "images/char/sakura_worried.png"

image ren cool = "images/char/ren_cool.png"
image ren smirk = "images/char/ren_smirk.png"

# ============================================
# 音乐定义
# ============================================

define audio.mystery = "audio/bgm/mystery_theme.mp3"
define audio.peaceful = "audio/bgm/peaceful_theme.mp3"

# ============================================
# 主线剧情
# ============================================

label start:
    $ sakura_affection = 0
    $ ren_affection = 0
    
    jump prologue

# ============================================
# 序章
# ============================================

label prologue:
    play music mystery fadein 1.0 volume 0.6
    
    scene bg school_gate with fade(1.5)
    
    "星光学院，这所历史悠久的名校，隐藏着不为人知的秘密。"
    
    "我转学到这里已经一周了，却总觉得有什么不对劲。"
    
    with flash(0.3)
    
    player "（奇怪...那个旧校舍，为什么总是锁着？）"
    
    show sakura curious at left with slide_left
    
    sakura "新同学！你在看什么呢？"
    
    player "啊，樱井同学...没什么，只是在发呆。"
    
    sakura smile "你的表情可不像在发呆哦。对了，放学后学生会有个活动，要来吗？"
    
    menu:
        "好啊，我去看看":
            $ sakura_affection += 1
            $ trust_points += 1
            jump accept_student_council
        
        "抱歉，我还有事":
            $ sakura_affection -= 1
            jump decline_student_council
        
        "学生会有什么活动？":
            jump ask_about_activity

# ============================================
# 分支：接受邀请
# ============================================

label accept_student_council:
    sakura happy "太好了！那放学后见！"
    
    show sakura happy at left
    
    jump after_school_choice

# ============================================
# 分支：拒绝邀请
# ============================================

label decline_student_council:
    sakura worried "这样啊...那下次有机会再说吧。"
    
    show sakura disappointed at left
    hide sakura with slide_left_out
    
    player "（她的表情...好像有些失落。）"
    
    jump after_school_choice

# ============================================
# 分支：询问活动
# ============================================

label ask_about_activity:
    sakura "嘿嘿，这是秘密哦。来了你就知道了~"
    
    player "（秘密？）"
    
    jump after_school_choice

# ============================================
# 放学后选择
# ============================================

label after_school_choice:
    scene bg classroom with fade(1.0)
    
    "放学后，我面临一个选择。"
    
    show ren cool at right with slide_right
    
    ren "喂，新来的。要不要跟我去个地方？"
    
    player "黑崎同学？去哪里？"
    
    ren smirk "旧校舍。我知道怎么进去。"
    
    menu:
        "跟樱井去学生会" if sakura_affection >= 0:
            $ school_route = True
            player "（还是先去学生会看看吧。）"
            ren "切，无聊。"
            hide ren with slide_right_out
            call chapter1_school
        
        "跟黑崎去旧校舍":
            $ school_route = True
            $ ren_affection += 2
            player "（旧校舍...也许能发现什么。）"
            ren smirk "明智的选择。"
            call chapter1_old_building
        
        "回家":
            $ home_route = True
            player "（今天还是先回家吧。）"
            ren "随你便。"
            call chapter1_home

# ============================================
# 学生会线
# ============================================

label chapter1_school:
    play music peaceful fadein 1.0 volume 0.5
    
    scene bg student_council with fade(1.0)
    
    show sakura smile at center
    
    sakura "欢迎来到学生会！"
    
    show yuki gentle at left with slide_left
    
    yuki "啊，新同学也来了。欢迎欢迎。"
    
    player "白雪同学也在啊。"
    
    yuki smile "我是学生会的书记。樱井是副会长。"
    
    sakura proud "没错！我们学生会可是学校的重要组织！"
    
    player "（学生会...应该能了解到一些学校的事情。）"
    
    yuki serious "对了，新同学，有件事想请你帮忙。"
    
    menu:
        "什么事？":
            $ yuki_affection += 1
            $ trust_points += 2
            jump ask_for_help
        
        "我很忙...":
            $ yuki_affection -= 1
            $ sakura_affection -= 1
            jump refuse_help

label ask_for_help:
    yuki serious "最近学校里发生了一些奇怪的事情。"
    
    sakura worried "嗯...有学生说在旧校舍附近看到了奇怪的光。"
    
    player "奇怪的光？"
    
    yuki thinking "我们想调查一下，但是旧校舍一直锁着..."
    
    sakura determined "如果你能帮忙调查的话，我们会很感激的！"
    
    menu:
        "我愿意帮忙":
            $ trust_points += 2
            sakura happy "太好了！谢谢你！"
            yuki relieved "有你帮忙就放心多了。"
            jump investigation_start
        
        "这听起来很危险...":
            sakura reassuring "别担心！我们只是想看看是什么情况。"
            yuki gentle "如果你害怕的话，可以和我们一起行动。"
            $ sakura_affection += 1
            jump investigation_start

label refuse_help:
    sakura disappointed "这样啊..."
    
    yuki disappointed "没关系，我们理解。"
    
    player "（他们的表情...好像很需要帮助。）"
    
    menu:
        "等等，我改变主意了":
            jump ask_for_help
        
        "告辞了":
            jump leave_student_council

label investigation_start:
    scene bg school_hallway with fade(0.8)
    
    player "好，我来帮忙调查。"
    
    show sakura happy at center
    
    sakura "太好了！那我们明天放学后在学校门口集合！"
    
    with fade_out(1.0)
    
    call chapter2_investigation

label leave_student_council:
    scene bg school_corridor with fade(0.8)
    
    player "（也许我不该卷入这件事...）"
    
    with fade_out(1.0)
    
    call chapter1_home

# ============================================
# 结局判定
# ============================================

label endings:
    if ending_type == "public":
        jump public_truth_ending
    elif ending_type == "restore":
        jump restore_memory_ending
    elif ending_type == "investigate":
        jump true_ending
    else:
        jump normal_ending

label sakura_good_ending:
    scene bg cherry_blossom with fade(1.5)
    
    play music romantic fadein 1.0 volume 0.6
    
    show sakura blush at center
    
    sakura "那个...我有话想对你说。"
    
    player "什么事？"
    
    sakura shy "这段时间...谢谢你一直陪在我身边。"
    
    sakura "我...我好像..."
    
    player "？"
    
    sakura determined "我喜欢你！"
    
    with heart_float(1.0)
    
    player "樱井同学..."
    
    sakura happy "以后...请多多指教了！"
    
    with fade_out(2.0)
    
    centered """
    {b}樱井美咲结局：樱花之约{/b}
    
    你与樱井美咲成为了恋人。
    在樱花树下，你们许下了永远的约定。
    """
    
    return

# ============================================
# 游戏统计
# ============================================

label show_stats:
    centered """
    ---
    
    {b}游戏统计{/b}
    
    • 樱井美咲好感度: [sakura_affection]
    • 黑崎莲好感度: [ren_affection]
    • 白雪悠希好感度: [yuki_affection]
    • 信任点数: [trust_points]
    • 发现线索: [clues_found]
    
    ---
    
    感谢游玩《星光学院的秘密》！
    """
    
    return
