#!/usr/bin/env python3
"""z審 — 梯5 (mic→z, crate 機関/歌口) 独立path解析器.
役=子鐘·信号解析係 (批根丁 @lampas 配下, 批側 二季). 依存=stdlib only (numpy禁).
被審Rust code (機関/歌口) を一切参照せず — 契約源 = 文書/環統合.md §主座標 · 丙-C1/C2/C3
+ docs/adversary/2026-08-11-梯45審.md 乙節のみ。着地前に完成させる (梯5 = 未着地, 2026-08-11実測).

対象log形式 (梯2互換, 実物=proof/環制御/z再生log.txt):
  # 環z 梯N 起動 ts=...
  # param Z変param { key: val, ... }
  Z ts=... x=... y=... theta=... r=... lap=...
寛容parser: 未知の行形式 (欠落field·異形式混入·header不在) は例外を投げず
「形式差分」として記録し, 解析可能な行のみで検査を継続する — 差分自体が欠陥報告の対象。

用:
  python3 z審.py <zログ> [--注入 注入目録.json --名 <エントリ名>]
                 [--決定論2 <別zログ>] [--定常比 0.3] [--r閾 0.15] [--縫目閾 rad] [--pretty]
  python3 z審.py --自己校正      # 合成陽性/陰性対照で全検査を自走判定, 結果を表印字

検査項目 (② 契約検査, 各々 合/否/未判定 を返す):
  検_無契約        — 丙-C1: r=0 ⟺ lap=0 ⟺ z=0. r=0 だが lap≠0 の区間を列挙.
  検_総角連続性    — 丙-C3: 総角(θ+2π·lap)のtick間差分. 縫目跳躍(閾既定0.9π)を列挙.
  検_lap単調性     — 指定区間でlap列が単調非減少/非増加か. chatter(±1往復)回数計数.
  検_r連続性       — rのtick間跳躍(閾既定0.15)を列挙 (0→r 瞬時立上り=出現律違反 検出).
  検_期待値突合    — 注入目録.jsonの期待lap/家/thetaと実測(区間後半の代表値)を突合.
                     cent誤差=1200*log2(f実/f期) · θ誤差(rad, 最短角距離).
  検_遅延          — z列最初のr>0時刻 − 目録onset秒 = 実測遅延 (C9).
  検_決定論        — 同一log二回読み(または2ファイル比較)のbit一致 (C16補助).

限界 (正直申告 — z審校正.md末尾にも複写):
  - 期待値突合/lap単調性は「区間後半の代表値(最頻lap内θ中央値)」方式 — 区間内で
    音高が連続的に移り変わる注入 (vibrato/glide, C8) の精密判定には向かない。単一定常
    値を仮定する簡易法であり, C8の厳密検査(ヒステリシス帯の正誤)は範囲外。
  - r連続性の閾値(既定0.15)・縫目跳躍の閾値(既定0.9π)は経験的既定であり普遍定数では
    ない — 実mic波形・実機tick周期での再校正必須 (本具は合成logでのみ校正済).
  - 遅延(C9)は「zログ先頭tsが録音/収音開始基準である」と仮定する。梯5実装がこの前提
    を満たさなければ数値は無意味 — 前提自体は本具では検証できない。
  - 決定論検査は「同一ファイルを2回読んでも同じ結果」というparserの安定性確認、また
    は2ファイルのbit比較に留まる。真のC16 (同一wav入力→梯5を2回実行して得た2つの
    実ログが一致するか) を検証するには実行そのもの(梯5起動)が要る — 本具は比較機能
    のみ提供し、実行はしない。
  - 期待値突合のcent誤差は「lap+frac から周波数を逆算する式的近似」であり、梯5内部
    が実際にどう周波数/音高を推定したか(FFT窓·hop等)は不問 — ブラックボックス突合。
  - 家境界tie-break (丙-D2既知欠陥, 202.5°のみ反転) のような単一tick規約逸脱は、
    代表値方式(中央値/最頻値)で均されるため検出できない可能性がある — 個別tick単位
    のtie-break検査は未実装。
  - wavファイル自体は読まない (wav審.py/歌口注入生成.pyの領分) — 本具はzログ (テキ
    スト行) のみを対象とする。バイナリ破損・エンコーディング異常は検出対象外。
"""
import argparse
import json
import math
import os
import re
import shutil
import sys
from statistics import median

