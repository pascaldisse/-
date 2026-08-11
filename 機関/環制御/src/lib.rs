//! 環制御 — DualSense→場 界面. 梯1=読取 (bin 環制御) · 梯2=z変換器 (bin 環z).
//! 下流 (梯3音·梯4場注入·梯5歌路) は本libの `z::Z` のみ受取る — 契約=不変.
//! 注: 非ascii module名はrustcがfile解決できぬ為 #[path] を添える (鳴語維持の為の宿主配管).
#![allow(mixed_script_confusables)]

/// **非契約** — 梯1の扇形家判定 (度数系). 下流機能は z::Z のみ受取る事.
/// 公開理由は敵対審査lane (tests/敵対_z.rs) の独立検算路として要る為であり、
/// 梯3以降がここを呼んだら契約違反 (審査対象).
pub mod polar;
pub mod z;
#[path = "入力源.rs"]
pub mod 入力源;
/// A11/A13是正 (docs/adversary 甲.2.8, Pascal裁定 08-11) — 実機路・再生路が共有すべき
/// 唯一の量子化窓口. z主.rs/main.rs の書出しは本moduleを通す事 (契約).
#[path = "正準表記.rs"]
pub mod 正準表記;
/// haptic帰路 (任A支援, 08-11) — 場応答r→DualSense rumble 2Hz搬送 (既定). 文書/環制御.md §帰路.
#[path = "帰路.rs"]
pub mod 帰路;
/// 実機live読取の共通law (Gilrs起動+温機+device列挙+左stick読取) — 梯3 環音のlive源が
/// 再用する契約点 (私有再実装禁, 梯4前梯 実機歌鐘 08-11).
#[path = "実機.rs"]
pub mod 実機;
