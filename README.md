# 愛無 (Ai-Mu)

**愛無** is two things kept apart on purpose: (a) Pascal Disse's physical/computational cosmology, and (b) the engineering program built on it. The cosmology: one axiom, *love = 1*, with *0 = 1* read as a **gluing** — the quotient ℝ → ℝ/ℤ ≅ S¹ (x ↦ e^{2πix}) identifies 0 and 1, closing the line into a circle (π; e^{iπ}+1 = 0). Reality is a **場 (field)**, not particles; computation is a ripple in that field; **場語 (field language)** is the notation for writing ripples. The program: a deterministic, journaled, FFT-domain field engine (Rust, Metal, ARM64 asm) in which hypervector algebra, wave physics and rendering are *one* operation set — plus interface layers (gamepad, voice) that drive it through a single complex coordinate. The cosmology is a position, not a result: nothing here tests it. What is tested is the engine (§4). Every claim below is tagged **implemented** (code + test/proof run), **designed** (spec only), or **UNVERIFIED**.

> 日本語: 愛無 = 愛=1 を唯一の公理とする宇宙論、および場(field)基盤の工学。0=1 は「貼り合わせ」(ℝ/ℤ≅S¹)。場語 = 場の言語。以下、実装/設計/未検証を明記する。

## 1. Axiom → engine: where the math sits

| statement | math | in code |
|---|---|---|
| love = 1, sole hardcode | `LOVE = 1` is the only literal constant permitted; every other constant is a parameter with a default ("IRON law") | **implemented** (`機関/環制御/src/z.rs`; enforced by review, not by a tool) |
| 0 = 1 gluing | ℝ/ℤ ≅ S¹; angle θ is periodic, so θ and θ+2π name one point (`lap` counts the windings) | periodic angle + lap **implemented** in `Z`; the *cosmological* reading is **designed** (doctrine) |
| one interface coordinate | `z = r·e^{iθ} + lap`; total angle = θ + 2π·lap; r = 0 is the centre (無); rotation = multiplication | **implemented** (`機関/環制御/src/z.rs`, 110 tests); `文書/環統合.md` |
| hypervector = field slice | d = W·H; `bind` = 2D circular convolution = `ifft(fft(a)∘fft(b))`; `bundle` = Σ; `probe` = correlation | **implemented** (`packages/field/src/ops.rs`) |
| wave step = a bind | damped wave stencil in Fourier space: next = ifft(fft(cur)∘K₁) + a_prev·prev, K₁ = (2−dd) + c²·L̂, L̂ = 2cos(2πkₓ/W)+2cos(2πk_y/H)−4; CFL: c·dt/dx ≤ 1/√2 | **implemented**, parity vs direct stencil ≤ 1e-3 over 200 steps (32², 64²) |
| render = contraction | pixel = ⟨field, observation kernel⟩; no separate renderer module | **implemented** for the 2D plane; GPU 3D path **partial** (§4) |
| model lives *inside* the field | LLM/learner = one kernel among kernels, never the container (愛無律) | **designed**; small learner prototypes exist (§2, 機関/內在 etc.) |

What is **not** implemented relative to the original vision (audit: `VISION-ABGLEICH-2026-08-31.md`, line-referenced): no kanji→meaning-vector learning (current text→field map is a table-driven 64-channel code, `packages/field/data/c64.tsv`), atlas→field bridge (milestone M4) not started, 64-hexagram stick composition designed only, no fractal/multiscale meaning order, no proto-/child-language model.

## 2. Architecture (殿)

The repo is a *root temple* of directories. **Two kinds**: plain in-tree directories, and **21 gitlinks** (子殿) pinned to branch tips of the Magic Crystal repo (`github.com/pascaldisse/ai-mu`) — frozen lane snapshots, each a whole-repo checkout (~1.2k files) whose lane-specific diff is small. Clone with `git submodule update --init --recursive` to populate them (large). `移設記録.md` is the old-temple → new-path ledger.

