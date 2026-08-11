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
/// 実機live読取の共通law (Gilrs起動+温機+device列挙+左stick読取) — 梯3 環音のlive源が
/// 再用する契約点 (私有再実装禁, 梯4前梯 実機歌鐘 08-11).
#[path = "実機.rs"]
pub mod 実機;
