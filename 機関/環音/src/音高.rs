//! 音高律 (梯3 一部) — z → 家番号/家率/周波数.
//! 文書/環統合.md §実装梯3: freq=基音·2^{lap}·家率, stickを回すと歌う.
//! 鉄則: hardcode禁 — 全定数param既定経由 (例外LOVE=1のみ). 唯一所有file (機関/環音/src/音高.rs).

use crate::契約::Z;
use std::f64::consts::TAU;

/// 律 (scale) 種別 — 八家=八卦8音 或 十二平均律12半音.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum 律 {
    /// 八家 (八卦) — 8音, 家率=2^(家番号/8).
    八家,
    /// 十二平均律 — 12音, 半音比=2^(1/12).
    十二平均律,
}

/// 音高設定 — 基音Hz + 律選択. 全既定値経由 (鉄則).
#[derive(Debug, Clone, Copy)]
pub struct 音高律 {
    /// 基音 [Hz] — lap=0 · 家率=1.0 時の周波数.
    pub 基音: f64,
    pub 律: 律,
}

impl Default for 音高律 {
    fn default() -> Self {
        Self { 基音: 220.0, 律: 律::八家 }
    }
}

/// θ→八扇形snap. 家0=θ=0中心±22.5° (θ増加方向=反時計回り=家番号増加方向, atan2慣習と同一).
/// 境界: [h·45°−22.5°, h·45°+22.5°) — 上限側open (丁度22.5°は次家へ倒れる).
pub fn 家番号(z: &Z) -> u8 {
    let theta = z.theta正規();
    let 幅 = TAU / 8.0;
    let 半幅 = 幅 / 2.0;
    let idx = ((theta + 半幅) / 幅).floor() as i64;
    idx.rem_euclid(8) as u8
}

/// z→家率∈[1,2). 律により算出法分岐:
/// - 八家 = 2^(家番号/8)
/// - 十二平均律 = 2^(round(θ/τ·12)/12), 12番目 (θ→τ端) は次lapへ持ち越さず家率1.0へ折返す
///   (連続性より同値性優先 — 契約通り, 家率∈[1,2)を常に保つ).
pub fn 家率(z: &Z, 律: 律) -> f64 {
    match 律 {
        律::八家 => {
            let h = 家番号(z) as f64;
            2f64.powf(h / 8.0)
        }
        律::十二平均律 => {
            let theta = z.theta正規();
            let n = (theta / TAU * 12.0).round() as i64;
            let n = n.rem_euclid(12);
            2f64.powf(n as f64 / 12.0)
        }
    }
}

