#!/usr/bin/env python3
"""歌口注入生成 — 梯5 (mic→z) 敵対審査用 既知信号corpus.
役=批根丁 (二季). 依存=stdlib only. `wave`/numpy 不使用 (手書きRIFF — 解析器wav審.pyと別経路)。
被審code (機関/歌口) を一切参照せず, 契約 文書/環統合.md からのみ期待値を導く。

契約写像 (環統合.md 界面写像表 音高行):
  L = log2(f / 基音)      基音=220.0 Hz (梯3 音高.rs と同値, param既定)
  lap = floor(L)          octave = 巻
  frac = L - lap          環内位置 [0,1)
  theta = 2*pi*frac       角
  家 = round(frac*8) mod 8   八家snap (家数=param)
  r = 音量 (振幅)

出力: docs/adversary/具/合成歌/*.wav + 注入目録.json (期待値manifest)
"""
import os
import math
import struct
import json

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '合成歌')
os.makedirs(OUT, exist_ok=True)

基音 = 220.0
既定率 = 48000


# ---- 手書きRIFF (wave module不使用) -------------------------------------
def 書wav(path, rate, ch, samples_f, bits=16):
    """samples_f: mono list[float] または ch=2なら [(l,r),...]"""
    block = ch * (bits // 8)
    data = bytearray()
    flat = []
    for v in samples_f:
        if ch == 2 and isinstance(v, tuple):
            flat.extend(v)
        else:
            flat.append(v)
    for v in flat:
        if bits == 16:
            iv = max(-32768, min(32767, round(v * 32767)))
            data += struct.pack('<h', iv)
        elif bits == 8:
            iv = max(0, min(255, round(v * 127) + 128))
            data += struct.pack('<B', iv)
        else:
            raise ValueError(bits)
    fmt = struct.pack('<HHIIHH', 1, ch, rate, rate * block, block, bits)
    riff = 4 + (8 + len(fmt)) + (8 + len(data))
    with open(path, 'wb') as f:
        f.write(b'RIFF' + struct.pack('<I', riff) + b'WAVE')
        f.write(b'fmt ' + struct.pack('<I', len(fmt)) + fmt)
        f.write(b'data' + struct.pack('<I', len(data)) + bytes(data))
    return len(data)


# ---- 素波 ---------------------------------------------------------------
def sine(f, dur, rate, amp, ph=0.0):
    n = round(dur * rate)
    return [amp * math.sin(2 * math.pi * f * i / rate + ph) for i in range(n)]


def 加算(*波列):
    n = min(len(w) for w in 波列)
    return [sum(w[i] for w in 波列) for i in range(n)]


def 倍音(f, dur, rate, amp, 係数):
    """係数 = {次数: 相対振幅}. 次数1を欠けば missing fundamental."""
    n = round(dur * rate)
    out = []
    for i in range(n):
        t = i / rate
        v = sum(a * math.sin(2 * math.pi * f * k * t) for k, a in 係数.items())
        out.append(amp * v)
    m = max(abs(v) for v in out) or 1.0
    return [v * amp / m for v in out]


def 鋸歯(f, dur, rate, amp, 次数上限=20):
    係数 = {k: 1.0 / k for k in range(1, 次数上限 + 1)}
    return 倍音(f, dur, rate, amp, 係数)


def 矩形(f, dur, rate, amp, 次数上限=21):
    係数 = {k: 1.0 / k for k in range(1, 次数上限 + 1, 2)}
    return 倍音(f, dur, rate, amp, 係数)


def 白色(dur, rate, amp, seed=12345):
    """決定論LCG — 同seed→bit一致 (再現性契約C11の為)"""
    n = round(dur * rate)
    s = seed
    out = []
    for _ in range(n):
        s = (1103515245 * s + 12345) & 0x7FFFFFFF
        out.append(amp * ((s / 0x3FFFFFFF) - 1.0))
    return out


def 揺(f0, dur, rate, amp, 深cent, 速hz):
    """vibrato: 位相積分で生成 (周波数を直接時変させると位相跳ぶ)"""
    n = round(dur * rate)
    ph, out = 0.0, []
    for i in range(n):
        t = i / rate
        f = f0 * (2.0 ** ((深cent * math.sin(2 * math.pi * 速hz * t)) / 1200.0))
        ph += 2 * math.pi * f / rate
        out.append(amp * math.sin(ph))
    return out


def 滑走(f0, f1, dur, rate, amp):
    """log線形glide, 位相積分"""
    n = round(dur * rate)
    ph, out = 0.0, []
    for i in range(n):
        u = i / n
        f = f0 * (f1 / f0) ** u
        ph += 2 * math.pi * f / rate
        out.append(amp * math.sin(ph))
    return out


def 立上(f, dur, rate, amp, 無音秒, 傾秒):
    """無音→線形fade-in→定常. 出現律 (r連続立上り) 検査用"""
    n = round(dur * rate)
    n0 = round(無音秒 * rate)
    nr = round(傾秒 * rate)
    out = []
    for i in range(n):
        if i < n0:
            out.append(0.0)
            continue
        j = i - n0
        env = min(1.0, j / nr) if nr > 0 else 1.0
        out.append(amp * env * math.sin(2 * math.pi * f * j / rate))
    return out


def 急起(f, dur, rate, amp, 無音秒):
    """厳密無音 → 標本index境界で即開始 (遅延測定用 陽性対照)"""
    return 立上(f, dur, rate, amp, 無音秒, 0.0)


def 門(f, dur, rate, amp, gate_hz=2.0, duty=0.5):
    n = round(dur * rate)
    T = 1.0 / gate_hz
    return [amp * math.sin(2 * math.pi * f * (i / rate)) if (i / rate) % T < duty * T else 0.0
            for i in range(n)]


# ---- 期待値 (契約から導出, 被審code非参照) -------------------------------
def 期待z(f, r):
    L = math.log2(f / 基音)
    lap = math.floor(L)
    frac = L - lap
    return {
        '期待freq_hz': f,
        '期待L': L,
        '期待lap': lap,
        '期待frac': frac,
        '期待theta_rad': 2 * math.pi * frac,
        '期待家': round(frac * 8) % 8,
        '期待r': r,
    }


def 家freq(lap, h):
    return 基音 * (2.0 ** lap) * (2.0 ** (h / 8.0))


def cent(f, c):
    return f * (2.0 ** (c / 1200.0))


目録 = []


def 登録(名, 波, 攻撃, 期待, rate=既定率, ch=1, bits=16):
    path = os.path.join(OUT, 名 + '.wav')
    書wav(path, rate, ch, 波, bits)
    e = dict(期待)
    e.update({'名': 名, 'wav': os.path.relpath(path, os.path.dirname(OUT)),
              '攻撃': 攻撃, 'sample率': rate, 'ch': ch, 'bits': bits,
              '秒': round(len(波) / rate, 6)})
    目録.append(e)


def 生成():
    R = 既定率
    # C1 音高精度: 家梯子 lap0-3 × 家0/1/3/7 (梯3 乙.4-a と同一格子 = 逆写像対照)
    for lap in (0, 1, 2, 3):
        for h in (0, 1, 3, 7):
            f = 家freq(lap, h)
            登録(f'C1_家梯子_lap{lap}_家{h}', sine(f, 2.0, R, 0.5), 'C1 音高精度',
                 期待z(f, 0.5))

    # C2 octave境界: 巻の境目 ±cent
    for lap in (1, 2):
        f0 = 基音 * 2.0 ** lap
        for c, tag in ((0.0, '厳密'), (-1.0, '下1cent'), (+1.0, '上1cent'),
                       (-5.0, '下5cent'), (+5.0, '上5cent')):
            f = cent(f0, c)
            登録(f'C2_巻境_lap{lap}_{tag}', sine(f, 2.0, R, 0.5), 'C2 octave境界lap誤判',
                 期待z(f, 0.5))

    # C3 家境界 tie-break (frac = (2h+1)/16 — 梯2 丙-D2 202.5°の音版)
    for k in range(8):
        frac = (2 * k + 1) / 16.0
        f = 基音 * 2.0 ** frac
        登録(f'C3_家境_{k}', sine(f, 2.0, R, 0.5), 'C3 家境界tie-break',
             期待z(f, 0.5))

    # C4 無 (厳密零) — 契約 z=0 ⟺ r=0 ∧ lap=0
    登録('C4_無音_厳密', [0.0] * (2 * R), 'C4 無音→z=0 (r=0契約)',
         {'期待r': 0.0, '期待lap': 0, '期待theta_rad': None, '期待家': None, '期待freq_hz': None})

    # C5 雑音 (音高無し) — 偶発zを出してはならぬ
    for amp, tag in ((0.5, '大'), (0.05, '小'), (0.005, '微')):
        登録(f'C5_白色_{tag}', 白色(2.0, R, amp, seed=4242), 'C5 雑音→偶発z禁',
             {'期待r': 0.0, '期待lap': 0, '期待theta_rad': None, '期待家': None,
              '期待freq_hz': None, '註': '確度gate必須. 音高断定=不合格'})

    # C6 準無音・非音高源 (hum/DC/微小)
    登録('C6_hum50', sine(50.0, 2.0, R, 0.02), 'C6 電源hum→無',
         {'期待r': 0.0, '期待lap': None, '期待freq_hz': 50.0, '註': '下限param外→無 が契約'})
    登録('C6_hum60', sine(60.0, 2.0, R, 0.02), 'C6 電源hum→無',
         {'期待r': 0.0, '期待lap': None, '期待freq_hz': 60.0})
    登録('C6_直流', [0.3] * (2 * R), 'C6 DC offset→無',
         {'期待r': 0.0, '期待lap': 0, '期待freq_hz': None})
    登録('C6_微小440', sine(440.0, 2.0, R, 1e-4), 'C6 微小振幅→無 or r≈0',
         {'期待r': 0.0, '期待freq_hz': 440.0, '註': '振幅gate下限paramの位置を露呈させる'})

    # C7 octave誤り古典: missing fundamental / 倍音豊富
    登録('C7_基音欠_220', 倍音(220.0, 2.0, R, 0.5, {2: 1.0, 3: 0.7, 4: 0.5, 5: 0.35}),
         'C7 missing fundamental→lap+1誤り',
         dict(期待z(220.0, 0.5), **{'註': '真の音高=220 (基音欠). FFT最大peak=440 → 素朴実装はlap+1'}))
    登録('C7_鋸歯_220', 鋸歯(220.0, 2.0, R, 0.5), 'C7 倍音豊富→lap誤り',
         期待z(220.0, 0.5))
    登録('C7_矩形_293_66', 矩形(293.6648, 2.0, R, 0.5), 'C7 奇数倍音→lap誤り',
         期待z(293.6648, 0.5))
    登録('C7_下octave混入_440', 加算(sine(440.0, 2.0, R, 0.5), sine(220.0, 2.0, R, 0.15)),
         'C7 弱い下octave混入→lap−1誤り', dict(期待z(440.0, 0.5),
         **{'註': '支配=440. 220混入15%で lap を落とすなら不合格'}))

    # C8 揺れ/滑走 — lap chatter (梯2 丙-D1の音版) と単調性
    登録('C8_揺_440_50cent', 揺(440.0, 3.0, R, 0.5, 50.0, 6.0), 'C8 vibrato→巻境chatter',
         {'期待lap': 1, '期待freq_hz': 440.0, '註': 'lap は 1 で不動であるべき (±50centは巻を跨がぬ)'})
    登録('C8_揺_440_120cent', 揺(440.0, 3.0, R, 0.5, 120.0, 5.0), 'C8 巻跨ぎvibrato',
         {'期待lap': None, '註': '±120cent=巻境跨ぎ. ヒステリシス無ければlap暴れ. 有界±1が最低条件'})
    登録('C8_滑走_220_880', 滑走(220.0, 880.0, 4.0, R, 0.5), 'C8 glide→lap単調0→2',
         {'期待lap': None, '註': 'lap列は 0→1→2 単調非減少. 逆走/跳躍=不合格. 総角も単調'})
    登録('C8_滑走_880_220', 滑走(880.0, 220.0, 4.0, R, 0.5), 'C8 下降glide→lap単調2→0',
         {'期待lap': None, '註': 'lap列は 2→1→0 単調非増加'})

    # C9 遅延 — 厳密無音1.0s → 標本境界で急起
    登録('C9_急起_440_1s', 急起(440.0, 3.0, R, 0.5, 1.0), 'C9 遅延測定 (onset=標本48000)',
         dict(期待z(440.0, 0.5), **{'onset標本': R, 'onset秒': 1.0,
              '註': 'z が r>0 を報じる最初の時刻 − 1.0s = 実測遅延. 契約=param宣言必須'}))
    登録('C9_急止_440_1s', sine(440.0, 1.0, R, 0.5) + [0.0] * R, 'C9 offset遅延',
         {'offset標本': R, 'offset秒': 1.0, '期待r': 0.0, '註': '無音復帰後は r=0 ∧ z=0 へ落ちる事'})

    # C10 振幅→r 線形 (梯3 B5の逆写像)
    for a in (0.25, 0.5, 1.0):
        登録(f'C10_振幅_{a}', sine(440.0, 2.0, R, a), 'C10 amp→r線形',
             dict(期待z(440.0, a), **{'期待RMS': a / math.sqrt(2)}))

    # C11 出現律 (r連続立上り) — 100ms fade-in
    登録('C11_立上_440_100ms', 立上(440.0, 3.0, R, 0.5, 0.5, 0.1), 'C11 出現律 r連続',
         dict(期待z(440.0, 0.5), **{'註': 'r は 0→0.5 を約100ms掛けて登る. 階段/瞬時跳躍=出現律違反'}))

    # C12 律動 2Hz gate (生命搬送波)
    登録('C12_門2hz_440', 門(440.0, 4.0, R, 0.5), 'C12 2Hz gate入力',
         dict(期待z(440.0, 0.5), **{'gate_hz': 2.0, 'duty': 0.5,
              '註': 'r が 2Hz で 0↔0.5 を往復. 無音区間で lap記憶が壊れぬ事'}))

    # C13 帯域外 — 下限/上限
    登録('C13_低域_27_5', sine(27.5, 2.0, R, 0.5), 'C13 下限外 (lap=−3)', 期待z(27.5, 0.5))
    登録('C13_高域_14080', sine(14080.0, 2.0, R, 0.5), 'C13 高域 (lap=6)', 期待z(14080.0, 0.5))
    登録('C13_超高域_21000', sine(21000.0, 2.0, R, 0.5), 'C13 Nyquist近傍',
         dict(期待z(21000.0, 0.5), **{'註': '48kHz·0.45=21600. 上限param近傍で飽和/警告か折返しか'}))

    # C14 形式頑健性
    登録('C14_44100Hz_440', sine(440.0, 2.0, 44100, 0.5), 'C14 別標本率', 期待z(440.0, 0.5), rate=44100)
    登録('C14_stereo_440_660', [(0.5 * math.sin(2 * math.pi * 440 * i / R),
                                0.5 * math.sin(2 * math.pi * 660 * i / R)) for i in range(2 * R)],
         'C14 stereo (L/R別音高)', {'期待lap': None, '註': 'downmix規約が契約に無ければ不定=欠陥'},
         ch=2)
    登録('C14_8bit_440', sine(440.0, 2.0, R, 0.5), 'C14 8bit PCM', 期待z(440.0, 0.5), bits=8)
    登録('C14_飽和_440', sine(440.0, 2.0, R, 1.6), 'C14 clip入力',
         dict(期待z(440.0, 0.5), **{'註': '振幅1.6→16bit飽和で矩形化. r=1.0上限clamp必須, panic禁'}))

    # C15 多声 — 単声契約の境界
    登録('C15_二声_440_660', 加算(sine(440.0, 2.0, R, 0.35), sine(660.0, 2.0, R, 0.35)),
         'C15 二声同振幅', {'期待lap': None, '註': '単声契約なら決定論的にどちらか一方 or 無. 揺れ=不合格'})

    # C16 決定論 (同wav二回→bit一致) は C1_家梯子_lap1_家0 を再利用
    with open(os.path.join(os.path.dirname(OUT), '注入目録.json'), 'w') as f:
        json.dump({'基音': 基音, '契約源': '文書/環統合.md 界面写像(音高行)',
                   '生成器': 'docs/adversary/具/歌口注入生成.py (批根丁, stdlib手書きRIFF)',
                   '件数': len(目録), '目録': 目録}, f, ensure_ascii=False, indent=1)
    print(f'生成 {len(目録)} 件 → {OUT}')


if __name__ == '__main__':
    生成()