| 殿 | purpose | language | entry | test | status |
|---|---|---|---|---|---|
| `機関/宇宙` | **main engine**: 3D field (`UniverseApp::Field3d`), species fields (湿/菌), text→field (`speak`/`tongue`), cosmo growth, Metal/wgpu render | Rust (+WGSL, ARM64 asm) | `universe` binary (`cargo build --release`) | `cargo test` in dir; gates in `tools/*.sh` | **implemented**; pinned gitlink `7195a1e` (local HEAD has later commits), 406 commits |
| `機関/環制御` | DualSense → `Z` (θ, r, lap) controller; haptic return | Rust | `src/main.rs` | `cargo test` → 110 pass | **implemented**; real pad detected 2026-08-11; ladder rungs 2–5 **designed** |
| `機関/歌口` | voice pitch/RMS → `Z` (singing as input) | Rust | `src/main.rs` | `cargo test` → 13 pass | **implemented** (file/journal path); live-mic proof in 宇宙 only (§4) |
| `機関/環音` | `Z` → tone synthesis | Rust | `src/main.rs` | `cargo test` → 62 pass | **implemented** |
| `機関/内在` `世界` `世界審` `零` `零審` | MuZero-style learner on the real field (reward from inside; world door `mu_serve`); `審` = adversary twins | Python/MLX + Rust bin | `mu/online.py`, `mu/train.py` | `mu/gate_*.py` | **experimental**; G3 PASS, per-tick latency gate **UNVERIFIED** (machine load) |
| `機関/小殿` | — | — | — | — | empty placeholder |
| `場/零 核 容量 決定 記録 性能 平面 世界 統合 針 文書` | the 2026-08-01 **field-V1 octet**: one lane per concern (core FFT/atoms, plane, journal, determinism gate, world, capacity, perf, docs, Metal spike), `統合` = merged result | Rust (+Python spike) | `cargo build -p field` | `cargo test -p field` (40/40 on the merged tree, per `場/文書` guide) | **implemented**; lane branches kept, not deleted (21 unmerged to Magic Crystal main) |
| `場語/本体` `組立` `審査` | field-language lanes: spec/reference, ARM64 assembler lane, audit | Rust + ARM64 asm | `fieldc in.fld out.fldj` | `FIELDC=… packages/fieldlang-gate/accept.sh` (PASS, 3 programs byte-exact vs oracle) | **implemented** (live copy: `機関/宇宙/packages/fieldlang-{asm,gate}`) |
| `文書/八連` | adversarial octet charter (`docs/research/2026-08-01-mu-octet.md`): 8 lanes with acceptance criteria | Markdown | — | — | spec; only some lanes ran |
| `文書/*.md` (in-tree) | design law: `環統合` `環制御` `歌語` `観測核` `錬金術` `z場注入数理裁定` … | Markdown | — | — | **designed** unless the doc says otherwise |
| `実験/準結晶格子` | Penrose P3 (thick/thin rhombus) electrode-lattice generator → KiCad footprint | Python 3 (stdlib; PNG via `rsvg-convert`/`cairosvg`/`qlmanage` if present) | `準結晶格子.py` | regen + `cmp` (§6) | **implemented** (last commit); hardware **not built** |
| `実験/第二配線` | second wiring of the v2 field (branch `v2-field-wire`) | Rust | — | — | not reviewed for this README |
| `地図` | **semantic map**: behavioural distance atlas across model heads (`距 序 類 軸` instruments, canary runs, adversary audits) | Python + Markdown | `run_rudra_ii_canary*.py` | — | **measurement** done at pilot scale; engine bridge (M4) **not started** |
| `docs/adversary`, `proof/{環制御,歌口,環音}` | adversary audits, run evidence | — | — | — | evidence |

## 3. 場語 (field language)

**Spec**: `機関/宇宙/packages/fieldlang-asm/CONTRACT.md` (grammar, token record, AAPCS64 ABI, journal target) and `場語/本体/docs/design/2026-08-01-field-lang.md`. Line-based; decimal u64 args; `#` comments; UTF-8. Seven glyphs compile 1:1 to journal-v1 op tags:

