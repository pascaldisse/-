#!/usr/bin/env python3
# 批根丙 独立検証具 — 梯1入力log → z 独立再計算 → 梯2出力log と突合 (C12/C13).
# 法: 被審Rust code 不参照. 契約=文書/環統合.md §主座標 + 出力log header の param値のみを源とする.
# 用: python3 z再生審.py <入力log> <z再生log>

import math
import re
import sys


def 入力読(path):
    行 = []
    with open(path, encoding="utf-8") as f:
        for ln in f:
            if not ln.startswith("TICK "):
                continue
            ts = int(re.search(r"ts=(\d+)", ln).group(1))
            L = re.search(r"L\(x=(-?[\d.]+) y=(-?[\d.]+)", ln)
            行.append((ts, float(L.group(1)), float(L.group(2))))
    return 行


def z読(path):
    param = {}
    行 = []
    with open(path, encoding="utf-8") as f:
        for ln in f:
            if ln.startswith("# param"):
                for k, v in re.findall(r"([^\s{,]+): ([^,}]+)", ln.split("{", 1)[1]):
                    param[k.strip()] = v.strip()
            if not ln.startswith("Z "):
                continue
            d = dict(re.findall(r"(\w+)=(-?[\d.e+-]+)", ln))
            行.append(
                (
                    int(d["ts"]),
                    float(d["x"]),
                    float(d["y"]),
                    float(d["theta"]),
                    float(d["r"]),
                    int(d["lap"]),
                )
            )
    return param, 行


def 独立z(入力, 死域, 上限, 再正規化, 許容, 半回転規約):
    """契約からの独立再実装. 返= (theta, r, lap) 列."""
    出 = []
    前総 = None  # 環記憶 (lap計数用, 死域で解除)
    保持θ = 0.0  # 死域中θ保持用 (解除されない)
    lap = 0
    for _ts, x, y in 入力:
        mag = math.hypot(x, y)
        活性 = mag >= 死域 - 許容
        if not 活性:
            # 死域: r=0, θ保持, 環記憶解除
            出.append((保持θ, 0.0, lap))
            前総 = None
            continue
        θ = math.atan2(y, x)
        r = (mag - 死域) / (1.0 - 死域) if 再正規化 else mag
        r = min(max(r, 0.0), 上限)
        if 前総 is not None:
            d = θ - 前総
            if d > math.pi:
                lap -= 1
            elif d < -math.pi:
                lap += 1
            elif abs(abs(d) - math.pi) <= 1e-15:  # 半回転 tie
                lap += 半回転規約 if d > 0 else -半回転規約
        前総 = θ
        保持θ = θ
        出.append((θ, r, lap))
    return 出


def main():
    入p, zp = sys.argv[1], sys.argv[2]
    入 = 入力読(入p)
    param, z = z読(zp)
    print(f"# 入力TICK={len(入)} · z行={len(z)} · param={param}")
    if len(入) != len(z):
        print(f"!! 行数不一致 {len(入)} vs {len(z)}")
    死域 = float(param.get("死域", 0.08))
    上限 = float(param.get("r上限", 1.0))
    再 = param.get("死域再正規化", "true") == "true"
    許容 = float(param.get("境界許容", 1e-12))
    半 = int(param.get("半回転規約", 1))
    独 = 独立z(入, 死域, 上限, 再, 許容, 半)

    # 突合 (出力log は {:.6}/{:.4} 量子化 → 量子化後で比較)
    dθ最大 = dr最大 = 0.0
    lap不一致 = 0
    θ不一致 = r不一致 = 0
    for (ts, x, y, θz, rz, lz), (θi, ri, li) in zip(z, 独):
        dθ = abs(round(θi, 6) - θz)
        dr = abs(round(ri, 6) - rz)
        dθ最大 = max(dθ最大, dθ)
        dr最大 = max(dr最大, dr)
        if dθ > 1e-6:
            θ不一致 += 1
        if dr > 1e-6:
            r不一致 += 1
        if li != lz:
            lap不一致 += 1
    print(f"θ 最大差={dθ最大:.9g} 不一致行={θ不一致}")
    print(f"r 最大差={dr最大:.9g} 不一致行={r不一致}")
    print(f"lap 不一致行={lap不一致} (独立末lap={独[-1][2]} · 被審末lap={z[-1][5]})")

    # C12: 生値→log量子化の情報損 (log再生では原理的に回復不能な量)
    q = sum(1 for _ts, x, y in 入 if abs(x) > 0 and abs(x) < 1e-4)
    print(f"# 量子化下限未満で零化され得た標本(|x|<1e-4 かつ非零)={q}")


if __name__ == "__main__":
    main()