/// z→周波数 [Hz] = 基音 · 2^lap · 家率.
pub fn 周波数(z: &Z, 設定: &音高律) -> f64 {
    設定.基音 * 2f64.powi(z.lap as i32) * 家率(z, 設定.律)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lap成一_周波数厳密二倍() {
        let 設定 = 音高律::default();
        let z0 = Z::new(0.7, 0.5, 3);
        let z1 = Z::new(0.7, 0.5, 4);
        let f0 = 周波数(&z0, &設定);
        let f1 = 周波数(&z1, &設定);
        let 誤差 = ((f1 / f0) - 2.0).abs();
        assert!(誤差 < 1e-12, "誤差={誤差}");
    }

    #[test]
    fn lap成一_十二平均律側でも厳密二倍() {
        let 設定 = 音高律 { 基音: 330.0, 律: 律::十二平均律 };
        let z0 = Z::new(2.1, 0.9, -2);
        let z1 = Z::new(2.1, 0.9, -1);
        let 比 = 周波数(&z1, &設定) / 周波数(&z0, &設定);
        assert!((比 - 2.0).abs() < 1e-12, "比={比}");
    }

    #[test]
    fn theta零_家零_家率一_周波数基音倍数() {
        let 設定 = 音高律::default();
        let z = Z::new(0.0, 1.0, 2);
        assert_eq!(家番号(&z), 0);
        assert!((家率(&z, 律::八家) - 1.0).abs() < 1e-15);
        let 期待 = 設定.基音 * 2f64.powi(2);
        assert!((周波数(&z, &設定) - 期待).abs() < 1e-9);
    }

    #[test]
    fn 八家_全到達_単調増_範囲() {
        let mut rates = vec![];
        for h in 0u8..8 {
            let theta = (h as f64) * TAU / 8.0; // 各家の中心角
            let z = Z::new(theta, 1.0, 0);
            assert_eq!(家番号(&z), h, "h={h}");
            let r = 家率(&z, 律::八家);
            assert!(r >= 1.0 && r < 2.0, "h={h} r={r}");
            rates.push(r);
        }
        for w in rates.windows(2) {
            assert!(w[1] > w[0], "単調増崩れ: {:?}", w);
        }
    }

    #[test]
    fn 十二平均律_全音到達() {
        for n in 0i64..12 {
            let theta = (n as f64) * TAU / 12.0;
            let z = Z::new(theta, 1.0, 0);
            let r = 家率(&z, 律::十二平均律);
            let 期待 = 2f64.powf(n as f64 / 12.0);
            assert!((r - 期待).abs() < 1e-12, "n={n} r={r} 期待={期待}");
        }
    }

    #[test]
    fn 十二平均律_半音比() {
        let z0 = Z::new(0.0, 1.0, 0);
        let z1 = Z::new(TAU / 12.0, 1.0, 0);
        let r0 = 家率(&z0, 律::十二平均律);
        let r1 = 家率(&z1, 律::十二平均律);
        let 比 = r1 / r0;
        assert!((比 - 2f64.powf(1.0 / 12.0)).abs() < 1e-12, "比={比}");
    }

    #[test]
    fn 十二平均律_十二番目_家率一へ折返し() {
        // θ→τ端 (round結果n=12) は次lapへ持ち越さず家率1.0へ折返る (契約: 連続性より同値性優先).
        let z = Z::new(TAU - 1e-9, 1.0, 5);
        let r = 家率(&z, 律::十二平均律);
        assert!((r - 1.0).abs() < 1e-9, "r={r}");
    }

    #[test]
    fn 負角_正規化一致() {
        let z_neg = Z::new(-std::f64::consts::FRAC_PI_4, 1.0, 0); // -45°
        let z_pos = Z::new(TAU - std::f64::consts::FRAC_PI_4, 1.0, 0); // 315°
        assert_eq!(家番号(&z_neg), 家番号(&z_pos));
        let 差 = (家率(&z_neg, 律::八家) - 家率(&z_pos, 律::八家)).abs();
        assert!(差 < 1e-12, "差={差}");
    }

    #[test]
    fn 三周回_正規化一致() {
        let z0 = Z::new(0.9, 1.0, 0);
        let z3 = Z::new(0.9 + 3.0 * TAU, 1.0, 0);
        assert_eq!(家番号(&z0), 家番号(&z3));
        let 差 = (家率(&z0, 律::十二平均律) - 家率(&z3, 律::十二平均律)).abs();
        assert!(差 < 1e-12, "差={差}");
    }

    #[test]
    fn 家境界_22度4対22度6() {
        let rad = |deg: f64| deg.to_radians();
        let z_low = Z::new(rad(22.4), 1.0, 0);
        let z_high = Z::new(rad(22.6), 1.0, 0);
        assert_eq!(家番号(&z_low), 0);
        assert_eq!(家番号(&z_high), 1);
    }

    #[test]
    fn 既定値_基音220_律八家() {
        let 設定 = 音高律::default();
        assert!((設定.基音 - 220.0).abs() < 1e-15);
        assert_eq!(設定.律, 律::八家);
    }

    #[test]
    fn 基音param_変更反映_hardcode無し() {
        let 設定 = 音高律 { 基音: 440.0, 律: 律::八家 };
        let z = Z::new(0.0, 1.0, 0);
        let f = 周波数(&z, &設定);
        assert!((f - 440.0).abs() < 1e-9, "f={f}");
    }

    #[test]
    fn 十二平均律_家率範囲() {
        for n in 0..12 {
            let theta = (n as f64) * TAU / 12.0 + 0.001; // snap境界近傍もずらして検査
            let z = Z::new(theta, 1.0, 0);
            let r = 家率(&z, 律::十二平均律);
            assert!(r >= 1.0 && r < 2.0, "n={n} r={r}");
        }
    }
}
