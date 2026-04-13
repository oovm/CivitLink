# Ren'Py 格式 - 第一章 旧校舍线

label chapter1_old_building:
    play music audio.suspense fadein 1.0 volume 0.6
    
    scene bg old_building_exterior with fade(1.0)
    
    show ren smirk at right
    
    ren "到了。这就是旧校舍。"
    
    player "看起来已经废弃很久了..."
    
    ren "表面上是这样。"
    
    ren "但实际上，这里藏着很多秘密。"
    
    player "秘密？"
    
    ren "跟我来。"
    
    scene bg old_building_side with fade(0.8)
    
    play sound "audio/se/metal_gate.wav"
    
    with vpunch
    
    "黑崎带我来到旧校舍的侧面，那里有一扇生锈的铁门。"
    
    ren "这扇门的锁早就坏了。"
    
    play sound "audio/se/door_open_creak.wav"
    
    scene bg old_building_corridor with fade(1.0)
    
    "我们进入了旧校舍内部。"
    
    with flash(0.5)
    
    player "好暗..."
    
    ren "小心脚下。"
    
    menu:
        "打开手电筒":
            jump use_flashlight
        
        "跟着黑崎走":
            jump follow_ren
        
        "询问黑崎为什么知道这里":
            jump ask_ren_knowledge

label use_flashlight:
    $ clues_found += 1
    
    play sound "audio/se/flashlight_on.wav"
    
    with flash(0.5)
    
    player "（手电筒的光照亮了走廊...）"
    
    player "（墙上好像有什么字...）"
    
    centered """
    墙上写着：
    "不要相信他们"
    "记忆是假的"
    "救救我"
    """
    
    player "这些字...是什么意思？"
    
    ren "看来有人在这里待过很久。"
    
    jump explore_corridor

label follow_ren:
    $ ren_affection += 1
    
    player "（相信黑崎同学吧。）"
    
    ren "...聪明。"
    
    player "（他好像对我有点改观？）"
    
    jump explore_corridor

label ask_ren_knowledge:
    $ clues_found += 2
    
    player "黑崎同学，你怎么知道这个地方？"
    
    ren "..."
    
    ren "我...曾经在这里待过。"
    
    player "什么意思？"
    
    ren "我是...实验的幸存者之一。"
    
    $ knows_secret = True
    $ knows_ren_secret = True
    
    player "实验...？"
    
    ren "以后再解释。先往前走。"
    
    jump explore_corridor

label explore_corridor:
    scene bg old_building_hallway with fade(0.8)
    
    "我们沿着走廊前进，两边是废弃的教室。"
    
    with flash(0.2)
    
    play sound "audio/se/strange_sound.wav"
    
    "突然，一阵奇怪的声音从远处传来！"
    
    player "什么声音？"
    
    ren "有人在这里。"
    
    menu:
        "躲起来观察":
            jump hide_and_observe
        
        "直接去查看":
            jump go_check
        
        "让黑崎去查看":
            jump send_ren

label hide_and_observe:
    $ clues_found += 2
    $ trust_points += 1
    
    player "（先躲起来看看情况。）"
    
    scene bg old_building_shadow with fade(0.5)
    
    "我们躲在一扇破旧的门后。"
    
    show unknown hooded at center with fade
    
    unknown "（低声自语）数据已经转移完毕..."
    
    unknown "下一个目标...新来的转学生。"
    
    player "（他们...在说我？！）"
    
    ren "（小声）看来你被盯上了。"
    
    hide unknown with fade
    
    player "（这到底是怎么回事...）"
    
    jump discover_room

label go_check:
    $ ren_affection += 1
    
    player "（去看看是什么。）"
    
    ren "...有胆量。"
    
    scene bg old_building_intersection with fade(0.5)
    
    "我们走向声音传来的方向，但那里已经没有人了。"
    
    player "跑了..."
    
    ren "但是留下了这个。"
    
    $ has_mysterious_document = True
    $ clues_found += 1
    
    player "这是..."
    
    jump discover_room

