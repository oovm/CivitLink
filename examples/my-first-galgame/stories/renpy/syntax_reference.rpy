# Ren'Py 风格 DSL - 语法参考
# Ren'Py 引擎的脚本语法

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
default knows_ren_secret = False
default has_lab_key = False
default has_lab_access = False
default school_route = False
default home_route = False
default ending_type = ""

# ============================================
# 图片定义
# ============================================

image bg school_gate = "images/bg/school_gate_sunset.png"
image bg classroom = "images/bg/classroom_afternoon.png"
image bg student_council = "images/bg/student_council_room.png"
image bg cherry_blossom = "images/bg/cherry_blossom.png"

image sakura smile = "images/char/sakura_smile.png"
image sakura happy = "images/char/sakura_happy.png"
image sakura blush = "images/char/sakura_blush.png"
image sakura curious = "images/char/sakura_curious.png"

image ren cool = "images/char/ren_cool.png"
image ren smirk = "images/char/ren_smirk.png"

image yuki gentle = "images/char/yuki_gentle.png"

# ============================================
# 音乐定义
# ============================================

define audio.mystery = "audio/bgm/mystery_theme.mp3"
define audio.peaceful = "audio/bgm/peaceful_theme.mp3"
define audio.romantic = "audio/bgm/romantic_theme.mp3"

# ============================================
# 语法规则说明
# ============================================

"""
1. 角色定义：define 角色名 = Character("显示名", color="#颜色")
2. 变量定义：default 变量名 = 初始值
3. 图片定义：image 类型 名称 = "路径"
4. 音乐定义：define audio.名称 = "路径"
5. 标签定义：label 标签名:
6. 场景切换：scene bg 图片 with 过渡效果
7. 角色显示：show 角色 表情 at 位置 with 动画
8. 对话显示：角色名 "对话内容" 或 "叙述内容"
9. 选择分支：menu: ... "选项": 动作
10. 条件选项："选项" if 条件:
11. 变量操作：$ 变量名 += 1
12. 跳转：jump 标签名
13. 调用：call 标签名
14. 条件判断：if 条件: ... elif: ... else:
"""

# ============================================
# 完整示例场景
# ============================================

label example_scene:
    play music peaceful fadein 1.0 volume 0.5
    
    scene bg classroom with fade(1.0)
    
    show sakura smile at center with fade
    
    sakura "你好！欢迎来到星光学院！"
    
    menu:
        "你好！":
            $ sakura_affection += 1
            player "你好！"
            jump next_part
        
        "请问你是？":
            player "请问你是？"
            jump introduce
        
        "关于那个秘密..." if knows_secret:
            player "关于那个秘密..."
            jump secret_talk

label introduce:
    sakura "我是樱井美咲，学生会副会长！"
    jump next_part

label next_part:
    sakura "要不要参观一下学校？"
    
    menu:
        "好啊":
            player "好啊！"
            jump tour
        
        "下次吧":
            player "下次吧，谢谢。"
            jump end

label tour:
    scene bg school_hallway with slide_right(0.8)
    sakura "这里是教学楼，那边是图书馆..."
    jump end

label end:
    sakura "再见！"
    with fade_out(1.0)
    return

# ============================================
# 复杂分支示例
# ============================================

label complex_choice:
    ren "你想知道真相吗？"
    
    menu:
        "我想知道一切":
            $ trust_points += 2
            $ ren_affection += 1
            jump reveal_all
        
        "太危险了，算了吧":
            $ trust_points -= 1
            jump avoid_danger
        
        "我已经知道一些了" if clues_found >= 3:
            $ ren_affection += 2
            player "我已经发现了一些线索..."
            jump share_intel
        
        "你也是实验对象？" if knows_ren_secret:
            $ ren_affection += 3
            player "黑崎同学，你也是..."
            jump ren_confession

# ============================================
# 多条件判断示例
# ============================================

label determine_ending:
    if sakura_affection >= 8 and trust_points >= 10:
        jump sakura_true_ending
    elif ren_affection >= 8 and knows_ren_secret:
        jump ren_true_ending
    elif yuki_affection >= 6 and clues_found >= 5:
        jump yuki_true_ending
    elif sakura_affection >= 5:
        jump sakura_good_ending
    elif ren_affection >= 5:
        jump ren_good_ending
    elif yuki_affection >= 5:
        jump yuki_good_ending
    elif trust_points >= 8:
        jump good_ending
    else:
        jump normal_ending

# ============================================
# 结局定义
# ============================================

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
    
    jump show_stats

label ren_good_ending:
    scene bg night_city with fade(1.5)
    
    show ren smile at center
    
    ren "喂。"
    
    player "黑崎同学？"
    
    ren smirk "没想到你还挺有本事的。"
    
    ren "我对你刮目相看了。"
    
    player "谢谢...？"
    
    ren "以后...有什么事可以找我。"
    
    ren smile "我会帮你的。"
    
    player "真的？"
    
    ren "别让我说第二遍。"
    
    with fade_out(2.0)
    
    centered """
    {b}黑崎莲结局：暗夜的盟约{/b}
    
    你获得了黑崎莲的认可。
    虽然他表面冷淡，但你知道，
    他会是你最可靠的伙伴。
    """
    
    jump show_stats

label yuki_good_ending:
    scene bg library with fade(1.5)
    
    show yuki smile at center
    
    yuki "那个...谢谢你。"
    
    player "怎么了，白雪同学？"
    
    yuki gentle "谢谢你让我变得勇敢。"
    
    yuki "以前的我...总是害怕面对真相。"
    
    yuki determined "但是现在...我想和你一起面对未来。"
    
    player "我也是。"
    
    yuki blush "那...我们约定好了？"
    
    with sparkle(1.0)
    
    with fade_out(2.0)
    
    centered """
    {b}白雪悠希结局：温柔的约定{/b}
    
    你与白雪悠希建立了深厚的羁绊。
    在图书馆的角落，你们约定一起面对未来。
    """
    
    jump show_stats

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
    
    如果想要体验其他结局，请尝试不同的选择路径。
    """
    
    return

# ============================================
# 常用命令参考
# ============================================

# 音频命令
# play music "文件路径" fadein 秒数 volume 音量
# play sound "文件路径"
# stop music fadeout 秒数

# 场景命令
# scene bg 图片 with 过渡效果(秒数)
# 过渡效果：fade, dissolve, pixellate, vpunch, hpunch

# 角色命令
# show 角色 表情 at 位置 with 动画
# hide 角色 with 动画
# 位置：left, center, right
# 动画：fade, dissolve, moveinleft, moveinright

# 特效命令
# with flash(秒数)
# with vpunch (垂直震动)
# with hpunch (水平震动)
# with dissolve(秒数)

# 文本显示
# 角色 "对话内容"
# "叙述内容"
# centered "居中文本"
