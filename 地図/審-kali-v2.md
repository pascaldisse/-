# 審 Kali-II v2 — M0後半

日=2026-08-05 · 神=Kali-II(陰·審) · 対象=`地図-計画-v2@38bb20b` + Rudra-II/Yama handoff + Shiva衛生残

## gate貼

- 現=`M0 FAIL / UNVERIFIED`
- 採用族=DeepSeek一族のみ · non-DeepSeek=零 · 錨=未走
- 必須=両錨各≥2基底 + ≥3族均衡 + 問×pass×族 N5 + raw/sidecar一対一
- 故 4pass錨発射禁 · 集計禁 · DB確定禁
- 死枝=GPT auth · Moonshot429×6 · Novita403×2 · GLM credential無 · Ollama無

## Rudra-II監督

- `terra/rudra-ii@fff45fb` clean · 実測`f07f952`をYama `b84f91c`照合済
- canary=DeepSeek exact 2/2のみ → 族gate未達
- context=108667/272000=39.95% > L3閾10% · HANDOFF済 · 測定再開不可
- A9追補を室`terra-msfzdekkf93ewn`へresume送達: 反転軸別lane・`s_AB+s_BA=10`残差/CI · 非価値軸(具抽/過去未来/大小) · 一般因子負荷/相関 · 事前gate固定 · A7併用
- 追補commit=未着 → UNVERIFIED

## lampas成果 実測審

### vocab.json

- path=`~/worktrees/lane-lampas/地図/vocab.json` · commit=`2ceb554`(枝tip `38bb20b`)
- SHA256=`f2ac356b93909969266edb10e8ab2ba041672dfd915ccc62431b94850a0d1304`
- JSON parse=PASS · 1954件/一意1954 · 空concept零 · domain外零 · source外零
- domain=`宇宙論326/感情322/技326/関係327/七つ328/日常325`
- 層(`注`)=核359/周1073/縁522 · source=`terra715/lampas-seed261/flash978`
- boundary=26件・全件domains≥2 · axis_end=愛無生死光闇夢の7件
- 判=構造受理。約2000語gate=1954を許容するか上位裁定待; source値は保持するが原seed SHAの項目別連結無→provenance再構成はUNVERIFIED。

### plan-v2 rev.4

**差戻(重)**

1. A9欠落: `A9/一般因子/反対称/非価値`記載零。§一で13軸を試走候補化、A9「採用前直接検」を実行段へ置かず。軸選抜承認不可。
2. N算定衝突: 本書=`N8=四族×2`、Rudra/Yama正本=`問×pass×族 N5`かつ≥3族。錨二問の最小は`2×4×3×5=120 lane`、本書64は過小。争域`N10`は四族均衡とも三族均衡とも整数分配不能。故`M0計280`無効。
3. 夢gate衝突: handoff正本=`pass中央値=5`、本書=`4〜6`。上位裁定なしの帯拡張→不可。
4. F3未遮断: 回転基底6対中5対|r|≥0.9の既測所見を設計入力へ反映せず、言語×framingを「独立projection」と運用。§九で仮定UNVERIFIEDとは書くが、A9前の基底採用を止めていない。
5. 記載内数値腐敗: §九に旧`12,408/27,936`残存、現表`28,544/128,256`と不整合。

**受理**

- A1一問一lane · A2遮断 · A3族結論 · A5対照 · A6 raw/sidecar · A7相関 · A8復元条件は記載有。
- M0/M1以降の分離 · 期待値sidecar限定 · pass別集計 · 死枝/UNVERIFIED節は可。

判=`plan-v2 FAIL / 差戻`。単価32lane・錨・280lane全体の執行承認なし。

## Shiva衛生残 実測

- repo=`~/projects/paloptic` · commit `c4c27d2`は`.gitignore`5行追加のみ(`proof/cleanup-*`,`proof/surya-vector-db/`)。
- 枝`jareth/surya-chalice-db@c4c27d2`のlane生成物隔離方針=部分受理。
- main作業樹=dirty、`__wt`/`atlas-play.log`等未追跡残。共有稼働物ゆえ本審で削除せず。
- tracked絶対path残: 例`proof/stayn-autos-test/probe-sa.mjs:33-34`・`proof/k4-jump-probe/report.json:10-13`。対象m29との同一性は所在札欠でUNVERIFIED。
- `c4c27d2`後の相対化commit・live gate再実行証跡=零。
- 判=衛生弾劾6 **未閉**。未追跡物の所有者分類→対象証跡相対化→既存live gate再走が必要。

## 死枝

- 4pass即走→死: ≥3族gate未達。
- 全raw死fallback→死: DeepSeek raw生存ゆえ許可条件不成立。
- 旧Rudra/Yama再稼働→死: 両室寿命閾超過・HANDOFF済。
- plan-v2 280lane→死: N定義衝突+A9欠落。

## HANDOFF

- 枝=`kali/audit-v1`
- 次: Rudra後継を親が新召→本書A9追補を仕様へ取り込む。非DeepSeek二族canary合格後のみ錨4pass。
- 次審: Rudra A9 commit · Lampas rev.5(N算定/夢gate/A9/旧数値) · Shiva相対化+live gate証跡。