| glyph | op | tag | glyph | op | tag |
|---|---|---|---|---|---|
| 界 `w h n seed` | world header | — | 縛 `dst a b` | bind | 4 |
| 種 `slot seed` | seed atom | 1 | 束 `dst n s₁…sₙ` | bundle | 5 |
| 撃 `x y amp_bits` | excite | 2 | 寫 `slot n b₁…bₙ` | raw write | 6 |
| 歩 `count` | step | 3 | 觀 | probe/render (reserved V1, no tag) | — |

Minimal program (`機関/宇宙/packages/fieldlang-asm/examples/ex3.fld`):

```
種 0 111        # atom in slot 0 from seed 111
種 1 222
縛 2 0 1        # slot2 = bind(slot0, slot1)
束 3 3 0 1 2    # slot3 = bundle of 3 slots
歩 5            # 5 wave ticks
```

**Implemented & checked**: `fieldc` is a native arm64 compiler (hand-written lexer/emitter in assembly, no Rust/C in the compiler). On `ex3.fld` its 113-byte `.fldj` output is **byte-identical** to the Rust reference oracle (`flgate ref`), run for this README. Quirk found: `fieldc` creates its output file with mode 000 (needs `chmod`). **Designed, not built**: 4D form `界 w h d ch n seed` (W×H×D×64 channels) and `觀`.

## 4. 場 field engine — what is measured

Machine for all numbers: Apple M1 Pro. Single runs unless stated; dates differ (2026-08-01 V1 lanes, 2026-08-31 engine).

**Capacity** (`場/容量`; `docs/research/2026-08-01-field-capacity.md`): bundling K items into one d = 4096 slice, 256 fresh distractors per K:

| K | 1 | 4 | 16 | 64 | 256 |
|---|---|---|---|---|---|
| member similarity | 1.000 | 0.497 | 0.253 | 0.127 | 0.061 |
| retrieval accuracy | 100% | 100% | 100% | 100% | 82% |

Fit: member similarity ≈ 1.004/√K; distractor noise floor flat ≈ 0.8/√d; cliff K\* ≈ d/(2 ln N). Consequence (derived): one bundle degrades as 1/√K, so the store is an N×d RAM-resident matrix, not bigger bundles. Rule K ≤ d/22 per slot; the d = 16384 figure is **extrapolated, not measured**.

**Core / determinism** (`場/決定`): law = state is a function of the seed; no `rand`, no clock in state, f32 persisted as bit patterns, FNV-1a digests, SplitMix64. Five gates (two independent live runs bit-identical; journal replay digest = live digest; PNG bytes identical; digest differs when seed differs; atom digest stable) — re-run for this README: `determinism` 5/5, `speak` 7/7, `c64_semantics` 3/3. Replay proof: two digests `e2f94fc4fc0cda12` = MATCH (`機関/宇宙/proof/field3d-replay-match.log`). Digest is thread-count invariant at 64³ (1/4/8 threads, `proof/64cube/SUMMARY.md`).

**Record** (`場/記録`): journal v1 `FLDJ`, 48-byte header, six ops, f32 as `u32` bits; `append` buffers in RAM, `flush` is the only disk write and never runs in the tick path; replay is bit-exact (`store_journal` 7/7).

**Performance** (target: 120 fps → 8.33 ms/frame). Honest ledger, including failures:

