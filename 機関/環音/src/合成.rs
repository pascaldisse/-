//! 位相連続sine合成 — zの角=音高, 径=振幅, 巻=octave。

use crate::契約::{Z構, Z};
use crate::音高::{周波数_上限付, 周波数上限param, 音高律};

const TAU: f64 = std::f64::consts::TAU;

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

/// frame間補間param — z frame境界 (通常60Hz) でのr/周波数階段をclick無く均す.
/// 監査乙.4-c 欠4 (出現律violation) 対応: 添先は総角 θ+2π·lap (丙-C3) —
/// θ単体を補間すると ±π縫目で不連続 (大きな規則の飛び) になる為, 必ず総角を先に線形補間してから
/// リモードする.
#[derive(Debug, Clone, Copy)]
pub struct 補間param {
    /// 補間標本数. frame境界からこの標本数かけて 前総角/振幅 → 次へ線形移行.
    /// 既定=frame長そのもの (別途のhardcode定数は持たず, 呼出側(主=main.rs)がz毎さんぷる数を
    /// そのまま渡す事で「既定=frame長」を成立させる).
    pub 標本数: usize,
}

/// 総角 (丙-C3) — センター値を含むunwrap済み角. 旋回履歴を失わず連続量として扱える.
fn 総角(z: &Z) -> f64 {
    z.theta + TAU * z.lap as f64
}

