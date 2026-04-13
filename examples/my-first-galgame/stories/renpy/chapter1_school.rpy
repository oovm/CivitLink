# Ren'Py 格式 - 第一章 学生会线

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

label sakura_secret_talk:
    play music mystery fadein 1.0 volume 0.5
    
    sakura "其实...我有件事想告诉你。"
    
    player "什么事？"
    
    sakura "这所学校...隐藏着一个秘密。"
    
    sakura "关于"记忆"的秘密..."
    
    player "记忆？"
    
    sakura "我不能说太多...但是请小心黑崎同学。"
    
    sakura "他...不是普通人。"
    
    player "（黑崎同学？他到底是谁？）"
    
    $ clues_found += 1
    
    jump investigation_start

label yuki_information:
    yuki "我查到了一些资料..."
    
    yuki "十年前，这所学校曾经进行过一项秘密实验。"
    
    player "秘密实验？"
    
    yuki "代号"星之子计划"。"
    
    yuki "目的是...操纵人类的记忆。"
    
    player "操纵记忆？！"
    
    yuki "实验后来被终止了，但是..."
    
    yuki "据说有些实验对象还在学校里。"
    
    $ clues_found += 2
    $ knows_secret = True
    
    player "（实验对象...还在学校里？）"
    
    jump investigation_start
