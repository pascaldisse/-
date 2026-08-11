//! 環制御 — DualSense→場 界面. 梯1=読取 (bin 環制御) · 梯2=z変換器 (bin 環z).
//! 下流 (梯3音·梯4場注入·梯5歌路) は本libの `z::Z` のみ受取る — 契約=不変.
//! 注: 非ascii module名はrustcが file解決できぬ為 #[path] を添える (鳴語維持の為の配管).
#![allow(mixed_script_confusables)]

pub mod polar;
pub mod z;
#[path = "入力源.rs"]
pub mod 入力源;