/// 総角→(theta∈(-π,π], lap) へ分解 (契約の表現範囲へ復元).
fn 逆総角(総: f64) -> (f64, i64) {
    let theta = (総 + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI;
    let lap = ((総 - theta) / TAU).round() as i64;
    (theta, lap)
}

/// 位相連続の単声sine合成器。
pub struct 合成器 {
    sample率: u32,
    音高: 音高律,
    律動: 律動param,
    上限: 周波数上限param,
    補間: 補間param,
    位相: f64,
    律動位相: f64,
    fade標本: f64,
    上限ログ済: bool,
    現在z: Option<Z>,
    補間起点総角: f64,
    補間起点振幅: f64,
    目標総角: f64,
    目標振幅: f64,
    補間進行標本: usize,
}

impl 合成器 {
    /// 新規合成器。gate端fade既定=2ms。上限/補間はparam必須 (hardcode回避—呼出側が決める).
    pub fn 新(
        sample率: u32,
        音高: 音高律,
        律動: 律動param,
        上限: 周波数上限param,
        補間: 補間param,
    ) -> 合成器 {
        let fade秒 = 律動.端fade秒.max(0.0);
        合成器 {
            sample率,
            音高,
            律動,
            上限,
            補間,
            位相: 0.0,
            律動位相: 0.0,
            fade標本: fade秒 * sample率 as f64,
            上限ログ済: false,
            現在z: None,
            補間起点総角: 0.0,
            補間起点振幅: 0.0,
            目標総角: 0.0,
            目標振幅: 0.0,
            補間進行標本: 0,
        }
    }

    /// 現在の補間進行度に応じた (総角, 振幅) — 進行を更新せずに参照のみ.
    fn 補間後現在値(&self) -> (f64, f64) {
        let 標本数 = self.補間.標本数.max(1) as f64;
        let t = (self.補間進行標本 as f64 / 標本数).min(1.0);
        let 総角 = self.補間起点総角 + (self.目標総角 - self.補間起点総角) * t;
        let 振幅 = self.補間起点振幅 + (self.目標振幅 - self.補間起点振幅) * t;
        (総角, 振幅)
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
    /// frame境界 (z変化検知) では 総角/振幅を補間.標本数 にかけて線形ランプ (欣受け 欠4).
    /// 周波数はNyquist上限paramで飽和 (欣受け 欠3) — 初回飽和時のみ1行log.
    pub fn 次sample(&mut self, z: &Z) -> f32 {
        if self.sample率 == 0 {
            return 0.0;
        }

        if self.現在z != Some(*z) {
            let (起点総角, 起点振幅) = self.補間後現在値();
            self.補間起点総角 = 起点総角;
            self.補間起点振幅 = 起点振幅;
            self.目標総角 = 総角(z);
            self.目標振幅 = z.r.clamp(0.0, 1.0);
            self.補間進行標本 = 0;
            self.現在z = Some(*z);
        }

        let (現総角, 現振幅) = self.補間後現在値();
        self.補間進行標本 = (self.補間進行標本 + 1).min(self.補間.標本数.max(1));

        let (theta, lap) = 逆総角(現総角);
        let z_eff = Z::new(theta, 現振幅, lap);
        let (周波数, 飽和) = 周波数_上限付(&z_eff, &self.音高, self.sample率, self.上限);
        if 飽和 && !self.上限ログ済 {
            eprintln!(
                "# 警告: 周波数上限飽和 (欠3射壁) lap={} theta={:.4} 上限={:.1}Hz — 以後同一run内は再ログせず",
                lap,
                theta,
                self.上限.上限hz(self.sample率)
            );
            self.上限ログ済 = true;
        }
        if !周波数.is_finite() || 周波数 <= 0.0 {
            return 0.0;
        }

        let sine = self.位相.sin();
        self.位相 = (self.位相 + TAU * 周波数 / self.sample率 as f64).rem_euclid(TAU);
        self.律動位相 = (self.律動位相 + self.律動.律動Hz / self.sample率 as f64).rem_euclid(1.0);

        if 現振幅 <= 0.0 {
            return 0.0;
        }
        (sine * 現振幅 * self.gate()) as f32
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
        // 既定補間標本数=1 — 旧来試験の「喳間への即時切替」与矛n (frame長の概念が無い単一z連続試験群) を壊さない.
        合成器::新(
            1_000,
            音高律 {
                基音: 10.0,
                律: 律::八家,
            },
            律動,
            周波数上限param::default(),
            補間param { 標本数: 1 },
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
        let mut a = 合成(律動param {
            有効: false,
            ..Default::default()
        });
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
        let mut a = 合成(律動param {
            有効: false,
            ..Default::default()
        });
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
        let mut a = 合成(律動param {
            端fade秒: 0.05,
            ..Default::default()
        });
        let samples = a.描画(&[z(0.0, 1.0)], 600);
        assert!(samples.windows(2).all(|w| (w[1] - w[0]).abs() < 0.1));
    }

    #[test]
    fn 端fade秒は既定二ms且つparam() {
        assert!((律動param::default().端fade秒 - 0.002).abs() < f64::EPSILON);
        let mut 急 = 合成(律動param {
            端fade秒: 0.0,
            ..Default::default()
        });
        let s = 急.描画(&[z(0.0, 1.0)], 600);
        let 跳 = s
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f32, f32::max);
        let mut 緩 = 合成(律動param {
            端fade秒: 0.05,
            ..Default::default()
        });
        let s2 = 緩.描画(&[z(0.0, 1.0)], 600);
        let 跳2 = s2
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f32, f32::max);
        assert!(跳 > 跳2, "fade無={跳} fade有={跳2}");
    }

    #[test]
    fn 振幅は一を越えない() {
        let mut a = 合成(律動param {
            有効: false,
            ..Default::default()
        });
        let samples = a.描画(&[z(0.0, 5.0), z(1.0, -2.0)], 1_000);
        assert!(samples.iter().all(|s| s.abs() <= 1.0));
    }

    #[test]
    fn 描画長はz数掛標本数() {
        let mut a = 合成(律動param::default());
        assert_eq!(a.描画(&[z(0.0, 1.0), z(1.0, 1.0)], 17).len(), 34);
    }

    // — 欣乙.4-c 欠4 (出現律violation, frame境界click) 実走検審—

    #[test]
    fn frame境界のr足跍は補間で均される() {
        let frame長 = 200usize;
        let z_low = z(0.0, 0.1);
        let z_high = z(0.0, 0.9);
        let 律動 = 律動param {
            有効: false,
            ..Default::default()
        }; // 律動gateはfade済み別件 (欠5) — 本件はframe境界単体を見る

        let mut 即時 = 合成器::新(
            48_000,
            音高律 {
                基音: 220.0,
                律: 律::八家,
            },
            律動,
            周波数上限param::default(),
            補間param { 標本数: 1 }, // 補間無 (旧来相当) — 対照区
        );
        let s即時 = 即時.描画(&[z_low, z_high], frame長);
        let 足即時 = s即時
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);

        let mut 均し = 合成器::新(
            48_000,
            音高律 {
                基音: 220.0,
                律: 律::八家,
            },
            律動,
            周波数上限param::default(),
            補間param {
                標本数: frame長
            }, // 既定=frame長
        );
        let s均し = 均し.描画(&[z_low, z_high], frame長);
        let 足均し = s均し
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);

        assert!(
            足均し < 足即時 * 0.2,
            "補間有={足均し} 補間無(即時)={足即時}"
        );
    }

    #[test]
    fn 高lapは上限飽和しても発音は有限・panic無し() {
        let mut a = 合成器::新(
            8_000,
            音高律 {
                基音: 220.0,
                律: 律::八家,
            },
            律動param {
                有効: false,
                ..Default::default()
            },
            周波数上限param::default(), // 上限=8000*0.45=3600Hz
            補間param { 標本数: 1 },
        );
        let 高lap = Z {
            theta: 0.0,
            r: 1.0,
            lap: 6,
        }; // 220*2^6=14080Hz → 上限超え
        let samples = a.描画(&[高lap], 200);
        assert!(samples.iter().all(|s| s.abs() <= 1.0 && s.is_finite()));
    }

    #[test]
    fn 振幅線形性_rms比一対二対四() {
        use crate::波形::実効値;
        let 律動 = 律動param {
            有効: false,
            ..Default::default()
        };
        let frame長 = 4_000usize; // 補間立上りを避ける為後半のみ使用
        let rms_of = |r: f64| -> f64 {
            let mut a = 合成器::新(
                48_000,
                音高律 {
                    基音: 220.0,
                    律: 律::八家,
                },
                律動,
                周波数上限param::default(),
                補間param { 標本数: 1 },
            );
            let s = a.描画(&[z(0.0, r)], frame長);
            実効値(&s[frame長 / 2..])
        };
        let r1 = rms_of(0.25);
        let r2 = rms_of(0.5);
        let r4 = rms_of(1.0);
        assert!((r2 / r1 - 2.0).abs() < 0.01, "r2/r1={}", r2 / r1);
        assert!((r4 / r1 - 4.0).abs() < 0.01, "r4/r1={}", r4 / r1);
    }
}
