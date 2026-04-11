# Ren'Py 格式 - 结局

label endings:
    if ending_type == "public":
        jump public_truth_ending
    elif ending_type == "restore":
        jump restore_memory_ending
    elif ending_type == "investigate":
        jump true_ending
    else:
        jump normal_ending

label public_truth_ending:
    play music audio.dramatic fadein 1.0 volume 0.7
    
    scene bg school_assembly with fade(1.5)
    
    with flash(0.3)
    
    "我们决定公开真相。"
    
    player "（这是正确的选择。）"
    
    show sakura determined at center
    show yuki serious at left
    show ren serious at right
    
    sakura "各位同学，我们有重要的事情要宣布..."
    
    with flash(0.5)
    
    centered """
    几天后...
    
    学校的秘密实验被媒体曝光，
    校长被撤职，
    所有参与实验的学生都接受了心理辅导。
    """
    
    scene bg school_rooftop with fade(1.0)
    
    play music audio.bittersweet fadein 1.0 volume 0.5
    
    sakura "虽然真相大白了..."
    
    sakura "但是学校现在一片混乱。"
    
    player "这是必要的代价。"
    
    yuki "至少，大家现在都知道真相了。"
    
    ren "哼，总算结束了。"
    
    if sakura_affection >= 5:
        jump sakura_good_ending
    elif ren_affection >= 5:
        jump ren_good_ending
    elif yuki_affection >= 5:
        jump yuki_good_ending
    else:
        jump normal_public_ending

label restore_memory_ending:
    play music audio.hopeful fadein 1.0 volume 0.6
    
    scene bg old_lab with fade(1.0)
    
    "我们决定寻找恢复记忆的方法。"
    
    player "（一定要找到实验数据。）"
    
    show sakura hopeful at center
    
    sakura "我找到了！这里有备份数据！"
    
    with flash(0.5)
    
    centered """
    经过几天的努力，
    我们成功恢复了所有被改写的记忆。
    
    那些曾经失去记忆的学生，
    终于找回了属于自己的过去。
    """
    
    scene bg cherry_blossom with fade(1.5)
    
    play music audio.happy_ending fadein 1.0 volume 0.6
    
    sakura "谢谢你...帮我找回了记忆。"
    
    sakura "我现在...终于完整了。"
    
    player "不用谢，这是我应该做的。"
    
    if sakura_affection >= 6:
        sakura "那个...以后...我们可以..."
        player "可以什么？"
        sakura "一起...一起走下去吗？"
        jump true_love_ending
    else:
        jump good_ending

label true_ending:
    play music audio.mystery_deep fadein 1.0 volume 0.7
    
    scene bg old_lab with fade(1.0)
    
    "我们决定继续调查。"
    
    player "（还有太多谜团没有解开。）"
    
    show ren serious at right
    
    ren "等等，看这个。"
    
    with flash(0.5)
    
    ren "这里有一个隐藏的文件..."
    
    centered """
    文件内容：
    "实验并未结束。
    负责人已转移至新校区。
    实验代号：星之子计划"
    """
    
    player "实验还在继续？！"
    
    sakura "新校区...那是刚建成的..."
    
    yuki "我们必须阻止他们。"
    
    with pause(1.0)
    
    scene bg sunset with fade(1.5)
    
    play music audio.epic fadein 1.0 volume 0.7
    
    centered """
    我们发现了更深层的阴谋。
    实验并未结束，
    只是转移到了新的地点。
    
    我们的战斗，才刚刚开始...
    """
    
    with fade_out(2.0)
    
    centered """
    {b}真结局：星之子{/b}
    
    你发现了隐藏在幕后的真相。
    故事将在续作中继续...
    """
    
    jump ending_summary

