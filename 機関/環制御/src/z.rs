//! 梯2 — stick生値 → **z** 変換器 (純粋計算部, 機器非依存).
//! 文書/環統合.md §主座標: z = r·e^{iθ} + lap.
//! 全下流 (梯3音·梯4場注入·梯5歌路) は **zのみ** 受取る — 契約=不変.
//!
//! 律: hardcode禁 — 全定数は Z変param 既定つき (唯一例外 LOVE=1).

// 律判定 (terra審査への回答, 08-11): π/τ は **param化しない**。
// hardcode禁の対象=「他の値でも成立し得る選択」。π=環の定義そのもので選択の余地が無く,
// 可変にすれば e^{iπ}+1=0 (公理形) が壊れる。LOVE=1と同列の公理側定数として置く。

/// 半環 π. 対蹠 (θ=π) = −1 = amm = 孤独 (環統合.md).
pub const 半環: f64 = std::f64::consts::PI;
/// 全環 2π = 一巻 (lap +1 分).
pub const 全環: f64 = std::f64::consts::TAU;
/// 愛=1 — 唯一のhardcode許可定数.
pub const LOVE: f64 = 1.0;

/// 螺旋上の一点 — 全界面の唯一表現 (建根甲·乙 共有契約, 不変).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Z {
    /// 角 [-π, π] — 環位置. θ=0 → +1=愛 · θ=π → −1=対蹠.
    pub theta: f64,
    /// 径 [0, 1] — 大きさ. r=0 = 無 = 中心 = 沈黙.
    pub r: f64,
    /// 巻 — ±π横断の符号つき累計 (反時計+1 · 時計−1). octave/shellに対応.
    pub lap: i64,
}

impl Z {
    /// 無 = 中心 = 零vector (θ=0, r=0, lap=0).
    pub fn 無() -> Z {
        Z {
            theta: 0.0,
            r: 0.0,
            lap: 0,
        }
    }

    /// 直交成分 (x, y) = r·(cosθ, sinθ) — 場注入用.
    pub fn 直交(&self) -> (f64, f64) {
        (self.r * self.theta.cos(), self.r * self.theta.sin())
    }

    /// 総角 = θ + 2π·lap — 螺旋を巻数込みで展開した連続角 (単調性検査用).
    pub fn 総角(&self) -> f64 {
        self.theta + 全環 * (self.lap as f64)
    }

    /// 無か否か (r==0).
    pub fn 無か(&self) -> bool {
        self.r == 0.0
    }
}

/// 変換param — 全て既定つき (鉄則: hardcode禁).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Z変param {
    /// 中央死域. 生径がこれ未満 → r=0 (無). 既定 0.08.
    pub 死域: f64,
    /// 死域外を [0,1] に再写像するか. true = 死域境界で r が 0 から連続に立上がる
    /// (出現律: 可視flag/pop-in禁 → 既定 true).
    pub 死域再正規化: bool,
    /// 八家 (45°) snap. 既定 off — 生θをそのまま返す.
    pub 八家snap: bool,
    /// snap分割数. 8 = 八卦 (45°刻). 既定 8.
    pub 家数: u32,
    /// r の上限clamp (stick生値が正方形域で√2まで出る為). 既定 1.0.
    pub r上限: f64,
    /// 死域中に直前の活性θを保持するか. false なら θ=0 を返す. 既定 true
    /// (無へ落ちた瞬間に角が飛ぶのを防ぐ — 出現律).
    pub 死域中θ保持: bool,
    /// 死域に入った時点で環記憶 (前θ) を解除するか. true = 解除 →
    /// 手を離して逆側から入れ直しても偽の巻を数えない. 既定 true.
    pub 死域で環記憶解除: bool,
    /// 径再写像の最小幅 (零除算防止下限). 既定 f64::EPSILON.
    pub 最小幅: f64,
    /// 活性判定の境界許容. 生径 + これ >= 死域 なら活性。
    /// 理由 (敵対審査 a6, 08-11): r·(cos,sin) を hypot で戻すと浮動丸めで
    /// 千分の一の確率で 死域化 を下回り, 「境界ちょうど=活性」仕様が
    /// 実測不能になる。許容幅無しの >= は仕様ではなく丸め誤差の役。既定 1e-12.
    pub 境界許容: f64,
}

