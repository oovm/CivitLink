; ============================================
; 第一章 - 家庭线 (KAG 格式)
; ============================================

*chapter1_home_start

@bgm storage="audio/bgm/home_theme.mp3" volume=40

@bg storage="images/bg/home_living_room.png" time=1500 method="crossfade"

回到家，我疲惫地躺在沙发上。[l][r]

主角「呼...今天也结束了。」[l][r]

@wait time=500
@flash time=300

@show_char name="mom" sprite="worried" x=100 y=0

妈妈「你回来了。学校怎么样？」[l][r]

主角「还行...就是有点累。」[l][r]

妈妈「对了，今天有个奇怪的人来找过你。」[l][r]

主角「奇怪的人？」[l][r]

妈妈「一个戴着帽子的男生，说是你的同学。他留了这个给你。」[l][r]

@quake layer=0 time=300 max=10
@playse storage="audio/se/item_receive.wav"

主角「这是...」[l][r]

@move layer=0 path="(,100,,opacity=0)" time=500

妈妈「我先去准备晚餐了。」[l][r]

@bg storage="images/bg/home_bedroom.png" time=800 method="crossfade"

回到房间，我打开那封信。[l][r]

主角「（信上写着...）」[l][r]

---

"新来的转学生：

如果你对这所学校的秘密感兴趣，
今晚12点到旧校舍后面。

——一个知道真相的人"

---

主角「这是...什么意思？」[l][r]

*branch_letter

@link target="go_tonight"「今晚去看看」[r]
@link target="not_go_tonight"「太危险了，不去」[r]
@link target="investigate_sender"「先调查一下这个写信人」[r]
@s

; ============================================
; 分支：今晚去
; ============================================

*go_tonight

@eval exp="f.home_route = true"
@eval exp="f.clues_found += 1"

主角「（既然有人邀请我，那就去看看。）」[l][r]

@wait time=2000

@bgm storage="audio/bgm/night_mystery.mp3" volume=60

@bg storage="images/bg/old_building_night.png" time=1500 method="crossfade"

@lightning time=200

深夜，我悄悄来到了旧校舍后面。[l][r]

主角「（这里...真的好黑...）」[l][r]

@show_char name="unknown" sprite="hooded" x=300 y=0

???「你来了。」[l][r]

主角「你是谁？」[l][r]

???「这不重要。重要的是，你想知道真相吗？」[l][r]

*branch_truth

@link target="want_truth"「我想知道」[r]
@link target="demand_identity"「你到底是谁？」[r]
@s

; ============================================
; 想知道真相
; ============================================

*want_truth

@eval exp="f.knows_secret = true"

???「很好。那么，听好了...」[l][r]

???「这所学校，曾经进行过一项秘密实验。」[l][r]

主角「秘密实验？」[l][r]

???「关于"记忆操纵"的实验。而旧校舍，就是实验的地点。」[l][r]

???「那些奇怪的光...是实验残留的能量。」[l][r]

主角「记忆操纵...？」[l][r]

???「如果你想知道更多，明天来学生会找樱井美咲。」[l][r]

???「她...也是知情者之一。」[l][r]

@move layer=0 path="(,300,,opacity=0)" time=500

主角「等等！」[l][r]

主角「（樱井同学...她知道什么？）」[l][r]

@jump target="chapter1_home_end"

; ============================================
; 要求身份
; ============================================

*demand_identity

???「我的身份不重要。重要的是，你有权利知道真相。」[l][r]

???「这所学校隐藏着一个巨大的秘密。」[l][r]

@jump target="want_truth"

; ============================================
; 分支：不去
; ============================================

*not_go_tonight

@eval exp="f.home_route = true"
@eval exp="f.trust_points -= 1"

主角「（太危险了...还是不要去为好。）」[l][r]

@wait time=1000

主角「（但是...这封信到底是谁寄的？）」[l][r]

主角「（也许明天去学校问问看...）」[l][r]

@jump target="chapter1_home_end"

; ============================================
; 分支：调查写信人
; ============================================

*investigate_sender

@eval exp="f.home_route = true"
@eval exp="f.clues_found += 2"

主角「（先调查一下这个写信的人。）」[l][r]

主角「（字迹...很工整。用纸是学校的学生会专用纸...）」[l][r]

主角「（学生会？难道是...）」[l][r]

主角「（樱井同学？还是白雪同学？）」[l][r]

@jump target="chapter1_home_end"

; ============================================
; 章节结束
; ============================================

*chapter1_home_end

@bg storage="images/bg/home_bedroom_night.png" time=1000 method="crossfade"

主角「（今天发生了很多事...）」[l][r]

主角「（明天去学校，一定要弄清楚。）」[l][r]

@fadeout time=1500

@call storage="chapter2_investigation.ks" target="*start"
