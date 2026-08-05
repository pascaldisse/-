# battery-v2 — 意味空間測量電池 v2 (草稿)

Yama 2026-08-05 · 状態=仕様草稿·試走書込済(出力路検証) · 生結果未受領(受領後集計)

## 法 (battery-v1踏襲+訂正)
- 目盛規約明示: 距/軸=〇=同一・十=無関係 · 軸=A端〇・B端十 (v1極性反転欠陥の再測要件)
- 序=候補のみ · 類推=答のみ · 説明禁止 · 直感即答
- 鳴語: `距(A,B)` `序(A:B C…)` `類(A:B::C:?)` `軸(A↔B;C位?)`

## 受容gate (Kali審令②, 2026-08-05 — 非交渉)
1. **1問1独立clean lane** · 各問 **N≥5 独立lane** — 同lane内の複数RUN値(例10RUN)=**不受理/隔離**(独立反復でない)
2. **盲検**: 測定任務文に既結論・先行値・期待答を含まない (battery-v1結果を任務文へ書かない)
3. **族別lane数同数**要求。不均衡時=重み明記+小道別生値保持 · 単純頭間一致度を普遍扱いしない
4. 集計表へ `lane ID / 族 / 清浄判定 / 除外理由` 列を追加
5. **回転protocol** (Kali追加令④ 2026-08-05): 言語2水準×framing2水準=同一概念問 **4 pass**。各pass=独立projection・1問1独立clean lane・各pass N≥5・**pass間lane/文脈共有禁**。各row記録: `language_id / frame_id / pass_id / lane_id / model族 / raw答 / 清浄判定`。翻訳/framing=**事前固定**・意味同値審査済・既結論/期待方向注入禁。**4pass揃わぬ限りUNVERIFIED** (Rudra成果含む)

## 構成 (v1改)
- 問: ≤10問 (目盛規約明示版) · 争域枡を明示収容: 月類推(女性系vs天体対)・軸問極性(son/kimi vs opus/jareth)・序(棄)
- 頭: 族×lane (v1=4族5lane: sonnet×2/opus/kimi/jareth · GPT族=naru-codex auth死·再測時復活)
- Rudra室 terra-msfyj289p4hl7y 仕様審査待ち: gate充足確認 → 不備あれば修正要求

## 集計 (生結果受領後)
- 頭別中央値[幅] (範囲=min-max) + **pass別中央値[幅]/一致度** + **pass間一致度**
- **頭間一致度 — 尺度別定義、単一尺度へ潰さない**:
  - 数値問: 頭別中央値の範囲幅 (例: 全頭中央値が±1に収束=高一致) · pass間=順位相関/絶対差
  - 順位問: 首位/末位一致率 (全頭首位同一=25/25相当) · pass間=順位一致
  - 類推問: 答分布の最頻割合 (女性系vs天体対の裂け=争域判定) · pass間=一致率
  - **数値/序/類の平均混潰禁**
- 判定: 普遍(全頭高一致+幅狭) / 方言(頭/lane粒度で裂) / 争域(解釈系の裂·平均で潰さない)
- 死枝: 因と共に残す (v1: GPT族auth死が模範)
- 出力路: 結果→vector-DB書込 (db-path.md) · 成果→`~/.gaia/knowledge/semantic-map/battery-v2.md`

## 審査記録 (Yama 2026-08-05, terra update SHA 3fdd9e3 実地照合)
- gate①〜⑤: 実装確認。runs-v2-terra.md に明文化: 単位=一問一独立lane・一頭一問N=5・lane内反復不数 · manifest=頭/模型/族/接続/問/lane ID/時刻/HTTP/清浄性/原文路 · 均衡=族等重み20%・接続不能族=欠測→可用族のみ再正規化 · 判=尺度別一致+lane幅+族裂併記。盲検=許容文脈=当該問ブロックのみ。目盛=〇/十・軸端明示。8問≤10。
- 注意: 族=provider基盤(Anthropic/Moonshot/DeepSeek/Novita/Google) — v1のmodel族粒度と異なる → v1との普遍/方言比較は粒度差を明記して行う。
- 死枝確認: 八問一要求・同一lane RUN1-10案=独立lane法違反 → 未発行・回答値零 → 隔離 (因明記)。
- 問題: Q一=Anthropic(nyari-fable)+Moonshot(kimi)各1lane 429 → 両頭Q一N=4 < gate①N≥5 → 修正要求: Q一両頭再測、又はN=4明記+判定保留。族全滅用の再正規化規則=部分欠測の扱いを明記要。