impl Default for Z変param {
    fn default() -> Self {
        Z変param {
            死域: 0.08,
            死域再正規化: true,
            八家snap: false,
            家数: 8,
            r上限: LOVE,
            死域中θ保持: true,
            死域で環記憶解除: true,
            最小幅: f64::EPSILON,
            境界許容: 1e-12,
        }
    }
}

/// 角を (-π, π] へ環正規化.
pub fn 環正規化(θ: f64) -> f64 {
    let t = θ.rem_euclid(全環);
    if t > 半環 {
        t - 全環
    } else {
        t
    }
}

/// θ を 家数 等分の格子へ snap し (-π, π] へ戻す.
pub fn 家snap(θ: f64, 家数: u32) -> f64 {
    if 家数 == 0 {
        return 環正規化(θ);
    }
    let 刻 = 全環 / (家数 as f64);
    環正規化((θ / 刻).round() * 刻)
}

/// stick生値の流 → zの流 変換器 (状態=巻計数 + 前角).
#[derive(Debug, Clone)]
pub struct Z変換器 {
    param: Z変param,
    前θ: Option<f64>,
    保持θ: f64,
    lap: i64,
}

impl Z変換器 {
    pub fn 新(param: Z変param) -> Self {
        Z変換器 {
            param,
            前θ: None,
            保持θ: 0.0,
            lap: 0,
        }
    }

    pub fn 既定() -> Self {
        Self::新(Z変param::default())
    }

    pub fn param(&self) -> &Z変param {
        &self.param
    }

    /// 巻を零へ (再生開始時など決定論再現用).
    pub fn 巻戻し(&mut self) {
        self.前θ = None;
        self.保持θ = 0.0;
        self.lap = 0;
    }

    /// 生 (x, y) → Z. 巻は ±π 横断で符号つきに増減する.
    ///
    /// 横断判定: 連続tick間の生角差 d を最短路と見做し
    /// d < −π → 反時計に +π を越えた (lap +1) · d > +π → 時計に −π を越えた (lap −1).
    /// 前提=標本間の実移動が π 未満 (Nyquist条件) — 満たさぬ高速回転は原理上不可分.
    pub fn 変換(&mut self, x: f64, y: f64) -> Z {
        // 壊れ入力防壁 (敵対審査 a14 回帰): NaN/Inf は方位ではない → 無へ落とす.
        // param化しない: 「非数を方位として採る」選択肢は存在しない.
        if !x.is_finite() || !y.is_finite() {
            if self.param.死域で環記憶解除 {
                self.前θ = None;
            }
            return Z {
                theta: if self.param.死域中θ保持 { self.保持θ } else { 0.0 },
                r: 0.0,
                lap: self.lap,
            };
        }
        let 生r = x.hypot(y);
        let 活性 = 生r + self.param.境界許容 >= self.param.死域;

        if !活性 {
            if self.param.死域で環記憶解除 {
                self.前θ = None;
            }
            let theta = if self.param.死域中θ保持 {
                self.保持θ
            } else {
                0.0
            };
            return Z {
                theta,
                r: 0.0,
                lap: self.lap,
            };
        }

        let 生θ = 環正規化(y.atan2(x));

        // 巻計数は必ず **生θ** で行う (snapは下流の量子化であって環の実位置ではない).
        if let Some(前) = self.前θ {
            let d = 生θ - 前;
            if d < -半環 {
                self.lap += 1;
            } else if d > 半環 {
                self.lap -= 1;
            }
        }
        self.前θ = Some(生θ);

        let theta = if self.param.八家snap {
            家snap(生θ, self.param.家数)
        } else {
            生θ
        };
        self.保持θ = theta;

        let r = self.径写像(生r);

        Z {
            theta,
            r,
            lap: self.lap,
        }
    }

