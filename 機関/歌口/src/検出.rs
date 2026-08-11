//! 音高検出 — YIN / 自己相関。入力一窓→hz・明瞭度・rms。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum 検出法 {
    YIN,
    自己相関,
}

#[derive(Debug, Clone, Copy)]
pub struct 検出param {
    pub 標本率: u32,
    pub 窓長: usize,
    pub 跳幅: usize,
    pub 法: 検出法,
    pub 下限hz: f64,
    pub 上限hz: f64,
    pub 明瞭閾: f64,
    pub 無音閾rms: f64,
}

impl Default for 検出param {
    fn default() -> Self {
        Self {
            標本率: 48_000,
            窓長: 4_096,
            跳幅: 1_024,
            法: 検出法::YIN,
            下限hz: 60.0,
            上限hz: 1_200.0,
            明瞭閾: 0.70,
            無音閾rms: 0.01,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct 検出結果 {
    pub hz: Option<f64>,
    pub 明瞭度: f64,
    pub rms: f64,
}

fn rms(標本: &[f32]) -> f64 {
    if 標本.is_empty() {
        return 0.0;
    }
    let 二乗和: f64 = 標本.iter().map(|&x| (x as f64).powi(2)).sum();
    (二乗和 / 標本.len() as f64).sqrt()
}

fn 範囲(p: &検出param, 標本長: usize) -> Option<(usize, usize)> {
    if p.標本率 == 0 || p.下限hz <= 0.0 || p.上限hz <= p.下限hz || 標本長 < 2 {
        return None;
    }
    let 最小 = ((p.標本率 as f64 / p.上限hz).floor() as usize).max(1);
    let 最大 = ((p.標本率 as f64 / p.下限hz).ceil() as usize).min(標本長 - 1);
    (最小 <= 最大).then_some((最小, 最大))
}

fn 中心化(標本: &[f32]) -> Vec<f64> {
    let 平均 = 標本.iter().map(|&x| x as f64).sum::<f64>() / 標本.len() as f64;
    標本.iter().map(|&x| x as f64 - 平均).collect()
}

fn yin(標本: &[f32], p: &検出param, 最小: usize, 最大: usize) -> (Option<f64>, f64) {
    let x = 中心化(標本);
    let mut 累積 = 0.0;
    let mut 値 = Vec::with_capacity(最大 + 1);
    値.resize(最大 + 1, 1.0);
    for τ in 1..=最大 {
        let 差: f64 = x[..x.len() - τ]
            .iter()
            .zip(&x[τ..])
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        累積 += 差;
        値[τ] = if 累積 > 0.0 {
            差 * τ as f64 / 累積
        } else {
            1.0
        };
    }
    let mut 候補 = None;
    for τ in 最小..=最大 {
        if 値[τ] <= p.明瞭閾.clamp(0.0, 1.0) {
            let mut 谷 = τ;
            while 谷 < 最大 && 値[谷 + 1] < 値[谷] {
                谷 += 1;
            }
            候補 = Some(谷);
            break;
        }
    }
    let τ = 候補.or_else(|| {
        (最小..=最大).min_by(|&a, &b| {
            値[a]
                .partial_cmp(&値[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    let Some(τ) = τ else { return (None, 0.0) };
    let 明瞭度 = (1.0 - 値[τ]).clamp(0.0, 1.0);
    if 明瞭度 < p.明瞭閾 || !明瞭度.is_finite() {
        return (None, 明瞭度);
    }
    let 補正 = if τ > 最小 && τ < 最大 {
        let 分母 = 値[τ - 1] - 2.0 * 値[τ] + 値[τ + 1];
        if 分母.abs() > f64::EPSILON {
            (0.5 * (値[τ - 1] - 値[τ + 1]) / 分母).clamp(-0.5, 0.5)
        } else {
            0.0
        }
    } else {
        0.0
    };
    (Some(p.標本率 as f64 / (τ as f64 + 補正)), 明瞭度)
}

fn 自己相関(
    標本: &[f32], p: &検出param, 最小: usize, 最大: usize
) -> (Option<f64>, f64) {
    let x = 中心化(標本);
    let 基準: f64 = x.iter().map(|a| a * a).sum();
    if 基準 <= f64::EPSILON {
        return (None, 0.0);
    }
    let mut 最良 = (最小, f64::NEG_INFINITY);
    for τ in 最小..=最大 {
        let 相関: f64 = x[..x.len() - τ]
            .iter()
            .zip(&x[τ..])
            .map(|(a, b)| a * b)
            .sum();
        let 明瞭 = (相関 / 基準).clamp(0.0, 1.0);
        if 明瞭 > 最良.1 {
            最良 = (τ, 明瞭);
        }
    }
    if 最良.1 < p.明瞭閾 || !最良.1.is_finite() {
        (None, 最良.1)
    } else {
        (Some(p.標本率 as f64 / 最良.0 as f64), 最良.1)
    }
}

pub fn 音高検出(標本: &[f32], p: &検出param) -> 検出結果 {
    let 長 = p.窓長.min(標本.len());
    let 窓 = &標本[..長];
    let 音量 = rms(窓);
    if 音量 < p.無音閾rms.max(0.0) {
        return 検出結果 {
            hz: None,
            明瞭度: 0.0,
            rms: 音量,
        };
    }
    let Some((最小, 最大)) = 範囲(p, 窓.len()) else {
        return 検出結果 {
            hz: None,
            明瞭度: 0.0,
            rms: 音量,
        };
    };
    let (hz, 明瞭度) = match p.法 {
        検出法::YIN => yin(窓, p, 最小, 最大),
        検出法::自己相関 => 自己相関(窓, p, 最小, 最大),
    };
    検出結果 {
        hz,
        明瞭度,
        rms: 音量,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn 正弦(hz: f64, p: &検出param) -> Vec<f32> {
        (0..p.窓長)
            .map(|i| (std::f64::consts::TAU * hz * i as f64 / p.標本率 as f64).sin() as f32)
            .collect()
    }

    #[test]
    fn yin正弦を捉える() {
        let p = 検出param::default();
        let r = 音高検出(&正弦(220.0, &p), &p);
        assert!((r.hz.unwrap() - 220.0).abs() < 1.0, "{r:?}");
        assert!(r.明瞭度 >= p.明瞭閾, "{r:?}");
    }

    #[test]
    fn 自己相関正弦を捉える() {
        let p = 検出param {
            法: 検出法::自己相関,
            ..Default::default()
        };
        let r = 音高検出(&正弦(220.0, &p), &p);
        assert!((r.hz.unwrap() - 220.0).abs() < 2.0, "{r:?}");
    }

    #[test]
    fn 無音は無声() {
        let p = 検出param::default();
        let r = 音高検出(&vec![0.0; p.窓長], &p);
        assert_eq!(r.hz, None);
        assert_eq!(r.rms, 0.0);
    }
}
