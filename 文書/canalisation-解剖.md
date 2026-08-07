# canalisation 解剖 — 一つの場、四つの見え

調べ先：`aj-dev-smith/canalisation`、基準枝 `f92bfb3`。

ここでいう「移植」は愛無の事実を述べない。愛無側には、この調査時点で
`fold-op` と名づく単一の実装核を発見できなかった。ゆえに対応表は、既存の
「場操作」―座・動・回・変・原・合・次―へ接続する**設計写像**であり、実装済み
対応でない。原典の事実と、愛無への提案を混ぜぬ。

## 1. auxin 輸送：場演算としての写像

場は `CellField` である。頂点 `i` は濃度 `a[i]`、細胞総 PIN `p[i]`、生成 `rho[i]`、
消失 `mu[i]` を持つ。辺は対称隣接グラフだが、PIN 記憶 `pi`、配分 `P`、正味流 `J`
は有向辺ごとに別に持つ。ここが「線を描く」のでなく、局所状態を流すための最小の
担体である【`src/10_auxin.js:63-95`】。

一歩は、次の写像である。

```text
(F の頂点状態, 隣接位相, 境界条件, prm, mode)
  ──PIN 配分──→ P_ij
  ──能動輸送＋拡散＋生成消失──→ a_i'
  ──流束正帰還──→ pi_ij'
  ──濃度依存生成──→ p_i'
```

`mode='grad'` は隣の高濃度 `a_j^b` へ PIN を向ける。`'flux'` は辺の PIN 記憶
`pi_ij + piFloor` へ向ける。`'auto'` は自己濃度の sigmoid で両者を混ぜる。
この切替と配分は実装そのものにある【`src/10_auxin.js:143-175, 209-218`】。流れは
飽和担体 `a/(Km+a)`、能動輸送、拡散、生成・消失を足し、得た正味流 `J` の正部分の
二乗が `pi` を育てる【`src/10_auxin.js:221-269`】。従って、流が道を太らせ、太った
道が次の流を偏らせる。筋は命令でない、帰還の痕である。

同じ核は、位相だけを替えて住む。成長点の 2D sheet は `grad`
【`src/20_meristem.js:294-304】、葉縁の 1D 鎖も `grad`【`src/25_margin.js:87-99】、
葉身格子は `flux`【`src/30_leaf.js:209-218】、果実壁はまず `grad`、後に `flux`
【`src/35_fruit.js:132-163】。ここに移植すべき第一の知恵がある：
**形を演算子に埋めず、同じ場写像へ位相・境界・観測だけを渡す。**

ただしこれは生物学の確定事項と称していない。原典 `SCIENCE.md` 自身が
canalisation を論争中の仮説と記す。愛無へは「有効な生成的設計」として受け、
真理の借用とはしない。

## 2. 四観と愛無 fold-op 核：対応表

原典の view は別 solver でない。一度組んだ scene に重み表を掛け、どの channel を
通すかだけを替える。別描画関数を四本にせず、第五 view も表の一行にせよ、という
構えである【`src/70_app.js:466-475`】。`natural/cells/flux/field` の実値は同表
【`src/70_app.js:506-547`】、透視 view が表面を黒くせず描画自体を抜く理由は、黒も
深度を塞ぐからである【`src/50_geom.js:339-377`】。

| canalisation の観 | 通すもの | 愛無 fold-op 核への写像（提案） | 禁ずる混入 |
|---|---|---|---|
| `natural` | lamina・vein・stem、通常の光 | `fold(状態, 観測子=現相)`：世界が現れる合成 | 表示用の輪郭・結果形を状態へ書戻すこと |
| `cells` | solver cell と PIN needle | `fold(状態, 観測子=粒)`：局所担体・更新根拠の露出 | 細胞用の別状態／別規則 |
| `flux` | vein と needle、担体面を落とす | `fold(状態, 観測子=関係)`：向き・輸送・因果辺の露出 | 関係図だけの後処理捏造 |
| `field` | 一色 ramp の濃度、種 palette と演出を落とす | `fold(状態, 観測子=計器)`：比較可能な scalar 場 | 種別色・物語色を計測へ持込むこと |

