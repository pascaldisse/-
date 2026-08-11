pub use wa::z::Z;

/// 構築補助 — 甲契約 (機関/環制御 src/z.rs) の Z に構築子が無い為の殿内補助trait.
/// 契約本体は不変 · 此は下流の組立糖衣のみ (場は追加せぬ).
pub trait Z構 {
    fn new(theta: f64, r: f64, lap: i64) -> Self;
}

impl Z構 for Z {
    fn new(theta: f64, r: f64, lap: i64) -> Z {
        Z { theta, r, lap }
    }
}
