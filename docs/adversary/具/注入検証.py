#!/usr/bin/env python3
"""注入検証 — 生成corpus(歌口注入生成.py)を 別経路解析器(wav審.py)で実測し, 期待値と突合。
役=批根丁. 目的: 被審(機関/歌口)へ入れる前に「注入波そのものが正しい」を独立path で立証する。
生成=struct手書き / 解析=自前FFT — 経路独立。
出力: docs/adversary/具/注入検証結果.md + 注入検証.jsonl
"""
import os
import sys
import json
import math

D = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, D)
import importlib
wav審 = importlib.import_module('wav審')

目録 = json.load(open(os.path.join(D, '注入目録.json')))
行 = []
不一致 = []

for e in 目録['目録']:
    path = os.path.join(D, e['wav'].replace('具/', '') if e['wav'].startswith('具/') else e['wav'])
    if not os.path.exists(path):
        path = os.path.join(D, '合成歌', e['名'] + '.wav')
    a = wav審.analyze(path)
    sp = a.get('spectrum') or {}
    amp = a.get('amplitude') or {}
    実 = sp.get('refined_freq_hz') or sp.get('dominant_freq_hz')
    分解 = sp.get('resolution_hz')
    期 = e.get('期待freq_hz')
    誤差 = (実 - 期) if (実 is not None and 期 is not None) else None
    判定 = '—'
    if 誤差 is not None and 分解 is not None:
        判定 = '合' if abs(誤差) <= max(分解, 1.0) else '不一致'
        if 判定 == '不一致':
            不一致.append((e['名'], 期, 実, 誤差, 分解))
    rec = {
        '名': e['名'], '攻撃': e['攻撃'], 'sample率': e['sample率'], 'ch': e['ch'], 'bits': e['bits'],
        '期待freq': 期, '実測freq': 実, '誤差Hz': 誤差, '分解能Hz': 分解,
        'RMS': amp.get('rms'), 'peak': amp.get('peak_abs'), '厳密無音': amp.get('strict_zero'),
        '期待lap': e.get('期待lap'), '期待家': e.get('期待家'),
        '期待theta': e.get('期待theta_rad'), '註': e.get('註'), 'header整合': (a.get('riff') or {}).get('data_size_matches'),
        '判定': 判定,
    }
    行.append(rec)

with open(os.path.join(D, '注入検証.jsonl'), 'w') as f:
    for r in 行:
        f.write(json.dumps(r, ensure_ascii=False) + '\n')

def s(v, n=4):
    return '—' if v is None else (f'{v:.{n}f}' if isinstance(v, float) else str(v))

md = ['# 注入波 独立検証表 (批根丁 二季)', '',
      f'生成器=`具/歌口注入生成.py` (struct手書きRIFF) · 解析器=`具/wav審.py` (自前FFT, 校正11/11済) — **別経路**。',
      f'契約源=`文書/環統合.md` 音高写像 (基音={目録["基音"]}Hz, L=log2(f/基音), lap=floor(L), θ=2π·frac, 家=round(frac·8) mod 8)。', '',
      '| 名 | 攻撃 | 期待Hz | 実測Hz | 誤差Hz | 分解±Hz | RMS | peak | 期待lap | 期待家 | 判定 |',
      '|---|---|---|---|---|---|---|---|---|---|---|']
for r in 行:
    md.append('| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |'.format(
        r['名'], r['攻撃'], s(r['期待freq']), s(r['実測freq']), s(r['誤差Hz']), s(r['分解能Hz']),
        s(r['RMS']), s(r['peak']), s(r['期待lap']), s(r['期待家']), r['判定']))
md += ['', f'不一致 {len(不一致)} 件 / 全 {len(行)} 件。',
       '', '註: 非音高信号 (雑音·無音·DC·多声·倍音族) は「支配周波数」が意味を持たぬ故 判定=— (期待freq未定義)。',
       'これらは被審z出力側で判定する (r=0契約·確度gate)。']
open(os.path.join(D, '注入検証結果.md'), 'w').write('\n'.join(md) + '\n')
print(f'検証 {len(行)} 件 · 不一致 {len(不一致)}')
for x in 不一致:
    print('  不一致:', x)