label send_ren:
    player "黑崎同学，你能去看看吗？"
    
    ren "...切。"
    
    ren "在这里等着。"
    
    hide ren with slide_right_out
    
    with pause(2.0)
    
    show ren serious at right with slide_right
    
    ren "没人。但是发现了这个。"
    
    $ has_lab_access = True
    
    player "一张门禁卡？"
    
    ren "旧校舍的地下实验室用的。"
    
    jump discover_room

label discover_room:
    scene bg old_building_door with fade(0.8)
    
    "我们来到一扇厚重的金属门前。"
    
    ren "这就是目的地。"
    
    if has_lab_access:
        jump enter_with_card
    else:
        jump locked_door

label locked_door:
    player "门锁着..."
    
    ren "我们需要找到开门的方法。"
    
    menu:
        "寻找其他入口":
            jump find_entrance
        
        "尝试撬锁":
            jump try_lockpick
        
        "回去找樱井帮忙":
            jump get_sakura_help

label enter_with_card:
    play sound "audio/se/card_beep.wav"
    
    with flash(0.5)
    
    player "门开了！"
    
    ren "走吧。"
    
    jump enter_lab

label find_entrance:
    $ clues_found += 1
    
    player "（一定有其他入口。）"
    
    scene bg old_building_vent with fade(0.8)
    
    ren "通风管道...可以试试。"
    
    player "这...真的能进去吗？"
    
    ren "跟着我。"
    
    jump enter_lab

label try_lockpick:
    player "让我试试..."
    
    play sound "audio/se/lockpick_fail.wav"
    
    player "不行，锁太复杂了。"
    
    ren "我就说没用。"
    
    jump find_entrance

label get_sakura_help:
    $ sakura_affection += 1
    $ trust_points += 1
    
    player "（樱井同学可能有办法...）"
    
    ren "还要找那个女人？"
    
    player "她可能有钥匙。"
    
    with flash(0.5)
    
    sakura "喂？怎么了？"
    
    player "我们在旧校舍...门锁着。"
    
    sakura "我马上来！"
    
    with dissolve(1.5)
    
    show sakura worried at left with slide_left
    
    sakura "给你们钥匙。"
    
    play sound "audio/se/key_unlock.wav"
    
    sakura "但是...请小心。"
    
    jump enter_lab

label enter_lab:
    scene bg old_lab with fade(1.5)
    
    play music audio.revelation fadein 1.0 volume 0.7
    
    with flash(0.2)
    
    "我们进入了地下实验室。"
    
    player "这是..."
    
    ren "记忆操纵实验室。"
    
    centered """
    房间中央是一台巨大的机器，
    周围散落着各种文件和设备。
    """
    
    player "记忆操纵...真的存在？"
    
    ren "是的。而且..."
    
    ren "我是实验对象之一。"
    
    player "黑崎同学..."
    
    ren "我失去了一部分记忆。"
    
    ren "但是我知道...这所学校还在继续实验。"
    
    $ knows_secret = True
    $ clues_found += 3
    
    menu:
        "我们一起阻止他们":
            jump promise_together
        
        "告诉我更多细节":
            jump more_details

label promise_together:
    $ ren_affection += 3
    $ trust_points += 2
    
    player "我们一起阻止他们。"
    
    ren "..."
    
    ren "...谢了。"
    
    ren "第一次有人说要帮我。"
    
    jump chapter1_old_building_end

label more_details:
    player "告诉我更多细节。"
    
    ren "实验代号"星之子"。"
    
    ren "目的是创造没有痛苦记忆的人。"
    
    ren "但是副作用...是人格分裂。"
    
    player "人格分裂？"
    
    ren "我有时候会变成另一个人。"
    
    ren "那个我...更冷酷，更危险。"
    
    $ knows_ren_secret = True
    
    jump chapter1_old_building_end

label chapter1_old_building_end:
    scene bg school_sunset with fade(1.5)
    
    play music audio.bittersweet fadein 1.0 volume 0.5
    
    "离开旧校舍后，夕阳已经西沉。"
    
    player "（今天发现了太多事情...）"
    
    show ren serious at right
    
    ren "明天放学后，我们继续调查。"
    
    player "好。"
    
    ren "...你比我想象的可靠。"
    
    player "谢谢。"
    
    with fade_out(1.5)
    
    call chapter2_investigation