# ---------------------------------------------------------------------------
# ① 寛容parser — 梯2互換 z ログ読取
# ---------------------------------------------------------------------------

_KV_RE = re.compile(r"(\w+)=(-?[0-9.eE+\-]+)")
_必須field = {"ts", "x", "y", "theta", "r", "lap"}


def z読(path):
    """z ログ読取. 戻= (param:dict, rows:list[dict], 形式差分:list[str]).
    未知行形式は例外を投げず 形式差分 に記録し, 解析可能行のみ rows へ積む。"""
    param = {}
    rows = []
    diffs = []
    header_seen = False
    with open(path, encoding="utf-8") as f:
        for lineno, raw in enumerate(f, 1):
            ln = raw.rstrip("\n")
            if not ln.strip():
                continue
            if ln.startswith("#"):
                if "param" in ln and "{" in ln and "}" in ln:
                    body = ln.split("{", 1)[1].rsplit("}", 1)[0]
                    for k, v in re.findall(r"([^\s{,]+):\s*([^,}]+)", body):
                        param[k.strip()] = v.strip()
                    header_seen = True
                continue
            if not ln.startswith("Z "):
                diffs.append(f"L{lineno}: 未知行形式 (Z/#以外) — {ln[:80]!r}")
                continue
            d = dict(_KV_RE.findall(ln))
            missing = _必須field - set(d.keys())
            if missing:
                diffs.append(f"L{lineno}: 欠落field {sorted(missing)} — {ln[:100]!r}")
                continue
            try:
                row = {
                    "ts": int(float(d["ts"])),
                    "x": float(d["x"]),
                    "y": float(d["y"]),
                    "theta": float(d["theta"]),
                    "r": float(d["r"]),
                    "lap": int(float(d["lap"])),
                    "lineno": lineno,
                }
            except ValueError as e:
                diffs.append(f"L{lineno}: 型変換失敗 {e} — {ln[:100]!r}")
                continue
            rows.append(row)
    if not header_seen:
        diffs.append("# param header 不在 (梯2互換契約 — proof/環制御/z再生log.txt 参照)")
    return param, rows, diffs


# ---------------------------------------------------------------------------
# ヘルパ
# ---------------------------------------------------------------------------


def _角差(a, b):
    """最短角距離 (rad, 符号付, [-π, π])."""
    d = (a - b) % (2 * math.pi)
    if d > math.pi:
        d -= 2 * math.pi
    return d


def _分布(vals):
    if not vals:
        return {}
    s = sorted(vals)
    n = len(s)

    def pct(p):
        return s[min(n - 1, int(p * n))]

    return {"min": s[0], "p50": pct(0.5), "p90": pct(0.9), "p99": pct(0.99), "max": s[-1]}


# ---------------------------------------------------------------------------
# ② 契約検査
# ---------------------------------------------------------------------------


def 検_無契約(rows, tol=1e-9):
    """丙-C1: r=0 ⟺ lap=0 ⟺ z=0. r=0 だが lap≠0 の区間を列挙."""
    if not rows:
        return {"合否": "未判定", "理由": "rows空"}
    違反区間 = []
    cur = None
    for row in rows:
        bad = abs(row["r"]) <= tol and row["lap"] != 0
        if bad and cur is None:
            cur = [row["lineno"], row["lineno"], row["lap"]]
        elif bad:
            cur[1] = row["lineno"]
        elif not bad and cur is not None:
            違反区間.append(tuple(cur))
            cur = None
    if cur is not None:
        違反区間.append(tuple(cur))
    return {"合否": "否" if 違反区間 else "合", "違反区間": 違反区間, "件数": len(違反区間)}


