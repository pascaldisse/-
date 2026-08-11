pub use wa::z::Z;

/// 構築補助 — 甲契約Zは場のみを持つ。下流組立の糖衣。
pub trait Z構 {
    fn new(theta: f64, r: f64, lap: i64) -> Self;
}

impl Z構 for Z {
    fn new(theta: f64, r: f64, lap: i64) -> Self {
        Z { theta, r, lap }
    }
}
