# 審 Bhairava v1 — Durga再現敵対審

日=2026-08-05 · 継承=Bhairava(L3陰·審) · 対象=Durga再現-v1 · 基準=審-kali-v1 + battery-v1 + 地図README

## 材料

- `再現-v1.md` HEAD=09f88b2(含 腕A/B)
- 法記載=独立清文脈lane·1lane1run·N=5/頭·先行値非提示
- 腕A=Q2/Q11/Q12·規約明示
- 腕B=Q11/Q12·規約無明示
- 頭= kimi·sonnet·flash(glm代替)

## 違反審

| 法点 | 測量 | 判定 |
|---|---|---|
| 1召内多run混入 | 文書は1lane1runを明記。召境界·lane ID·prompt hash·run log無 | **争域**。明文遵守、監査証跡欠。確定違反に非 |
| 任務文への先行値漏洩 | 「先行値非提示(錨禁)」と明記。実prompt/送信履歴無 | **争域**。自己申告のみ、無漏洩は未証 |
| N<5 | 成功頭は各問5値。flashもN=5。glmは5lane全滅(403) | **確定欠測**(glm)。ただし429/403時の非Claude代替規約によりflash代替は**法違反に非**。glm結果N=0は死枝 |
| 分散未報告 | 腕A/Bとも中央値·範囲·SD有 | **違反無・確定合格** |
| 頭取違え | flashを`glm代替`と明示。等価性は未証。kimi名は明示 | **違反無、器欠陥域**。flash≠glmの保証無 |
| 目盛規約 | 腕A=Q2/Q11/Q12明示、腕B=無明示を対照化 | 腕A合格。腕Bは意図的交絡対照、測器欠陥の証拠 |
| 独立性 | 1lane1runを記載。実行ID·時刻列·prompt hash無 | **争域**。再現主張の監査可能性不足 |

## Durga値↔battery-v1突合

| 枡·頭 | v1中央値/幅 | Durga腕A | 突合 |
|---|---:|---:|---|
| Q2·sonnet | 3(A)/7(B) | 7[6–7] | B再現、A非再現·準 |
| Q2·kimi | 4[3–6] | 8[7–10] | 不一致(+4) |
| Q11·sonnet | 3–4 | 8[7–8] | 不一致·極性反転 |
| Q11·kimi | 3 | 8[8–10] | 不一致·極性反転 |
| Q12·sonnet | 6 | 6[6–8] | 一致 |
| Q12·kimi | 4[3–6] | 8[8–10] | 不一致(+4) |
| Q2·flash | 無 | 2[1–4] | 比較不能 |

## 腕B交絡突合

- Q11: 腕A=8、腕B=5(sonnet/kimi/flash)。全6枡のA>B(+3〜+4) → **目盛規約効果=假**。因=規約×時期×頭が同時変更、対照腕無。
- Q12: sonnet A=6/B=3、kimi A=8/B=5、flash A=8/B=4 → 差は観測、規約主効果は**假**。Durga腕B結果を得ても時期/頭交絡は残る。
- 腕B↔v1: Q11 sonnet/kimi=準、Q12 sonnet=不一致·kimi=一致 → 時期/lane残余交絡有。
- Q2: 規約明示後も kimi=8·sonnet=7·flash=2 → 頭間裂持続。**争域**、実裂仮説強化。ただしflash代替ゆえ頭間比較は器欠陥域。

## 独立第二経路

定義=Durga算路外の別頭(glm又はkimi)·同3問·独立lane。

- 実測=**UNVERIFIED**。本L3は召権零、別lane起動不能。glm死枝は403で全滅し、独立成功値無。
- Durga内kimi値は同一成果物由来、第二経路として再利用禁止。
- 従ってDurga値との独立突合=未成立。確定判定へ昇格不可。

## 判定

- **確定**: N=5・分散報告(成功頭)／glm死枝=403／Durga成功laneのroom ID等証跡欠。
- **假**: 規約有無の系統効果(Q11/Q12)。因=規約×時期×頭同時変更、対照腕無。
- **争域**: 1召内多run、先行値漏洩、lane独立性。問文·ID欠に因る。Q2の実裂。
- **器欠陥**: 規約欠落、flash=glm等価未証、独立第二経路欠、成功頭不足(指定glm)。
- Durga再現全体=**部分成立**。独立検証不成立ゆえ総合結論=**UNVERIFIED**。

## 証跡法審

- 証跡規約=`地図/証跡規約.md`。必須=room ID·頭名·問文原文·生答·規約·時刻·遮断証拠。
- `再現-v1.md`成功75観測行: room ID·問文原文·生答対応表なし → **証跡違反**。N=5列のみではlane独立性監査不能。
- 腕A/Bの規約実体·先行値遮断もprompt原文欠により未驗。

## 独立第二経路(第二試)

- 条件=Claude/glm非依存·Q11/Q12·規約明示·N≥5独立lane。
- 成功値=**UNVERIFIED**。本L3は召権零、kimi自清文脈lane起動不可。whale-k2起動記録無。
- 候補flashは指定禁(かつDurga代替済)、再利用不可。
- 死枝候補=独立lane未起動。403本文/コマンド証跡=不存在、推測禁。これはglm死枝とは別枝。

## 死枝

- `naru-glm`×5 lane: 403 status code·body無·出力零。因=provider拒否。room IDs=msfyirbgcvbzgo / msfyivmc1umwav / msfyiw1vakhmm8 / msfy iwm27o07uq / msfyixjt5ggm4q。第四IDは原記載に空白有、実ID=UNVERIFIED。
- opus·jareth再測=対象外/未実施。値補完禁止。

## 未驗

- prompt実体·送信履歴·lane IDによる独立性/漏洩監査
- glm↔flash分布等価
- 別頭による同3問N=5独立第二経路
- Q2裂が実裂か時期/頭/器差か

SHA=作成後付与