label normal_ending:
    play music audio.peaceful_ending fadein 1.0 volume 0.5
    
    scene bg school_gate with fade(1.0)
    
    "一切结束后，学校恢复了平静。"
    
    player "（虽然还有很多谜团...）"
    
    player "（但至少现在，大家都安全了。）"
    
    show sakura smile at left with slide_left
    
    sakura "谢谢你，新同学。"
    
    sakura "如果没有你，我们可能永远不会知道真相。"
    
    player "这是我应该做的。"
    
    with fade_out(1.5)
    
    centered """
    {b}普通结局：平静的日常{/b}
    
    你帮助学校揭开了秘密，
    但真相似乎不止于此...
    """
    
    jump ending_summary

label sakura_good_ending:
    scene bg cherry_blossom with fade(1.5)
    
    play music audio.romantic fadein 1.0 volume 0.6
    
    show sakura blush at center
    
    sakura "那个...我有话想对你说。"
    
    player "什么事？"
    
    sakura "这段时间...谢谢你一直陪在我身边。"
    
    sakura "我...我好像..."
    
    player "？"
    
    sakura "我喜欢你！"
    
    with heart_float(1.0)
    
    player "樱井同学..."
    
    sakura "以后...请多多指教了！"
    
    with fade_out(2.0)
    
    centered """
    {b}樱井美咲结局：樱花之约{/b}
    
    你与樱井美咲成为了恋人。
    在樱花树下，你们许下了永远的约定。
    """
    
    jump ending_summary

label ren_good_ending:
    scene bg night_city with fade(1.5)
    
    play music audio.cool fadein 1.0 volume 0.5
    
    show ren smile at center
    
    ren "喂。"
    
    player "黑崎同学？"
    
    ren "没想到你还挺有本事的。"
    
    ren "我对你刮目相看了。"
    
    player "谢谢...？"
    
    ren "以后...有什么事可以找我。"
    
    ren "我会帮你的。"
    
    player "真的？"
    
    ren "别让我说第二遍。"
    
    with fade_out(2.0)
    
    centered """
    {b}黑崎莲结局：暗夜的盟约{/b}
    
    你获得了黑崎莲的认可。
    虽然他表面冷淡，但你知道，
    他会是你最可靠的伙伴。
    """
    
    jump ending_summary

label yuki_good_ending:
    scene bg library with fade(1.5)
    
    play music audio.gentle fadein 1.0 volume 0.5
    
    show yuki smile at center
    
    yuki "那个...谢谢你。"
    
    player "怎么了，白雪同学？"
    
    yuki "谢谢你让我变得勇敢。"
    
    yuki "以前的我...总是害怕面对真相。"
    
    yuki "但是现在...我想和你一起面对未来。"
    
    player "我也是。"
    
    yuki "那...我们约定好了？"
    
    with sparkle(1.0)
    
    with fade_out(2.0)
    
    centered """
    {b}白雪悠希结局：温柔的约定{/b}
    
    你与白雪悠希建立了深厚的羁绊。
    在图书馆的角落，你们约定一起面对未来。
    """
    
    jump ending_summary

label true_love_ending:
    play music audio.love fadein 1.0 volume 0.7
    
    scene bg cherry_blossom_sunset with fade(1.5)
    
    with dissolve(1.0)
    
    show sakura happy at center
    
    sakura "从今以后...我们一起走吧。"
    
    player "嗯，一起走。"
    
    with heart_float(1.0)
    
    centered """
    {b}真结局：永恒的星光{/b}
    
    你不仅揭开了学校的秘密，
    还找到了真正的爱情。
    
    樱井美咲的笑容，
    将是你心中永远的光芒。
    """
    
    jump ending_summary

label good_ending:
    centered """
    {b}好结局：新的开始{/b}
    
    你帮助恢复了所有人的记忆。
    虽然还有未解的谜团，
    但你知道，未来会更好。
    """
    
    jump ending_summary

label normal_public_ending:
    centered """
    {b}结局：真相大白{/b}
    
    学校的秘密被公之于众。
    虽然付出了代价，
    但正义得到了伸张。
    """
    
    jump ending_summary

label ending_summary:
    with fade_out(2.0)
    
    scene bg black with fade(1.0)
    
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
    
    如果想要体验其他结局，
    请尝试不同的选择路径。
    """
    
    return
