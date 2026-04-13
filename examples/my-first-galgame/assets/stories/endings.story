# ============================================
# 结局 - 星光学院的秘密
# ============================================
# 多线多结局系统
# ============================================

== start ==

{
    - ending_type == "public":
        -> public_truth_ending
    - ending_type == "restore":
        -> restore_memory_ending
    - ending_type == "investigate":
        -> true_ending
    - else:
        -> normal_ending
}

# ============================================
# 结局：公布真相
# ============================================

== public_truth_ending ==

%audio::play("bgm/dramatic_theme.mp3", 0.7)
%scene::change("bg/school_assembly.png", "fade", 1.5)

%effect::flash(0.3)

我们决定公开真相。

主角：（这是正确的选择。）

%character::show("sakura", "determined", "center")
%character::show("yuki", "serious", "left")
%character::show("ren", "serious", "right")

樱井美咲：各位同学，我们有重要的事情要宣布...

%effect::flash(0.5)

---

几天后...

学校的秘密实验被媒体曝光，
校长被撤职，
所有参与实验的学生都接受了心理辅导。

---

%scene::change("bg/school_rooftop.png", "fade", 1.0)

%audio::play("bgm/bittersweet.mp3", 0.5)

樱井美咲：虽然真相大白了...

樱井美咲：但是学校现在一片混乱。

主角：这是必要的代价。

白雪悠希：至少，大家现在都知道真相了。

黑崎莲：哼，总算结束了。

{
    - sakura_affection >= 5:
        -> sakura_good_ending
    - ren_affection >= 5:
        -> ren_good_ending
    - yuki_affection >= 5:
        -> yuki_good_ending
    - else:
        -> normal_public_ending
}

# ============================================
# 结局：恢复记忆
# ============================================

== restore_memory_ending ==

%audio::play("bgm/hopeful_theme.mp3", 0.6)
%scene::change("bg/old_lab.png", "fade", 1.0)

我们决定寻找恢复记忆的方法。

主角：（一定要找到实验数据。）

%character::show("sakura", "hopeful", "center")

樱井美咲：我找到了！这里有备份数据！

%effect::flash(0.5)

---

经过几天的努力，
我们成功恢复了所有被改写的记忆。

那些曾经失去记忆的学生，
终于找回了属于自己的过去。

---

%scene::change("bg/cherry_blossom.png", "fade", 1.5)

%audio::play("bgm/happy_ending.mp3", 0.6)

樱井美咲：谢谢你...帮我找回了记忆。

樱井美咲：我现在...终于完整了。

主角：不用谢，这是我应该做的。

{
    - sakura_affection >= 6:
        樱井美咲：那个...以后...我们可以...
        主角：可以什么？
        樱井美咲：一起...一起走下去吗？
        -> true_love_ending
    - else:
        -> good_ending
}

# ============================================
# 真结局：星之子
# ============================================

== true_ending ==

%audio::play("bgm/mystery_deep.mp3", 0.7)
%scene::change("bg/old_lab.png", "fade", 1.0)

我们决定继续调查。

主角：（还有太多谜团没有解开。）

%character::show("ren", "serious", "right")

黑崎莲：等等，看这个。

%effect::flash(0.5)

黑崎莲：这里有一个隐藏的文件...

---

文件内容：
"实验并未结束。
负责人已转移至新校区。
实验代号：星之子计划"

---

主角：实验还在继续？！

樱井美咲：新校区...那是刚建成的...

白雪悠希：我们必须阻止他们。

%effect::pause(1.0)

%scene::change("bg/sunset.png", "fade", 1.5)

%audio::play("bgm/epic_theme.mp3", 0.7)

---

我们发现了更深层的阴谋。
实验并未结束，
只是转移到了新的地点。

我们的战斗，才刚刚开始...

---

%effect::fade_out(2.0)

---

**真结局：星之子**

你发现了隐藏在幕后的真相。
故事将在续作中继续...

---

-> ending_summary

# ============================================
# 普通结局
# ============================================

== normal_ending ==

%audio::play("bgm/peaceful_ending.mp3", 0.5)
%scene::change("bg/school_gate.png", "fade", 1.0)

一切结束后，学校恢复了平静。

主角：（虽然还有很多谜团...）

主角：（但至少现在，大家都安全了。）

%character::show("sakura", "smile", "left", "slide_left")

樱井美咲：谢谢你，新同学。

樱井美咲：如果没有你，我们可能永远不会知道真相。

主角：这是我应该做的。

%effect::fade_out(1.5)

---

**普通结局：平静的日常**

你帮助学校揭开了秘密，
但真相似乎不止于此...

---

-> ending_summary

# ============================================
# 樱井美咲好结局
# ============================================

== sakura_good_ending ==

%scene::change("bg/cherry_blossom.png", "fade", 1.5)

%audio::play("bgm/romantic_theme.mp3", 0.6)

%character::show("sakura", "blush", "center")

樱井美咲：那个...我有话想对你说。

主角：什么事？

樱井美咲：这段时间...谢谢你一直陪在我身边。

樱井美咲：我...我好像...

主角：？

樱井美咲：我喜欢你！

%effect::heart_float(1.0)

主角：樱井同学...

樱井美咲：以后...请多多指教了！

%effect::fade_out(2.0)

---

**樱井美咲结局：樱花之约**

