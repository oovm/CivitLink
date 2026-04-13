; ============================================
; 结局 (KAG 格式)
; ============================================

*endings_start

@if exp="f.ending_type == 'public'"
@jump target="public_truth_ending"
@elsif exp="f.ending_type == 'restore'"
@jump target="restore_memory_ending"
@elsif exp="f.ending_type == 'investigate'"
@jump target="true_ending"
@else
@jump target="normal_ending"
@endif

; ============================================
; 结局：公布真相
; ============================================

*public_truth_ending

@bgm storage="audio/bgm/dramatic_theme.mp3" volume=70
@bg storage="images/bg/school_assembly.png" time=1500 method="crossfade"
@flash time=300

我们决定公开真相。[l][r]

主角「（这是正确的选择。）」[l][r]

@show_char name="sakura" sprite="determined" x=300 y=0
@show_char name="yuki" sprite="serious" x=100 y=0
@show_char name="ren" sprite="serious" x=500 y=0

樱井美咲「各位同学，我们有重要的事情要宣布...」[l][r]

@flash time=500

---

几天后...学校的秘密实验被媒体曝光，校长被撤职，所有参与实验的学生都接受了心理辅导。

---

@bg storage="images/bg/school_rooftop.png" time=1000 method="crossfade"
@bgm storage="audio/bgm/bittersweet.mp3" volume=50

樱井美咲「虽然真相大白了...但是学校现在一片混乱。」[l][r]
主角「这是必要的代价。」[l][r]
白雪悠希「至少，大家现在都知道真相了。」[l][r]
黑崎莲「哼，总算结束了。」[l][r]

@if exp="f.sakura_affection >= 5"
@jump target="sakura_good_ending"
@elsif exp="f.ren_affection >= 5"
@jump target="ren_good_ending"
@elsif exp="f.yuki_affection >= 5"
@jump target="yuki_good_ending"
@else
@jump target="normal_public_ending"
@endif

; ============================================
; 结局：恢复记忆
; ============================================

*restore_memory_ending

@bgm storage="audio/bgm/hopeful_theme.mp3" volume=60
@bg storage="images/bg/old_lab.png" time=1000 method="crossfade"

我们决定寻找恢复记忆的方法。[l][r]

@show_char name="sakura" sprite="hopeful" x=300 y=0

樱井美咲「我找到了！这里有备份数据！」[l][r]
@flash time=500

---

经过几天的努力，我们成功恢复了所有被改写的记忆。那些曾经失去记忆的学生，终于找回了属于自己的过去。

---

@bg storage="images/bg/cherry_blossom.png" time=1500 method="crossfade"
@bgm storage="audio/bgm/happy_ending.mp3" volume=60

樱井美咲「谢谢你...帮我找回了记忆。我现在...终于完整了。」[l][r]
主角「不用谢，这是我应该做的。」[l][r]

@if exp="f.sakura_affection >= 6"
樱井美咲「那个...以后...我们可以...」[l][r]
主角「可以什么？」[l][r]
樱井美咲「一起...一起走下去吗？」[l][r]
@jump target="true_love_ending"
@else
@jump target="good_ending"
@endif

; ============================================
; 真结局：星之子
; ============================================

*true_ending

@bgm storage="audio/bgm/mystery_deep.mp3" volume=70
@bg storage="images/bg/old_lab.png" time=1000 method="crossfade"

我们决定继续调查。[l][r]

@show_char name="ren" sprite="serious" x=500 y=0

黑崎莲「等等，看这个。」[l][r]
@flash time=500

---

文件内容："实验并未结束。负责人已转移至新校区。实验代号：星之子计划"

---

主角「实验还在继续？！」[l][r]
樱井美咲「新校区...那是刚建成的...」[l][r]
白雪悠希「我们必须阻止他们。」[l][r]

@wait time=1000
@bg storage="images/bg/sunset.png" time=1500 method="crossfade"
@bgm storage="audio/bgm/epic_theme.mp3" volume=70

---

我们发现了更深层的阴谋。实验并未结束，只是转移到了新的地点。我们的战斗，才刚刚开始...

---

@fadeout time=2000

---

**真结局：星之子**

你发现了隐藏在幕后的真相。故事将在续作中继续...

---

@jump target="ending_summary"

; ============================================
; 普通结局
; ============================================

*normal_ending

@bgm storage="audio/bgm/peaceful_ending.mp3" volume=50
@bg storage="images/bg/school_gate.png" time=1000 method="crossfade"

一切结束后，学校恢复了平静。[l][r]

@show_char name="sakura" sprite="smile" x=100 y=0

樱井美咲「谢谢你，新同学。如果没有你，我们可能永远不会知道真相。」[l][r]
主角「这是我应该做的。」[l][r]

