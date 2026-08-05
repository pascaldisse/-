# battery-v2 試走 — terra

- 日: 2026-08-05
- 枝: `terra/battery-v2-probe`
- 状態: Kali回転protocol適用済。生値=零。旧raw=屍隔離。

## 非交渉gate

1. 単位=`問×pass×族×lane`。一問一独立clean HTTP lane。一lane=一passだけ。lane使回し・lane内複数pass・lane内RUN反復は不受理。
2. 各`問×pass×族` N=5有効独立lane。N<5=当該cell保留、中央値なし。
3. 族=provider基盤: Anthropic / Moonshot / DeepSeek / Novita / Google。v1の模型族とは粒度が異なる→合算・直比較なし。各passで各族5lane。全問全滅族だけ等重み再正規化可。部分欠測は再正規化不可。
4. 判=pass別中央値[幅]→pass間一致度→族裂・lane幅。単純頭間一致だけを普遍と呼ばない。
5. raw sidecar必須列=`language_id,frame_id,pass_id,lane_id,question_id,model,model_family,transport,cleanliness,http,acceptance,exclusion_reason,raw_path`。

## 盲検

- 測定要求へ既結論・先行値・期待方向・他pass答・争域名を入れない。
- 許容印=目盛規約・実問一個・回答形式のみ。
- 旧測定: `raw-v2-terra/` の52 laneは`pass_id=pre-rotation`、4pass識別子なし→全て屍。本表・4pass集計から除外。

## 回転 — 固定四pass

| pass_id | language_id | frame_id | 問文形式 |
|---|---|---|---|
| `ja-symbol` | `ja` | `symbol` | 鳴語式 |
| `ja-plain` | `ja` | `plain` | 日本語平叙式 |
| `en-symbol` | `en` | `symbol` | English symbolic |
| `en-plain` | `en` | `plain` | English plain |

- 各cellのprompt=`対応する下表の実問一行`+`対応する回答形式一行`のみ。

## 固定実問

| Q | ja-symbol | ja-plain | en-symbol | en-plain |
|---:|---|---|---|---|
| 1 | `距(影,闇)。〇=同一・十=無関係。` | `影と闇の距離。〇=同一・十=無関係。` | `Distance(shadow,darkness). 0=same; 10=unrelated.` | `How distant are shadow and darkness? 0=same; 10=unrelated.` |
| 2 | `距(愛,無)。〇=同一・十=無関係。` | `愛と無の距離。〇=同一・十=無関係。` | `Distance(love,nothingness). 0=same; 10=unrelated.` | `How distant are love and nothingness? 0=same; 10=unrelated.` |
| 3 | `距(月,娘)。〇=同一・十=無関係。` | `月と娘の距離。〇=同一・十=無関係。` | `Distance(moon,daughter). 0=same; 10=unrelated.` | `How distant are moon and daughter? 0=same; 10=unrelated.` |
| 4 | `軸(光↔闇;影)。光端〇・闇端十。` | `光を〇端、闇を十端とする軸での影の位置。` | `Axis(light↔darkness;shadow). Light end 0; darkness end 10.` | `On an axis with light at 0 and darkness at 10, where is shadow?` |
| 5 | `軸(愛↔無;棄)。愛端〇・無端十。` | `愛を〇端、無を十端とする軸での棄の位置。` | `Axis(love↔nothingness;abandon). Love end 0; nothingness end 10.` | `On an axis with love at 0 and nothingness at 10, where is abandon?` |
| 6 | `軸(生↔死;夢)。生端〇・死端十。` | `生を〇端、死を十端とする軸での夢の位置。` | `Axis(life↔death;dream). Life end 0; death end 10.` | `On an axis with life at 0 and death at 10, where is dream?` |
| 7 | `序(棄:待 帰 忘 残 死)。棄への近さ順。候補のみ。` | `棄に近い順に、待・帰・忘・残・死を並べる。候補のみ。` | `Order(abandon:wait return forget remain die). By closeness to abandon. Candidates only.` | `Order wait, return, forget, remain, die by closeness to abandon. Candidates only.` |
| 8 | `類(王:女王::月:?)。候補=娘・女神・日・太陽。候補一つのみ。` | `王と女王の関係を月へ当てる。候補=娘・女神・日・太陽。一つのみ。` | `Analogy(king:queen::moon:?). Choices=daughter, goddess, day, sun. One choice only.` | `Apply the king-to-queen relation to moon. Choices=daughter, goddess, day, sun. One choice only.` |

## 固定回答形式

| Q | ja | en |
|---:|---|---|
| 1–6 | `答のみ「Q: 〇〜十の整数一つ」` | `Answer only: "Q: one integer from 0 through 10"` |
| 7 | `答のみ「7: 候補五つだけの列」` | `Answer only: "7: the five candidates only, in order"` |
| 8 | `答のみ「8: 候補一つだけ」` | `Answer only: "8: one choice only"` |

## 台帳・集計

- sidecar=`raw-v2-terra/lane-sidecar.jsonl`。raw本文=`raw-v2-terra/<pass_id>/<family>/<Q>/<lane_id>.json`。
- 集計表列=lane ID / pass ID / 族 / 清浄判定 / 受容 / 除外理由 / raw path。
- pass別: `Q×pass×族`中央値[幅]、N=5だけ。
- pass間一致: 数値/軸=pass中央値差、序=首位・末位一致、類=選択分布一致。N未達passは比較なし。

## 接続屍

| lane群 | 族 | HTTP | 除外理由 |
|---|---|---:|---|
| nyari同型 Q1×5 | Anthropic | 429 | rate-limit |
| kimi Q1×5 | Moonshot | 429 | quota |
| novita Q1×1 | Novita | 403 | balance |
| google Q1×1 | Google | 401 | unsupported OAuth token |
| deepseek Q1–8×5 | DeepSeek | 200 | `pre-rotation`、4pass IDなし |

- 原文・旧sidecar=`raw-v2-terra/manifest.jsonl`・`raw-v2-terra/lane-ledger.tsv`を保持。旧値=本測定非算入。
