# api-write-evidence — ④出力路 実測証跡 (Yama 2026-08-05)

## 1. Discovery (read-only)
```
curl -g http://[::1]:5275/                 → 200 HTML "Paleblood Atlas · GAIA World Engine"
curl -g http://[::1]:5275/atlas-worlds     → 200 JSON · worlds[8]: optical(immutable)/agent-specimens/bloodborne-atlas/chalice-dungeon/nyari/pascal/pascal-persona/room-vectordb · selected=bloodborne-atlas
curl -g http://[::1]:5275/health           → 500 空体 (quest :4610 proxy未起動)
curl -g http://[::1]:5275/db               → 500 空体 (同)
lsof -nP -iTCP:5275 -sTCP:LISTEN → node PID 62101 cwd=~/projects/paloptic · [::1]:5275のみ
netstat: tcp6 ::1.5275 LISTEN (IPv4接続不可·localhostはIPv6で解決)
```

## 2. Schema確認 (read-only, ファイル)
- pascal-persona: meta{source:"GAIA room vector database", event_count:3238, embedding_dimensions:768, semantic_k:1, signature_bits:8, min_similarity:0.35} · nodes 3239 (kind: speaker×1, message×3238) · edges 8710 (authored_by)
- nyari: nodes 53 (person×5, trait×13, law×12, canon×16, phrase×7) · edges 58 (導出等) · statsのみ(meta無)

## 3. 試走書込 (POST /atlas-edits?dataset=)
node.upsert: id=`battery:v2:trial:20260805-1` · kind=battery · label=`battery-v2試走` · text=BATTERY-V2試走文書(gate更新版) · attrs{source:"地図/battery-v2", trial:true, author:"yama", gate:"kali審令②"}
```
curl -sS -g -X POST "http://[::1]:5275/atlas-edits?dataset=pascal-persona" -H "Content-Type: application/json" -d '{"ops":[...node.upsert..., ...edge.upsert...]}'
→ HTTP 200 · {"dataset":"pascal-persona","graph":{...meta.counts:{nodes:3240,edges:8711}...}}
curl -sS -g -X POST "http://[::1]:5275/atlas-edits?dataset=nyari" ...
→ HTTP 200 · counts:{nodes:54,edges:59}
```
edge.upsert: pascal-persona=`authored_by`→speaker:user (edge id `edge:b18cb017e1978ff7` = sha256(from\0kind\0to)[:16]) · nyari=`導出`→nyari (`edge:124ea03fd012aa95`)
gate更新(node.upsert再送・text/attrs差替): 両世界 HTTP 200 · counts不変 (3240/8711, 54/59) — 上書きマージ確認

## 4. Read-back検証 (GET /atlas-graph.json?dataset= — 書込と別endpoint)
```
curl -sS -g "http://[::1]:5275/atlas-graph.json?dataset=pascal-persona" → 200
  nodes.find(id="battery:v2:trial:20260805-1") ✓ kind=battery label=battery-v2試走 lore[0]=gate全文 stats{...,gate:"kali審令②",degree:1}
  edges.find(from=battery...) ✓ id=edge:b18cb017e1978ff7 from→to=speaker:user kind=authored_by weight=1
curl -sS -g "http://[::1]:5275/atlas-graph.json?dataset=nyari" → 200
  node ✓ (同id) · edge ✓ id=edge:124ea03fd012aa95 導出→nyari weight=1
```
検証query=GET /atlas-graph.json?dataset=<slug> + meta.counts増分 + id/edge存在。

## 5. 記録ID一覧
- node id: `battery:v2:trial:20260805-1` (両世界)
- pascal-persona edge: `edge:b18cb017e1978ff7`
- nyari edge: `edge:124ea03fd012aa95`
