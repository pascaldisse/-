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
    ///
    /// D4是正 (敵対審査乙.3, 08-11): 3600角中589 (16%) が死域値ちょうどを三角経路で生成すると
    /// 1ulp下振れする (実測 — tests/敌対_丙_a6.rs). つまり「mag==死域は厳密等値で活性」を
    /// 厳密==として課す契約は確率的に到達不能で実効無意味 — 本契約は「境界許容幅内での
    /// 活性包含」 + 「死域再正規化によるr連続立上り」の両方で構成されると再定義する.
    /// 1e-12 は f64 hypot(trig) の実測誤差 (~10^-17オーダ) を十分吸収する幅.
    pub 境界許容: f64,
    /// |d| = π 厳密 (半回転) の巻 tie-break 規約。反時計/時計が原理上不分別
    /// (solas審査欠陷3): (0,+r)→(0,-r) は d=-π 厳密で, strict比較のみだと
    /// 巻不動→総角がπ後退する非対称に黙って倒れる。
    /// +1=反時計優先 (既定) · -1=時計優先 · 0=巻不動.
    pub 半回転規約: i8,
    /// 縫目 (±π) lap flicker抑制 (D1是正, 敵対審査乙.3, 08-11): 跨ぐ縫目での
    /// 微振動(ジッタ)は生θが縫目を毎ティック微小に跳び越える現象として現れ,
    /// 単純な |d|>半環 判定だけだと全環2πのワープ差として毎回確定してしまい,
    /// lapが毎ティック ±1 でフリッカーする (有界だが下流可聴/可視ノイズ). 判定は全環-|d|
    /// (このワープが含意する「真の短路進み量」) で行う — 着地点が縫目の(-π,π]半開区間規約上
    /// ちょうど折り返される入力でも(着地距離=0になっても)誤抜かない対称形. 全環-|d| が本値以上
    /// なら確定する (微小な進み量のジッタは抜かれ, ティック幅相当の進み量の真の回転は確定される).
    /// 既定 0.005 rad (≈死域・センサの典型ジッタ(1e-4〜1e-3rad幅)の10倍以上だが,
    /// 60Hz直近の実回転 (360標本/周, 1ティック≈ 0.01745 rad) よりは十分小さく —
    /// 真の徒歩速回転の交差検出を妨げない).
    pub 縫目ヒステリシス: f64,
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
            半回転規約: 1,
            縫目ヒステリシス: 0.005,
        }
    }
}

/// 角を (-π, π] へ環正規化.
/// 非有限値は 0 へ吸収 — rem_euclid は NaN/±∞ をそのまま通し契約 (-π,π] を破る
/// (solas審査欠陷1, 08-11).
pub fn 環正規化(θ: f64) -> f64 {
    if !θ.is_finite() {
        return 0.0;
    }
    let t = θ.rem_euclid(全環);
    if t > 半環 {
        t - 全環
    } else {
        t
    }
}

/// θ (弧度, 任意実数) → 家数 等分格子の家番号 (0..家数). 家数=0なら0.
///
/// B9是正 (敵対審査乙.4 B9, 08-11): 家番号算出の**契約層唯一実装** — polar.rs (機関/環制御,
/// 度数系) と 音高.rs (機関/環音, 弧度系) はここへ委譲する (私有再実装は削除済).
/// 規約 (D2是正済, 甲.2.7): 常に正の[0,2π)域で floor((θ+半刻)/刻) を取ってから 家数で
/// 環正規化する — 符号に依らず「境界ちょうどは常に上位家」を一貫させる (旧実装の
/// (-π,π]符号域`.round()`は正負でtie-break方向が反転する既知欠陥だった).
pub fn 家番号(θ: f64, 家数: u32) -> u32 {
    if 家数 == 0 {
        return 0;
    }
    let 刻 = 全環 / (家数 as f64);
    let θ正 = θ.rem_euclid(全環);
    let idx = ((θ正 + 刻 / 2.0) / 刻).floor() as i64;
    idx.rem_euclid(家数 as i64) as u32
}

