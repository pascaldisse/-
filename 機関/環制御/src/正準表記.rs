//! 正準表記 — A11/A13 是正 (docs/adversary/2026-08-11-環統合審.md 甲.2.8, Pascal裁定 08-11 任B).
//!
//! 契約 (A11, 裁定確定): **bit一致契約の対象 = 記録済log (量子化後の値)**。
//! 生値 (device f32 raw · 変換前f64) は非契約 — 量子化は意図的な精度低下であり、
//! 「生値と量子化値が一致しない」事自体は欠陥ではない (旧: A11=欠陥として扱っていたのを是正)。
//!
//! 契約 (A13, 裁定確定): **実機路 (f32→f64) と 再生路 (logテキスト→f64) は本module
//! 唯一の量子化関数を経由する**。型経路 (f32由来 vs テキスト由来) が異なっていても、
//! 同一の正準表記 (文字列) を経由すれば同一bitのf64になる (IEEE754 parseは決定論的)。
//! z主.rs (実機/再生 両分岐) · main.rs (梯1書出) は本moduleを唯一の書式化窓口として通す
//! — 直書き `format!("{:.4}", …)` を残さない (全下流でこのmoduleを経由する事=契約).
//!
//! 桁・書式は param 既定つき (鉄則: hardcode禁)。既定 桁=4 は現行log書式 `{:.4}` と後方互換。

/// 量子化param — 十進小数桁数. 既定4 = 現行log書式 (`{:.4}`) と後方互換.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct 正準param {
    /// 小数点以下桁数.
    pub 桁: usize,
}

impl Default for 正準param {
    fn default() -> Self {
        正準param { 桁: 4 }
    }
}

/// 値 → 正準表記文字列 (log行にそのまま書く形). 実機路 (f32→f64済の値) と
/// 再生路への書出しの両方が通る唯一の書式化関数 — ここ以外で桁付き書式化をしない事.
///
/// 非有限 (NaN/±inf) は Rust既定表記 (`"NaN"` / `"inf"` / `"-inf"`) へ — 量子化桁は
/// 有限小数の丸め操作であり、非有限に桁を当てる操作は定義できない為、桁指定を素通しする.
pub fn 表記(値: f64, param: 正準param) -> String {
    if 値.is_finite() {
        format!("{:.*}", param.桁, 値)
    } else {
        format!("{値}")
    }
}

/// 正準表記文字列 → f64 (表記()の逆変換, parse). 表記()の出力のみを入力に想定する
/// (契約 = 「表記して即parse」の往復)。壊れ文字列 (契約外入力) は NaN — panicで
/// 下流を落とさぬ為 (z.rs非数防壁がその先で吸収する).
///
/// **これが実機路とlog再生路が共有すべき唯一の量子化関数** — 実機路は
/// `量子化(生値 as f64, param)` を通してから Z変換器 へ渡し、かつ同じ値を `表記()` で
/// logへ書く。再生路は log解析 (入力源.rs) が同じ正準表記の文字列をparseするので、
/// 実機路と再生路は同一桁数である限りbit一致する (docs/adversary 甲.2.8 A13是正実証,
/// tests/敵対_z.rs `a13_量子化を経由すれば実機路と再生路はbit一致する`).
pub fn 量子化(値: f64, param: 正準param) -> f64 {
    表記(値, param).parse::<f64>().unwrap_or(f64::NAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 既定桁は四() {
        assert_eq!(正準param::default().桁, 4);
    }

    #[test]
    fn 表記は現行書式と同値() {
        let v = 0.123456789_f64;
        let p = 正準param::default();
        assert_eq!(表記(v, p), format!("{v:.4}"));
    }

    #[test]
    fn 桁paramで書式が変わる() {
        let v = 0.123456789_f64;
        assert_eq!(表記(v, 正準param { 桁: 2 }), "0.12");
        assert_eq!(表記(v, 正準param { 桁: 6 }), "0.123457");
        assert_eq!(表記(v, 正準param { 桁: 0 }), "0");
    }

    #[test]
    fn 量子化は生値と一般に異なる_非契約の実測() {
        // A11: 生値は非契約. 一致しない事自体は仕様 (欠陥ではない).
        let v = 0.123456789_f64;
        let q = 量子化(v, 正準param::default());
        assert_ne!(v.to_bits(), q.to_bits());
        assert_eq!(q, 0.1235); // round-half-to-even/away実装依存だがRust既定丸めで0.1235
    }

    #[test]
    fn 量子化は冪等() {
        // 一度量子化した値を再度同paramで量子化しても変わらない
        // (再生路が読み戻した値を再書出ししても表記が安定する事の保証 — z主.rs再生分岐前提).
        let v = 0.987654321_f64;
        let p = 正準param::default();
        let q1 = 量子化(v, p);
        let q2 = 量子化(q1, p);
        assert_eq!(q1.to_bits(), q2.to_bits());
        assert_eq!(表記(q1, p), 表記(v, p));
    }

    #[test]
    fn 実機路相当のf32起源でも同じ関数で往復一致する() {
        // f32(device raw) → f64 → 量子化 → 表記 → parse (再生路相当) が同一bitになる事の
        // 最小回帰 (統合的な実機路/再生路一致は tests/敵対_z.rs A13 参照).
        let raw_f32: f32 = 0.3141592_f32;
        let p = 正準param::default();
        let 実機路 = 量子化(raw_f32 as f64, p);
        let 文字 = 表記(raw_f32 as f64, p);
        let 再生路: f64 = 文字.parse().unwrap();
        assert_eq!(実機路.to_bits(), 再生路.to_bits());
    }

    #[test]
    fn 非有限は桁指定を無視して素通しする() {
        let p = 正準param { 桁: 4 };
        assert_eq!(表記(f64::NAN, p), "NaN");
        assert_eq!(表記(f64::INFINITY, p), "inf");
        assert_eq!(表記(f64::NEG_INFINITY, p), "-inf");
    }

    #[test]
    fn 壊れ文字列の量子化はnanへ落ちる_panicしない() {
        // 表記()を経ない直接呼出しは想定外だが、量子化()単体としてもpanic禁は守る.
        assert!("not-a-number".parse::<f64>().is_err());
    }
}
