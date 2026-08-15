#!/usr/bin/env python3
"""atom6 独立 Q30 波 oracle + 敵対 vectors。製品碼(Rust/C/Swift/asm)再利用零。
法出典= scratch/audit/atom6/LAW.md (qplane.rs 引用)。"""
from __future__ import annotations
import argparse, hashlib, struct, sys
from pathlib import Path

I32_MIN, I32_MAX = -(1 << 31), (1 << 31) - 1
HALF, FRAC = 1 << 29, 30
M32 = (1 << 32) - 1


def sar(v: int, n: int) -> int:  # 算術右shift = floor除
    return v >> n


def q30(c: int, x: int) -> int:  # 係数毎 丸め (LAW 定1)
    return sar(c * x + HALF, FRAC)


def wave(w: int, h: int, c_cur: int, c_lap: int, c_prev: int,
         cur: list[int], prev: list[int]) -> tuple[list[int], int]:
    out = [0] * (w * h)
    sat = 0
    for y in range(h):
        ym = h - 1 if y == 0 else y - 1
        yp = 0 if y == h - 1 else y + 1
        for x in range(w):
            xm = w - 1 if x == 0 else x - 1
            xp = 0 if x == w - 1 else x + 1
            i = y * w + x
            c0 = cur[i]
            lap = cur[y * w + xm] + cur[y * w + xp] + cur[ym * w + x] + cur[yp * w + x] - 4 * c0
            acc = q30(c_cur, c0) + q30(c_lap, lap) + q30(c_prev, prev[i])
            if acc > I32_MAX:
                out[i], sat = I32_MAX, sat + 1
            elif acc < I32_MIN:
                out[i], sat = I32_MIN, sat + 1
            else:
                out[i] = acc
    return out, sat


def lcg(seed: int):
    while True:
        seed = (1664525 * seed + 1013904223) & M32
        yield seed - (1 << 32) if seed & (1 << 31) else seed


EXTREMA = [0, 1, -1, 2, -2, 3, -3, HALF, -HALF, 1 << 30, -(1 << 30),
           I32_MAX, I32_MIN, I32_MAX - 1, I32_MIN + 1, (1 << 30) + 1]
# |c_lap| < 2^29 が i64 安全域 (LAW 域節)。境界直下まで。
COEFS = [(1 << 30, 1 << 28, -(1 << 30)),          # 標準級 k=0.25
         (2 << 30 if False else (1 << 31) - 1, (1 << 29) - 1, -(1 << 30)),  # c_lap 上限直下
         (0, 0, 0),                                # 全零
         (1 << 30, -((1 << 29) - 1), 1 << 30),     # 負 c_lap
         (-(1 << 31), (1 << 29) - 1, (1 << 31) - 1),  # 係数極値
         (1, -1, 1),                               # 微小: 半値丸め露出
         (I32_MAX, 1, I32_MIN)]


def cases():
    out = []
    # ① 退化寸法: 自己重複 lap (w=1 / h=1 / 1x1)
    for (w, h) in [(1, 1), (1, 2), (2, 1), (1, 5), (5, 1), (1, 8), (8, 1), (2, 2), (3, 3), (4, 4), (5, 4), (4, 5), (7, 3), (3, 7)]:
        n = w * h
        for ci, co in enumerate(COEFS):
            cur = [EXTREMA[(i * 7 + ci) % len(EXTREMA)] for i in range(n)]
            prev = [EXTREMA[(i * 11 + ci * 3) % len(EXTREMA)] for i in range(n)]
            out.append((f"dim-{w}x{h}-c{ci}", w, h, co, cur, prev))
    # ② 行跨ぎ/tail: width が 4 の倍数でない場合 lane tail
    for w in range(1, 13):
        h = 3
        n = w * h
        co = COEFS[0]
        cur = [I32_MAX if (i % 2) else I32_MIN for i in range(n)]   # 最悪交互 → |lap| 最大
        prev = [I32_MIN if (i % 3) else I32_MAX for i in range(n)]
        out.append((f"tail-w{w}", w, h, co, cur, prev))
    # ③ 飽和域: acc が両側に飛ぶ
    for ci, co in enumerate(COEFS):
        w = h = 6
        n = w * h
        cur = [I32_MAX] * n
        prev = [I32_MAX] * n
        out.append((f"sat-hi-c{ci}", w, h, co, cur, prev))
        out.append((f"sat-lo-c{ci}", w, h, co, [I32_MIN] * n, [I32_MIN] * n))
    # ④ 半値丸め: acc 端 ±0.5 ulp 近傍 (c=2^30 → 積の丸めが恰度半値)
    n = 16
    cur = [(1 << 29) + k for k in range(-4, 12)]
    out.append(("halfway", 4, 4, (1 << 30, 1, 1 << 30), cur, [(-(1 << 29)) + k for k in range(-4, 12)]))
    out.append(("halfway-neg", 4, 4, (-(1 << 30), -1, -(1 << 30)), [-v for v in cur], cur))
    # ⑤ 非対称大盤 + 乱数
    rng = lcg(0xA5A5F00D)
    for (w, h) in [(17, 5), (5, 17), (13, 13), (64, 3), (3, 64), (31, 9)]:
        n = w * h
        out.append((f"rand-{w}x{h}", w, h, COEFS[0], [next(rng) for _ in range(n)], [next(rng) for _ in range(n)]))
    # ⑥ 大盤 digest 用
    w, h = 128, 97
    n = w * h
    out.append(("rand-128x97", w, h, COEFS[1], [next(rng) for _ in range(n)], [next(rng) for _ in range(n)]))
    return out


MAGIC = b"Q30WAVE1\0"


def emit(path: Path) -> str:
    hsh = hashlib.sha256()
    with path.open("wb") as f:
        def put(b: bytes):
            f.write(b); hsh.update(b)
        put(MAGIC)
        for name, w, h, (cc, cl, cp), cur, prev in cases():
            exp, sat = wave(w, h, cc, cl, cp, cur, prev)
            nm = name.encode("ascii")
            put(struct.pack("<H", len(nm)) + nm)
            put(struct.pack("<iiiii", w, h, cc, cl, cp))
            put(struct.pack(f"<{len(cur)}i", *cur))
            put(struct.pack(f"<{len(prev)}i", *prev))
            put(struct.pack(f"<{len(exp)}i", *exp))
            put(struct.pack("<Q", sat))
        put(struct.pack("<H", 0))
    return hsh.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--emit", type=Path, required=True)
    ap.add_argument("--expect-digest")
    a = ap.parse_args()
    d = emit(a.emit)
    print(f"vectors={a.emit} cases={len(cases())} sha256={d}")
    if a.expect_digest and a.expect_digest != d:
        print(f"DIGEST MISMATCH expected={a.expect_digest}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
