//! 歌口 tests 共通 — 合成波生成 + 独立導出式 (期待値oracle). hardcode禁: 全て式から.
//! 真値源=docs/adversary/2026-08-11-歌口数審.md 定1/定2 (建根乙裁定 08-11 14:31) + src/写像.rs 契約.
#![allow(dead_code)]

pub const TAU: f64 = std::f64::consts::TAU;

/// 正弦波 (振幅amp, 位相0起点).
pub fn 正弦(hz: f64, sr: u32, n: usize, amp: f64) -> Vec<f32> {
    (0..n)
        .map(|i| (amp * (TAU * hz * i as f64 / sr as f64).sin()) as f32)
        .collect()
}

/// 矩形波 (sign(sin) — フーリエ級数上, 奇数倍音のみ持つ既知信号).
pub fn 矩形(hz: f64, sr: u32, n: usize, amp: f64) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let s = (TAU * hz * i as f64 / sr as f64).sin();
            (amp * s.signum()) as f32
        })
        .collect()
}

/// 鋸波 (位相0..1を-1..1へ線形写像 — 全倍音を持つ既知信号).
pub fn 鋸(hz: f64, sr: u32, n: usize, amp: f64) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let phase = (hz * i as f64 / sr as f64).rem_euclid(1.0);
            (amp * (2.0 * phase - 1.0)) as f32
        })
        .collect()
}

/// 倍音混合波 — (倍数,振幅比)表を基音へ重畳 (例: [(1.0,1.0),(2.0,0.5),(3.0,0.33)]).
pub fn 倍音波(基音: f64, 表: &[(f64, f64)], sr: u32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let t = i as f64 / sr as f64;
            let s: f64 = 表
                .iter()
                .map(|&(倍, 振)| 振 * (TAU * 基音 * 倍 * t).sin())
                .sum();
            s as f32
        })
        .collect()
}

/// 決定論的白色雑音 — xorshift64 (外部crate不使用, 再現可能・独立実装).
pub fn 白色雑音(n: usize, amp: f64, seed: u64) -> Vec<f32> {
    let mut s = seed | 1; // 0除け
    let mut 次 = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    (0..n)
        .map(|_| {
            let u = (次() >> 11) as f64 / (1u64 << 53) as f64; // [0,1)
            (amp * (2.0 * u - 1.0)) as f32
        })
        .collect()
}

/// L = log2(hz/基音) の期待値 (真値源=注入周波数そのもの — 検出hzのノイズに依らぬ).
pub fn 期待l(hz: f64, 基音: f64) -> f64 {
    (hz / 基音).log2()
}

/// 定1 round規約: lap = floor(L+0.5).
pub fn 期待lap(l: f64) -> i64 {
    (l + 0.5).floor() as i64
}

/// 定1: θ = 全環·(L - lap).
pub fn 期待theta(l: f64) -> f64 {
    TAU * (l - (l + 0.5).floor())
}

/// 定2: L2 = round(L·家数)/家数 (家snap時, lap/θ算出前のL域量子化).
pub fn 期待l2(l: f64, 家数: u32) -> f64 {
    (l * 家数 as f64 + 0.5).floor() / 家数 as f64
}

/// 総角 = 全環·L (定1不変式 — lap/theta分割に依らぬ真値. 誤差<1e-12を主張根拠とする).
pub fn 期待総角(l: f64) -> f64 {
    TAU * l
}

/// hzの相対誤差から導くθ許容 (dθ/dL=全環, dL/dhz≈1/(hz·ln2) ⟹ θ誤差≈全環/ln2·相対誤差).
pub fn theta許容(相対誤差上限: f64) -> f64 {
    TAU / std::f64::consts::LN_2 * 相対誤差上限
}

/// hzの相対誤差ε許容から導くcent許容 (定義そのもの: 1200·log2(1+ε)).
pub fn cent許容(相対誤差上限: f64) -> f64 {
    1200.0 * (1.0 + 相対誤差上限).log2()
}

pub fn hz相対誤差(検出: f64, 真値: f64) -> f64 {
    (検出 - 真値).abs() / 真値
}

pub fn cent誤差(検出: f64, 真値: f64) -> f64 {
    1200.0 * (検出 / 真値).log2()
}