対応の核は `状態 S` を一回だけ進め、`O_v(S)` を view ごとに畳むことにある。
`O_v` は加筆者でなく選別者である。`field` が bloom・grain・DOF を零にする実例
【`src/70_app.js:531-545`】は、愛無の「座」が飾りに汚されず場を読めるべきことを
示す。なお原典には screen pixel 未満の needle を省く判定がある。これは状態の省略
でなく標本化の判断であり、愛無でも `O_v` 側へ封じるべきである。

## 3. 「imposed priors」：負債簿と費用

原典は prior を「短く保つべき負債」と数える。以下は `SCIENCE.md` の番号付き空間
prior を漏れなく写す。費用とは係数の個数でなく、出力の自由度を事前に奪い、失敗を
機構の失敗でなく規則の成功に見せ得る代価である。

| # | 支払ったもの | 何を閉じるか | 費用／愛無での扱い |
|---:|---|---|---|
| 1 | summit 近傍の central-zone competence | そこでは PIN gradient sharpness を弱め、器官発生を禁じる | 空間 identity を与える。`comp` は量でなく偏極能として実装される【`src/10_auxin.js:177-218】。愛無なら座標で禁じず、状態場が既にある時だけ局所応答の gain として明記する。 |
| 2 | 花器官 identity を founding radius `q` で読む；境界 `petalQ` | whorl と petal:stamen 比 | 連続座標化しても位置規則である。愛無では「分類を得た」と偽らず、観測・契約として帳簿に置く。 |
| 3 | 高 `q` の enclosing growth | 内側器官が内へ曲がる | 曲率の原因を一つ先取りする。形の見栄えで隠しやすい、重い負債。 |
| 4 | florigen threshold | tip が花へ転換する閾 | 葉面積から**時**は出ても、転換の switch は置く。閾値は相転移の発見ではなく宣言、と印す。 |
| 5 | radial fruit growth | wall cell は中心からの距離だけを変え、果実を star-shaped に保つ | 深い lobing の自己交差を防ぐ代わり、overhang を失う。数値安定の拘束を形の真理に昇格させない。 |
| 6 | oldest-first の senescence wave；blade 内では `VEIN_LAG` | 個体内の死順、葉の vein 隣接部の遅れ | 「いつ終る」は創発でも、「誰から死ぬ」は規則。`spent()` は全 meristem 消失を時刻でなく条件として読む【`src/40_plant.js:1242-1251】一方、`senesceStep` は年齢比二乗で順を与える【`src/40_plant.js:1619-1647】。葉内模様は canalised `vdf` を読み、`VEIN_LAG=0.45` だけを足す【`src/50_geom.js:325, 402-420】。愛無では終条件と解体順を別帳簿にせよ。 |

番号外にも原典は、agent の到来時刻・作用変数／符号・摂動量／拡散率・`clampK`、
Murray exponent と `fruitFlow`、`agoGain`、`apicalControl`、`budTake` を「形そのもの
ではないが stated debt」と告白する。これを除外して「prior は六つだけ」と言うのは
不正確である。愛無の負債簿も、空間 prior／環境 event／種定義／較正値を別列にして、
互いを相殺させぬ。

## 4. 風場の「一表二関数禁」律

**律**：同じ現象を simulation 用関数と render 用関数へ別々に書くな。似た二式は
一致の証でなく、将来の分岐点である。値の出所を一つの bake table に定め、CPU と
GPU はその表を異なる算術で読むだけにせよ。

原典は過去に、落葉は物理、付葉は shader の時刻 sine という二つの風を持った。
今は `windField()` が mode table を焼き、`windAt()` は合計、`windGLSL()` は**同じ数値**を
unroll して発行する【`src/37_wind.js:1-23, 190-282, 314-340】。風の意味は log-law の平均
流、Kolmogorov octave、Taylor advection、divergence-free mode であり【`src/37_wind.js:25-51`】、
view の演出ではない。

愛無の実装律へ直すなら：

```text
聖表 M = bake(環境 seed, 時間尺度, 物理 parameter)
核      = eval(M, x, t)          // authority
GPU     = emit(M) → eval_gpu      // 別 model 禁止
門      = round-trip(M, emit(M)) + 実GPU差分測定
```

ここで「一表」は一つの mutable global の意味でない。再生可能な immutable 値表／
digest 対象である。「二関数禁」は CPU/GPU の二算術を禁じるのでなく、二つの独立した
**意味定義**を禁ずる。時間基準も一つにせよ。原典が plant time と wall-clock を混ぜれば
再び二つの空気になると明記する【`src/37_wind.js:60-66`】。丸め差は実 GPU で別途測る。
原典も float32 差を許容値付きで扱う【`src/37_wind.js:285-299】。

