#!/usr/bin/env python3
"""wav審 — 独立path波形検証器 (被審Rust code非参照·wav bytesのみ実測).
役=審具丁. 対象=環統合 梯3 (z→音) 出力wav. 依存=stdlib only (numpy無).
使用: python3 wav審.py <wav path> [--pretty]
出力: JSON (stdout).

法: 推定値と実測値を混ぜるな — 本ファイルは実測のみ. 期待値は合成試験.py側で別途算出.
"""
import sys
import json
import struct
import cmath
import math


# ---------------------------------------------------------------------------
# RIFF/fmt/data chunk 自前parse (Python `wave` module不使用 — 独立性維持)
# ---------------------------------------------------------------------------

def _u16(b, o):
    return struct.unpack_from('<H', b, o)[0]


def _u32(b, o):
    return struct.unpack_from('<I', b, o)[0]


def _i16(b, o):
    return struct.unpack_from('<h', b, o)[0]


def parse_riff(raw: bytes) -> dict:
    """RIFF/WAVE chunk構造を自前parse. 例外は投げず notes へ異常記録."""
    notes = []
    if len(raw) < 12 or raw[0:4] != b'RIFF' or raw[8:12] != b'WAVE':
        return {'riff_ok': False, 'wave_ok': False, 'notes': ['RIFF/WAVEヘッダ不整合 — 先頭12byte異常']}

    riff_declared_size = _u32(raw, 4)
    chunks = []  # (id, size, body_start, body_end_clipped)
    pos = 12
    n = len(raw)
    while pos + 8 <= n:
        cid = raw[pos:pos + 4]
        try:
            csize = _u32(raw, pos + 4)
        except struct.error:
            notes.append(f'chunk size読取失敗 @pos={pos}')
            break
        body_start = pos + 8
        body_end = body_start + csize
        clipped = False
        if body_end > n:
            notes.append(f'chunk {cid!r} 宣言size={csize} がfile末尾を超過 — 切詰 (実際={n - body_start})')
            body_end = n
            clipped = True
        chunks.append({'id': cid, 'declared_size': csize, 'start': body_start, 'end': body_end, 'clipped': clipped})
        pos = body_start + csize + (csize % 2)  # 奇数sizeはpad byte 1つ

    fmt_chunk = next((c for c in chunks if c['id'] == b'fmt '), None)
    data_chunk = next((c for c in chunks if c['id'] == b'data'), None)
    if fmt_chunk is None:
        notes.append('fmt chunk不在')
    if data_chunk is None:
        notes.append('data chunk不在')

    result = {
        'riff_ok': True, 'wave_ok': True,
        'riff_declared_chunk_size': riff_declared_size,
        'actual_file_size_bytes': n,
        'chunks_found': [{'id': c['id'].decode('ascii', 'replace'), 'declared_size': c['declared_size']} for c in chunks],
        'notes': notes,
    }

    if fmt_chunk is None or data_chunk is None:
        return result

    fb = raw[fmt_chunk['start']:fmt_chunk['end']]
    if len(fb) < 16:
        notes.append(f'fmt chunk短すぎ ({len(fb)} byte < 16)')
        return result

    audio_format = _u16(fb, 0)
    num_channels = _u16(fb, 2)
    sample_rate = _u32(fb, 4)
    byte_rate = _u32(fb, 8)
    block_align = _u16(fb, 12)
    bits_per_sample = _u16(fb, 14)

    actual_format = audio_format
    if audio_format == 0xFFFE and len(fb) >= 40:
        # WAVE_FORMAT_EXTENSIBLE — subformat GUID先頭2byteが真の format code
        try:
            actual_format = _u16(fb, 24)
            notes.append(f'WAVE_FORMAT_EXTENSIBLE検出 — subformat先頭={actual_format} を実効formatとして使用')
        except struct.error:
            notes.append('EXTENSIBLE subformat読取失敗 — 宣言audio_formatをそのまま使用')

    result['fmt'] = {
        'chunk_size_bytes': len(fb),
        'audio_format_code': audio_format,
        'effective_format_code': actual_format,
        'num_channels': num_channels,
        'sample_rate_hz': sample_rate,
        'byte_rate_bytes_per_sec': byte_rate,
        'block_align_bytes': block_align,
        'bits_per_sample': bits_per_sample,
    }

    data_bytes = raw[data_chunk['start']:data_chunk['end']]
    data_declared_size = data_chunk['declared_size']
    data_actual_present = len(data_bytes)

    expected_byte_rate = sample_rate * num_channels * (bits_per_sample // 8) if bits_per_sample % 8 == 0 else None
    expected_block_align = num_channels * (bits_per_sample // 8) if bits_per_sample % 8 == 0 else None

    remainder = data_actual_present % block_align if block_align else -1
    frame_count = data_actual_present // block_align if block_align else 0

    consistency = {
        'byte_rate_expected_from_rate_ch_bits': expected_byte_rate,
        'byte_rate_matches': (expected_byte_rate == byte_rate) if expected_byte_rate is not None else None,
        'block_align_expected_from_ch_bits': expected_block_align,
        'block_align_matches': (expected_block_align == block_align) if expected_block_align is not None else None,
        'data_size_multiple_of_block_align': (remainder == 0) if block_align else None,
        'data_size_remainder_bytes': remainder,
        'riff_declared_vs_actual_file_size_delta_bytes': (n - 8) - riff_declared_size,
        'data_declared_size_bytes': data_declared_size,
        'data_actual_bytes_present': data_actual_present,
        'data_declared_vs_actual_match': data_declared_size == data_actual_present,
        'data_size_delta_bytes': data_declared_size - data_actual_present,
    }

    result['data'] = {
        'declared_size_bytes': data_declared_size,
        'actual_bytes_present': data_actual_present,
        'frame_count': frame_count,
        'duration_declared_sec': (frame_count / sample_rate) if sample_rate else None,
        'raw_bytes': data_bytes,  # decode側で使用、JSON化直前に除去する
    }
    result['consistency'] = consistency
    result['notes'] = notes
    return result


# ---------------------------------------------------------------------------
# sample decode
# ---------------------------------------------------------------------------

def decode_samples(data_bytes: bytes, fmt: dict, frame_count: int):
    """raw bytes → チャネル別 正規化float list (-1..1想定, 32bit float型はそのまま).
    返却: (channels: list[list[float]], peak_raw, strict_zero: bool)
    """
    ch = fmt['num_channels']
    bits = fmt['bits_per_sample']
    fmt_code = fmt['effective_format_code']
    bytes_per_sample = bits // 8

    channels = [[] for _ in range(ch)]
    peak_raw = 0
    strict_zero = True
    is_float = (fmt_code == 3)

    for i in range(frame_count):
        base = i * fmt['block_align_bytes']
        for c in range(ch):
            off = base + c * bytes_per_sample
            if is_float and bits == 32:
                v = struct.unpack_from('<f', data_bytes, off)[0]
                raw = v
                norm = v
            elif is_float and bits == 64:
                v = struct.unpack_from('<d', data_bytes, off)[0]
                raw = v
                norm = v
            elif bits == 8:
                raw = data_bytes[off]  # unsigned 8bit, 中心128
                norm = (raw - 128) / 128.0
            elif bits == 16:
                raw = struct.unpack_from('<h', data_bytes, off)[0]
                norm = raw / 32768.0
            elif bits == 24:
                b0, b1, b2 = data_bytes[off], data_bytes[off + 1], data_bytes[off + 2]
                raw = b0 | (b1 << 8) | (b2 << 16)
                if raw & 0x800000:
                    raw -= 0x1000000
                norm = raw / 8388608.0
            elif bits == 32:
                raw = struct.unpack_from('<i', data_bytes, off)[0]
                norm = raw / 2147483648.0
            else:
                raise ValueError(f'未対応bit深: {bits}')
            channels[c].append(norm)
            if raw != 0:
                strict_zero = False
            ar = abs(raw)
            if ar > peak_raw:
                peak_raw = ar

    return channels, peak_raw, strict_zero


# ---------------------------------------------------------------------------
# 自前FFT (radix-2 Cooley-Tukey, stdlib cmath のみ)
# ---------------------------------------------------------------------------

def fft(a):
    """a: list[complex], len(a)=2^k 必須. in-place不要 (新list返却)."""
    n = len(a)
    if n <= 1:
        return list(a)
    if n & (n - 1) != 0:
        raise ValueError('fft: 長さは2の冪である事')
    # bit-reversal permutation
    j = 0
    out = list(a)
    for i in range(1, n):
        bit = n >> 1
        while j & bit:
            j ^= bit
            bit >>= 1
        j |= bit
        if i < j:
            out[i], out[j] = out[j], out[i]
    length = 2
    while length <= n:
        ang = -2 * math.pi / length
        wlen = cmath.exp(1j * ang)
        for start in range(0, n, length):
            w = 1 + 0j
            half = length // 2
            for k in range(half):
                u = out[start + k]
                v = out[start + k + half] * w
                out[start + k] = u + v
                out[start + k + half] = u - v
                w *= wlen
        length <<= 1
    return out


def hann(n):
    if n == 1:
        return [1.0]
    return [0.5 - 0.5 * math.cos(2 * math.pi * i / (n - 1)) for i in range(n)]


def next_pow2_le(n):
    """n以下の最大の2冪 (zero padding不使用 — 分解能を偽らぬ為, 末尾crop方式)."""
    p = 1
    while p * 2 <= n:
        p *= 2
    return p


MIN_PEAK_BIN_SEPARATION = 4  # Hann主lobe幅 ≈4bin — この間隔未満は同一peakと見做し非最大抑制


def spectrum_analysis(mono: list, sample_rate: int):
    total = len(mono)
    if total < 2 or sample_rate <= 0:
        return {'error': '標本不足 or sample_rate不正 — spectrum解析skip', 'usable': False}

    n_fft = next_pow2_le(total)
    if n_fft < 2:
        return {'error': f'標本数{total}はFFT最小長未満 — spectrum解析skip', 'usable': False}
    cropped = total - n_fft

    mean = sum(mono[:n_fft]) / n_fft
    win = hann(n_fft)
    windowed = [(mono[i] - mean) * win[i] for i in range(n_fft)]
    complex_in = [complex(v, 0.0) for v in windowed]
    spec = fft(complex_in)

    half = n_fft // 2
    mags = [abs(spec[k]) for k in range(half + 1)]
    resolution = sample_rate / n_fft

    # peak探索: DC(bin0)除外, 降順走査+非最大抑制
    order = sorted(range(1, half + 1), key=lambda k: mags[k], reverse=True)
    picked = []
    for k in order:
        if all(abs(k - p) >= MIN_PEAK_BIN_SEPARATION for p in picked):
            picked.append(k)
        if len(picked) >= 3:
            break

    peaks = []
    for rank, k in enumerate(picked, start=1):
        # 放物線内挿 (3点, 振幅ドメイン) — 主lobe中心の精密化
        if 1 <= k <= half - 1:
            a0, b0, g0 = mags[k - 1], mags[k], mags[k + 1]
            denom = (a0 - 2 * b0 + g0)
            p = 0.5 * (a0 - g0) / denom if denom != 0 else 0.0
            p = max(-1.0, min(1.0, p))
        else:
            p = 0.0
        refined_bin = k + p
        peaks.append({
            'rank': rank,
            'bin_index': k,
            'raw_bin_freq_hz': k * resolution,
            'refined_freq_hz': refined_bin * resolution,
            'magnitude_relative': mags[k],
            'error_bound_hz': resolution / 2.0,
        })

    max_mag = max(mags[1:], default=0.0)
    note = None
    if max_mag < 1e-9:
        note = '信号がほぼ無音 (最大magnitude≈0) — peak周波数は雑音床由来で無意味, 参考値として出力のみ'

    return {
        'usable': True,
        'method': 'radix-2 FFT (自前実装, cmath) + Hann窓 + 3点放物線内挿, DC成分は平均減算で除去',
        'fft_length_used': n_fft,
        'total_frames_available': total,
        'frames_cropped_from_tail': cropped,
        'frequency_resolution_hz': resolution,
        'frequency_uncertainty_note': '±resolution/2 (peaks[].error_bound_hz) — raw_bin基準. refined_freq_hzは内挿値で誤差はこれより小さいが保証値ではない',
        'min_peak_bin_separation': MIN_PEAK_BIN_SEPARATION,
        'silent_signal_note': note,
        'peaks': peaks,
    }


# ---------------------------------------------------------------------------
# 振幅 (RMS/peak/厳密無音)
# ---------------------------------------------------------------------------

def amplitude_stats(mono: list, peak_raw, strict_zero: bool):
    n = len(mono)
    if n == 0:
        return {'rms_normalized': 0.0, 'peak_abs_normalized': 0.0, 'peak_abs_raw': peak_raw, 'strict_silence_peak_is_exact_zero': strict_zero}
    ss = sum(v * v for v in mono)
    rms = math.sqrt(ss / n)
    peak_norm = max(abs(v) for v in mono)
    return {
        'rms_normalized': rms,
        'peak_abs_normalized': peak_norm,
        'peak_abs_raw': peak_raw,
        'strict_silence_peak_is_exact_zero': strict_zero,
    }


# ---------------------------------------------------------------------------
# 包絡解析 (有音/無音境界, gate周期, duty)
# ---------------------------------------------------------------------------

FRAME_MS = 1.0          # frame長 (ms) — 粗boundary検出用
COARSE_THRESH_FRAC = 0.10   # frame RMSしきい値 = peak_abs * この割合
FINE_THRESH_FRAC = 0.01     # sample単位精密化しきい値 = peak_abs * この割合


def envelope_analysis(mono: list, sample_rate: int, peak_abs: float):
    n = len(mono)
    if n == 0 or peak_abs == 0.0:
        return {'gate_detected': False, 'reason': '無音 or 標本無 — 包絡解析不能', 'segments': []}

    frame_len = max(1, round(sample_rate * FRAME_MS / 1000.0))
    n_frames = (n + frame_len - 1) // frame_len
    coarse_thresh = peak_abs * COARSE_THRESH_FRAC
    fine_thresh = peak_abs * FINE_THRESH_FRAC

    frame_active = []
    for f in range(n_frames):
        s = f * frame_len
        e = min(n, s + frame_len)
        seg = mono[s:e]
        rms = math.sqrt(sum(v * v for v in seg) / len(seg))
        frame_active.append(rms >= coarse_thresh)

    # frame runs → coarse boundary (frame idx)
    runs = []  # (state, frame_start, frame_end_excl)
    cur = frame_active[0]
    run_start = 0
    for f in range(1, n_frames):
        if frame_active[f] != cur:
            runs.append((cur, run_start, f))
            run_start = f
            cur = frame_active[f]
    runs.append((cur, run_start, n_frames))

    def refine_onset(sample_guess):
        lo = max(0, sample_guess - frame_len)
        hi = min(n, sample_guess + frame_len)
        for i in range(lo, hi):
            if abs(mono[i]) >= fine_thresh:
                return i
        return sample_guess

    def refine_offset(sample_guess):
        lo = max(0, sample_guess - frame_len)
        hi = min(n, sample_guess + frame_len)
        last_active = sample_guess
        for i in range(lo, hi):
            if abs(mono[i]) >= fine_thresh:
                last_active = i
        return last_active + 1  # 非活性化した最初のindex

    segments = []
    for state, fs, fe in runs:
        if not state:
            continue
        guess_start = fs * frame_len
        guess_end = min(n, fe * frame_len)
        start_i = refine_onset(guess_start)
        end_i = refine_offset(max(start_i, guess_end - 1))
        end_i = max(end_i, start_i + 1)
        end_i = min(end_i, n)
        segments.append({
            'start_sample': start_i, 'end_sample': end_i,
            'start_sec': start_i / sample_rate, 'end_sec': end_i / sample_rate,
            'duration_sec': (end_i - start_i) / sample_rate,
        })

    if len(segments) < 1:
        return {'gate_detected': False, 'reason': '有音区間0 — gate無 or 連続無音', 'segments': segments}
    if len(segments) < 2:
        return {'gate_detected': False, 'reason': '有音区間1個のみ — gate周期算出不能 (連続音の可能性)', 'segments': segments}

    onsets = [s['start_sample'] / sample_rate for s in segments]
    periods = [onsets[i + 1] - onsets[i] for i in range(len(onsets) - 1)]
    duties = [s['duration_sec'] / p for s, p in zip(segments[:-1], periods)] if periods else []

    period_mean = sum(periods) / len(periods)
    period_var = sum((p - period_mean) ** 2 for p in periods) / len(periods)
    period_std = math.sqrt(period_var)
    duty_mean = sum(duties) / len(duties) if duties else None
    duty_var = sum((d - duty_mean) ** 2 for d in duties) / len(duties) if duties else None
    duty_std = math.sqrt(duty_var) if duty_var is not None else None

    return {
        'gate_detected': True,
        'frame_len_samples_used_for_coarse': frame_len,
        'coarse_threshold_normalized': coarse_thresh,
        'fine_threshold_normalized': fine_thresh,
        'n_segments': len(segments),
        'segments': segments,
        'gate_period_sec_mean': period_mean,
        'gate_period_sec_std': period_std,
        'gate_period_samples_all': periods,
        'duty_mean': duty_mean,
        'duty_std': duty_std,
        'duty_samples_all': duties,
    }


# ---------------------------------------------------------------------------
# 段差 (click) 検出
# ---------------------------------------------------------------------------

CLICK_FACTOR = 3.0       # 期待最大差分の何倍を段差と見做すか
CLICK_ABS_FLOOR = 1e-4   # 無音区間での過検出防止 下限 (正規化振幅)
CLUSTER_GAP = 3           # この標本数以内に隣接するflagは同一click eventへ併合


def discontinuity_analysis(mono: list, sample_rate: int, dominant_freq_hz, peak_abs: float, gate_segments):
    n = len(mono)
    if n == 0:
        return {'click_present': False, 'events': [], 'note': '標本無'}

    f_dom = dominant_freq_hz if dominant_freq_hz else 0.0
    expected_max_diff = 2 * math.pi * f_dom * peak_abs / sample_rate if sample_rate else 0.0
    thresh = max(expected_max_diff * CLICK_FACTOR, CLICK_ABS_FLOOR)

    # 仮想無音 (file前後) を含めた拡張差分列
    ext = [0.0] + mono + [0.0]
    flags = []
    for i in range(1, len(ext)):
        d = ext[i] - ext[i - 1]
        if abs(d) > thresh:
            flags.append((i - 1, d))  # sample index系はmono基準 (0-indexed, i-1: 0=先頭virtual→0番目実sample直前)

    # index系を実sample indexへ揃える: ext[0]=virtual, ext[1..n]=mono[0..n-1], ext[n+1]=virtual
    # flag位置 i-1 が指すのは ext[i]-ext[i-1] の"到達側" (i-1). i=1→実sample0 (先頭差), i=n+1→末尾virtual後 (末尾差, sample=n-1扱い)
    events_raw = []
    for i, d in flags:
        idx = min(max(i, 0), n - 1)
        events_raw.append((idx, d))

    # cluster併合
    events_raw.sort(key=lambda t: t[0])
    clusters = []
    for idx, d in events_raw:
        if clusters and idx - clusters[-1]['last_idx'] <= CLUSTER_GAP:
            if abs(d) > abs(clusters[-1]['delta']):
                clusters[-1]['delta'] = d
                clusters[-1]['sample_index'] = idx
            clusters[-1]['last_idx'] = idx
        else:
            clusters.append({'sample_index': idx, 'delta': d, 'last_idx': idx})

    boundary_idx = set()
    for seg in (gate_segments or []):
        boundary_idx.add(seg['start_sample'])
        boundary_idx.add(seg['end_sample'] - 1)
        boundary_idx.add(seg['end_sample'])

    events = []
    for c in clusters:
        idx = c['sample_index']
        kind = 'unexpected_click'
        if idx == 0:
            kind = 'head_discontinuity'
        elif idx == n - 1:
            kind = 'tail_discontinuity'
        elif any(abs(idx - b) <= CLUSTER_GAP for b in boundary_idx):
            kind = 'gate_boundary_click'
        events.append({
            'sample_index': idx,
            'time_sec': idx / sample_rate,
            'delta_normalized': c['delta'],
            'kind': kind,
        })

    return {
        'click_threshold_normalized': thresh,
        'expected_max_sample_diff_normalized': expected_max_diff,
        'dominant_freq_hz_used_for_expectation': f_dom,
        'click_present': len(events) > 0,
        'n_click_events': len(events),
        'events': events,
    }


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

def analyze(path: str) -> dict:
    with open(path, 'rb') as f:
        raw = f.read()

    parsed = parse_riff(raw)
    out = {'file': path, 'header': {}}
    out['header'] = {k: v for k, v in parsed.items() if k not in ('notes',)}
    out['notes'] = list(parsed.get('notes', []))

    if 'data' not in parsed or 'fmt' not in parsed:
        out['error'] = 'fmt/data chunk不在 — 以降の解析不能'
        return out

    fmt = parsed['fmt']
    data_bytes = parsed['data']['raw_bytes']
    frame_count = parsed['data']['frame_count']
    del out['header']['data']['raw_bytes']  # JSON化前にbytes除去 (header出力からも消す)

    if fmt['effective_format_code'] not in (1, 3):
        out['error'] = f"未対応audio_format_code={fmt['effective_format_code']} (対応=1:PCM整数, 3:IEEE float)"
        return out
    if fmt['bits_per_sample'] not in (8, 16, 24, 32, 64):
        out['error'] = f"未対応bits_per_sample={fmt['bits_per_sample']}"
        return out

    channels, peak_raw, strict_zero = decode_samples(data_bytes, fmt, frame_count)
    ch_n = len(channels)
    if ch_n == 0 or frame_count == 0:
        out['error'] = '標本0 — 以降解析不能'
        return out
    if ch_n == 1:
        mono = channels[0]
    else:
        mono = [sum(vals) / ch_n for vals in zip(*channels)]

    out['amplitude'] = amplitude_stats(mono, peak_raw, strict_zero)
    spec = spectrum_analysis(mono, fmt['sample_rate_hz'])
    out['spectrum'] = spec

    dominant = None
    if spec.get('usable') and spec.get('peaks'):
        dominant = spec['peaks'][0]['refined_freq_hz']

    env = envelope_analysis(mono, fmt['sample_rate_hz'], out['amplitude']['peak_abs_normalized'])
    out['envelope'] = env

    disc = discontinuity_analysis(mono, fmt['sample_rate_hz'], dominant, out['amplitude']['peak_abs_normalized'], env.get('segments'))
    out['discontinuity'] = disc

    out['analysis_note'] = 'mono解析はチャネル平均downmix. RIFF/fmt/data chunkは本ファイル内で自前parse (Python wave module不使用)。被審Rust codeの値は一切参照していない。'
    return out


def main():
    if len(sys.argv) < 2:
        print('使用: python3 wav審.py <wav path> [--pretty]', file=sys.stderr)
        sys.exit(2)
    path = sys.argv[1]
    pretty = '--pretty' in sys.argv[2:]
    result = analyze(path)
    if pretty:
        print(json.dumps(result, ensure_ascii=False, indent=2, default=str))
    else:
        print(json.dumps(result, ensure_ascii=False, default=str))


if __name__ == '__main__':
    main()
