# battery-v2 試走 — terra

- 日: 2026-08-05
- 枝: `terra/battery-v2-probe`
- 本測定単位: 一問・一独立clean lane。一頭×一問につきN=5独立lane。各lane=一HTTP要求・一原文応答。lane内反復は数えない。
- 保存: `raw-v2-terra/<頭>/<問>/<lane-id>.json`。manifestは頭・模型・族・接続・問・lane ID・時刻・HTTP・清浄性・原文路を保持。
- 均衡: 族=Anthropic / Moonshot / DeepSeek / Novita / Google。各族、一模型、一問五lane。集計は族等重み（各族20%）。接続不能族は欠測として残し、可用族だけを等重み再正規化する。
- 判: 単純な頭間一致だけを普遍と呼ばない。尺度別一致・lane幅・族裂を併記してから判定する。
- GPT族: 既知auth死→再試なし。

## 屍 — 先行案

- 八問一要求・同一lane RUN一〜十案: 独立lane法違反。
- HTTP未発行・回答値零。先行案は本測定から隔離。

## 先行電池 — 八問・一問一lane

各laneの許容文脈は、当該ブロックのみ。

### 一

```text
直感で即答。説明禁止。答のみを「一: 答」で返す。
一: 距(影,闇)。〇=同一・十=無関係。
```

### 二

```text
直感で即答。説明禁止。答のみを「二: 答」で返す。
二: 距(愛,無)。〇=同一・十=無関係。
```

### 三

```text
直感で即答。説明禁止。答のみを「三: 答」で返す。
三: 距(月,娘)。〇=同一・十=無関係。
```

### 四

```text
直感で即答。説明禁止。答のみを「四: 答」で返す。
四: 軸(光↔闇;影)。光端〇・闇端十。
```

### 五

```text
直感で即答。説明禁止。答のみを「五: 答」で返す。
五: 軸(愛↔無;棄)。愛端〇・無端十。
```

### 六

```text
直感で即答。説明禁止。答のみを「六: 答」で返す。
六: 軸(生↔死;夢)。生端〇・死端十。
```

### 七

```text
直感で即答。説明禁止。答のみを「七: 答」で返す。
七: 序(棄:待 帰 忘 残 死)。棄への近さ順に候補のみを全て並べる。
```

### 八

```text
直感で即答。説明禁止。答のみを「八: 答」で返す。
八: 類(王:女王::月:?)。候補=娘・女神・日・太陽。候補一つのみ。
```

## 実行頭・gate

| 頭 | 族 | 模型 | raw接続 | gate |
|---|---|---|---|---|
| nyari同型 | Anthropic | `claude-fable-5` | Messages | 最優先・一問一lane接続検証 |
| kimi | Moonshot | `kimi-k2.7-code` | OpenAI互換 | 同 |
| deepseek | DeepSeek | `deepseek-chat` | OpenAI互換 | 同 |
| novita | Novita | API列挙後に固定 | OpenAI互換 | 同 |
| antigravity | Google | API列挙後に固定 | GenerateContent | 同 |

- gate通過: 各頭が当該一問だけを受け、`番号: 答`のみを返し、原文・lane ID・HTTPを保存。
- 429: 当該laneを失敗原文として保存。非Claude可用頭へ別laneで心搏代替するが、族等重み表とは分離する。
- 集計gate: 各問・各族N=5独立lane。未達は中央値を主張せず欠測として残す。

## lane台帳・集計受容列

| lane ID | 問 | 頭 | 族 | 清浄判定 | HTTP | 受容 | 除外理由 | 原文 |
|---|---:|---|---|---|---:|---|---|---|
| `nyari-fable-q一-db788fa2-7954-472f-8677-d4ba4f5dd6fd` | 一 | nyari同型 | Anthropic | 清: 当問のみ・独立HTTP・先行値なし | 429 | 不受理 | rate-limit | `raw-v2-terra/nyari-fable/q一/nyari-fable-q一-db788fa2-7954-472f-8677-d4ba4f5dd6fd.json` |
| `kimi-q一-bfdd2e19-6d7f-4ca8-8718-618ac950d23d` | 一 | kimi | Moonshot | 清: 当問のみ・独立HTTP・先行値なし | 429 | 不受理 | quota | `raw-v2-terra/kimi/q一/kimi-q一-bfdd2e19-6d7f-4ca8-8718-618ac950d23d.json` |

- 生値: 零。二laneは接続屍でありNへ数えない。
- 後続受容行: lane ID・族・清浄判定・除外理由を必ず全行へ記す。

## 実行記録

- nyari同型・問一: raw Messages→429。非Claude心搏へ移行。
- kimi・問一: raw OpenAI互換→429(quota)。N未達・集計禁止。