def 検_総角連続性(rows, 縫目閾=None):
    """丙-C3: 総角=theta+2π·lap の tick間差分. ±π縫目跳躍 (未補正wrap) を検出."""
    if len(rows) < 2:
        return {"合否": "未判定", "理由": "行不足(<2)"}
    総角列 = [r["theta"] + 2 * math.pi * r["lap"] for r in rows]
    diffs = [abs(総角列[i + 1] - 総角列[i]) for i in range(len(総角列) - 1)]
    閾 = 縫目閾 if 縫目閾 is not None else math.pi * 0.9
    跳躍点 = [
        (i + 1, rows[i]["lineno"], rows[i + 1]["lineno"], diffs[i])
        for i in range(len(diffs))
        if diffs[i] >= 閾
    ]
    return {
        "合否": "否" if 跳躍点 else "合",
        "最大差分rad": max(diffs),
        "平均差分rad": sum(diffs) / len(diffs),
        "分布": _分布(diffs),
        "縫目跳躍閾rad": 閾,
        "縫目跳躍": 跳躍点,
        "件数": len(跳躍点),
    }


def 検_lap単調性(rows, 区間=None):
    """指定区間で lap列が単調非減少/非増加か. chatter(±1往復)回数計数."""
    sub = rows if 区間 is None else rows[区間[0]:区間[1]]
    laps = [r["lap"] for r in sub]
    if len(laps) < 2:
        return {"合否": "未判定", "理由": "行不足(<2)"}
    非減少 = all(b >= a for a, b in zip(laps, laps[1:]))
    非増加 = all(b <= a for a, b in zip(laps, laps[1:]))
    単調 = 非減少 or 非増加
    deltas = [b - a for a, b in zip(laps, laps[1:]) if b != a]
    chatter = sum(1 for i in range(1, len(deltas)) if (deltas[i] > 0) != (deltas[i - 1] > 0))
    return {
        "合否": "合" if 単調 else "否",
        "非減少": 非減少,
        "非増加": 非増加,
        "chatter回数": chatter,
        "lap範囲": [min(laps), max(laps)],
    }


def 検_r連続性(rows, 閾=None):
    """r の tick間跳躍が閾超える点を列挙 (0→r 瞬時立上り=出現律違反 検出)."""
    if len(rows) < 2:
        return {"合否": "未判定", "理由": "行不足(<2)"}
    閾 = 0.15 if 閾 is None else 閾
    diffs = [abs(rows[i + 1]["r"] - rows[i]["r"]) for i in range(len(rows) - 1)]
    跳躍 = [
        (i + 1, rows[i]["lineno"], rows[i + 1]["lineno"], rows[i]["r"], rows[i + 1]["r"], diffs[i])
        for i in range(len(diffs))
        if diffs[i] >= 閾
    ]
    return {
        "合否": "否" if 跳躍 else "合",
        "閾": 閾,
        "最大差分": max(diffs),
        "跳躍": 跳躍,
        "件数": len(跳躍),
    }


def _代表値(rows, 定常比=0.3):
    """区間後半(定常比)から lap最頻値・(同lap内)theta中央値・r中央値を取る."""
    n = len(rows)
    start = max(0, int(n * (1 - 定常比)))
    sub = rows[start:] or rows
    laps = [r["lap"] for r in sub]
    lap代表 = max(set(laps), key=laps.count)
    thetas = [r["theta"] for r in sub if r["lap"] == lap代表] or [r["theta"] for r in sub]
    theta代表 = median(thetas)
    r代表 = median([r["r"] for r in sub])
    return lap代表, theta代表, r代表


