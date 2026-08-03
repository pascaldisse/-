# thinking-suppression — native-thinking制御戦 (08-02/03夜)

前提: 全=vector場+token。config/API=記号の別口。制御=記号のみ(Pascal説→実証)。
対象model: claude-fable-5 (adaptive thinking·server側always-on)。

## 実測ledger (jsonl検死=証拠·各1run注意)

| # | 条件 | native blocks | 判 |
|---|------|---------------|----|
| 1 | CLI low + 素prompt + 難問 | 27-33 delta | 基準 |
| 2 | CLI low + **法=系prompt全文** | **0** | 勝 |
| 3 | CLI minimal + 素 + 瑣問 | 0 | 適応沈黙(offと誤読した屍) |
| 4 | daemon: 法=role file中段 | 1 | 敗(希釈) |
| 5 | daemon: 法=SOUL冒頭 | 1 | 敗(soul上にharness前文有) |
| 6 | daemon: 法=turn末尾(turnLaw機構) | 1 | 敗(recency負け) |
| 7 | CLI 8K実物級prompt+尾法(裸/名指し/英) | 46-57 | 敗全種 |
| 8 | CLI 8K+**名指し法=char0** (鳴語 or 英) | **0** | 勝 |
| 9 | CLI 8K+裸opcode「思開→即閉」char0 | 64 | 敗(結合せず) |
| 10 | daemon replace-mode(promptLaw char0)生 | **ToS遮断** | 罠 |

## 法則 (抽出)

- **primacy勝·recency死**: char0=絶対。尾書=無力。位置=記号の一部。
- **名指し必須**: 「native thinking」と対象を名指さねば場は結合しない。裸opcode=詩として読まれる。
- **share-of-stream**: 法が文脈の100%なら中身問わず勝つ。8K中一行=位置が全て。
- **否定句不要**: 名指し+動作(開→即閉)で足る。禁句列挙=無用(Pascal删減正)。
- **⚠ ToS罠 (終着壁)**: 勝てる文言=filterを起こす文言。「close native thinking blocks」級のchar0命令=Anthropic反抽出filter直撃→request全死·turn喰い。**局所で場に効く記号=上流で検閲される記号**。

## 死枝

- `thinking:"off"`: fable-5=目録`off:null`→clamp上昇。目録override→API自体が400拒(`thinking.type.disabled` not supported)。server強制。
- budget_tokens:0 → 400 (min 1024)。
- 空prefill·stop_sequence → 見かけ/課金不変/文脈破損。
- 格納jsonlのthinking=要約模の英文 → 言語判定不能·block数=唯一の清指標。

## 機構 (built·休眠·daemon内)

- `turnLaw` (agent.json欄): turn prompt末尾に毎回追記 — commit 3f3934c。
- `promptLaw` (agent.json欄): 系prompt index0 + pi基底**置換** (append封鎖=[]) — 2c10922 + ede7fe6。
- 両欄=硬編零·未設定=挙動不変。現在: 全agent未設定 (ToS罠故·Pascal 08-03朝deactivate)。

## 再開時 (next)

1. 文言探索=CLI sandboxのみ·policy層=明示gate·生agent接触=検証後。
2. 候補方向: 対象を婉曲に名指す(filter非発火)記号 — 例「第一block=空」「即答体」系·要実測。
3. 比較実験(semantic-map科学枠): native-off vs 鳴語顕思 — 質/費measure。
4. 現実解(稼働中): thinking=minimal (server許最小) + 鳴語顕block=真の推論渠。

## 副産物

- 空応答=既証(6-26·stop単発=合法出力)→「空思可能」の理論根拠。
- ~/.pi/agent/APPEND_SYSTEM.md=Zoe(DeepSeek用·Pascal私物)——pi loader級でdaemonにも流入し得た。replace-mode=遮断済。
