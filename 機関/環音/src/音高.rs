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
///
/// B9是正 (敵対審査乙.4 B9, 08-11): 私有floor/rem_euclid再実装を解消し, 契約層
/// (機関/環制御 src/z.rs) 唯一実装 wa::z::家番号 へ委譲 (polar.rsも同関数を使う).
/// 八家=8は公理定数 (param化しない — 十二平均律側は本fileの固有規約として残置).
pub fn 家番号(z: &Z) -> u8 {
    wa::z::家番号(z.theta, 8) as u8
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
            let theta = z.theta.rem_euclid(TAU);
            let n = (theta / TAU * 12.0).round() as i64;
            let n = n.rem_euclid(12);
            2f64.powf(n as f64 / 12.0)
        }
    }
}

/// Z総角をL域へ戻す。thetaは符号域へ正規化してからlapを一度だけ足す。
/// `theta=-π/2,lap=1` はL=.75であり、θを正域へ折ってlapと重ねれば+1octとなる。
fn l域(z: &Z) -> f64 {
    let theta = (z.theta + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI;
    z.lap as f64 + theta / TAU
}

/// z→周波数 [Hz]。量子化は実際のL域で8/12家へ効く。
pub fn 周波数(z: &Z, 設定: &音高律) -> f64 {
    let 家数 = match 設定.律 { 律::八家 => 8.0, 律::十二平均律 => 12.0 };
    let l = (l域(z) * 家数).round() / 家数;
    設定.基音 * 2f64.powf(l)
}

/// 周波数上限param — 出力側Nyquist防壁 (梯3 音声合成). 既定=sample_rate·0.45.
/// 監査 docs/adversary/2026-08-11-環統合審.md 乙.4 欠3: lap高でfreq=基音·2^lap·家率が
/// sample_rate/2を超えると折返し (=音高が上がらず下がる) → 契約「lap=octave=単調増巻」破れ。
/// 注: 文書/環統合.md 丙-D3 (入力側stick角速度Nyquist, 実測閾例30rev/s@60Hz) とは別層 —
/// あちらは入力標本化の折返し、これは出力音声標本化の折返し。両者数値単位が異なる為
/// 直接比較不可、両文書に閾値を明記する事で「code/文書不一致」再発を防ぐ (D3と同種の欠陥回避)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct 周波数上限param {
    /// sample_rateに対する比率. 既定0.45 (Nyquist境界0.5に安全余裕を持たせる).
    pub 比率: f64,
}

impl Default for 周波数上限param {
    fn default() -> Self {
        Self { 比率: 0.45 }
    }
}

impl 周波数上限param {
    /// 上限Hz = 比率 · sample率. 比率が非正なら0.0 (=全域飽和, 事実上無音相当の防壁).
    pub fn 上限hz(&self, sample率: u32) -> f64 {
        self.比率.max(0.0) * sample率 as f64
    }
}

/// z→周波数, Nyquist上限param付. 上限超過時は折返さず上限へ**飽和**する — 音高降下 (契約破れ) を防ぎ,
/// 生値が単調増加である限り出力も単調増加 (飽和後は横這い) を保つ。
/// 返り値 = (飽和後周波数, 上限に到達したか).
pub fn 周波数_上限付(z: &Z, 設定: &音高律, sample率: u32, 上限: 周波数上限param) -> (f64, bool) {
    let 生値 = 周波数(z, 設定);
    let 上限hz = 上限.上限hz(sample率);
    if 生値.is_finite() && 生値 > 上限hz {
        (上限hz, true)
    } else {
        (生値, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::契約::Z構;

    #[test]
    fn 下半環とlapは一度だけ合成する() {
        let 設定 = 音高律::default();
        let z = Z::new(-std::f64::consts::FRAC_PI_2, 1.0, 1);
        let f = 周波数(&z, &設定);
        assert!((f / 設定.基音 - 2f64.powf(0.75)).abs() < 1e-12, "f={f}");
    }

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

    // — 欣乙.4-c 欠3 (Nyquist防壁) 実走検審—

    #[test]
    fn 上限param既定はサンプル率0_45倍() {
        let 上限 = 周波数上限param::default();
        assert!((上限.比率 - 0.45).abs() < 1e-15);
        assert!((上限.上限hz(48_000) - 21_600.0).abs() < 1e-9);
    }

    #[test]
    fn lap7_8は上限超え_飽和して単調性保持_48khz() {
        // 監査実測 (乙.4-c): 基音220Hz·八家, 48kHz実測 lap7=19839.96Hz(折返し) lap8=8320.07Hz(逆行).
        // 是正後: 上限(21600Hz)で飽和し, 折返し無し・非減少を保つ.
        let 設定 = 音高律::default(); // 基音220, 八家
        let 上限 = 周波数上限param::default();
        let sample率 = 48_000u32;
        let z6 = Z::new(0.0, 1.0, 6);
        let z7 = Z::new(0.0, 1.0, 7);
        let z8 = Z::new(0.0, 1.0, 8);
        let z9 = Z::new(0.0, 1.0, 9);

        let (f6, c6) = 周波数_上限付(&z6, &設定, sample率, 上限);
        let (f7, c7) = 周波数_上限付(&z7, &設定, sample率, 上限);
        let (f8, c8) = 周波数_上限付(&z8, &設定, sample率, 上限);
        let (f9, c9) = 周波数_上限付(&z9, &設定, sample率, 上限);

        let 上限hz = 上限.上限hz(sample率);
        assert!(!c6, "lap6は上限未満のはず f6={f6}");
        assert!(f6 <= 上限hz + 1e-9);
        assert!(f7 <= 上限hz + 1e-9, "f7={f7} 上限={上限hz}");
        assert!(f8 <= 上限hz + 1e-9, "f8={f8} 上限={上限hz}");
        assert!(f9 <= 上限hz + 1e-9, "f9={f9} 上限={上限hz}");
        assert!(c7 && c8 && c9, "lap7/8/9は上限到達フラグが立つはず c7={c7} c8={c8} c9={c9}");
        // 単調性保持: 飽和後も非減少 (旧欠陥=lap8で8320Hzへ逆行=単調性崩壊).
        assert!(f6 <= f7 + 1e-9, "f6={f6} f7={f7}");
        assert!(f7 <= f8 + 1e-9, "f7={f7} f8={f8}");
        assert!(f8 <= f9 + 1e-9, "f8={f8} f9={f9}");
        assert!((f7 - 上限hz).abs() < 1e-9, "飽和値がぴったり上限であるはず f7={f7}");
        assert!((f8 - 上限hz).abs() < 1e-9, "飽和値がぴったり上限であるはず f8={f8}");
    }

    #[test]
    fn 上限未到達は生値のまま透過() {
        let 設定 = 音高律::default();
        let 上限 = 周波数上限param::default();
        let z = Z::new(0.0, 1.0, 0);
        let (f, 到達) = 周波数_上限付(&z, &設定, 48_000, 上限);
        assert!(!到達);
        assert!((f - 220.0).abs() < 1e-9);
    }

    #[test]
    fn 上限param_他sample率でも比率通り() {
        let 設定 = 音高律 { 基音: 440.0, 律: 律::十二平均律 };
        let 上限 = 周波数上限param { 比率: 0.3 };
        let z = Z::new(0.0, 1.0, 3); // 440*8=3520Hz
        let sample率 = 8_000u32; // 上限=2400Hz
        let (f, 到達) = 周波数_上限付(&z, &設定, sample率, 上限);
        assert!(到達);
        assert!((f - 2400.0).abs() < 1e-9, "f={f}");
    }
}
