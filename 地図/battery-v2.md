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