def 検_期待値突合(rows, 期待, 基音=220.0, 家数=8, 定常比=0.3, cent許容=50.0):
    """注入目録.jsonエントリと実測(区間後半代表値)を突合. cent誤差·θ誤差(rad)算出."""
    if not rows:
        return {"合否": "未判定", "理由": "rows空"}
    lap代表, theta代表, r代表 = _代表値(rows, 定常比)
    frac = (theta代表 % (2 * math.pi)) / (2 * math.pi)
    家実 = round(frac * 家数) % 家数
    f実 = 基音 * (2 ** (lap代表 + frac))
    f期 = 期待.get("期待freq_hz")
    cent誤差 = 1200.0 * math.log2(f実 / f期) if (f実 and f期) else float("nan")
    θ期 = 期待.get("期待theta_rad", float("nan"))
    θ誤差 = _角差(theta代表, θ期) if not math.isnan(θ期) else float("nan")
    lap期 = 期待.get("期待lap")
    家期 = 期待.get("期待家")
    lap合 = lap代表 == lap期 if lap期 is not None else None
    家合 = 家実 == 家期 if 家期 is not None else None
    ok = bool(lap合) and bool(家合) and not math.isnan(cent誤差) and abs(cent誤差) < cent許容
    return {
        "合否": "合" if ok else "否",
        "実測lap": lap代表, "期待lap": lap期, "lap一致": lap合,
        "実測家": 家実, "期待家": 家期, "家一致": 家合,
        "実測theta_rad": theta代表, "期待theta_rad": θ期, "θ誤差rad": θ誤差,
        "実測freq_hz概算": f実, "期待freq_hz": f期, "cent誤差": cent誤差, "cent許容": cent許容,
        "実測r中央値": r代表, "期待r": 期待.get("期待r"),
    }


def 検_遅延(rows, onset秒, r閾=1e-9):
    """z列の最初の r>0 時刻 − 目録onset秒 = 実測遅延 (C9).
    前提: zログ先頭tsを収音開始基準とする (検証不能な仮定 — 限界申告参照)."""
    if not rows:
        return {"合否": "未判定", "理由": "rows空"}
    r0 = next((row for row in rows if row["r"] > r閾), None)
    if r0 is None:
        return {"合否": "未判定", "理由": "r>0行なし"}
    t0 = rows[0]["ts"]
    起動相対秒 = (r0["ts"] - t0) / 1000.0
    実測遅延 = 起動相対秒 - onset秒
    return {
        "合否": "参考(上限契約は文書/環統合.md未記載, 数値申告のみ)",
        "実測遅延秒": 実測遅延,
        "z起動相対秒": 起動相対秒,
        "目録onset秒": onset秒,
        "r>0初出lineno": r0["lineno"],
    }


def 検_決定論(path_a, path_b=None):
    """同一log二回読み (path_b省略時) または2ファイル比較の bit一致 (C16補助)."""
    p1, r1, d1 = z読(path_a)
    比較先 = path_a if path_b is None else path_b
    p2, r2, d2 = z読(比較先)
    一致 = (r1 == r2) and (p1 == p2)
    差分行 = [] if 一致 else [i for i, (a, b) in enumerate(zip(r1, r2)) if a != b]
    差分行 += list(range(min(len(r1), len(r2)), max(len(r1), len(r2)))) if len(r1) != len(r2) else []
    return {
        "合否": "合" if 一致 else "否",
        "比較対象": [path_a, 比較先],
        "行数a": len(r1), "行数b": len(r2),
        "差分行数": len(差分行),
    }


# ---------------------------------------------------------------------------
# 総合検査 (実ログ解析)
# ---------------------------------------------------------------------------


def 総合検査(zlog_path, 注入path=None, 名=None, 決定論2=None, 定常比=0.3, r閾=None, 縫目閾=None):
    param, rows, diffs = z読(zlog_path)
    報告 = {
        "file": zlog_path,
        "行数": len(rows),
        "形式差分": diffs,
        "param": param,
        "無契約(C1)": 検_無契約(rows),
        "総角連続性(C3)": 検_総角連続性(rows, 縫目閾),
        "lap単調性": 検_lap単調性(rows),
        "r連続性(出現律)": 検_r連続性(rows, r閾),
        "決定論(C16補助)": 検_決定論(zlog_path, 決定論2),
    }
    if 注入path and 名:
        with open(注入path, encoding="utf-8") as f:
            目録 = json.load(f)
        基音 = 目録.get("基音", 220.0)
        entry = next((e for e in 目録["目録"] if e.get("名") == 名), None)
        if entry is None:
            報告["期待値突合"] = {"合否": "未判定", "理由": f"名={名} が注入目録.json内に不在"}
            報告["遅延(C9)"] = {"合否": "未判定", "理由": f"名={名} が注入目録.json内に不在"}
        else:
            報告["期待値突合"] = 検_期待値突合(rows, entry, 基音=基音, 定常比=定常比)
            報告["遅延(C9)"] = 検_遅延(rows, entry.get("onset秒", 0.0))
    return 報告