/// θ を 家数 等分の格子へ snap し (-π, π] へ戻す. 家番号() の角度側表現 (契約層内部でのみ使用).
pub fn 家snap(θ: f64, 家数: u32) -> f64 {
    if 家数 == 0 {
        return 環正規化(θ);
    }
    let 刻 = 全環 / (家数 as f64);
    環正規化((家番号(θ, 家数) as f64) * 刻)
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
                theta: if self.param.死域中θ保持 {
                    self.保持θ
                } else {
                    0.0
                },
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
                // D1是正 (敵対審査乙.3, 08-11): 縫目を跨ぐ確定は, このワープが含意する
                // 「真の短路進み量」(全環-|d|) が縫目ヒステリシス以上あって初めて成立する.
                // (着地点の縫目からの距離だけでは判定しない — (-π,π]の半開区間規約上
                // 真のθ=-πが+πへ折り返される回転もあり, その場合着地距離は常に0になり
                // 真の回転を誤抜くバグがあった. 全環-|d| は着地側の符号に依存せず
                // 微小ジッタ(真の進み量≈ 0)と真の回転(進み量≈ティック幅)を対称に分ける).
                if 全環 - d.abs() >= self.param.縫目ヒステリシス {
                    self.lap += 1;
                }
            } else if d > 半環 {
                if 全環 - d.abs() >= self.param.縫目ヒステリシス {
                    self.lap -= 1;
                }
            } else if d.abs() == 半環 {
                // 半回転 = 方向不分別点. 規約で明示的に決める (黙って後退させない).
                // (厳密d=±半環は既に境界ちょうどの確定点 — D1ヒステリシスは適用しない, D2領域).
                match self.param.半回転規約.signum() {
                    1 if d < 0.0 => self.lap += 1,
                    -1 if d > 0.0 => self.lap -= 1,
                    _ => {}
                }
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
        // r上限 <= 死域 の逆転構成では再正規化の幅が最小幅へ丸まり瞬時飽和する
        // → 連続立上り契約 (出現律) が壊れる (solas審査欠陷4) → 生径clampへ退避.
        if self.param.死域再正規化 && 上 > self.param.死域 {
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
    fn 境界ちょうどは回転構成でも活性() {
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
            assert!(
                z.theta > -半環 - 1e-12 && z.theta <= 半環 + 1e-12,
                "θ={}",
                z.theta
            );
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
    fn 環正規化は非有限を吸収() {
        for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let t = 環正規化(v);
            assert!(t.is_finite() && t > -半環 && t <= 半環, "{v} → {t}");
        }
    }

    #[test]
    fn 半回転は規約で巻を決める() {
        // solas審査欠陷3 回帰: (0,+r)→(0,-r) は d=-π 厳密.
        let 上下 = |規: i8| {
            let mut 変 = Z変換器::新(Z変param {
                半回転規約: 規,
                ..Default::default()
            });
            変.変換(0.0, 0.9);
            変.変換(0.0, -0.9)
        };
        assert_eq!(上下(1).lap, 1, "反時計優先規約が効かぬ");
        assert_eq!(上下(0).lap, 0, "不動規約が効かぬ");
        assert!(上下(1).総角() > 0.0, "既定規約で総角が後退した");
    }

    #[test]
    fn r上限が死域以下の退化構成も境界を守る() {
        // solas審査欠陷4. 判定: r上限<=死域 は **退化構成** — 活性標本は必ず
        // 生径>=死域>=上限 で連続立上りは原理上不可能。求め得るのは
        // 「爆発せず境界を守り決定論的」まで — 再正規化を切って生clampへ退避する.
        let p = Z変param {
            r上限: 0.05,
            ..Default::default()
        };
        let mut 変 = Z変換器::新(p);
        for i in 0..100 {
            let z = 変.変換(0.08 + (i as f64) * 1e-9, 0.0);
            assert!(z.r.is_finite() && z.r <= p.r上限, "境界破り r={}", z.r);
        }
        // 死域未満は従前通り無.
        assert_eq!(変.変換(0.05, 0.0).r, 0.0);
    }

    #[test]
    fn 環正規化の端() {
        assert!(近(環正規化(半環), 半環, 1e-12));
        assert!(近(環正規化(-半環), 半環, 1e-12));
        assert!(近(環正規化(全環), 0.0, 1e-12));
        assert!(近(環正規化(3.0 * 半環), 半環, 1e-12));
    }
}