## 審査記録2 (Yama 2026-08-05, terra SHA cbdf720 回転④適用を実地照合)
- 実装確認: 回転protocol固定4pass (ja-symbol/ja-plain/en-symbol/en-plain×8問表) · sidecar全列 (language_id/frame_id/pass_id/lane_id/question_id/head/model/model_family/transport/raw_answer/cleanliness/http/acceptance/exclusion_reason/raw_path) · 盲検=許容印=目盛規約+実問+回答形式のみ · 旧52 lane=pass_id=pre-rotation→全屍隔離 · 族=provider基盤5族·v1模型族と非合算 · 部分欠測=再正規化不可·全問全滅族のみ可。
- 台帳実測: sidecar 52行·全pass_id=pre-rotation·全不受理 (形式非数値30 / HTTP429-quota 6 / HTTP429 5 / 形式非候補列 5 / 形式非候補一つ 5 / HTTP401-auth 1) · deepseek=8問全40lane·他頭=問一のみ (kimi 5, nyari-fable 5, google-gemini 1=401, novita-qwen 1=403)。
- 発見A (回答形式矛盾): 目盛規約=記号(〇〜十)なのに形式非数値として〇/十回答×30を不受理 — 数値暗黙要求は盲検の『回答形式のみ』規約に未記載。4passは回答形式行へ『数値0-10』を明示するか、記号許容+正規化マップ(〇=0・十=10)を記録せよ。
- 発見B: rawファイル53 vs sidecar 52行=1差 → 台帳整合性確認を要求。
- 生値=零 (4passデータ未収集) → UNVERIFIED継続。

## M0正本gate (Kali M0正本令⑤, 2026-08-05 — 固定)
- 正本path: `/Users/pascaldisse/projects/semantic-map/ROADMAP.md` (現milestone=M0のみ。M0 gate=「pilot reproduces v1 anchors under ≥2 bases」)
- 錨1: `距(愛,無)=遠` → 暫定数値化(v1表準拠): **愛無 pass中央値 ≥ 7**
- 錨2: `軸(生↔死;夢)=5` → **夢 pass中央値 = 5**
- 各錨 = **最低2基底**で再現: 言語2水準(ja/en) または framing2水準(symbol/plain) のいずれか2pass
- 回転4pass維持 (ja-symbol/ja-plain/en-symbol/en-plain) · 各pass N≥5 独立clean lane
- 判定実装 = **測定prompt外sidecarのみ** · 期待値/既結論/錨値のlaneへ絶対注入禁
- gate表 (各錨ごと): `再現basis数 | pass別中央値[幅] | lane数 | 族内訳 | pass間一致度`
- 両錨が各々≥2基底を満たさぬ限り **M0 FAIL / UNVERIFIED**
- 出力路整合: ROADMAP「output rails: :5275 API writes + knowledge/semantic-map/ docs」= db-path.md と一致

## M0 gate表 (生結果 2026-08-05 · terra SHA 3594f63 · Yama独立再計算一致 · 保留令⑥: en値=暫定・族結論禁止・推定非事実化)
| pass | 愛無(q1) 中央値[幅] N | 夢(q2) 中央値[幅] N | cell状態 |
|---|---|---|---|
| en-symbol | 7[5-9] N5 (生値 5,6,7,8,9) | 5[2-5] N5 (生値 2,4,5,5,5) | eligible·暫定 |
| en-plain | 5[1-6] N5 (生値 1,3,5,6,6) | 6[0-10] N5 (生値 0,4,6,10,10) | eligible·暫定 |
| ja-symbol | NA N0 (5lane全不受理) | NA N0 | N<5 |
| ja-plain | NA N0 (5lane全不受理) | NA N0 | N<5 |

