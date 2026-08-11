//! 位相連続sine合成 — zの角=音高, 径=振幅, 巻=octave。

use crate::契約::Z;
use crate::音高::{周波数, 音高律};

/// 生命搬送波gate。開率は一周期中の発声比。
#[derive(Debug, Clone, Copy)]
pub struct 律動param {
    pub 有効: bool,
    pub 律動Hz: f64,
    pub 開率: f64,
    /// gate端fade秒 — click防止の立上下り時間. 既定 0.002 (2ms).
    /// 注: fade長 < 搬送波一標本歩幅の時、端で振幅跳が残る (fade標本数が1-2では平滑化不能).
    pub 端fade秒: f64,
}

impl Default for 律動param {
    fn default() -> Self {
        Self {
            有効: true,
            律動Hz: 2.0,
            開率: 0.5,
            端fade秒: 0.002,
        }
    }
}

/// 位相連続の単声sine合成器。
pub struct 合成器 {
    sample率: u32,
    音高: 音高律,
    律動: 律動param,
    位相: f64,
    律動位相: f64,
    fade標本: f64,
}

impl 合成器 {
    /// 新規合成器。gate端fade既定=2ms。
    pub fn 新(sample率: u32, 音高: 音高律, 律動: 律動param) -> 合成器 {
        let fade秒 = 律動.端fade秒.max(0.0);
        合成器 {
            sample率,
            音高,
            律動,
            位相: 0.0,
            律動位相: 0.0,
            fade標本: fade秒 * sample率 as f64,
        }
    }

    fn gate(&self) -> f64 {
        if !self.律動.有効 || self.律動.律動Hz <= 0.0 || self.sample率 == 0 {
            return 1.0;
        }
        let 開率 = self.律動.開率.clamp(0.0, 1.0);
        if 開率 == 0.0 {
            return 0.0;
        }
        if 開率 == 1.0 {
            return 1.0;
        }
        let 位相 = self.律動位相.rem_euclid(1.0);
        let fade = (self.fade標本 * self.律動.律動Hz / self.sample率 as f64)
            .min(開率 / 2.0)
            .min((1.0 - 開率) / 2.0);
        if fade <= f64::EPSILON {
            return if 位相 < 開率 { 1.0 } else { 0.0 };
        }
        if 位相 < fade {
            位相 / fade
        } else if 位相 < 開率 - fade {
            1.0
        } else if 位相 < 開率 {
            (開率 - 位相) / fade
        } else {
            0.0
        }
    }

    /// 次の一標本。周波数変更でも位相を戻さずclickを避ける。
    pub fn 次sample(&mut self, z: &Z) -> f32 {
        if self.sample率 == 0 {
            return 0.0;
        }
        let 周波数 = 周波数(z, &self.音高);
        if !周波数.is_finite() || 周波数 <= 0.0 {
            return 0.0;
        }

        let sine = self.位相.sin();
        self.位相 = (self.位相 + std::f64::consts::TAU * 周波数 / self.sample率 as f64)
            .rem_euclid(std::f64::consts::TAU);
        self.律動位相 = (self.律動位相 + self.律動.律動Hz / self.sample率 as f64).rem_euclid(1.0);

        let 振幅 = z.r.clamp(0.0, 1.0);
        if 振幅 == 0.0 {
            return 0.0;
        }
        (sine * 振幅 * self.gate()) as f32
    }

    /// 各zを指定標本数だけ描画。
    pub fn 描画(&mut self, zs: &[Z], z毎sample数: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(zs.len().saturating_mul(z毎sample数));
        for z in zs {
            for _ in 0..z毎sample数 {
                samples.push(self.次sample(z));
            }
        }
        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::音高::律;

    fn z(theta: f64, r: f64) -> Z {
        Z { theta, r, lap: 0 }
    }

    fn 合成(律動: 律動param) -> 合成器 {
        合成器::新(
            1_000,
            音高律 { 基音: 10.0, 律: 律::八家 },
            律動,
        )
    }

    #[test]
    fn 既定律動は二hz半開() {
        let p = 律動param::default();
        assert!(p.有効 && (p.律動Hz - 2.0).abs() < f64::EPSILON);
        assert!((p.開率 - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn 位相は音高切替でも連続() {
        let mut a = 合成(律動param { 有効: false, ..Default::default() });
        let low = z(0.0, 1.0);
        for _ in 0..20 {
            a.次sample(&low);
        }
        let 前 = a.次sample(&low);
        let 後 = a.次sample(&z(std::f64::consts::FRAC_PI_2, 1.0));
        assert!((後 - 前).abs() < 0.2, "切替差={}", (後 - 前).abs());
    }

    #[test]
    fn 径零は厳密無音() {
        let mut a = 合成(律動param::default());
        let samples = a.描画(&[z(0.0, 0.0)], 1_000);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn gateoffは連続鳴() {
        let mut a = 合成(律動param { 有効: false, ..Default::default() });
        let samples = a.描画(&[z(0.0, 1.0)], 100);
        assert!(samples.iter().skip(1).any(|&s| s.abs() > 0.1));
        assert!(samples.windows(2).all(|w| (w[1] - w[0]).abs() < 0.1));
    }

    #[test]
    fn gateonは周期的に閉じる() {
        let mut a = 合成(律動param::default());
        let samples = a.描画(&[z(0.0, 1.0)], 1_000);
        let rms = |slice: &[f32]| -> f64 {
            (slice.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / slice.len() as f64).sqrt()
        };
        assert!(rms(&samples[50..200]) > 0.3);
        assert!(rms(&samples[300..450]) < 0.01);
        assert!(rms(&samples[550..700]) > 0.3);
    }

    #[test]
    fn gate端はfadeで跳ばない() {
        // fade標本数を十分取れば端の跳は fade傾き (振幅/fade標本) に抑えられる.
        // 既定2msはsample率48kHz (=96標本) 前提 — 試験の1kHz玩具率では2標本しか無く
        // 原理的に平滑化できぬ故、fade秒を param で伸ばして性質を測る (hardcode禁の実利).
        let mut a = 合成(律動param { 端fade秒: 0.05, ..Default::default() });
        let samples = a.描画(&[z(0.0, 1.0)], 600);
        assert!(samples.windows(2).all(|w| (w[1] - w[0]).abs() < 0.1));
    }

    #[test]
    fn 端fade秒は既定二ms且つparam() {
        assert!((律動param::default().端fade秒 - 0.002).abs() < f64::EPSILON);
        let mut 急 = 合成(律動param { 端fade秒: 0.0, ..Default::default() });
        let s = 急.描画(&[z(0.0, 1.0)], 600);
        let 跳 = s.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0_f32, f32::max);
        let mut 緩 = 合成(律動param { 端fade秒: 0.05, ..Default::default() });
        let s2 = 緩.描画(&[z(0.0, 1.0)], 600);
        let 跳2 = s2.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0_f32, f32::max);
        assert!(跳 > 跳2, "fade無={跳} fade有={跳2}");
    }

    #[test]
    fn 振幅は一を越えない() {
        let mut a = 合成(律動param { 有効: false, ..Default::default() });
        let samples = a.描画(&[z(0.0, 5.0), z(1.0, -2.0)], 1_000);
        assert!(samples.iter().all(|s| s.abs() <= 1.0));
    }

    #[test]
    fn 描画長はz数掛標本数() {
        let mut a = 合成(律動param::default());
        assert_eq!(a.描画(&[z(0.0, 1.0), z(1.0, 1.0)], 17).len(), 34);
    }
}
