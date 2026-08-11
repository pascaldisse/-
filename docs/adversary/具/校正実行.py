#!/usr/bin/env python3
"""校正実行 — manifest.expected.json と wav審.py実測を突合し校正表を生成.
役=審具丁. 実行結果をそのまま docs/adversary/具/校正結果.md へ書出す (未実行値混入禁止).
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MANIFEST = os.path.join(HERE, '合成', 'manifest.expected.json')
ANALYZER = os.path.join(HERE, 'wav審.py')

with open(MANIFEST, encoding='utf-8') as f:
    manifest = json.load(f)

lines = []
lines.append('# 校正結果 — wav審.py 実測 (2026-08-11)')
lines.append('')
lines.append('全件 `/opt/homebrew/bin/python3 wav審.py <path>` を実走し、標準出力JSONから抽出。推定値なし、実測値のみ。')
lines.append('')
lines.append('| 信号 | 種別 | 期待Hz | 実測Hz(内挿) | 誤差Hz | 分解能±Hz | 期待RMS | 実測RMS | 期待peak | 実測peak | gate周期期待 | gate周期実測 | duty実測 | click検出 |')
lines.append('|---|---|---|---|---|---|---|---|---|---|---|---|---|---|')

for key, exp in manifest.items():
    p = exp['path']
    out = subprocess.run(['/opt/homebrew/bin/python3', ANALYZER, p], capture_output=True, text=True, check=True)
    d = json.loads(out.stdout)

    exp_freq = exp.get('expected_freq_hz')
    meas_freq = None
    err_freq = None
    res_hz = None
    if d.get('spectrum', {}).get('usable') and d['spectrum'].get('peaks'):
        top = d['spectrum']['peaks'][0]
        meas_freq = top['refined_freq_hz']
        res_hz = d['spectrum']['frequency_resolution_hz']
        if exp_freq is not None:
            err_freq = meas_freq - exp_freq

    exp_rms = exp.get('expected_rms_normalized') or exp.get('expected_rms_normalized_active_region_only')
    meas_rms = d.get('amplitude', {}).get('rms_normalized')
    exp_peak = exp.get('expected_peak_normalized_approx')
    meas_peak = d.get('amplitude', {}).get('peak_abs_normalized')

    exp_period = exp.get('expected_gate_period_sec')
    meas_period = d.get('envelope', {}).get('gate_period_sec_mean')
    meas_duty = d.get('envelope', {}).get('duty_mean')

    click_present = d.get('discontinuity', {}).get('click_present')
    exp_click = exp.get('expected_click_present')
    click_cell = f'{click_present}'
    if exp_click is not None:
        click_cell += f' (期待{exp_click}, {"一致" if click_present == exp_click else "不一致"})'

    def fmt(v, nd=4):
        return f'{v:.{nd}f}' if isinstance(v, (int, float)) else str(v)

    row = [
        key, exp.get('kind', ''),
        fmt(exp_freq) if exp_freq is not None else '—',
        fmt(meas_freq) if meas_freq is not None else '—',
        fmt(err_freq, 4) if err_freq is not None else '—',
        fmt(res_hz / 2, 4) if res_hz is not None else '—',
        fmt(exp_rms) if exp_rms is not None else '—',
        fmt(meas_rms) if meas_rms is not None else '—',
        fmt(exp_peak) if exp_peak is not None else '—',
        fmt(meas_peak) if meas_peak is not None else '—',
        fmt(exp_period, 3) if exp_period is not None else '—',
        fmt(meas_period, 6) if meas_period is not None else '—',
        fmt(meas_duty, 4) if meas_duty is not None else '—',
        click_cell,
    ]
    lines.append('| ' + ' | '.join(row) + ' |')

    # header整合系 (期待キーがあれば追記)
    if 'expected_data_declared_vs_actual_match' in exp:
        c = d['header']['consistency']
        ok = c['data_declared_vs_actual_match'] == exp['expected_data_declared_vs_actual_match']
        delta_ok = c['data_size_delta_bytes'] == exp.get('expected_data_size_delta_bytes')
        lines.append(f'  - {key}: data整合性検査 match={c["data_declared_vs_actual_match"]} (期待{exp["expected_data_declared_vs_actual_match"]}, {"一致" if ok else "不一致"}) · delta={c["data_size_delta_bytes"]}byte (期待{exp.get("expected_data_size_delta_bytes")}, {"一致" if delta_ok else "不一致"})')
    if 'expected_byte_rate_matches' in exp:
        c = d['header']['consistency']
        ok = c['byte_rate_matches'] == exp['expected_byte_rate_matches']
        lines.append(f'  - {key}: byte_rate整合性検査 matches={c["byte_rate_matches"]} (期待{exp["expected_byte_rate_matches"]}, {"一致" if ok else "不一致"}) · 宣言={d["header"]["fmt"]["byte_rate_bytes_per_sec"]} 期待算出={c["byte_rate_expected_from_rate_ch_bits"]}')

out_path = os.path.join(HERE, '校正結果.md')
with open(out_path, 'w', encoding='utf-8') as f:
    f.write('\n'.join(lines) + '\n')

print('\n'.join(lines))
print(f'\n書出: {out_path}', file=sys.stderr)
