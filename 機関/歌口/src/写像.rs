//! 声→Z写像。比の小数巻を環角、整数巻をlapへ分離する。

use wa::z::{全環, 家snap, 環正規化};

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
            満音rms: 1.0,
            明瞭閾: 0.70,
            無音閾rms: 0.01,
            lap下限: -8,
            lap上限: 8,
        }
    }
}

pub fn 声z(検出: &検出結果, p: &写像param) -> Z {
    let Some(hz) = 検出.hz else { return Z::無() };
    if !hz.is_finite()
        || !p.基音.is_finite()
        || p.基音 <= 0.0
        || !検出.rms.is_finite()
        || 検出.rms < p.無音閾rms.max(0.0)
        || !検出.明瞭度.is_finite()
        || 検出.明瞭度 < p.明瞭閾
        || !p.満音rms.is_finite()
        || p.満音rms <= 0.0
    {
        return Z::無();
    }
    let 巻実 = (hz / p.基音).log2();
    if !巻実.is_finite() {
        return Z::無();
    }
    let 巻床 = 巻実.floor();
    let mut theta = 環正規化(全環 * (巻実 - 巻床));
    if p.家snap {
        // B9: 家番号の私有算出を持たぬ。甲契約層だけが格子規約を所有する。
        theta = 家snap(theta, p.家数);
    }
    let lap = if p.lap下限 <= p.lap上限 {
        巻床.clamp(p.lap下限 as f64, p.lap上限 as f64) as i64
    } else {
        p.lap下限
    };
    Z {
        theta,
        r: (検出.rms / p.満音rms).clamp(0.0, 1.0),
        lap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn 検(hz: Option<f64>, 明瞭度: f64, rms: f64) -> 検出結果 {
        検出結果 { hz, 明瞭度, rms }
    }

    #[test]
    fn 基音は環零_lap零() {
        let z = 声z(&検(Some(220.0), 1.0, 0.5), &写像param::default());
        assert!(z.theta.abs() < 1e-12, "{z:?}");
        assert_eq!(z.lap, 0);
        assert!((z.r - 0.5).abs() < 1e-12);
    }

    #[test]
    fn octaveはlapへ入る() {
        let z = 声z(&検(Some(440.0), 1.0, 1.0), &写像param::default());
        assert!(z.theta.abs() < 1e-12, "{z:?}");
        assert_eq!(z.lap, 1);
    }

    #[test]
    fn 無音無声欠hzは無() {
        let p = 写像param::default();
        for r in [
            検(None, 1.0, 1.0),
            検(Some(220.0), 0.0, 1.0),
            検(Some(220.0), 1.0, 0.0),
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
        assert_eq!(声z(&検(Some(220.0 * 16.0), 1.0, 1.0), &p).lap, 2);
        assert_eq!(声z(&検(Some(220.0 / 16.0), 1.0, 1.0), &p).lap, -2);
    }

    #[test]
    fn 家snapは甲契約格子へ委譲() {
        let p = 写像param {
            家snap: true,
            家数: 8,
            ..Default::default()
        };
        let z = 声z(&検(Some(220.0 * 2f64.powf(0.14)), 1.0, 1.0), &p);
        assert!((z.theta - 家snap(z.theta, p.家数)).abs() < 1e-12, "{z:?}");
    }
}
