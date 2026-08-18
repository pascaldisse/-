# Yama·IRON/A8·Q誓審

⚓対象=`naru/yama-r2`。審時刻=2026-08-05。主樹未觸。

## 取込対象

- `naru/yama-l2-watch@133649c468ae7872dc7657e22fe01ed383a05b1c`
- 前審=`kali/q-audit-0804@f0c62bf9`、`docs/adversary/2026-08-04-kali-q-oath.md`
- 再審=`kali/q-audit-2@1d27fce3`、`docs/adversary/2026-08-05-kali-q-oath-2.md`

## Yama枝·主張vs証跡

|主張|判|証跡/根|
|---|---|---|
|係数溢出経路発見|PASS|`yama-coefs-overflow.log` debug=`qplane.rs:130` panic、release同入力=`[2147483647,-2147483648,-2147483648,2147483647]`。問題検知は真。|
|90m watch完走|FAIL|`yama-watch-90m.log` は08:52:21→08:58:21、SCAN 3回、`END`無し。90分非証明。|
|watch対象枝観測|PASS|base包含として`naru/int-core@556304ec`→`kali/q-audit-2@1d27fce3`を観測。|
|Q replay再現|UNVERIFIED|`actual-v2.rows`=`expected-v2.rows`は同blob（独立期待値非）。log/scriptは外部`mc-ua-q-study2`依存、当枝だけで再現不能。|
|失敗注入gate検出|FAIL|`fake-universe-fail.sh` はFAILを出すが、`tools/replay-gate.sh` はdigest同値のみ判定。`replay-fail-inject.log`最終PASSは偽陽性、FAIL字句検証非。|
|overflow修正|UNVERIFIED|133649c4は証跡のみ、本体修正/overflow gate追加無し。release挙動安全性未証。|

## IRON/A8所在・接続

|面|所在|判|
|---|---|---|
|IRON Q入力|`packages/universe/src/lib.rs`: `DEFAULT_MAX_STRIKE_AMP=64.0`, `DEFAULT_MAX_SOURCES_PER_TICK=256`, `strike`契約|PASS·実門で確認|
|IRON excite shape|`packages/field/src/qplane.rs`: `ExciteShape{sigma,r}`、既定`{2.0,8}`|PASS·実門で確認|
|A8 keyframe/replay|`packages/universe/src/lib.rs` A8 gates、`packages/universe/src/main.rs` A8 knobs|PASS·既存門で確認|
|Yama枝との変更接続|133649c4変更は証跡18檔のみ、`packages/universe/**`変更無し|UNVERIFIED·本枝は保証変更非|

`rg -n -i '\bIRON\b|\bA8\b|DEFAULT_MAX_STRIKE_AMP|DEFAULT_MAX_SOURCES_PER_TICK|ExciteShape|extreme_cell_count'` にて上記所在を特定。文字列所在≠保証成立と扱う。

## 前審五FAIL·独立再判

|面|前判|再判|独立根/実走|
|---|---|---|---|
|Q20選定根拠|FAIL|PASS|`q-study` test=`q20_3d_real_order_gate` 1/1。v2実場順序、特性式`A=1.8905`,`B=-0.893475`をDECISIONと照合。|
|10k+長走|FAIL|PASS（証跡再確認）|`q-study2-v2-rerun.log`に10k/100k・Q20/Q24全4行、`linf=4.547834421e-5→4.627309096e-5`。独立release再走は未実行。再走未実行=UNVERIFIED（ログ主張自体はPASS）。|
|3D/z/whole-volume|UNVERIFIED|PASS|`q20_3d_real_order_gate` 1/1。64×64×16、z周期、F64/Q比較の門。|
|最大振幅/連打飽和|FAIL|PASS|`cargo test -p universe --lib`: `input_contract_prevents_saturation_and_bounds_journal`を含む51/51。契約後`extreme_cell_count()==0`。|
|IRON shape parameter|FAIL|PASS|`cargo test -p field --lib`: `excite_shape_is_iron_param_with_legacy_default`を含む14 passed/0 failed（1 ignored）。|

## 独立gate貼

実走殿=`/Users/pascaldisse/projects/mc-ua-int-core`、HEAD=`691b2fc3`（対象commitを含む履歴）。

```text
cargo test -p field --lib
14 passed; 0 failed; 1 ignored

cargo test -p field --bin q-study
1 passed; 0 failed

cargo test -p universe --lib
51 passed; 0 failed
```

未走=field GPU長走 ignored（`qgpu_long`）、q-study release CSV再走、90m watch完走、overflow修正後安全性。`q_step_replay_bit_exact`は同実装二走の残留故、独立oracle代替（手計算vector+RED）が必要。前審五FAILは修正内容/門についてPASS、主張全体は上記UNVERIFIEDを保持。