    /// 生径 → r ∈ [0, r上限].
    fn 径写像(&self, 生r: f64) -> f64 {
        let 上 = self.param.r上限;
        if self.param.死域再正規化 {
            let 幅 = (上 - self.param.死域).max(self.param.最小幅);
            (((生r - self.param.死域) / 幅) * 上).clamp(0.0, 上)
        } else {
            生r.clamp(0.0, 上)
        }
    }

    /// 現在の巻.
    pub fn 巻(&self) -> i64 {
        self.lap
    }
}

/// 生値列を一括変換 (決定論再生test用 — 同じ列は必ず同じz列を産む).
pub fn 列変換(param: Z変param, 列: &[(f64, f64)]) -> Vec<Z> {
    let mut 変 = Z変換器::新(param);
    列.iter().map(|&(x, y)| 変.変換(x, y)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn 近(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    /// 円周上を n 標本で 巻数 周する生値列を作る (反時計=正).
    fn 回転列(巻数: f64, n: usize, 径: f64) -> Vec<(f64, f64)> {
        (0..=n)
            .map(|i| {
                let θ = 巻数 * 全環 * (i as f64) / (n as f64);
                (径 * θ.cos(), 径 * θ.sin())
            })
            .collect()
    }

    #[test]
    fn 中心は無() {
        let mut 変 = Z変換器::既定();
        let z = 変.変換(0.0, 0.0);
        assert_eq!(z.r, 0.0);
        assert_eq!(z.lap, 0);
        assert!(z.無か());
    }

    #[test]
    fn 境界ちょうどは回転構成でも活性 () {
        // 敵対審査 a6 回帰: dz·(cosθ,sinθ) の hypot は丸めで dz を微小に下回る事があり,
        // 許容幅無しの >= だと境界を丸め誤差で非活性側へ落としていた.
        // 活性の外部観測は **theta更新** で行う — lapは最短路判定なので
        // ±πを跨がぬ跨び (0→200°=時計回り160°) では動かぬのが正しい.
        let dz = Z変param::default().死域;
        let mut 変 = Z変換器::既定();
        変.変換(0.9, 0.0); // 前θ=0
        let θ: f64 = 200.0_f64.to_radians();
        let z = 変.変換(dz * θ.cos(), dz * θ.sin());
        assert!(
            近(z.theta, 環正規化(θ), 1e-9),
            "境界が非活性側へ落ちた (thetaが前値保持のまま) z={z:?}"
        );
        assert_eq!(z.lap, 0, "最短路で±πを跨がぬのに巻が動いた z={z:?}");
    }

    #[test]
    fn 境界活性は跨ぎ時に巻を発火させる() {
        // 同じ境界問題を lap で見るなら, 実際に ±π を跨ぐ跨びで探る必要がある.
        let dz = Z変param::default().死域;
        let mut 変 = Z変換器::既定();
        let 前: f64 = 170.0_f64.to_radians();
        変.変換(0.9 * 前.cos(), 0.9 * 前.sin());
        let 後: f64 = -170.0_f64.to_radians();
        let z = 変.変換(dz * 後.cos(), dz * 後.sin());
        assert_eq!(z.lap, 1, "境界値で +π 跨ぎが数えられなかった z={z:?}");
    }

    #[test]
    fn 死域未満は無() {
        let mut 変 = Z変換器::既定();
        let z = 変.変換(0.05, 0.0);
        assert_eq!(z.r, 0.0);
    }

    #[test]
    fn 死域境界で連続に立上がる() {
        // 出現律: r は死域で 0、直上で ~0 → pop-in無し.
        let mut 変 = Z変換器::既定();
        let z0 = 変.変換(0.08, 0.0);
        assert!(近(z0.r, 0.0, 1e-9), "境界r={}", z0.r);
        let z1 = 変.変換(0.0801, 0.0);
        assert!(z1.r > 0.0 && z1.r < 1e-3, "直上r={}", z1.r);
    }

    #[test]
    fn 最大押幅でr一() {
        let mut 変 = Z変換器::既定();
        let z = 変.変換(1.0, 0.0);
        assert!(近(z.r, LOVE, 1e-12), "r={}", z.r);
    }

    #[test]
    fn 右はθ零() {
        let mut 変 = Z変換器::既定();
        let z = 変.変換(1.0, 0.0);
        assert!(近(z.theta, 0.0, 1e-12));
    }

    #[test]
    fn 左は対蹠π() {
        let mut 変 = Z変換器::既定();
        let z = 変.変換(-1.0, 0.0);
        assert!(近(z.theta.abs(), 半環, 1e-12), "θ={}", z.theta);
    }

    #[test]
    fn θは常に閉区間内() {
        let mut 変 = Z変換器::既定();
        for i in 0..1000 {
            let θ = -10.0 + 0.02 * (i as f64);
            let z = 変.変換(θ.cos(), θ.sin());
            assert!(z.theta > -半環 - 1e-12 && z.theta <= 半環 + 1e-12, "θ={}", z.theta);
        }
    }

    #[test]
    fn 反時計一周で巻正一() {
        let 列 = 回転列(1.0, 360, 0.9);
        let zs = 列変換(Z変param::default(), &列);
        assert_eq!(zs.last().unwrap().lap, 1, "巻={:?}", zs.last());
    }

    #[test]
    fn 時計一周で巻負一() {
        let 列 = 回転列(-1.0, 360, 0.9);
        let zs = 列変換(Z変param::default(), &列);
        assert_eq!(zs.last().unwrap().lap, -1);
    }

    #[test]
    fn 三周で巻三() {
        let 列 = 回転列(3.0, 1080, 0.9);
        let zs = 列変換(Z変param::default(), &列);
        assert_eq!(zs.last().unwrap().lap, 3);
    }

    #[test]
    fn 往復は巻零に戻る() {
        let mut 列 = 回転列(1.0, 360, 0.9);
        let mut 戻 = 回転列(-1.0, 360, 0.9);
        // 戻り列は θ=0 から時計回り → 連結して往復.
        列.append(&mut 戻);
        let zs = 列変換(Z変param::default(), &列);
        assert_eq!(zs.last().unwrap().lap, 0);
    }

    #[test]
    fn 総角は単調増加_反時計() {
        let 列 = 回転列(2.0, 720, 0.9);
        let zs = 列変換(Z変param::default(), &列);
        for w in zs.windows(2) {
            assert!(w[1].総角() >= w[0].総角() - 1e-9, "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn 死域跨ぎは偽巻を産まない() {
        // θ≈+3.0 で離す → 無 → θ≈−3.0 で入れ直す. 環記憶解除 (既定) なら巻=0.
        let mut 変 = Z変換器::既定();
        変.変換(0.9 * 3.0_f64.cos(), 0.9 * 3.0_f64.sin());
        変.変換(0.0, 0.0);
        let z = 変.変換(0.9 * (-3.0_f64).cos(), 0.9 * (-3.0_f64).sin());
        assert_eq!(z.lap, 0, "偽巻 z={z:?}");
    }

    #[test]
    fn 死域中も巻は保持される() {
        let 列 = 回転列(1.0, 360, 0.9);
        let mut 変 = Z変換器::既定();
        for &(x, y) in &列 {
            変.変換(x, y);
        }
        let z = 変.変換(0.0, 0.0);
        assert_eq!(z.lap, 1);
        assert_eq!(z.r, 0.0);
    }

    #[test]
    fn 死域中θ保持() {
        let mut 変 = Z変換器::既定();
        let z1 = 変.変換(0.0, 0.9);
        let z0 = 変.変換(0.0, 0.0);
        assert!(近(z0.theta, z1.theta, 1e-12), "保持失敗 {z0:?}");
    }

    #[test]
    fn 八家snap既定off() {
        let p = Z変param::default();
        assert!(!p.八家snap);
        let mut 変 = Z変換器::新(p);
        let θ: f64 = 0.3;
        let z = 変.変換(θ.cos(), θ.sin());
        assert!(近(z.theta, θ, 1e-12));
    }

    #[test]
    fn 八家snapは45度格子へ() {
        let p = Z変param {
            八家snap: true,
            ..Default::default()
        };
        let mut 変 = Z変換器::新(p);
        let 刻 = 全環 / 8.0;
        for i in 0..64 {
            let θ = -半環 + (i as f64) * 0.1;
            let z = 変.変換(θ.cos(), θ.sin());
            let 余 = (z.theta / 刻) - (z.theta / 刻).round();
            assert!(余.abs() < 1e-9, "非格子 θ={} → {}", θ, z.theta);
        }
    }

    #[test]
    fn snapは巻計数を汚さない() {
        let 列 = 回転列(2.0, 720, 0.9);
        let 生 = 列変換(Z変param::default(), &列);
        let s = 列変換(
            Z変param {
                八家snap: true,
                ..Default::default()
            },
            &列,
        );
        assert_eq!(生.last().unwrap().lap, s.last().unwrap().lap);
    }

    #[test]
    fn 家数はparam() {
        let p = Z変param {
            八家snap: true,
            家数: 4,
            ..Default::default()
        };
        let mut 変 = Z変換器::新(p);
        let 刻 = 全環 / 4.0;
        let z = 変.変換(0.6_f64.cos(), 0.6_f64.sin());
        let 余 = (z.theta / 刻) - (z.theta / 刻).round();
        assert!(余.abs() < 1e-9, "θ={}", z.theta);
    }

    #[test]
    fn 再生は決定論的() {
        let 列 = 回転列(2.5, 900, 0.7);
        let a = 列変換(Z変param::default(), &列);
        let b = 列変換(Z変param::default(), &列);
        assert_eq!(a, b);
    }

    #[test]
    fn 巻戻しで状態初期化() {
        let 列 = 回転列(1.0, 360, 0.9);
        let mut 変 = Z変換器::既定();
        for &(x, y) in &列 {
            変.変換(x, y);
        }
        assert_eq!(変.巻(), 1);
        変.巻戻し();
        assert_eq!(変.巻(), 0);
    }

    #[test]
    fn 直交往復() {
        let mut 変 = Z変換器::新(Z変param {
            死域再正規化: false,
            ..Default::default()
        });
        let z = 変.変換(0.6, -0.3);
        let (x, y) = z.直交();
        assert!(近(x, 0.6, 1e-12) && 近(y, -0.3, 1e-12), "({x},{y})");
    }

    #[test]
    fn r上限を超えない() {
        let mut 変 = Z変換器::既定();
        let z = 変.変換(1.0, 1.0); // 正方形域の角 → 生径√2
        assert!(z.r <= LOVE + 1e-12, "r={}", z.r);
    }

    #[test]
    fn 非数入力は無へ落ちる() {
        // 敵対審査 a14 (極側の既知欠陥) のz側確認: NaN/Inf で偽の方位を出さぬ事.
        let mut 変 = Z変換器::既定();
        変.変換(0.9, 0.0);
        for (x, y) in [
            (f64::NAN, 0.0),
            (0.0, f64::NAN),
            (f64::INFINITY, 0.0),
            (f64::NEG_INFINITY, f64::NAN),
        ] {
            let z = 変.変換(x, y);
            assert!(z.theta.is_finite(), "theta汚染 ({x},{y}) → {z:?}");
            assert!(z.r.is_finite(), "r汚染 ({x},{y}) → {z:?}");
        }
    }

    #[test]
    fn 環正規化の端() {
        assert!(近(環正規化(半環), 半環, 1e-12));
        assert!(近(環正規化(-半環), 半環, 1e-12));
        assert!(近(環正規化(全環), 0.0, 1e-12));
        assert!(近(環正規化(3.0 * 半環), 半環, 1e-12));
    }
}
