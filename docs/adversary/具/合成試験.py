#!/usr/bin/env python3
"""合成試験 — wav審.py 校正用. 既知信号を自前byte書出で生成し, 期待値(解析式)を併記する.
役=審具丁. 依存=stdlib only. `wave`モジュール不使用 (analyzer側と別経路を保つ為, 手書きheader).
出力: docs/adversary/具/合成/*.wav + 同名.expected.json
"""
import os
import math
import struct
import json

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), '合成')
os.makedirs(OUT_DIR, exist_ok=True)


def write_wav_pcm16(path, sample_rate, channels, samples_f):
    """samples_f: list[float] (-1..1, mono想定) → 16bit PCM wav 手書き."""
    n = len(samples_f)
    bits = 16
    block_align = channels * (bits // 8)
    byte_rate = sample_rate * block_align
    data = bytearray()
    for v in samples_f:
        iv = round(v * 32767)
        iv = max(-32768, min(32767, iv))
        data += struct.pack('<h', iv)
    data_size = len(data)
    fmt_chunk = struct.pack('<HHIIHH', 1, channels, sample_rate, byte_rate, block_align, bits)
    riff_size = 4 + (8 + len(fmt_chunk)) + (8 + data_size)
    with open(path, 'wb') as f:
        f.write(b'RIFF')
        f.write(struct.pack('<I', riff_size))
        f.write(b'WAVE')
        f.write(b'fmt ')
        f.write(struct.pack('<I', len(fmt_chunk)))
        f.write(fmt_chunk)
        f.write(b'data')
        f.write(struct.pack('<I', data_size))
        f.write(data)
    return data_size


def sine(freq, dur, rate, amp, phase0=0.0):
    n = round(dur * rate)
    return [amp * math.sin(2 * math.pi * freq * i / rate + phase0) for i in range(n)]


def zeros(dur, rate):
    return [0.0] * round(dur * rate)


def gated_sine(freq, dur, rate, amp, gate_hz, duty):
    n = round(dur * rate)
    period = 1.0 / gate_hz
    out = []
    for i in range(n):
        t = i / rate
        phase_in_period = t % period
        on = phase_in_period < (duty * period)
        v = amp * math.sin(2 * math.pi * freq * t) if on else 0.0
        out.append(v)
    return out


def expected_rms(amp):
    return amp / math.sqrt(2)


results_manifest = {}

RATE = 44100

# 1) 440Hz純音, amp=0.5, 2.0s
sig = sine(440.0, 2.0, RATE, 0.5)
p = os.path.join(OUT_DIR, 'sine_440.wav')
write_wav_pcm16(p, RATE, 1, sig)
results_manifest['sine_440'] = {
    'path': p, 'expected_freq_hz': 440.0, 'expected_rms_normalized': expected_rms(0.5),
    'expected_peak_normalized_approx': 0.5, 'sample_rate': RATE, 'duration_sec': 2.0,
    'kind': '純音・非gate',
}

# 2) 880Hz純音, amp=1.0 (フルスケール), 2.0s
sig = sine(880.0, 2.0, RATE, 1.0)
p = os.path.join(OUT_DIR, 'sine_880.wav')
write_wav_pcm16(p, RATE, 1, sig)
results_manifest['sine_880'] = {
    'path': p, 'expected_freq_hz': 880.0, 'expected_rms_normalized': expected_rms(1.0),
    'expected_peak_normalized_approx': 1.0, 'sample_rate': RATE, 'duration_sec': 2.0,
    'kind': '純音・フル振幅',
}

# 3) 2Hz gate付440Hz, duty=0.5, 4.0s (A4検証: period=0.500s±2%)
sig = gated_sine(440.0, 4.0, RATE, 0.5, 2.0, 0.5)
p = os.path.join(OUT_DIR, 'gate_2hz_440.wav')
write_wav_pcm16(p, RATE, 1, sig)
results_manifest['gate_2hz_440'] = {
    'path': p, 'expected_freq_hz': 440.0,
    'expected_rms_normalized_active_region_only': expected_rms(0.5),
    'expected_rms_normalized': expected_rms(0.5) * math.sqrt(0.5),  # duty=0.5混みfile全体RMS (無音半分込み)
    'expected_gate_period_sec': 0.5, 'expected_duty': 0.5, 'expected_n_segments': 8,
    'expected_peak_normalized_approx': 0.5,
    'sample_rate': RATE, 'duration_sec': 4.0,
    'note': '440*0.25=110 (整数) · 440*0.5=220 (整数) → gate境界は搬送波zero crossingと一致 → 理論上click無 (滑らかgate)',
    'kind': '2Hz gate',
}

# 4) 完全無音, 1.0s
sig = zeros(1.0, RATE)
p = os.path.join(OUT_DIR, 'silence.wav')
write_wav_pcm16(p, RATE, 1, sig)
results_manifest['silence'] = {
    'path': p, 'expected_peak_raw': 0, 'expected_rms_normalized': 0.0,
    'expected_strict_silence': True, 'sample_rate': RATE, 'duration_sec': 1.0,
    'kind': '無音',
}

# 5) 振幅較正列 1000Hz, amp=0.25/0.5/1.0, 1.0s
for amp_tag, amp in (('0_25', 0.25), ('0_5', 0.5), ('1_0', 1.0)):
    sig = sine(1000.0, 1.0, RATE, amp)
    p = os.path.join(OUT_DIR, f'amp_{amp_tag}.wav')
    write_wav_pcm16(p, RATE, 1, sig)
    results_manifest[f'amp_{amp_tag}'] = {
        'path': p, 'expected_freq_hz': 1000.0, 'expected_rms_normalized': expected_rms(amp),
        'expected_peak_normalized_approx': amp, 'sample_rate': RATE, 'duration_sec': 1.0,
        'kind': f'振幅較正 amp={amp}',
    }

# 6) 人工click注入 (300Hz, amp=0.6, 1.0s, 標本index=22050に単発spikeを加算)
sig = sine(300.0, 1.0, RATE, 0.6)
click_idx = 22050
sig[click_idx] = max(-1.0, min(1.0, sig[click_idx] + 0.8))
p = os.path.join(OUT_DIR, 'click_injected.wav')
write_wav_pcm16(p, RATE, 1, sig)
results_manifest['click_injected'] = {
    'path': p, 'expected_freq_hz': 300.0, 'injected_click_sample_index': click_idx,
    'expected_click_present': True, 'sample_rate': RATE, 'duration_sec': 1.0,
    'kind': '人工段差 (陽性対照)',
}

# 7) 滑らか対照 (440Hz, amp=0.5, dur=1.0s → 440*1.0=440整数 → 先頭/末尾ともzero crossing, click無を期待)
sig = sine(440.0, 1.0, RATE, 0.5)
p = os.path.join(OUT_DIR, 'no_click_smooth.wav')
write_wav_pcm16(p, RATE, 1, sig)
results_manifest['no_click_smooth'] = {
    'path': p, 'expected_freq_hz': 440.0, 'expected_click_present': False,
    'sample_rate': RATE, 'duration_sec': 1.0,
    'kind': '陰性対照 (click無であるべき)',
}

# 8) 破損header-A: data宣言sizeが実際より大 (切詰模擬)
sig = sine(500.0, 0.5, RATE, 0.4)
p = os.path.join(OUT_DIR, 'corrupt_data_size.wav')
data_size = write_wav_pcm16(p, RATE, 1, sig)
with open(p, 'r+b') as f:
    f.seek(40)  # data chunk size field位置 (RIFF12+fmt24+data8=44, sizeは data ID直後=offset 40)
    f.write(struct.pack('<I', data_size + 1000))  # 宣言を1000byte水増し (実file末尾は変えない)
results_manifest['corrupt_data_size'] = {
    'path': p, 'expected_data_declared_vs_actual_match': False,
    'expected_data_size_delta_bytes': 1000,
    'sample_rate': RATE, 'duration_sec': 0.5,
    'kind': '破損header (data size水増し)',
}

# 9) 破損header-B: byte_rate field不整合
sig = sine(500.0, 0.5, RATE, 0.4)
p = os.path.join(OUT_DIR, 'corrupt_byte_rate.wav')
write_wav_pcm16(p, RATE, 1, sig)
with open(p, 'r+b') as f:
    f.seek(28)  # fmt chunk内 byte_rate field位置 (RIFF12+id4+size4+audiofmt2+ch2+rate4=28)
    f.write(struct.pack('<I', 999999))
results_manifest['corrupt_byte_rate'] = {
    'path': p, 'expected_byte_rate_matches': False,
    'sample_rate': RATE, 'duration_sec': 0.5,
    'kind': '破損header (byte_rate不整合)',
}

manifest_path = os.path.join(OUT_DIR, 'manifest.expected.json')
with open(manifest_path, 'w', encoding='utf-8') as f:
    json.dump(results_manifest, f, ensure_ascii=False, indent=2)

print(f'生成完了: {len(results_manifest)}件 → {OUT_DIR}')
print(f'期待値manifest: {manifest_path}')
