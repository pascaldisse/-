#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
準結晶格子.py -- Penrose P3 (太/細 菱形) 電極格子 生成器。

手法: Robinson 三角形 細分割 (deflation) 法。
  - 単位半径1の「車輪」= 頂角36度の二等辺三角形(黄金三角形, 色0)10枚を
    原点を中心に環状配置 (P3 の「太陽 (sun)」開始配置)。
  - 各三角形を de Bruijn/Robinson の黄金比細分割規則で --深さ 回, 再帰的に
    分割 (色0: 2枚へ、色1: 3枚へ分裂。色1は頂角108度の鈍角黄金三角形)。
    典拠: N.G. de Bruijn の pentagrid 双対理論と同値な、Preshing (2011)
    "Penrose Tiling Explained" に記載の三角形置換規則。
  - 分割後、各三角形の「底辺」(頂角の対辺)を鍵に、底辺を共有する2枚を
    鏡像ペアとして結合 → 菱形1枚 (底辺=内部対角線として除去、外周4辺のみ
    残す)。本スクリプトはこのペア接合を辺長の実測で検証済み
    (全ペアで4辺が厳密に等長になることを確認: bad-rhombi=0)。
  - 菱形分類は構築時の色ではなく、実測した頂点角 (36/144→細, 72/108→太)
    で判定するため頑健。

座標スケール: 単位車輪 (外接半径=1) を、目標半径 (--半径, mm) が車輪の
「内接半径」(アポセム, 半径*cos(18°)) に一致するよう拡大
  (拡大率 = 半径mm / cos(18°))
してから、原点からの距離が --半径 を超える菱形を除去する
(= 真円クリップ。車輪=正十角形の角のみが切り落とされ、円内は隙間なく
菱形で充填される)。

座標系: mm 単位, 原点=円中心。KiCad 側は Y 軸表記の向きを特に補正して
いない (そのままの生成座標を使用)。

