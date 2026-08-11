//! 契約 (不変) — 建根甲 梯2 の z stream と同型. 全下流はzのみ受取る.
//! 文書/環統合.md §主座標: z = r·e^{iθ} + lap.

/// 螺旋上の一点. 全界面 (stick·音高·母音·律動·場座標) の唯一の界面型.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Z {
    /// 角 [rad] — 環位置. θ=0 → +1=愛 · θ=π → −1=対蹠.
    pub theta: f64,
    /// 径 — 大きさ. r=0 = 無 = 沈黙.
    pub r: f64,
    /// 巻 — 螺旋高度 (octave).
    pub lap: i64,
}

impl Z {
    pub fn new(theta: f64, r: f64, lap: i64) -> Self {
        Self { theta, r, lap }
    }
    /// 無 (中心) — r=0.
    pub fn 無() -> Self {
        Self { theta: 0.0, r: 0.0, lap: 0 }
    }
    /// θ を [0, 2π) へ正規化した値.
    pub fn theta正規(&self) -> f64 {
        self.theta.rem_euclid(std::f64::consts::TAU)
    }
}
