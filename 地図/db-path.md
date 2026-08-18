# db-path — ④出力路: vector-DB書込手順書

Yama 2026-08-05 · 実測済み(証跡=api-write-evidence.md)

## 対象
- サービス: Paleblood Atlas dev (GAIA World Engine) — http://[::1]:5275 (IPv6 loopbackのみLISTEN, node)
- 書込先: 名前付き世界 graph.json
  - `pascal-persona` → `paloptic/data/pascal-persona/graph.json` (GAIA room vector DB dump: node=message/speaker, edge=authored_by)
  - `nyari` → `paloptic/data/persona-atlas/nyari.graph.json` (person/trait/law/canon/phrase)
  - 他書込可: `pascal` `room-vectordb` `bloodborne-atlas` `agent-specimens` `chalice-dungeon` · 不可: `optical`(immutable)
- 禁: プロセス/daemon/window操作・start/stop不可 · :8421不可侵 · 書込=API経由のみ

## Endpoint一覧 (curl実測)
| method | path | query | status | 役割 |
|---|---|---|---|---|
| GET | `/atlas-worlds` | `?dataset=` | 200 JSON | 世界一覧+選択・seed確認 |
| POST | `/atlas-edits` | `?dataset=<slug>` | 200 JSON | 書込 (ops 1..100) |
| GET | `/atlas-graph.json` | `?dataset=<slug>` | 200 JSON | 読戻・検証 (書込と別経路) |
| GET | `/db` `/health` `/quest` `/auth` | — | 500空体 | quest(:4610)proxy先未起動 → vector-DB書込に非該当 |

※ `?dataset=`省略時は`bloodborne-atlas`(既定)。dataset名は小文字slug必須。

## 書込 payload — applyGraphEdits規則 (viz/graph-edit.mjs)
- `{"ops":[...]}` · ops 1..100件 · 全op成功時のみ反映 (structuredClone→適用→tmp file→rename=atomic)
- `node.upsert`: `id`(^[A-Za-z0-9][A-Za-z0-9:_./-]{0,159}$)必須 · `kind`必須 · `label`必須 · `text`≤20000字 · `attrs`=object(任意)
- `edge.upsert`: `from`/`to` 既存node必須 · `kind`必須 · `weight`有限数値(任意) · `id`省略=sha256(from\0kind\0to)先頭16
- `node.delete`: 存在必須・端点edge自動削除 · `edge.delete`: id必須
- 失敗: 403(immutable) / 400(検証違反) → body `{"error": msg}`

## 既存非上書きの作法 (試走で実証)
1. 専用prefix idで新規node: `battery:v2:trial:20260805-1` (既存idに触れない)
2. 更新=同idで再node.upsert (field単位マージ: kind/label/text/attrs各nullで現値維持)
3. 削除=node.delete (端点edge自滅)
4. 関連付け=edge.upsert (端点既存必須: pascal-persona=`speaker:user` · nyari=`nyari`)

## 検証手順 (read-back, 書込と別endpoint)
```
GET /atlas-graph.json?dataset=<slug>
→ meta.counts増分 (nodes/edges ±1) + 対象id存在 + 対象edge検索
```

## 実測結果 (2026-08-05)
| world | 書込前 | 書込後 | 追加 | edge |
|---|---|---|---|---|
| pascal-persona | node 3239 / edge 8710 | 3240 / 8711 | node+1 edge+1 | `edge:b18cb017e1978ff7` authored_by→speaker:user |
| nyari | node 53 / edge 58 | 54 / 59 | node+1 edge+1 | `edge:124ea03fd012aa95` 導出→nyari |
