; KAG/Kirikiri 风格 DSL - 语法参考
; Kirikiri 引擎的 KAG 脚本语法

; ============================================
; 宏定义
; ============================================

@macro name="dialogue"
@eval exp="f.characters[mp.character].name"
@font size=24
@emb exp="mp.text"
@font size=default
@endmacro

@macro name="show_char"
@image storage="images/char/%(mp.name)_%(mp.sprite).png" layer=0 page=fore visible=true
@move layer=0 path="(,%(mp.x),%(mp.y),opacity=255)" time=500
@endmacro

; ============================================
; 变量初始化
; ============================================

@iscript
f.sakura_affection = 0;
f.ren_affection = 0;
f.yuki_affection = 0;
f.trust_points = 0;
f.clues_found = 0;
f.knows_secret = false;
f.knows_ren_secret = false;
f.has_lab_key = false;
f.has_lab_access = false;
f.school_route = false;
f.home_route = false;
f.ending_type = "";

f.characters = {
    player: { name: "主角" },
    sakura: { name: "樱井美咲" },
    ren: { name: "黑崎莲" },
    yuki: { name: "白雪悠希" }
};
@endscript

; ============================================
; 语法规则说明
; ============================================

/*
1. 变量定义：@iscript ... @endscript 中使用 JavaScript
2. 标签定义：*label_name|显示名称
3. 跳转：@jump target="label"
4. 选择分支：@link target="label"「文本」[r]
5. 条件判断：@if exp="条件" ... @endif
6. 变量操作：@eval exp="f.变量名 += 1"
7. 场景切换：@bg storage="图片路径" time=毫秒
8. 角色显示：@image storage="图片路径" layer=图层
9. 音乐播放：@bgm storage="音频路径" volume=音量
10. 注释：; 单行注释 或 /* 多行注释 */
*/

; ============================================
; 完整示例场景
; ============================================

*example_scene|示例场景

@bgm storage="audio/bgm/peaceful.mp3" volume=50

@bg storage="images/bg/classroom.png" time=1000 method="crossfade"

@show_char name="sakura" sprite="smile" x=300 y=0

樱井美咲「你好！欢迎来到星光学院！」[l][r]

*branch_example

@link target="choice_hello"「你好！」[r]
@link target="choice_who"「请问你是？」[r]
@if exp="f.knows_secret"
@link target="choice_secret"「关于那个秘密...」[r]
@endif
@s

*choice_hello
主角「你好！」[l][r]
@eval exp="f.sakura_affection += 1"
@jump target="next_part"

*choice_who
主角「请问你是？」[l][r]
@jump target="introduce"

*choice_secret
主角「关于那个秘密...」[l][r]
@jump target="secret_talk"

*introduce
樱井美咲「我是樱井美咲，学生会副会长！」[l][r]
@jump target="next_part"

*next_part
樱井美咲「要不要参观一下学校？」[l][r]

*branch_tour
@link target="tour"「好啊」[r]
@link target="end"「下次吧」[r]
@s

*tour
@bg storage="images/bg/school_hallway.png" time=800 method="crossfade"
樱井美咲「这里是教学楼，那边是图书馆...」[l][r]
@jump target="end"

*end
樱井美咲「再见！」[l][r]
@fadeout time=1000
@jump target="*game_end"

; ============================================
; 复杂分支示例
; ============================================

*complex_choice
黑崎莲「你想知道真相吗？」[l][r]

*branch_complex
@link target="want_all"「我想知道一切」[r]
@link target="avoid_danger"「太危险了，算了吧」[r]
@if exp="f.clues_found >= 3"
@link target="share_intel"「我已经知道一些了」[r]
@endif
@if exp="f.knows_ren_secret"
@link target="ren_confession"「你也是实验对象？」[r]
@endif
@s

*want_all
@eval exp="f.trust_points += 2"
@eval exp="f.ren_affection += 1"
@jump target="reveal_all"

*avoid_danger
@eval exp="f.trust_points -= 1"
@jump target="avoid_danger_scene"

; ============================================
; 多条件判断示例
; ============================================

*determine_ending

@if exp="f.sakura_affection >= 8 && f.trust_points >= 10"
@jump target="sakura_true_ending"
@elsif exp="f.ren_affection >= 8 && f.knows_ren_secret"
@jump target="ren_true_ending"
@elsif exp="f.yuki_affection >= 6 && f.clues_found >= 5"
@jump target="yuki_true_ending"
@elsif exp="f.sakura_affection >= 5"
@jump target="sakura_good_ending"
@elsif exp="f.ren_affection >= 5"
@jump target="ren_good_ending"
@elsif exp="f.yuki_affection >= 5"
@jump target="yuki_good_ending"
@elsif exp="f.trust_points >= 8"
@jump target="good_ending"
@else
@jump target="normal_ending"
@endif

; ============================================
; 结局定义
; ============================================

*sakura_good_ending

@bg storage="images/bg/cherry_blossom.png" time=1500 method="crossfade"
@bgm storage="audio/bgm/romantic_theme.mp3" volume=60

@show_char name="sakura" sprite="blush" x=300 y=0

樱井美咲「那个...我有话想对你说。」[l][r]
主角「什么事？」[l][r]
樱井美咲「这段时间...谢谢你一直陪在我身边。」[l][r]
樱井美咲「我...我好像...」[l][r]
主角「？」[l][r]
樱井美咲「我喜欢你！」[l][r]

@quake layer=0 time=1000 max=10

主角「樱井同学...」[l][r]
樱井美咲「以后...请多多指教了！」[l][r]

@fadeout time=2000

=== 樱井美咲结局：樱花之约 ===

你与樱井美咲成为了恋人。
在樱花树下，你们许下了永远的约定。

@jump target="show_stats"

; ============================================
; 游戏统计
; ============================================

*show_stats

---

游戏统计

樱井美咲好感度: [emb exp="f.sakura_affection"][r]
黑崎莲好感度: [emb exp="f.ren_affection"][r]
白雪悠希好感度: [emb exp="f.yuki_affection"][r]
信任点数: [emb exp="f.trust_points"][r]
发现线索: [emb exp="f.clues_found"][r]

---

感谢游玩《星光学院的秘密》！

@title

; ============================================
; 游戏结束
; ============================================

*game_end

@close
