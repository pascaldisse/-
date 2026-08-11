#!/usr/bin/env python3
"""C8追跡gate→四corpus・非mic。raw octave補正無・lap契約を実走判定。"""
import argparse
import math
import pathlib
import re
import subprocess

根 = pathlib.Path(__file__).resolve().parents[3]
crate = 根 / "機関" / "歌口"
名前 = ["C8_揺_440_50cent.wav", "C8_揺_440_120cent.wav", "C8_滑走_220_880.wav", "C8_滑走_880_220.wav"]
上限 = 6.0
hz行 = re.compile(r"^Z .* lap=(-?\d+) hz=(none|[0-9.]+)$")

arg = argparse.ArgumentParser()
arg.add_argument("--wave-dir", type=pathlib.Path, required=True, help="C8 RIFF corpus directory")
a = arg.parse_args()
if not a.wave_dir.is_dir():
    raise SystemExit(f"FAIL input corpus missing: {a.wave_dir}")

結果 = []
for 名 in 名前:
    入力 = a.wave_dir / 名
    出力 = 根 / ".jareth-1336" / "c8追跡" / f"{入力.stem}.log"
    出力.parent.mkdir(parents=True, exist_ok=True)
    cmd = ["cargo", "run", "--quiet", "--", "--源", "wav", "--wav", str(入力), "--跳幅", "2048", "--出力", str(出力), "--最大跳幅半音", str(上限)]
    run = subprocess.run(cmd, cwd=crate, text=True, capture_output=True)
    if run.returncode:
        raise SystemExit(f"FAIL {名} rc={run.returncode}\n{run.stdout}{run.stderr}")
    rows = []
    for 行 in 出力.read_text().splitlines():
        m = hz行.match(行)
        if m:
            rows.append((int(m.group(1)), None if m.group(2) == "none" else float(m.group(2))))
    voiced = [(lap, hz) for lap, hz in rows if hz is not None]
    if not rows or not voiced:
        raise SystemExit(f"FAIL {名} frame={len(rows)} voiced={len(voiced)}")
    差 = [12 * abs(math.log2(b / aa)) for (_, aa), (_, b) in zip(voiced, voiced[1:])]
    最大 = max(差, default=0.0)
    laps = [lap for lap, _ in voiced]
    if not math.isfinite(最大) or 最大 > 上限 + 1e-9:
        raise SystemExit(f"FAIL {名} rawstep={最大:.6f} > {上限}")
    if 名 == "C8_揺_440_50cent.wav" and set(laps) != {1}:
        raise SystemExit(f"FAIL {名} lap={sorted(set(laps))}; expected [1]")
    if 名 == "C8_揺_440_120cent.wav" and max(laps) - min(laps) > 1:
        raise SystemExit(f"FAIL {名} lap span={min(laps)}..{max(laps)}")
    if 名 == "C8_滑走_220_880.wav" and (laps != sorted(laps) or min(laps) != 0 or max(laps) != 2):
        raise SystemExit(f"FAIL {名} lap={min(laps)}..{max(laps)} monotonic={laps == sorted(laps)}")
    if 名 == "C8_滑走_880_220.wav" and (laps != sorted(laps, reverse=True) or min(laps) != 0 or max(laps) != 2):
        raise SystemExit(f"FAIL {名} lap={min(laps)}..{max(laps)} monotonic={laps == sorted(laps, reverse=True)}")
    結果.append(f"PASS {名} frame={len(rows)} voiced={len(voiced)} raw-maxstep={最大:.6f} lap={min(laps)}..{max(laps)}")
print("\n".join(結果))
print("PASS C8=4/4 octave補正=無 無音stale=無")
