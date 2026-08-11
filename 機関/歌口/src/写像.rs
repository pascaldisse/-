//! 声→Z写像。総角=全環·log2(hz/基音) を巻込みで連続に保つ。

use wa::z::全環;

use crate::契約::Z;
use crate::検出::検出結果;
use crate::音高律::律;

#[derive(Debug, Clone, Copy)]
pub struct 写像param {
    pub 基音: f64,
    pub 律: 律,
    pub 家snap: bool,
    pub 家数: u32,
    pub 満音rms: f64,
    pub 明瞭閾: f64,
    pub 無音閾rms: f64,
    pub lap下限: i64,
    pub lap上限: i64,
}

impl Default for 写像param {
    fn default() -> Self {
        Self {
            基音: 220.0,
            律: 律::八家,
            家snap: false,
            家数: 8,
            // 既定=full-scale sine のRMS (1/√2)。合成C10の振幅はこの逆写像で
            // r=amp を保つ。実micは話者較正有声音RMS P95を --満音rms に明示注入する。
            満音rms: std::f64::consts::FRAC_1_SQRT_2,
            明瞭閾: 0.90,
            // −60dBFS相当。較正noise床+6dBへ上書き可能。
            無音閾rms: 0.001,
            lap下限: -8,
            lap上限: 8,
        }
    }
}

fn 螺旋(l: f64, p: &写像param) -> Option<(f64, i64)> {
    if !l.is_finite() || p.lap下限 > p.lap上限 {
        return None;
    }
    // 定1: round= floor(L+1/2)。θ+全環·lap は全環·L と厳密一致する。
    // 半巻tieでは θ=−π, lapが上位へ進む。総角保存を優先する同値端である。
    let lap実 = (l + 0.5).floor();
    let theta = 全環 * (l - lap実);
    let lap = lap実.clamp(p.lap下限 as f64, p.lap上限 as f64) as i64;
    Some((theta, lap))
}

pub fn 声z(検出: &検出結果, p: &写像param) -> Z {
    let Some(hz) = 検出.hz else { return Z::無() };
    if !hz.is_finite()
        || hz <= 0.0
        || !p.基音.is_finite()
        || p.基音 <= 0.0
        || !検出.rms.is_finite()
        || 検出.rms <= p.無音閾rms.max(0.0)
        || !検出.明瞭度.is_finite()
        || 検出.明瞭度 < p.明瞭閾
        || !p.満音rms.is_finite()
        || p.満音rms <= p.無音閾rms
    {
        return Z::無();
    }
    let mut l = (hz / p.基音).log2();
    if p.家snap {
        if p.家数 == 0 {
            return Z::無();
        }
        // 定2: θ域のwa::z::家snapは家0縫目でcarryを持てぬ為ここでは使わない。
        // L域の量子化は巻込み整数格子を直接丸め、337.5°/345°で必ずlapへcarryする。
        // 家番号を算出しない歌口固有写像であり、番号が要る層のB9契約を侵さない。
        l = (l * p.家数 as f64 + 0.5).floor() / p.家数 as f64;
    }
    let Some((theta, lap)) = 螺旋(l, p) else {
        return Z::無();
    };
    // RMS線形deadzone再正規化: 境界r=0、直上も連続。dBは較正だけに留める。
    let r = ((検出.rms - p.無音閾rms) / (p.満音rms - p.無音閾rms)).clamp(0.0, 1.0);
    Z { theta, r, lap }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn 検(hz: Option<f64>, 明瞭度: f64, rms: f64) -> 検出結果 {
        検出結果 { hz, 明瞭度, rms }
    }

    #[test]
    fn 基音は環零_lap零() {
        let p = 写像param::default();
        let z = 声z(&検(Some(p.基音), 1.0, p.満音rms), &p);
        assert!(z.theta.abs() < 1e-12, "{z:?}");
        assert_eq!(z.lap, 0);
        assert_eq!(z.r, 1.0);
    }

    #[test]
    fn 総角は連続真値と一致() {
        let p = 写像param::default();
        for l in [-2.49, -0.5, -0.14, 0.0, 0.49, 0.5, 1.31] {
            let hz = p.基音 * 2f64.powf(l);
            let z = 声z(&検(Some(hz), 1.0, p.満音rms), &p);
            assert!((z.総角() - 全環 * l).abs() < 1e-11, "L={l} z={z:?}");
        }
    }

    #[test]
    fn 家snapはl域carryを保つ() {
        let p = 写像param {
            家snap: true,
            家数: 8,
            ..Default::default()
        };
        let l = 0.99;
        let z = 声z(&検(Some(p.基音 * 2f64.powf(l)), 1.0, p.満音rms), &p);
        assert_eq!(z.lap, 1, "{z:?}");
        assert!(z.theta.abs() < 1e-12, "{z:?}");
    }

    #[test]
    fn 無音無声欠hzは無() {
        let p = 写像param::default();
        for r in [
            検(None, 1.0, p.満音rms),
            検(Some(p.基音), 0.0, p.満音rms),
            検(Some(p.基音), 1.0, p.無音閾rms),
        ] {
            assert_eq!(声z(&r, &p), Z::無());
        }
    }

    #[test]
    fn lapは飽和し折返さぬ() {
        let p = 写像param {
            lap下限: -2,
            lap上限: 2,
            ..Default::default()
        };
        assert_eq!(声z(&検(Some(p.基音 * 16.0), 1.0, p.満音rms), &p).lap, 2);
        assert_eq!(声z(&検(Some(p.基音 / 16.0), 1.0, p.満音rms), &p).lap, -2);
    }

    #[test]
    fn c10_既定較正は合成振幅を飽和せず線形に逆写像する() {
        let p = 写像param::default();
        let mut rs = Vec::new();
        for amp in [0.25_f64, 0.5, 1.0] {
            let rms = amp * std::f64::consts::FRAC_1_SQRT_2;
            rs.push(声z(&検(Some(p.基音), 1.0, rms), &p).r);
        }
        for (got, want) in rs.iter().zip([0.25, 0.5, 1.0]) {
            assert!((got - want).abs() <= 0.01, "r={got}, want={want}");
        }
        assert!(((rs[1] / rs[0]) - 2.0).abs() <= 0.01);
        assert!(((rs[2] / rs[1]) - 2.0).abs() <= 0.01);
    }
}
