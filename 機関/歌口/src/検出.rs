//! 音高検出 — YIN全段を必ず通す。自己相関はYIN候補の補助確認だけ。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum 検出法 {
    YIN,
    自己相関,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct 検出param {
    pub 標本率: u32,
    pub 窓長: usize,
    pub 跳幅: usize,
    pub 法: 検出法,
    pub 下限hz: f64,
    pub 上限hz: f64,
    /// YIN CMND谷への進入閾。既定0.10 = 明瞭度0.90。
    pub YIN谷閾: f64,
    pub 明瞭閾: f64,
    /// DC除去RMSの無音死域。既定=−60dBFS相当。
    pub 無音閾rms: f64,
    /// 入力側Nyquist防壁比。既定0.45。
    pub 入力Nyquist比: f64,
}

impl Default for 検出param {
    fn default() -> Self {
        Self {
            標本率: 48_000,
            窓長: 2_048,
            跳幅: 256,
            法: 検出法::YIN,
            下限hz: 80.0,
            上限hz: 400.0,
            YIN谷閾: 0.10,
            明瞭閾: 0.90,
            無音閾rms: 0.001,
            入力Nyquist比: 0.45,
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
    let 平均 = 標本.iter().map(|&x| x as f64).sum::<f64>() / 標本.len() as f64;
    let 二乗和: f64 = 標本.iter().map(|&x| (x as f64 - 平均).powi(2)).sum();
    (二乗和 / 標本.len() as f64).sqrt()
}

fn 範囲(p: &検出param, 標本長: usize) -> Option<(usize, usize, f64)> {
    if p.標本率 == 0
        || !p.下限hz.is_finite()
        || !p.上限hz.is_finite()
        || !p.入力Nyquist比.is_finite()
        || p.下限hz <= 0.0
        || p.入力Nyquist比 <= 0.0
    {
        return None;
    }
    // 入力側の上限はADC標本率に従い飽和。出力側音声Nyquistとは別層。
    let 上限 = p.上限hz.min(p.入力Nyquist比 * p.標本率 as f64);
    if 上限 <= p.下限hz {
        return None;
    }
    let 最小 = ((p.標本率 as f64 / 上限).floor() as usize).max(2);
    let 最大 = (p.標本率 as f64 / p.下限hz).ceil() as usize;
    // 固定比較幅 W=N−最大lag を確保。lag別の比較項数biasを許さぬ。
    if 最小 > 最大 || 標本長 < 最大.saturating_mul(2) {
        None
    } else {
        Some((最小, 最大, 上限))
    }
}

fn 中心化(標本: &[f32]) -> Vec<f64> {
    let 平均 = 標本.iter().map(|&x| x as f64).sum::<f64>() / 標本.len() as f64;
    標本.iter().map(|&x| x as f64 - 平均).collect()
}

#[derive(Debug, Clone, Copy)]
struct Yin結果 {
    hz: Option<f64>,
    明瞭度: f64,
    lag: usize,
}

fn yin(標本: &[f32], p: &検出param, 最小: usize, 最大: usize, 上限: f64) -> Yin結果 {
    let x = 中心化(標本);
    let 幅 = x.len() - 最大;
    let mut 累積 = 0.0;
    let mut 値 = vec![1.0; 最大 + 1];
    for lag in 1..=最大 {
        let 差: f64 = (0..幅).map(|i| (x[i] - x[i + lag]).powi(2)).sum();
        累積 += 差;
        // 定数DC/厳密無音では分母零。NaNを作らず無効へ退避する。
        if !累積.is_finite() || 累積 <= f64::EPSILON {
            return Yin結果 {
                hz: None,
                明瞭度: 0.0,
                lag: 0,
            };
        }
        値[lag] = lag as f64 * 差 / 累積;
    }
    let mut 候補 = None;
    for lag in 最小..=最大 {
        if 値[lag].is_finite() && 値[lag] < p.YIN谷閾 {
            let mut 谷 = lag;
            while 谷 < 最大 && 値[谷 + 1] < 値[谷] {
                谷 += 1;
            }
            候補 = Some(谷);
            break;
        }
    }
    let Some(lag) = 候補 else {
        return Yin結果 {
            hz: None,
            明瞭度: 0.0,
            lag: 0,
        };
    };
    let 明瞭度 = (1.0 - 値[lag]).clamp(0.0, 1.0);
    if 明瞭度 < p.明瞭閾 {
        return Yin結果 {
            hz: None,
            明瞭度,
            lag,
        };
    }
    let 補正 = if lag > 最小 && lag < 最大 {
        let 分母 = 値[lag - 1] - 2.0 * 値[lag] + 値[lag + 1];
        let δ = 0.5 * (値[lag - 1] - 値[lag + 1]) / 分母;
        if 分母.is_finite() && 分母 > 0.0 && δ.is_finite() && δ.abs() <= 1.0 {
            δ
        } else {
            0.0
        }
    } else {
        0.0
    };
    let hz = p.標本率 as f64 / (lag as f64 + 補正);
    if !hz.is_finite() || hz < p.下限hz {
        Yin結果 {
            hz: None,
            明瞭度,
            lag,
        }
    } else {
        Yin結果 {
            hz: Some(hz.min(上限)),
            明瞭度,
            lag,
        }
    }
}

fn 自己相関確認(標本: &[f32], 最大: usize, lag: usize) -> bool {
    if lag == 0 {
        return false;
    }
    let x = 中心化(標本);
    let 幅 = x.len() - 最大;
    let (mut 左, mut 右, mut 積) = (0.0, 0.0, 0.0);
    for i in 0..幅 {
        左 += x[i] * x[i];
        右 += x[i + lag] * x[i + lag];
        積 += x[i] * x[i + lag];
    }
    let 分母 = (左 * 右).sqrt();
    分母.is_finite() && 分母 > f64::EPSILON && (積 / 分母).is_finite() && 積 / 分母 > 0.0
}

pub fn 音高検出(標本: &[f32], p: &検出param) -> 検出結果 {
    let 長 = p.窓長.min(標本.len());
    let 窓 = &標本[..長];
    let 音量 = rms(窓);
    if 音量 <= p.無音閾rms.max(0.0) {
        return 検出結果 {
            hz: None,
            明瞭度: 0.0,
            rms: 音量,
        };
    }
    let Some((最小, 最大, 上限)) = 範囲(p, 窓.len()) else {
        return 検出結果 {
            hz: None,
            明瞭度: 0.0,
            rms: 音量,
        };
    };
    // 法に関わらず差分→CMND→最初の閾内谷→補間を通す。生自己相関単独路は無い。
    let y = yin(窓, p, 最小, 最大, 上限);
    let hz = match p.法 {
        検出法::YIN => y.hz,
        検出法::自己相関 if 自己相関確認(窓, 最大, y.lag) => y.hz,
        検出法::自己相関 => None,
    };
    検出結果 {
        hz,
        明瞭度: y.明瞭度,
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
    fn 自己相関もyinを通る() {
        let p = 検出param {
            法: 検出法::自己相関,
            ..Default::default()
        };
        let r = 音高検出(&正弦(220.0, &p), &p);
        assert!((r.hz.unwrap() - 220.0).abs() < 1.0, "{r:?}");
    }

    #[test]
    fn 無音とdcは無声() {
        let p = 検出param::default();
        for 標本 in [vec![0.0; p.窓長], vec![0.5; p.窓長]] {
            let r = 音高検出(&標本, &p);
            assert_eq!(r.hz, None, "{r:?}");
            assert_eq!(r.rms, 0.0);
        }
    }
}