你与樱井美咲成为了恋人。
在樱花树下，你们许下了永远的约定。

---

-> ending_summary

# ============================================
# 黑崎莲好结局
# ============================================

== ren_good_ending ==

%scene::change("bg/night_city.png", "fade", 1.5)

%audio::play("bgm/cool_theme.mp3", 0.5)

%character::show("ren", "smile", "center")

黑崎莲：喂。

主角：黑崎同学？

黑崎莲：没想到你还挺有本事的。

黑崎莲：我对你刮目相看了。

主角：谢谢...？

黑崎莲：以后...有什么事可以找我。

黑崎莲：我会帮你的。

主角：真的？

黑崎莲：别让我说第二遍。

%effect::fade_out(2.0)

---

**黑崎莲结局：暗夜的盟约**

你获得了黑崎莲的认可。
虽然他表面冷淡，但你知道，
他会是你最可靠的伙伴。

---

-> ending_summary

# ============================================
# 白雪悠希好结局
# ============================================

== yuki_good_ending ==

%scene::change("bg/library.png", "fade", 1.5)

%audio::play("bgm/gentle_theme.mp3", 0.5)

%character::show("yuki", "smile", "center")

白雪悠希：那个...谢谢你。

主角：怎么了，白雪同学？

白雪悠希：谢谢你让我变得勇敢。

白雪悠希：以前的我...总是害怕面对真相。

白雪悠希：但是现在...我想和你一起面对未来。

主角：我也是。

白雪悠希：那...我们约定好了？

%effect::sparkle(1.0)

%effect::fade_out(2.0)

---

**白雪悠希结局：温柔的约定**

你与白雪悠希建立了深厚的羁绊。
在图书馆的角落，你们约定一起面对未来。

---

-> ending_summary

# ============================================
# 真爱结局
# ============================================

== true_love_ending ==

%audio::play("bgm/love_theme.mp3", 0.7)

%scene::change("bg/cherry_blossom_sunset.png", "fade", 1.5)

%effect::flash(1.0)

%character::show("sakura", "happy", "center")

樱井美咲：从今以后...我们一起走吧。

主角：嗯，一起走。

%effect::heart_float(1.0)

---

**真结局：永恒的星光**

你不仅揭开了学校的秘密，
还找到了真正的爱情。

樱井美咲的笑容，
将是你心中永远的光芒。

---

-> ending_summary

# ============================================
# 好结局
# ============================================

== good_ending ==

---

**好结局：新的开始**

你帮助恢复了所有人的记忆。
虽然还有未解的谜团，
但你知道，未来会更好。

---

-> ending_summary

# ============================================
# 普通公布结局
# ============================================

== normal_public_ending ==

---

**结局：真相大白**

学校的秘密被公之于众。
虽然付出了代价，
但正义得到了伸张。

---

-> ending_summary

# ============================================
# 结局统计
# ============================================

== ending_summary ==

%effect::fade_out(2.0)

%scene::change("bg/black.png", "fade", 1.0)

---

# 游戏统计

- 樱井美咲好感度: %{sakura_affection}
- 黑崎莲好感度: %{ren_affection}
- 白雪悠希好感度: %{yuki_affection}
- 信任点数: %{trust_points}
- 发现线索: %{clues_found}

---

感谢游玩《星光学院的秘密》！

如果想要体验其他结局，
请尝试不同的选择路径。

---

-> DONE

# ============================================
# 隐藏结局：完美的真相
# ============================================

== perfect_ending ==

%audio::play("bgm/perfect_ending.mp3", 0.7)

%scene::change("bg/school_rooftop_sunset.png", "fade", 1.5)

{
    - sakura_affection >= 8 && ren_affection >= 5 && yuki_affection >= 5 && clues_found >= 8 && trust_points >= 10:
        -> perfect_ending_content
    - else:
        -> ending_summary
}

== perfect_ending_content ==

主角：（我们终于做到了...）

%character::show("sakura", "happy", "left")
%character::show("ren", "smile", "right")
%character::show("yuki", "smile", "center")

樱井美咲：谢谢你...谢谢你拯救了我们。

黑崎莲：哼...别这么肉麻。

白雪悠希：我们...终于可以开始新的生活了。

主角：是的，一起。

%effect::sparkle(2.0)

---

**完美结局：星光闪耀的未来**

你不仅揭开了所有的秘密，
还拯救了所有被实验影响的人。

你的勇气和善良，
点亮了所有人心中希望的星光。

这是最好的结局。

---

-> ending_summary

# ============================================
# 坏结局：被遗忘的记忆
# ============================================

== bad_ending ==

%audio::play("bgm/sad_ending.mp3", 0.5)

%scene::change("bg/dark_room.png", "fade", 1.5)

{
    - trust_points <= 0:
        -> bad_ending_content
    - else:
        -> normal_ending
}

== bad_ending_content ==

主角：（我...我是谁？）

---

你的记忆被抹去了。
你忘记了所有的事情。
你忘记了樱井美咲...
你忘记了黑崎莲...
你忘记了白雪悠希...

你甚至忘记了自己的名字。

---

**坏结局：被遗忘的记忆**

你被卷入了实验的漩涡，
最终失去了自己。

也许有一天，你会想起...
但现在，一切都已经结束了。

---

-> ending_summary
