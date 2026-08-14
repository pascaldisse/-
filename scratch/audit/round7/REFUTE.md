# round7 審 — Kali 独立反証 (対象=Vishnu round6 数値法)

源=機関/宇宙/packages/field/src/qplane.rs (読取のみ)。自読·再確定。

## (1) 丸め順序·飽和位置 再確定 (path:line)

| 主張 | 自読 | 判 |
|---|---|---|
| 定1 丸め=係数毎独立 `(c*v+2^29)>>30`、i64加算、融合丸め非 | `qplane.rs:74-79` mulc = `(c*x as i64 + (1<<29)) >> 30`。`qplane.rs:163-165` acc = mulc(c_cur,c0) + `((c_lap*lap + (1<<29))>>30)` + mulc(c_prev,prev[i]) | 一致 |
| 定2 lap=真正i64·中間飽和無 | `qplane.rs:156-161` 全項 `as i64`、-4*c0 も i64。sat呼出無 | 一致 |
| 定3 飽和=acc一回·上下計数 | `qplane.rs:166` sat32_counted 一回のみ。`qplane.rs:57-67` 上下両分岐で +1 | 一致 |
| 定5 w/h=1 自己重複、1x1→lap=0 | `qplane.rs:150-155` xm/xp/ym/yp wrap → w=1で xm=xp=x。1x1: lap=4*c0-4*c0=0 | 一致 |

補記(Vishnu未指摘): lap項は mulc を通さず **inline複製** (`qplane.rs:164`)。式は同一だが二重実装 = 将来乖離risk。F7とする。

## (2) 合算後 overflow 域 — 自力境界

真lap境界(w,h>=3, 交互極値で到達可能):
- lap_max = 4*(2^31-1) - 4*(-2^31) = **2^34-4 = 17179869180**、lap_min = -(2^34-4)。

段①(shift前, 唯一の危険点) `c_lap*lap + 2^29` が i64 必須:
- 条件 c_lap*(2^34-4)+2^29 <= 2^63-1 → **|c_lap| <= 2^29 = 536870912** (厳密。c_lap=2^29 で余裕 1610612735、2^29+1 で溢れ)。
- k=courant^2 として **k <= 0.5**。

段②(shift後 合算):
- |mulc項| <= 2^33 (c=±2^31, x=∓2^31 → 2^32… 実測上限 4294967296=2^32)。
- |lap項| <= 8589934574 ≈ 2^33。
- |acc| <= 1.7e10 ≈ 2^34 << 2^63 → **合算後 溢れ無**。∴ 危険は単項 c_lap*lap のみ。

判: **F5 は方向正 · 境界一 off-by-one**。Vishnu「|c_lap|<=2^29-1」→ 真は **|c_lap|<=2^29**。保守側なので実害無だが契約文言は 2^29 が正。
判: **F4 真** — 註釈 `|lap|<=6*2^31`(=12884901888) は真値 2^34-4 より **過小**。契約誤記確定 (`qplane.rs:162`)。
死枝: 「Kali単項2^62論」= 誤。2^62 は mulc 側の上限であり c_lap*lap 側(2^63級)を捉えぬ。因=最大の被乗数が胞(2^31)でなくlap(2^34)。

## (3) vectors 凍結digest — 独立再生

```
$ python3 oracle_copy.py --emit regen.bin --expect-digest 1973cf60...
vectors=regen.bin cases=133 sha256=1973cf60a0cf4b20e2117d64af66ed99a79a3751646d8f3f51837e33648e2397
$ shasum -a 256 regen.bin
1973cf60a0cf4b20e2117d64af66ed99a79a3751646d8f3f51837e33648e2397  regen.bin
```
別worktree·別path で再生一致。**digest 確認·不一致無**。
留保: 再生は同一 oracle scriptの再実行(決定性証明)。第二独立実装による相互検証は未 → UNVERIFIED。
副検: vectors 使用の最大 c_lap = 2^29-1 → 段①上限内。**vectors自体は i64 溢れを踏まぬ**(安全)。

## (4) F6 具体溢れ入力例

`c_cur = coef(2.0 - dd)`, dd = damping*dt, COEF_FRAC=30。
- **dd = 0 (damping=0 或 dt=0) → c_cur = 2147483648 = 2^31**。i32::MAX=2147483647 → **+1 で溢れ**。
- 一般閾: round((2-dd)*2^30) >= 2^31 ⟺ **dd <= 2^-31 = 4.6566e-10**。
- dd = 2^-30 = 9.3132e-10 → 2147483647 (境界内, 溢れ無)。
- 他係数: c_ret(dd=0)=2^30、c_prev=-(1-dd)*2^30 → i32内。**溢れるのは c_cur のみ**。
∴ 係数を i32 で持つ側(cfg/外部契約)は damping=0 の既定値で即破綻。Rust側 i64 は正。GPU側 `qgpu.rs:24 parts()` は i64→(u32,i32) 分割で正しく渡す ∴ GPU path は無傷。

## 総括
- 定1·定2·定3·定5·F4·F6 = **反証失敗 → 支持**。
- F5 = 境界 off-by-one 訂正 (2^29-1 → 2^29)。
- 新: F7 lap項の mulc 非利用(inline複製)。

## UNVERIFIED
- Rust実行(cargo test)未 — 静的読取のみ。
- 第二独立oracle実装による vectors 相互検証 未。