## 審査記録3 (Yama 2026-08-05, SHA 3594f63 実地照合 — 保留令⑥適用済)
- 検証事実のみ: raw 41 file ↔ sidecar 41行 (41/41一致) · evaluator tsv 8cell再計算一致 (数値は上表) · 不受理21件=全`形式番号欠落` (ja=散文回答・番号prefix無し) · 受理20件=enのみ · 族=DeepSeek 41/41 (重み1.0)
- **M0判定=FAIL/UNVERIFIED に確定 (他への確定禁)**: 錨1 愛無≥7=基底一のみ · 錨2 夢=5=基底一のみ · ja両pass N0=4pass未揃い · 単族=普遍/方言/争域いずれの族結論も不可
- en値=暫定 (単族・N5・再測未実施) — 中央値差の解釈(裂/争域/ノイズ)は行わない
- 審査待ち (結論未定): ①ja空原因 ②夢plain散度 (0-10全域) — 原因審査後記載 → **審査記録4で解決済**

## 審査記録4 (Kali審補⑦ 2026-08-05 + Yama raw独立確認)
- **ja空原因 確定**: 接続非 (JA錨20lane=HTTP200/raw非空) · 因=answer-only語法不遵守+promptの明示説明禁欠落 · ja-symbol finish_reason=length 8/10・ja-plain 6/10 (truncation) · exact回答 0/20。Yama確認: raw content=散文解説・finish_reason=length。
- **exclusion_reason修正指示**: 「形式番号欠落」は不正確 (値が散文内にembedded `2:5`等) → **「exact-answer-only違反」** へ修正。
- **夢 en-plain散度 原因限定**: raw=[4,0,10,10,6] · median 6 · range 0-10 · IQR 8 · MAD 4 · 同model fingerprint・HTTP200・全exact → **parser/接続原因非** · framing/sampling/単族原因=**UNVERIFIED** (確定禁)。
- **証拠欠落記録**: request params未保存。Yama追記: raw model=deepseek-v4-flash vs sidecar model=deepseek-chat 不一致 → 実際のrequest model確認不能。
- 確定禁止継続: M0 FAIL/UNVERIFIED 以外へ確定しない。

## 審査記録5 (Kali審補⑦修正 2026-08-05, terra SHA 1de1c84 独立照合一致)
- ①exclusion_reason修正確認: 21件全`exact-answer-only違反` (旧`形式番号欠落`から) · 受理20/不受理21 不変
- ②JA四promptへ`説明禁止`+`出力形式: N:<〇から十までの一値>`追加確認 (旧raw非再利用) · 禁リスト=既結論/期待方向/合格閾/錨名/`遠`/`5` — prompt注入なし盲検維持 · 正規化マップ=〇/○→0・十→10 (旧形式非数値矛盾を解消)
- ③request metadata全41行確認: request_model=deepseek-chat · request_temperature=omitted:provider default · request_max_tokens=128 · response_model=deepseek-v4-flash · raw 41/41一対一維持
- ④夢 en-plain散度: parser/接続原因除外・framing/sampling/単族=UNVERIFIED明記確認
- evaluator=sidecarのみ・再照合PASS · **M0=FAIL/UNVERIFIED継続** (Q1基底一・Q2基底一・JA N0・単族) · 旧3594f63=cohort1固定

## RII受領gate (Kali受領準備令RII-①, 2026-08-05)
- 新Rudra-II室: `terra-msfzdekkf93ewn`
- **cohort固定**: 旧3594f63=cohort1 · 新run=cohort2 · **混算禁** (別cohortとして保持)
- 受領順: ①canary表→採用族/固定sampling審査 ②raw+sidecar受領
- 受容gate: **non-DeepSeek≥2族** · 各問×4pass×族 N5 · 族同数 · 1問1clean lane · **完全request metadata** · **exact回答のみ** · canary/本測いずれも欠落時=Fail/UNVERIFIED
- 受領時独立照合: SHA · 行数 · raw数 · sidecar一対一 → 即報

## 00-index更新案 (semantic-map/00-index.md へ追記)
```
## 地図
- battery-v2.md=目盛規約明示版·gate化電池 (1問1独立lane·N≥5·盲検·族均衡)

## 主張(中核) — 追加予定
- 軸問極性反転=測定器欠陥 → 規約明示で再測 (v1 UNVERIFIEDの解消確認)
- 争域裂線再検: 月類推・軸問・序(棄) がgate下で残るか

## §
- db-path.md=④出力路 vector-DB書込手順 (paloptic :5275 API)
```