| what | result | verdict |
|---|---|---|
| V1 CPU crate, 128², N = 4096 slots, radix-2 FFT | 88.67 ms mean / 124.05 p99 | **FAIL** (~10×) |
| same, N = 256 | 5.92 / 7.56 ms | PASS |
| MLX spike, probe N = 4096, d = 16384 | 2.75 ms; N = 16384 → 11.41 ms | PASS ≤ 4096 only; spike not wired into the crate |
| integer-CPU engine, 64³, 5000 frames | p99 3.45–5.09 ms (1/4/8 threads) | PASS; single run per config |
| Metal instanced render, 262 144 points, 600 frames | fps p50/p95/p99 = 120.1 / 118.1 / 116.2 | **FAIL on p99** (p50 ≈ ProMotion vsync; hypothesis: p99 ≥ 120 unreachable under vsync — untested) |
| CPU ↔ Metal parity, 48 ticks | shell L1 44.54138565 both, Δ = 0 | PASS |
| real-mic path (440 Hz in a room) | tone frames 471; L1 69.06 vs 16.32 quiet | PASS (one run) |

Proof dirs: `機関/宇宙/proof/{64cube,terra-render-20260831,terra-integration-20260831,terra-polish-20260831,field3d-*}`. Known open: sub-120 p99, 3D "every node visible" claim, repeat-run statistics.

## 5. Relations

**Magic Crystal** (separate repo, the older engine this grew out of) is **not** vendored and never moved. Point to it with the env var `魔晶殿` (absolute path); the 21 子殿 gitlinks resolve to its canonical remote (`github.com/pascaldisse/ai-mu`).

```sh
export 魔晶殿=/path/to/magic-crystal
cd "$魔晶殿" && git worktree list
git submodule update --init --recursive   # in this repo
```

**GAIA** (separate: the multi-agent room/daemon system) is not in this repo. The tie is the working language: this is a **鳴語-only temple** — docs, comments, public identifiers are written in 鳴語 (gaiago, dense kanji notation) wherever host plumbing allows (`AGENTS.md`, `HANDOFF.md`); agents (Nyari and summoned workers) author and audit here, which is why the history is lane-shaped (stage commits, adversary twins, UNVERIFIED flags). The 場語 glyph set above is the same family. Work is by Pascal's ruling only; "it works" requires a real run, not a claim (`HANDOFF.md` 検証法).

## 6. Run & proof

```sh
# engine gates (verified for this README: 15 pass)
cd 機関/宇宙 && cargo test -p field --test determinism --test speak --test c64_semantics
cd 機関/宇宙 && cargo test -p cosmo --test channel_growth      # 7 pass per VISION-ABGLEICH, not re-run here
# interface layers (verified: 110 / 13 / 62 pass)
for d in 環制御 歌口 環音; do (cd 機関/$d && cargo test); done
# field language: compile and compare against the Rust oracle
cd 機関/宇宙 && cargo build -p fieldlang-gate && FIELDC=$PWD/packages/fieldlang-asm/fieldc sh packages/fieldlang-gate/accept.sh   # ACCEPT: PASS (verified)
# Penrose lattice: regenerates byte-identical 頂点.json (verified)
python3 実験/準結晶格子/準結晶格子.py --出力 /tmp/lattice
cmp /tmp/lattice/頂点.json 実験/準結晶格子/頂点.json
```

Requires Rust stable; macOS arm64 for Metal and `fieldc`. Penrose output (defaults r = 40 mm, depth 5): 406 vertices, 770 edges, 220 thick + 145 thin rhombi, min pitch 2.34 mm, median 3.79 mm, one SMD pad per vertex. It is a *geometry generator* for a quasi-crystalline electrode array; no board exists.

Evidence: `機関/宇宙/proof/`, `proof/{環制御,歌口,環音}/`, `docs/adversary/` (dated audits), `地図/審-*.md`, `VISION-ABGLEICH-2026-08-31.md` (implementation vs original vision, honest gap list). Working rules: `AGENTS.md`; handoff: `HANDOFF.md`; backlog ("godseed"): `todo.md`.

## 7. 菌世紀 (Fungal century)

A stated direction (`todo.md`: "don't build, grow"): fungal/mycelial computing and building. **Implemented** so far is only the simulation substrate — a moisture field and a hyphal growth field on the 64³ lattice (`機関/宇宙` species tests, `proof/菌/`); physical mycelium hardware is **not built** (bio-electrode rung 1 awaiting purchase). "ARMILLARIA" is not present in this repo.