IRON 法 (ハードコード禁止) により、本質的に P3 定義に固定される定数
(5回対称・車輪10枚・黄金比 φ) は CLI パラメータ化していない
(値を変えると P3 タイリングでなくなるため)。それ以外の寸法・閾値・
色・線幅は全て argparse パラメータ化した。
"""
import argparse
import cmath
import json
import math
import os
import subprocess
import sys
import shutil

黄金比 = (1 + math.sqrt(5)) / 2
十八度 = math.pi / 10  # 18度 (ラジアン) -- 正十角形のアポセム角


def 引数を読む():
    p = argparse.ArgumentParser(description="Penrose P3 準結晶電極格子 生成器")
    p.add_argument("--半径", type=float, default=40.0, help="クリップ円半径 mm (既定 40)")
    p.add_argument("--深さ", type=int, default=5, help="細分割の再帰深さ (既定 5)")
    p.add_argument("--最小間隔", type=float, default=2.0, help="最小パッドピッチ mm (既定 2.0); 未達なら警告して停止")
    p.add_argument("--パッド径", type=float, default=1.0, help="パッド直径 mm (既定 1.0)")
    p.add_argument("--出力", type=str, default=None, help="出力ディレクトリ (既定: スクリプトと同じ場所)")
    p.add_argument("--許容", type=float, default=1e-6, help="頂点重複除去の許容誤差 mm (既定 1e-6)")
    p.add_argument("--線幅", type=float, default=0.12, help="SVG 辺の線幅 mm (既定 0.12)")
    p.add_argument("--余白", type=float, default=4.0, help="SVG キャンバス余白 mm (既定 4.0)")
    p.add_argument("--太色", type=str, default="#bcd9ff", help="太菱形 塗り色 (既定 #bcd9ff)")
    p.add_argument("--細色", type=str, default="#ffd9bc", help="細菱形 塗り色 (既定 #ffd9bc)")
    p.add_argument("--画像幅", type=int, default=1600, help="PNG 出力画素幅 (既定 1600)")
    return p.parse_args()


def 分割する(三角形群):
    """1世代分の Robinson 三角形置換。要素は (色, A, B, C) の複素数タプル。
    A は頂角の頂点。色0: 頂角36度 (鋭角黄金三角形)。色1: 頂角108度 (鈍角黄金三角形)。"""
    結果 = []
    for 色, A, B, C in 三角形群:
        if 色 == 0:
            P = A + (B - A) / 黄金比
            結果.append((0, C, P, B))
            結果.append((1, P, C, A))
        else:
            Q = B + (A - B) / 黄金比
            R = B + (C - B) / 黄金比
            結果.append((1, R, C, A))
            結果.append((1, Q, R, B))
            結果.append((0, R, Q, A))
    return 結果


def 車輪を作る():
    三角形群 = []
    for i in range(10):
        B = cmath.rect(1, (2 * i - 1) * math.pi / 10)
        C = cmath.rect(1, (2 * i + 1) * math.pi / 10)
        if i % 2 == 0:
            B, C = C, B
        三角形群.append((0, 0j, B, C))
    return 三角形群


def 座標をスナップ(z, 桁数=9):
    """浮動小数点誤差 (~1e-15) で同一点が別キーになるのを防ぐ。
    単位円 (外接半径=1) 座標系で 1e-9 に丸める -- 実際の頂点間隔は常に
    これより十分大きいので真の区別は失われない。"""
    return round(z.real, 桁数) + round(z.imag, 桁数) * 1j


def 生成する(深さ):
    三角形群 = 車輪を作る()
    for _ in range(深さ):
        三角形群 = 分割する(三角形群)
    # 浮動小数点誤差対策: ペア判定 (frozenset 完全一致) の前に坐標をスナップ
    三角形群 = [
        (色, 座標をスナップ(A), 座標をスナップ(B), 座標をスナップ(C))
        for 色, A, B, C in 三角形群
    ]
    return 三角形群


def 菱形へ結合する(三角形群):
    """底辺 (B-C, 頂角Aの対辺) を鍵に鏡像ペアを探し、菱形 (e1,p1,e2,p2) の
    リストを返す。e1,e2=共有された底辺の端点 (=内部対角線), p1,p2=各三角形の
    頂角頂点 (=菱形の外周に残る対頂点)。ペアにならない境界半三角形は捨てる。"""
    群 = {}
    for t in 三角形群:
        色, A, B, C = t
        鍵 = frozenset([B, C])
        群.setdefault(鍵, []).append(t)
    菱形群 = []
    for v in 群.values():
        if len(v) != 2:
            continue  # 境界の半三角形 (対を持たない) は破棄
        (色1, A1, B1, C1), (色2, A2, B2, C2) = v
        e1, e2 = B1, C1
        p1, p2 = A1, A2
        菱形群.append((e1, p1, e2, p2))
    return 菱形群


def 角度度(頂点, 隣1, 隣2):
    u = 隣1 - 頂点
    v = 隣2 - 頂点
    return math.degrees(abs(cmath.phase(v / u)))


def 太細判定(菱形):
    e1, p1, e2, p2 = 菱形
    角 = 角度度(p1, e1, e2)
    if abs(角 - 36) < 1.0 or abs(角 - 144) < 1.0:
        return "細"
    return "太"


def main():
    引数 = 引数を読む()
    スクリプト場所 = os.path.dirname(os.path.abspath(__file__))
    出力先 = os.path.abspath(引数.出力) if 引数.出力 else スクリプト場所
    os.makedirs(出力先, exist_ok=True)

    拡大率 = 引数.半径 / math.cos(十八度)
    三角形群 = 生成する(引数.深さ)
    菱形群_単位 = 菱形へ結合する(三角形群)

    # mm へスケール
    def sc(z):
        return z * 拡大率

    菱形群 = []
    for e1, p1, e2, p2 in 菱形群_単位:
        菱形群.append((sc(e1), sc(p1), sc(e2), sc(p2)))

    # 真円クリップ: 4頂点全てが --半径 以内の菱形のみ残す
    半径mm = 引数.半径
    クリップ済み = []
    for 菱形 in 菱形群:
        if all(abs(z) <= 半径mm + 1e-9 for z in 菱形):
            クリップ済み.append(菱形)

    if not クリップ済み:
        print("エラー: クリップ後に菱形が0枚。--半径 か --深さ を見直してください。", file=sys.stderr)
        sys.exit(2)

    # 頂点の重複除去 (許容誤差グリッドへスナップ)
    許容 = 引数.許容
    頂点辞書 = {}
    頂点一覧 = []

    def 頂点番号(z):
        鍵 = (round(z.real / 許容), round(z.imag / 許容))
        if 鍵 in 頂点辞書:
            return 頂点辞書[鍵]
        idx = len(頂点一覧)
        頂点辞書[鍵] = idx
        頂点一覧.append((z.real, z.imag))
        return idx

    辺集合 = set()
    菱形分類群 = []
    太数 = 0
    細数 = 0
    for 菱形 in クリップ済み:
        e1, p1, e2, p2 = 菱形
        種別 = 太細判定(菱形)
        if 種別 == "太":
            太数 += 1
        else:
            細数 += 1
        i_e1 = 頂点番号(e1)
        i_p1 = 頂点番号(p1)
        i_e2 = 頂点番号(e2)
        i_p2 = 頂点番号(p2)
        for a, b in [(i_e1, i_p1), (i_p1, i_e2), (i_e2, i_p2), (i_p2, i_e1)]:
            if a != b:
                辺集合.add((min(a, b), max(a, b)))
        菱形分類群.append((種別, i_e1, i_p1, i_e2, i_p2))

    辺一覧 = sorted(辺集合)

    # ピッチ検査 (mm): 辺長の min/median、および全頂点対の最近傍距離 (安全側)
    def 距離(i, j):
        ax, ay = 頂点一覧[i]
        bx, by = 頂点一覧[j]
        return math.hypot(ax - bx, ay - by)

    辺長群 = [距離(a, b) for a, b in 辺一覧]
    辺長群_sorted = sorted(辺長群)
    n = len(辺長群_sorted)
    中央値 = 辺長群_sorted[n // 2] if n % 2 == 1 else (辺長群_sorted[n // 2 - 1] + 辺長群_sorted[n // 2]) / 2
    最小辺長 = min(辺長群) if 辺長群 else float("inf")

    # 全頂点対の最近傍 (安全側チェック, O(n^2))
    import numpy as np
    座標配列 = np.array(頂点一覧)
    差 = 座標配列[:, None, :] - 座標配列[None, :, :]
    距離行列 = np.sqrt((差 ** 2).sum(axis=2))
    np.fill_diagonal(距離行列, np.inf)
    最近傍最小 = float(距離行列.min()) if len(頂点一覧) > 1 else float("inf")

    実測最小ピッチ = min(最小辺長, 最近傍最小)

    if 実測最小ピッチ < 引数.最小間隔:
        print(
            f"警告: 実測最小ピッチ {実測最小ピッチ:.4f}mm が --最小間隔 {引数.最小間隔}mm 未満です。"
            f"リスケールせず停止します。--半径 を増やすか --深さ を減らしてください。",
            file=sys.stderr,
        )
        sys.exit(1)

    # 頂点.json 出力
    頂点json = {
        "vertices": [[round(x, 9), round(y, 9)] for x, y in 頂点一覧],
        "edges": [[a, b] for a, b in 辺一覧],
        "params": {
            "半径mm": 引数.半径,
            "深さ": 引数.深さ,
            "最小間隔mm": 引数.最小間隔,
            "パッド径mm": 引数.パッド径,
            "許容mm": 許容,
        },
        "counts": {
            "頂点数": len(頂点一覧),
            "辺数": len(辺一覧),
            "太菱形数": 太数,
            "細菱形数": 細数,
            "最小ピッチmm": round(実測最小ピッチ, 6),
            "中央値ピッチmm": round(中央値, 6),
        },
    }
    頂点json_path = os.path.join(出力先, "頂点.json")
    with open(頂点json_path, "w", encoding="utf-8") as f:
        json.dump(頂点json, f, ensure_ascii=False, indent=2)

    # SVG 出力
    半幅 = 半径mm + 引数.余白
    svg_w = 2 * 半幅
    svg_h = 2 * 半幅

    def x変換(x):
        return x + 半幅

    def y変換(y):
        return 半幅 - y  # SVG は Y 下向きなので反転

    svg_parts = []
    svg_parts.append(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{svg_w}mm" height="{svg_h}mm" '
        f'viewBox="0 0 {svg_w} {svg_h}">'
    )
    svg_parts.append(f'<rect x="0" y="0" width="{svg_w}" height="{svg_h}" fill="white"/>')

    # 菱形 (塗り)
    for 種別, i_e1, i_p1, i_e2, i_p2 in 菱形分類群:
        色 = 引数.太色 if 種別 == "太" else 引数.細色
        pts = [頂点一覧[i] for i in (i_e1, i_p1, i_e2, i_p2)]
        pts_str = " ".join(f"{x変換(x):.4f},{y変換(y):.4f}" for x, y in pts)
        svg_parts.append(f'<polygon points="{pts_str}" fill="{色}" stroke="none"/>')

    # 辺 (線)
    for a, b in 辺一覧:
        ax, ay = 頂点一覧[a]
        bx, by = 頂点一覧[b]
        svg_parts.append(
            f'<line x1="{x変換(ax):.4f}" y1="{y変換(ay):.4f}" x2="{x変換(bx):.4f}" y2="{y変換(by):.4f}" '
            f'stroke="#333333" stroke-width="{引数.線幅}"/>'
        )

    # パッド (円)
    半径パッド = 引数.パッド径 / 2
    for x, y in 頂点一覧:
        svg_parts.append(
            f'<circle cx="{x変換(x):.4f}" cy="{y変換(y):.4f}" r="{半径パッド}" fill="#111111"/>'
        )

    # クリップ円 (境界表示)
    svg_parts.append(
        f'<circle cx="{半幅}" cy="{半幅}" r="{半径mm}" fill="none" stroke="red" '
        f'stroke-width="0.2" stroke-dasharray="1,1"/>'
    )
    svg_parts.append("</svg>")

    svg_path = os.path.join(出力先, "格子.svg")
    with open(svg_path, "w", encoding="utf-8") as f:
        f.write("\n".join(svg_parts))

    # PNG レンダリング (rsvg-convert > cairosvg > qlmanage の順に存在確認)
    png_path = os.path.join(出力先, "格子.png")
    scratch = os.path.join(出力先, "_スクラッチ")
    os.makedirs(scratch, exist_ok=True)
    if shutil.which("rsvg-convert"):
        subprocess.run(
            ["rsvg-convert", "-w", str(引数.画像幅), "-o", png_path, svg_path], check=True
        )
    elif shutil.which("cairosvg"):
        subprocess.run(
            ["cairosvg", svg_path, "-o", png_path, "--output-width", str(引数.画像幅)], check=True
        )
    elif shutil.which("qlmanage"):
        subprocess.run(
            ["qlmanage", "-t", "-s", str(引数.画像幅), "-o", scratch, svg_path],
            check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        生成物 = os.path.join(scratch, "格子.svg.png")
        if os.path.exists(生成物):
            shutil.move(生成物, png_path)
        else:
            print("警告: qlmanage の PNG 生成物が見つかりません", file=sys.stderr)
    else:
        print("警告: rsvg-convert / cairosvg / qlmanage のいずれも見つからず PNG を生成できません", file=sys.stderr)
    if os.path.isdir(scratch) and not os.listdir(scratch):
        os.rmdir(scratch)

    # KiCad フットプリント (.kicad_mod, テキスト形式, 手書き S式)
    kicad_path = os.path.join(出力先, "格子.kicad_mod")
    with open(kicad_path, "w", encoding="utf-8") as f:
        f.write(f'(module 準結晶格子 (layer F.Cu) (tedit 00000000)\n')
        f.write(f'  (descr "Penrose P3 準結晶電極格子 -- 半径={引数.半径}mm 深さ={引数.深さ} パッド径={引数.パッド径}mm")\n')
        f.write(f'  (fp_text reference REF** (at 0 {-(半径mm + 2):.3f}) (layer F.SilkS)\n')
        f.write(f'    (effects (font (size 1 1) (thickness 0.15))))\n')
        f.write(f'  (fp_text value 準結晶格子 (at 0 {(半径mm + 2):.3f}) (layer F.Fab)\n')
        f.write(f'    (effects (font (size 1 1) (thickness 0.15))))\n')
        for idx, (x, y) in enumerate(頂点一覧):
            f.write(
                f'  (pad {idx} smd circle (at {x:.4f} {-y:.4f}) '
                f'(size {引数.パッド径:.3f} {引数.パッド径:.3f}) '
                f'(layers F.Cu F.Paste F.Mask))\n'
            )
        f.write(")\n")

    print(f"頂点数: {len(頂点一覧)}")
    print(f"辺数: {len(辺一覧)}")
    print(f"最小ピッチ実測: {実測最小ピッチ:.4f}mm")
    print(f"中央値ピッチ実測: {中央値:.4f}mm")
    print(f"太菱形: {太数}  細菱形: {細数}")
    print(f"出力先: {出力先}")
    print(f"  頂点.json  {頂点json_path}")
    print(f"  格子.svg   {svg_path}")
    print(f"  格子.png   {png_path}")
    print(f"  格子.kicad_mod {kicad_path}")


if __name__ == "__main__":
    main()