# ---------------------------------------------------------------------------
# ③ 自己校正 — 合成陽性/陰性対照で全検査を自走判定
# ---------------------------------------------------------------------------


def _合成行(θ_r_lap_列, ts0=1700000000000, dt_ms=16):
    行 = []
    for i, (θ, r, lap) in enumerate(θ_r_lap_列):
        ts = ts0 + i * dt_ms
        x = r * math.cos(θ)
        y = r * math.sin(θ)
        行.append(f"Z ts={ts} x={x:.4f} y={y:.4f} theta={θ:.6f} r={r:.6f} lap={lap}")
    return 行


def _書込(dir_, name, header, body):
    path = os.path.join(dir_, name)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(header) + "\n")
        f.write("\n".join(body) + "\n")
    return path


def 自己校正実行():
    """合成陽性/陰性対照を .scratch/z審校正/ に一時生成→全検査実走→表印字→撤去."""
    base = os.path.abspath(
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", ".scratch", "z審校正")
    )
    os.makedirs(base, exist_ok=True)
    ヘッダ = [
        "# 環z 梯5 起動(合成) ts=1700000000000",
        "# param Z変param { 死域: 0.08, 家数: 8, 半回転規約: 1 }",
        "# 源=合成校正 元=z審.py --自己校正",
    ]
    結果 = []
    try:
        # 陰性対照: 清浄定常 (家0, lap0, r=0.5一定, 40tick)
        清浄 = [(0.0, 0.5, 0)] * 40
        p清浄 = _書込(base, "陰性_清浄.log", ヘッダ, _合成行(清浄))
        _, r清浄, d清浄 = z読(p清浄)
        判1, 判2, 判3, 判4 = (
            検_無契約(r清浄), 検_総角連続性(r清浄), 検_lap単調性(r清浄), 検_r連続性(r清浄),
        )
        結果.append(("陰性_清浄", "無契約(C1)", "合", 判1["合否"], f"違反件数={判1['件数']}"))
        結果.append(("陰性_清浄", "総角連続性(C3)", "合", 判2["合否"], f"縫目件数={判2['件数']} 最大差={判2['最大差分rad']:.6f}"))
        結果.append(("陰性_清浄", "lap単調性", "合", 判3["合否"], f"chatter={判3['chatter回数']}"))
        結果.append(("陰性_清浄", "r連続性", "合", 判4["合否"], f"跳躍件数={判4['件数']}"))
        結果.append(("陰性_清浄", "寛容parser(形式差分)", "0件", f"{len(d清浄)}件", str(d清浄)))

        # 陽性1: 無契約違反 (r=0だがlap≠0の区間)
        列 = [(0.0, 0.5, 0)] * 10 + [(0.0, 0.0, 2)] * 8 + [(0.0, 0.5, 0)] * 10
        p1 = _書込(base, "陽性_無契約.log", ヘッダ, _合成行(列))
        _, r1, _ = z読(p1)
        判 = 検_無契約(r1)
        結果.append(("陽性_無契約(r=0∧lap≠0区間)", "無契約(C1)", "否", 判["合否"], f"違反区間={判['違反区間']}"))

        # 陽性2: lap chatter (±1往復)
        列 = [(0.0, 0.5, l) for l in [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]]
        p2 = _書込(base, "陽性_chatter.log", ヘッダ, _合成行(列))
        _, r2, _ = z読(p2)
        判 = 検_lap単調性(r2)
        結果.append(("陽性_chatter(lap 0/1往復)", "lap単調性", "否", 判["合否"], f"chatter={判['chatter回数']} lap範囲={判['lap範囲']}"))

        # 陽性3: 縫目跳躍 (総角不連続, wrap未補正) / 陰性: 正しく補正されたwrap
        列違反 = [(3.0, 0.5, 0), (3.10, 0.5, 0), (-3.10, 0.5, 0), (-3.0, 0.5, 0)]
        列補正 = [(3.0, 0.5, 0), (3.10, 0.5, 0), (-3.10, 0.5, 1), (-3.0, 0.5, 1)]
        p3 = _書込(base, "陽性_縫目未補正.log", ヘッダ, _合成行(列違反))
        p3b = _書込(base, "陰性_wrap補正済.log", ヘッダ, _合成行(列補正))
        _, r3, _ = z読(p3)
        _, r3b, _ = z読(p3b)
        判, 判b = 検_総角連続性(r3), 検_総角連続性(r3b)
        結果.append(("陽性_縫目未補正(lap不動)", "総角連続性(C3)", "否", 判["合否"], f"最大差={判['最大差分rad']:.6f} 跳躍={判['縫目跳躍']}"))
        結果.append(("陰性_wrap補正済(lap+1)", "総角連続性(C3)", "合", 判b["合否"], f"最大差={判b['最大差分rad']:.6f}"))

        # 陽性4: r階段(出現律違反, 0→0.9瞬時) / 陰性: r滑らかfade-in
        列 = [(0.0, 0.0, 0)] * 5 + [(0.0, 0.9, 0)] * 10
        p4 = _書込(base, "陽性_r階段.log", ヘッダ, _合成行(列))
        _, r4, _ = z読(p4)
        判 = 検_r連続性(r4)
        結果.append(("陽性_r階段(0→0.9瞬時)", "r連続性(出現律)", "否", 判["合否"], f"跳躍={判['跳躍']}"))

        n = 20
        列 = [(0.0, 0.9 * i / n, 0) for i in range(n + 1)]
        p5 = _書込(base, "陰性_r滑らか.log", ヘッダ, _合成行(列))
        _, r5, _ = z読(p5)
        判 = 検_r連続性(r5)
        結果.append(("陰性_r滑らか(fade-in)", "r連続性(出現律)", "合", 判["合否"], f"最大差分={判['最大差分']:.4f}"))

        # 期待値突合: 契約通り完全一致 (C1_家梯子_lap0_家0相当) / 陽性: lap誤り
        期待 = {"期待freq_hz": 220.0, "期待lap": 0, "期待家": 0, "期待theta_rad": 0.0, "期待r": 0.5}
        列 = [(0.0, 0.5, 0)] * 40
        p6 = _書込(base, "陽性_期待一致.log", ヘッダ, _合成行(列))
        _, r6, _ = z読(p6)
        判 = 検_期待値突合(r6, 期待, 基音=220.0)
        結果.append(("陽性_期待一致(契約通りlap0家0)", "期待値突合", "合", 判["合否"], f"cent誤差={判['cent誤差']:.4f} θ誤差={判['θ誤差rad']:.6f}"))

        列 = [(0.0, 0.5, 1)] * 40  # lap=1 だが期待lap=0
        p7 = _書込(base, "陽性_期待不一致.log", ヘッダ, _合成行(列))
        _, r7, _ = z読(p7)
        判 = 検_期待値突合(r7, 期待, 基音=220.0)
        結果.append(("陽性_期待不一致(lap誤り)", "期待値突合", "否", 判["合否"], f"実測lap={判['実測lap']} 期待lap={判['期待lap']} cent誤差={判['cent誤差']:.4f}"))

        # 遅延(C9): 無音(62tick≈0.992s)→立上り, 目録onset秒=1.0
        n無音 = 62
        列 = [(0.0, 0.0, 0)] * n無音 + [(0.0, 0.5, 0)] * 20
        p8 = _書込(base, "陽性_遅延.log", ヘッダ, _合成行(列))
        _, r8, _ = z読(p8)
        判 = 検_遅延(r8, 1.0)
        結果.append(("陽性_遅延(無音0.992s→立上り, onset目標1.0s)", "遅延(C9)", "実測遅延を数値化", 判.get("合否"), f"実測遅延秒={判.get('実測遅延秒')}"))

        # 決定論: 同一file2回読み(合) / 改変file比較(否)
        判 = 検_決定論(p6)
        結果.append(("陽性_期待一致(自己再読)", "決定論(C16補助)", "合", 判["合否"], f"差分行数={判['差分行数']}"))
        列改変 = [(0.0, 0.5, 0)] * 39 + [(0.0, 0.4, 0)]
        p9 = _書込(base, "陽性_改変版.log", ヘッダ, _合成行(列改変))
        判 = 検_決定論(p6, p9)
        結果.append(("陽性_期待一致 vs 改変版(末尾r相違)", "決定論(C16補助)", "否", 判["合否"], f"差分行数={判['差分行数']}"))

        # 寛容parser: 形式崩れ(欠落field·異形式混入·header込)
        崩れ = ヘッダ + [
            "Z ts=1700000000000 x=0.0000 y=0.0000 theta=0.000000 r=0.500000 lap=0",
            "Z ts=1700000000016 x=0.0000 theta=0.000000 r=0.500000 lap=0",  # y欠落
            "TICK ts=1700000000032 L(x=0.0 y=0.0)",  # 異形式混入
            "DEVICE id=GamepadId(0) name=\"PS5 Controller\" connected=true",  # proof実物にある header 亜種
            "Z ts=1700000000048 x=0.0000 y=0.0000 theta=0.000000 r=0.500000 lap=0",
        ]
        p10 = os.path.join(base, "陽性_形式崩れ.log")
        with open(p10, "w", encoding="utf-8") as f:
            f.write("\n".join(崩れ) + "\n")
        _, r10, d10 = z読(p10)
        結果.append(("陽性_形式崩れ(欠落field+異形式混入)", "寛容parser(形式差分)", "差分検出·正常行は継続parse", f"{len(d10)}件差分・正常行{len(r10)}件", " / ".join(d10)))

        print("| 対照 | 検査 | 期待判定 | 実測判定 | 詳細(実測) |")
        print("|---|---|---|---|---|")
        for 対照, 検査, 期待判定, 実測判定, 詳細 in 結果:
            print(f"| {対照} | {検査} | {期待判定} | {実測判定} | {詳細} |")
    finally:
        shutil.rmtree(base, ignore_errors=True)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main():
    ap = argparse.ArgumentParser(description="z審 — 梯5 (mic→z) 独立path解析器")
    ap.add_argument("zlog", nargs="?", help="被審zログ path")
    ap.add_argument("--注入", dest="注入", help="注入目録.json path")
    ap.add_argument("--名", dest="名", help="注入目録内エントリ名 (期待値突合/遅延に使用)")
    ap.add_argument("--決定論2", dest="決定論2", help="決定論比較対象の別zログ (省略時=自己読み比較)")
    ap.add_argument("--定常比", dest="定常比", type=float, default=0.3)
    ap.add_argument("--r閾", dest="r閾", type=float, default=None)
    ap.add_argument("--縫目閾", dest="縫目閾", type=float, default=None)
    ap.add_argument("--自己校正", dest="自己校正", action="store_true")
    ap.add_argument("--pretty", action="store_true")
    args = ap.parse_args()

    if args.自己校正:
        自己校正実行()
        return

    if not args.zlog:
        ap.error("zlog必須 (--自己校正 指定時を除く)")

    報告 = 総合検査(
        args.zlog, args.注入, args.名, args.決定論2, args.定常比, args.r閾, args.縫目閾,
    )
    print(json.dumps(報告, ensure_ascii=False, indent=2 if args.pretty else None, default=str))


if __name__ == "__main__":
    main()