@fadeout time=1500

---

**普通结局：平静的日常**

你帮助学校揭开了秘密，但真相似乎不止于此...

---

@jump target="ending_summary"

; ============================================
; 樱井美咲好结局
; ============================================

*sakura_good_ending

@bg storage="images/bg/cherry_blossom.png" time=1500 method="crossfade"
@bgm storage="audio/bgm/romantic_theme.mp3" volume=60
@show_char name="sakura" sprite="blush" x=300 y=0

樱井美咲「那个...我有话想对你说。」[l][r]
主角「什么事？」[l][r]
樱井美咲「这段时间...谢谢你一直陪在我身边。我...我好像...」[l][r]
主角「？」[l][r]
樱井美咲「我喜欢你！」[l][r]
@quake layer=0 time=1000 max=10
主角「樱井同学...」[l][r]
樱井美咲「以后...请多多指教了！」[l][r]

@fadeout time=2000

---

**樱井美咲结局：樱花之约**

你与樱井美咲成为了恋人。在樱花树下，你们许下了永远的约定。

---

@jump target="ending_summary"

; ============================================
; 黑崎莲好结局
; ============================================

*ren_good_ending

@bg storage="images/bg/night_city.png" time=1500 method="crossfade"
@bgm storage="audio/bgm/cool_theme.mp3" volume=50
@show_char name="ren" sprite="smile" x=300 y=0

黑崎莲「喂。」[l][r]
主角「黑崎同学？」[l][r]
黑崎莲「没想到你还挺有本事的。我对你刮目相看了。」[l][r]
主角「谢谢...？」[l][r]
黑崎莲「以后...有什么事可以找我。我会帮你的。」[l][r]
主角「真的？」[l][r]
黑崎莲「别让我说第二遍。」[l][r]

@fadeout time=2000

---

**黑崎莲结局：暗夜的盟约**

你获得了黑崎莲的认可。虽然他表面冷淡，但你知道，他会是你最可靠的伙伴。

---

@jump target="ending_summary"

; ============================================
; 白雪悠希好结局
; ============================================

*yuki_good_ending

@bg storage="images/bg/library.png" time=1500 method="crossfade"
@bgm storage="audio/bgm/gentle_theme.mp3" volume=50
@show_char name="yuki" sprite="smile" x=300 y=0

白雪悠希「那个...谢谢你。」[l][r]
主角「怎么了，白雪同学？」[l][r]
白雪悠希「谢谢你让我变得勇敢。以前的我...总是害怕面对真相。但是现在...我想和你一起面对未来。」[l][r]
主角「我也是。」[l][r]
白雪悠希「那...我们约定好了？」[l][r]
@flash time=1000
@fadeout time=2000

---

**白雪悠希结局：温柔的约定**

你与白雪悠希建立了深厚的羁绊。在图书馆的角落，你们约定一起面对未来。

---

@jump target="ending_summary"

; ============================================
; 真爱结局
; ============================================

*true_love_ending

@bgm storage="audio/bgm/love_theme.mp3" volume=70
@bg storage="images/bg/cherry_blossom_sunset.png" time=1500 method="crossfade"
@flash time=1000
@show_char name="sakura" sprite="happy" x=300 y=0

樱井美咲「从今以后...我们一起走吧。」[l][r]
主角「嗯，一起走。」[l][r]
@quake layer=0 time=1000 max=10

---

**真结局：永恒的星光**

你不仅揭开了学校的秘密，还找到了真正的爱情。樱井美咲的笑容，将是你心中永远的光芒。

---

@jump target="ending_summary"

; ============================================
; 好结局
; ============================================

*good_ending

---

**好结局：新的开始**

你帮助恢复了所有人的记忆。虽然还有未解的谜团，但你知道，未来会更好。

---

@jump target="ending_summary"

; ============================================
; 普通公布结局
; ============================================

*normal_public_ending

---

**结局：真相大白**

学校的秘密被公之于众。虽然付出了代价，但正义得到了伸张。

---

@jump target="ending_summary"

; ============================================
; 结局统计
; ============================================

*ending_summary

@fadeout time=2000
@bg storage="images/bg/black.png" time=1000 method="crossfade"

---

# 游戏统计

樱井美咲好感度: [emb exp="f.sakura_affection"][r]
黑崎莲好感度: [emb exp="f.ren_affection"][r]
白雪悠希好感度: [emb exp="f.yuki_affection"][r]
信任点数: [emb exp="f.trust_points"][r]
发现线索: [emb exp="f.clues_found"][r]

---

感谢游玩《星光学院的秘密》！

如果想要体验其他结局，请尝试不同的选择路径。

---

@title
