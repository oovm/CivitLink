# Ren'Py 格式 - 第一章 家庭线

label chapter1_home:
    play music audio.home_theme fadein 1.0 volume 0.4
    
    scene bg home_living_room with fade(1.5)
    
    "回到家，我疲惫地躺在沙发上。"
    
    player "呼...今天也结束了。"
    
    with flash(0.5)
    
    show mom worried at left with slide_left
    
    mom "你回来了。学校怎么样？"
    
    player "还行...就是有点累。"
    
    mom "对了，今天有个奇怪的人来找过你。"
    
    player "奇怪的人？"
    
    mom "一个戴着帽子的男生，说是你的同学。他留了这个给你。"
    
    with vpunch
    
    play sound "audio/se/item_receive.wav"
    
    $ has_mysterious_letter = True
    
    player "这是..."
    
    hide mom with slide_left_out
    
    mom "我先去准备晚餐了。"
    
    scene bg home_bedroom with fade(0.8)
    
    "回到房间，我打开那封信。"
    
    player "（信上写着...）"
    
    centered """
    "新来的转学生：
    
    如果你对这所学校的秘密感兴趣，
    今晚12点到旧校舍后面。
    
    ——一个知道真相的人"
    """
    
    player "这是...什么意思？"
    
    menu:
        "今晚去看看":
            jump go_tonight
        
        "太危险了，不去":
            jump not_go_tonight
        
        "先调查一下这个写信人":
            jump investigate_sender

label go_tonight:
    $ home_route = True
    $ clues_found += 1
    
    player "（既然有人邀请我，那就去看看。）"
    
    with dissolve(2.0)
    
    play music audio.night_mystery fadein 1.0 volume 0.6
    
    scene bg old_building_night with fade(1.5)
    
    with flash(0.2)
    
    "深夜，我悄悄来到了旧校舍后面。"
    
    player "（这里...真的好黑...）"
    
    show unknown hooded at center with fade
    
    unknown "你来了。"
    
    player "你是谁？"
    
    unknown "这不重要。重要的是，你想知道真相吗？"
    
    menu:
        "我想知道":
            jump want_truth
        
        "你到底是谁？":
            jump demand_identity

label want_truth:
    $ knows_secret = True
    $ met_mystery_person = True
    
    unknown "很好。那么，听好了..."
    
    unknown "这所学校，曾经进行过一项秘密实验。"
    
    player "秘密实验？"
    
    unknown "关于"记忆操纵"的实验。而旧校舍，就是实验的地点。"
    
    unknown "那些奇怪的光...是实验残留的能量。"
    
    player "记忆操纵...？"
    
    unknown "如果你想知道更多，明天来学生会找樱井美咲。"
    
    unknown "她...也是知情者之一。"
    
    hide unknown with fade
    
    player "等等！"
    
    player "（樱井同学...她知道什么？）"
    
    jump chapter1_home_end

label demand_identity:
    unknown "我的身份不重要。重要的是，你有权利知道真相。"
    
    unknown "这所学校隐藏着一个巨大的秘密。"
    
    jump want_truth

label not_go_tonight:
    $ home_route = True
    $ trust_points -= 1
    
    player "（太危险了...还是不要去为好。）"
    
    with dissolve(1.0)
    
    player "（但是...这封信到底是谁寄的？）"
    
    player "（也许明天去学校问问看...）"
    
    jump chapter1_home_end

label investigate_sender:
    $ home_route = True
    $ clues_found += 2
    
    player "（先调查一下这个写信的人。）"
    
    player "（字迹...很工整。用纸是学校的学生会专用纸...）"
    
    player "（学生会？难道是...）"
    
    player "（樱井同学？还是白雪同学？）"
    
    jump chapter1_home_end

label chapter1_home_end:
    scene bg home_bedroom_night with fade(1.0)
    
    player "（今天发生了很多事...）"
    
    player "（明天去学校，一定要弄清楚。）"
    
    with fade_out(1.5)
    
    call chapter2_investigation

label midnight_explore:
    play music audio.suspense fadein 1.0 volume 0.5
    
    player "（睡不着...出去走走吧。）"
    
    scene bg night_street with fade(1.0)
    
    "夜晚的街道格外安静。"
    
    player "（奇怪...那边有光？）"
    
    with flash(0.3)
    
    "远处，旧校舍的方向闪烁着奇异的光芒。"
    
    player "（那是...什么？）"
    
    $ clues_found += 1
    
    player "（明天一定要去调查。）"
    
    jump chapter1_home_end

label mysterious_call:
    play sound "audio/se/phone_ring.wav"
    
    player "（这么晚了，谁会打电话？）"
    
    unknown "喂...是转学生吗？"
    
    player "你是谁？"
    
    unknown "我是...你的朋友。"
    
    unknown "不要相信学生会的人。"
    
    unknown "他们在...监视你。"
    
    player "什么？！"
    
    unknown "嘟...嘟...嘟..."
    
    $ clues_found += 1
    $ trust_points -= 1
    
    player "（监视我？这到底是怎么回事？）"
    
    jump chapter1_home_end
