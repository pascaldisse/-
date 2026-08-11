//! 梯2 入力源 — {gilrs実機 | 入力log再生}. 実機不在のMac単体でも
//! 既定再生元が可読なら再生路へ落ちる (絶対断言ではない — file不可読なら失敗する).
//! 生値 (x, y) 列を産む口を一つに揃え、下流 (z変換器) は源を知らない.

use std::fs;
use std::path::Path;

/// 一標本 = 時刻(ms) + 左stick生値.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct 生標本 {
    pub ts: u128,
    pub x: f64,
    pub y: f64,
}

/// `key=` に続く数を取る (log行の自己記述形式を利用. 失敗=None).
fn 数取(行: &str, key: &str) -> Option<f64> {
    let i = 行.find(key)? + key.len();
    let 尾 = &行[i..];
    let 終 = 尾
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e'))
        .unwrap_or(尾.len());
    尾[..終].parse::<f64>().ok()
}

/// 梯1入力log (`TICK ts=… L(x=… y=…) …`) を生標本列へ.
/// 注釈行 (`#`) と DEVICE/EDGE 行は無視.
pub fn log解析(本文: &str) -> Vec<生標本> {
    本文.lines()
        .filter(|l| l.starts_with("TICK "))
        .filter_map(|l| {
            // ts欠損行は壊れ行として落とす (黙って0へ写像すると実時間再生が狂う).
            let ts = 数取(l, "ts=")? as u128;
            let 左 = l.find("L(")? + 2;
            let 右 = l[左..].find(')')? + 左;
            let 部 = &l[左..右];
            Some(生標本 {
                ts,
                x: 数取(部, "x=")?,
                y: 数取(部, "y=")?,
            })
        })
        .collect()
}

/// log fileから生標本列を読む.
pub fn log読込(path: &Path) -> std::io::Result<Vec<生標本>> {
    Ok(log解析(&fs::read_to_string(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const 見本: &str = "\
# 環制御 梯1 (読取) 起動 ts=1786442532613
# param deadzone=0.15 poll_hz=60
DEVICE id=GamepadId(0) name=\"PS5 Controller\" axes=6 buttons=17 connected=true
TICK ts=1786442533017 id=GamepadId(0) L(x=0.0000 y=0.0000 angle=0.00 mag=0.0000 house=無) R(x=0.0000 y=0.0000 angle=0.00 mag=0.0000 house=無) L2=0.0000 R2=0.0000
TICK ts=1786442533034 id=GamepadId(0) L(x=-0.5000 y=0.2500 angle=153.43 mag=0.5590 house=4) R(x=0.0000 y=0.0000 angle=0.00 mag=0.0000 house=無) L2=0.0000 R2=1.0000
EDGE ts=1786442533100 id=GamepadId(0) button=South state=down
";

    #[test]
    fn tick行のみ拾う() {
        let v = log解析(見本);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn 左stick値を正しく取る() {
        let v = log解析(見本);
        assert_eq!(v[1].x, -0.5);
        assert_eq!(v[1].y, 0.25);
        assert_eq!(v[1].ts, 1786442533034);
    }

    #[test]
    fn 右stickを混同しない() {
        let v = log解析(見本);
        assert_eq!(v[1].y, 0.25, "R(...)側を拾っていない事");
    }

    #[test]
    fn 空入力は空列() {
        assert!(log解析("").is_empty());
        assert!(log解析("# 注釈のみ\n").is_empty());
    }

    #[test]
    fn 壊れ行は落とすが後続は読む() {
        let 壊 = format!("TICK ts=1 id=X L(x= y=)\n{見本}");
        let v = log解析(&壊);
        assert_eq!(v.len(), 2);
    }
}
