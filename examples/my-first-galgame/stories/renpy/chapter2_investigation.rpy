# Ren'Py 格式 - 第二章 调查

label chapter2_investigation:
    play music audio.school_morning fadein 1.0 volume 0.5
    
    scene bg school_classroom with fade(1.0)
    
    "第二天早上。"
    
    player "（今天要继续调查...）"
    
    show sakura smile at left with slide_left
    show yuki gentle at right with slide_right
    
    sakura "早上好！准备好了吗？"
    
    yuki "我们今天放学后去旧校舍。"
    
    if knows_secret:
        player "（昨晚那个人说樱井同学是知情者...）"
    else:
        player "（希望能找到一些线索。）"
    
    menu:
        "我准备好了":
            jump ready_to_investigate
        
        "还有其他人要来吗？":
            jump ask_about_others

label ask_about_others:
    sakura "嗯...黑崎同学说他也要来。"
    
    show ren cool at center with fade
    
    ren "当然要来。旧校舍可是我的地盘。"
    
    yuki "黑崎同学...请不要太冲动。"
    
    ren "放心，我知道分寸。"
    
    jump ready_to_investigate

label ready_to_investigate:
    with dissolve(1.5)
    
    play music audio.mystery fadein 1.0 volume 0.6
    
    scene bg old_building_entrance with fade(1.0)
    
    "放学后，我们来到了旧校舍前。"
    
    sakura "这里...真的好旧了。"
    
    show ren cool at right
    
    ren "入口在这边。"
    
    scene bg old_building_interior with fade(1.0)
    
    play sound "audio/se/door_creak.wav"
    
    with vpunch
    
    "门发出刺耳的声音，缓缓打开。"
    
    yuki "好黑..."
    
    with flash(0.2)
    
    play sound "audio/se/strange_light.wav"
    
    "突然，一道奇异的光从走廊深处闪过！"
    
    player "那是...！"
    
    ren "在那边！追！"
    
    menu:
        "追上去":
            jump chase_light
        
        "小心前进":
            jump careful_advance
        
        "先观察一下":
            jump observe_first

label chase_light:
    $ clues_found += 1
    $ ren_affection += 1
    
    player "（不能让它跑了！）"
    
    scene bg old_building_hallway with fade(0.5)
    
    play sound "audio/se/running_footsteps.wav"
    
    "我们追着光跑过走廊，最后来到了一扇门前。"
    
    ren "这是...实验室？"
    
    jump discover_lab

label careful_advance:
    $ trust_points += 1
    $ yuki_affection += 1
    
    player "（太危险了，还是小心点。）"
    
    yuki "对，我们还是小心一点比较好。"
    
    scene bg old_building_hallway with fade(1.0)
    
    "我们小心翼翼地前进，发现地上有一些奇怪的痕迹。"
    
    $ has_lab_key = True
    
    player "这是...一把钥匙？"
    
    jump discover_lab

label observe_first:
    $ clues_found += 2
    $ sakura_affection += 1
    
    player "（先观察一下情况。）"
    
    sakura "看，光是从那边那个房间传来的。"
    
    player "（樱井同学好像对这里很熟悉...）"
    
    sakura "怎么了？"
    
    player "没什么，我们过去看看吧。"
    
    jump discover_lab

label discover_lab:
    scene bg old_lab with fade(1.5)
    
    with flash(0.2)
    
    play music audio.revelation fadein 1.0 volume 0.7
    
    "我们来到了一个废弃的实验室。"
    
    sakura "这...这是..."
    
    yuki "不可能...为什么会有这种东西..."
    
    ren "看来传说是真的。"
    
    player "（这些设备...看起来像是用来做某种实验的...）"
    
    centered """
    桌上散落着文件：
    "记忆操纵实验 - 第七次测试报告
    实验对象：学生志愿者
    结果：部分记忆成功被改写
    副作用：轻微头痛，偶尔出现幻觉
    ..."
    """
    
    player "记忆操纵...？"
    
    show sakura serious at left
    
    sakura "...你们想知道真相吗？"
    
    menu:
        "告诉我真相":
            jump reveal_truth
        
        "你早就知道了？":
            jump confront_sakura

label reveal_truth:
    sakura "是的...我知道。"
    
    sakura "这所学校...曾经进行过记忆操纵的实验。"
    
    sakura "而我...是实验的参与者之一。"
    
    player "什么？！"
    
    yuki "樱井同学..."
    
    sakura "我...我不记得那段时间发生了什么。"
    
    sakura "但是我知道，我的记忆...被改写过。"
    
    $ knows_secret = True
    
    jump final_choice

label confront_sakura:
    $ sakura_affection -= 2
    $ trust_points -= 1
    
    player "你早就知道了？为什么要瞒着我们？"
    
    sakura "对不起...我害怕..."
    
    sakura "害怕你们知道真相后会讨厌我..."
    
    menu:
        "我理解你的顾虑":
            jump forgive_sakura
        
        "我需要时间消化":
            jump need_time

label forgive_sakura:
    $ sakura_affection += 3
    $ trust_points += 2
    
    player "没关系，我理解。"
    
    sakura "谢谢你..."
    
    jump reveal_truth

label need_time:
    player "我需要时间消化这些信息。"
    
    sakura "我明白..."
    
    jump reveal_truth

label final_choice:
    sakura "现在，我们有一个选择。"
    
    sakura "我们可以把这一切公之于众，或者..."
    
    ren "或者什么？"
    
    sakura "或者...找到当初的实验数据，恢复所有人的记忆。"
    
    menu:
        "公布真相":
            $ ending_type = "public"
            player "真相应该被公开。"
            sakura "我同意。大家有权知道发生了什么。"
            call endings
        
        "恢复记忆":
            $ ending_type = "restore"
            player "如果能恢复记忆...那才是最好的结果。"
            yuki "是的，这样大家都能找回自己失去的记忆。"
            call endings
        
        "先找到更多证据":
            $ ending_type = "investigate"
            $ clues_found += 3
            player "我们需要更多证据才能做决定。"
            ren "说得对，不能草率行事。"
            call endings
