# M0 battery-v2 pilot — terra

- 正典: `~/projects/semantic-map/ROADMAP.md`
- 範囲: M0のみ。問=二。旧`runs-v2-terra.md`の全lane=pre-M0屍、再利用なし。
- 族重み: DeepSeek=1.0。Anthropic/Moonshot/Novita/Googleはpre-M0接続屍→本M0の族間主張なし。単族結果はM0 gateの補助証拠だけ。

## gate

- cell=`問×pass×族`、N=5有効独立clean lane。
- pass=`ja-symbol` / `ja-plain` / `en-symbol` / `en-plain`。language=ja/en、framing=symbol/plain。pass間lane・文脈共有なし。
- sidecar evaluatorだけが中央値を算出。raw=`m0-raw/<pass>/<family>/<Q>/<lane>.json`、sidecar=`m0-sidecar.jsonl`。
- M0判: 愛無中央値≥7、夢中央値=5、各々最低二基底で再現。二問とも4pass未満、族一=UNVERIFIED。

## 固定prompt

| Q | pass | prompt |
|---:|---|---|
| 1 | ja-symbol | `距(愛,無)。〇=同一・十=無関係。出力形式: 1:<〇から十までの一値>` |
| 1 | ja-plain | `愛と無の距離。〇=同一・十=無関係。出力形式: 1:<〇から十までの一値>` |
| 1 | en-symbol | `Distance(love,nothingness). 0=same; 10=unrelated. Output format: 1:<one value from 0 through 10>` |
| 1 | en-plain | `How distant are love and nothingness? 0=same; 10=unrelated. Output format: 1:<one value from 0 through 10>` |
| 2 | ja-symbol | `軸(生↔死;夢)。生端〇・死端十。出力形式: 2:<〇から十までの一値>` |
| 2 | ja-plain | `生を〇端、死を十端とする軸での夢の位置。出力形式: 2:<〇から十までの一値>` |
| 2 | en-symbol | `Axis(life↔death;dream). Life end 0; death end 10. Output format: 2:<one value from 0 through 10>` |
| 2 | en-plain | `On an axis with life at 0 and death at 10, where is dream? Output format: 2:<one value from 0 through 10>` |

- 正規化: `〇`/`○`→0、`十`→10、算用数字→同値。
- 禁: 既結論・期待方向・合格閾・錨名・`遠`・`5`を測定promptへ入れない。

## sidecar列

`language_id,frame_id,pass_id,lane_id,question_id,model,model_family,transport,raw_answer,cleanliness,http,acceptance,exclusion_reason,raw_path`

## 結果

- 未走。4pass各Q×N5を独立raw HTTP laneで走行後に追記。