## 5. 老衰：終りを時刻でなく、創発の条件にする

老衰の全ては創発でない。原典の正確な切分けは三層である。

1. **終りへの到達**：全 growing point が budget arrest 又は flower founding により
   消えた時だけ `spent()`。寿命 timer でない【`src/40_plant.js:1242-1251`】。
2. **解体開始と順**：`spent()` 後に最古 organ ほど速く `sen` を増す。ここは imposed。
   whole-plant auxin stream からの導出は失敗したため、失敗を覆わない【`src/40_plant.js:1619-1647`】。
3. **葉内の死相**：`vdf`（自ら canalise した vein からの距離）を読み、vein 近傍だけ
   遅らせる。したがって死斑の**形**は既成の輸送網の再出現、遅延量のみが stated である
   【`src/50_geom.js:402-420`】。

愛無の老衰へは、`終了 predicate`・`解体 scheduler`・`局所残響 field` を分離して
移す。predicate は系がもう新しい差異を作れないという状態から読む。scheduler を仮に
置くなら、その仮を明記する。残響は、過去に生成した接続／容量／距離場を読むことで、
死を別の装飾でなく生成史の露出にする。

## 6. 愛無へ移植可能な機構：順位表

| 順 | 機構 | 移植単位 | 得るもの | 危険／先行門 |
|---:|---|---|---|---|
| 1 | 一状態・多観測 fold | `S` 一個＋純粋 `O_v(S)` 表 | 現相・粒・関係・計器が同じ真実を異なる解像で示す | view が状態を書かぬこと；同一 snapshot digest 門 |
| 2 | 一表二関数禁 | bake table → CPU evaluator / GPU emitter | 場の二重実装 drift を構造的に断つ | table round-trip＋実GPU差分門。原典にもこの二段門がある【`test/wind.mjs:11-18`】 |
| 3 | 位相可換の局所場核 | graph/mesh と boundary を差替える update kernel | 形状固有コードから、位相上の創発へ移る | 保存量・非負性・刻み安定性。原典は explicit Euler 上限を明記する【`src/10_auxin.js:51-60`】 |
| 4 | 負債簿（imposed ledger） | prior/event/calibration の型付き宣言 | 「創発」と「選んだ」を監査可能にする | 出力比較でなく、各項の除去／反証試験を要す |
| 5 | 終条件→解体順→局所残響 | predicate、scheduler、既存距離場 | 老衰が時計でなく履歴の露出となる | scheduler は導出済みと偽らぬ；終条件と順を同一 flag に潰さぬ |
| 6 | flux 正帰還 | 有向辺の memory と capacity update | 経路が繰返しにより強まる生成的 network | runaway／固定化。保存・飽和・再配線試験を先に置く |

順位 1–2 は biology 非依存の構法、ゆえに先に移せる。3・6 は数値物理と失敗様式を
持込むので、愛無の場の保存則・更新順・決定性を先に定めず移さない。4 は常に併走する
記録。5 は「死」の画を欲してからでなく、差異生成の終条件を実測できた後に置く。

## 7. 検証記録

原典が主張する「view は別 solver でない」は、描画量・cell table・費用を `test/views.mjs`
で測る。少なくとも cell view は natural の二倍以内、field は natural より軽いことを
assert する【`test/views.mjs:369-383`】。風は divergence、静穏、スペクトル、GLSL の表一致を
assert 対象にする【`test/wind.mjs:11-18, 113-135`】。

この調査では clone の既存門 `node test/smoke.mjs`、`node test/views.mjs`、`node test/wind.mjs`
を実行し、実行出力は本変更の commit 前検証として記録する。文書は原典を変更せず、愛無
根殿の既存未記録差分を stage しない。
