#!/usr/bin/env python3
"""Rahu C8審 — corpus実走→歌口Z文法/有限値/巻列を独立判定。exit=0単独は合格にしない。"""
import json, math, re, shutil, subprocess, sys
from pathlib import Path

根 = Path(__file__).resolve().parents[3]
歌口 = 根 / '機関/歌口'
具 = 根 / 'docs/adversary/具'
波 = 具 / '合成歌'
証 = 根 / 'docs/adversary/証/Rahu梯5C8'
Z = re.compile(r'^Z\s+ts=(\d+)\s+x=([^ ]+)\s+y=([^ ]+)\s+theta=([^ ]+)\s+r=([^ ]+)\s+lap=(-?\d+)\s+hz=(.+)$')

def fail(msg):
    print('FAIL:', msg, file=sys.stderr)
    return msg

def z列(path):
    rows = []
    for line in path.read_text().splitlines():
        m = Z.match(line)
        if m:
            ts, x, y, theta, r, lap, hz = m.groups()
            rows.append(dict(ts=int(ts), x=float(x), y=float(y), theta=float(theta), r=float(r), lap=int(lap), hz=hz))
    return rows

def 条件(name, rows):
    active = [r for r in rows if r['r'] > 0.0]
    bad = [r for r in rows if not all(math.isfinite(r[k]) for k in ('x','y','theta','r'))]
    errors = []
    if not rows: errors.append('Z行零')
    if bad: errors.append(f'非有限={len(bad)}')
    if name.startswith('C13_') and active: errors.append(f'C13 active={len(active)} (期待0)')
    if name == 'C8_揺_440_50cent':
        laps = {r['lap'] for r in active}
        if not active: errors.append('C8 50cent active=0')
        elif laps != {1}: errors.append(f'C8 50cent lap={sorted(laps)} (期待[1])')
    if name == 'C8_揺_440_120cent':
        laps = [r['lap'] for r in active]
        if not active: errors.append('C8 120cent active=0')
        elif max(laps)-min(laps) > 1: errors.append(f'C8 120cent lap幅={max(laps)-min(laps)} (>1)')
    if name == 'C8_滑走_220_880':
        laps = [r['lap'] for r in active]
        if not active: errors.append('C8 上滑走 active=0')
        elif any(a > b for a,b in zip(laps,laps[1:])): errors.append('C8 上滑走 lap非単調')
        elif min(laps) != 0 or max(laps) != 2: errors.append(f'C8 上滑走 lap域={min(laps)}..{max(laps)} (期待0..2)')
    if name == 'C8_滑走_880_220':
        laps = [r['lap'] for r in active]
        if not active: errors.append('C8 下滑走 active=0')
        elif any(a < b for a,b in zip(laps,laps[1:])): errors.append('C8 下滑走 lap非単調')
        elif min(laps) != 0 or max(laps) != 2: errors.append(f'C8 下滑走 lap域={min(laps)}..{max(laps)} (期待0..2)')
    return active, errors

def main():
    subprocess.run(['cargo','build','--quiet'], cwd=歌口, check=True)
    binary = 歌口 / 'target/debug/歌口'
    if not binary.exists(): raise RuntimeError('歌口 binary無し')
    if 証.exists(): shutil.rmtree(証)
    証.mkdir(parents=True)
    records, failures = [], []
    for wav in sorted(波.glob('*.wav')):
        name = wav.stem; log = 証 / f'{name}.log'
        # 2048標本hop=42.7ms: C8の5–6Hz揺/4秒滑走を保持しつつ65波全走を有限化。
        p = subprocess.run([str(binary),'--源','wav','--wav',str(wav),'--跳幅','2048','--出力',str(log)], cwd=歌口, text=True, capture_output=True)
        rows = z列(log) if log.exists() else []
        active, errors = 条件(name, rows)
        if p.returncode: errors.append(f'exit={p.returncode}')
        rec = {'名':name,'exit':p.returncode,'z行':len(rows),'active':len(active),'lap':sorted(set(r['lap'] for r in active)),'errors':errors}
        records.append(rec)
        if errors: failures.append(rec)
    (証/'table.json').write_text(json.dumps(records, ensure_ascii=False, indent=2)+'\n')
    c8 = [r for r in records if r['名'].startswith('C8_')]
    c13 = [r for r in records if r['名'].startswith('C13_')]
    print(f'corpus={len(records)} exit0={sum(r["exit"]==0 for r in records)} failures={len(failures)}')
    for r in c8+c13:
        print(json.dumps(r, ensure_ascii=False))
    if failures:
        for r in failures: print(json.dumps(r, ensure_ascii=False), file=sys.stderr)
        return 1
    return 0
if __name__ == '__main__': sys.exit(main())
