//! z源 — 梯1入力log→唯一界面Z. 実streamは梯4接続口。

use std::fs;
use std::io;
use std::path::Path;

use crate::契約::Z;

/// zの来処。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum 源 {
    実stream,
    log再生,
}

fn 欄(line: &str, 名: &str) -> Option<f64> {
    line.split_whitespace()
        .find_map(|word| word.strip_prefix(名))?
        .trim_end_matches(')')
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

/// 梯1 `TICK … L(… angle=… mag=… house=…)` をz列へ読む。
///
/// deadzone内は中心=無へ畳む。中心滞在を跨ぐ角飛びは巻に数えない。
pub fn log再生(path: &Path, deadzone: f64) -> io::Result<Vec<Z>> {
    let text = fs::read_to_string(path)?;
    Ok(文字列再生(&text, deadzone))
}

fn 文字列再生(text: &str, deadzone: f64) -> Vec<Z> {
    let deadzone = deadzone.max(0.0);
    let mut zs = Vec::new();
    let mut 前theta: Option<f64> = None;
    let mut lap = 0_i64;

    for line in text.lines() {
        let Some(左始) = line.find("L(") else {
            continue;
        };
        let Some(左終相対) = line[左始..].find(')') else {
            continue;
        };
        let 左 = &line[左始..左始 + 左終相対];
        let (Some(角度), Some(大きさ)) = (欄(左, "angle="), 欄(左, "mag=")) else {
            continue;
        };
        if 大きさ < 0.0 {
            continue;
        }

        if 大きさ < deadzone {
            zs.push(Z::無());
            前theta = None;
            continue;
        }

        let theta = {
            let theta = 角度.to_radians().rem_euclid(std::f64::consts::TAU);
            if theta > std::f64::consts::PI {
                theta - std::f64::consts::TAU
            } else {
                theta
            }
        }; 
        if let Some(前) = 前theta {
            let 差 = theta - 前;
            if 差 < -std::f64::consts::PI {
                lap += 1;
            } else if 差 > std::f64::consts::PI {
                lap -= 1;
            }
        }
        zs.push(Z { theta, r: 大きさ.clamp(0.0, 1.0), lap });
        前theta = Some(theta);
    }
    zs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(angle: f64, mag: f64) -> String {
        format!("TICK ts=1 L(x=0 y=0 angle={angle} mag={mag} house=0) R(x=0 y=0)")
    }

    #[test]
    fn proof入力logを読む() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/入力log.txt");
        assert!(!log再生(&path, 0.15).unwrap().is_empty());
    }

    #[test]
    fn deadzone内は無() {
        let zs = 文字列再生(&format!("{}\n{}", tick(0.0, 0.0), tick(90.0, 0.14)), 0.15);
        assert!(zs.iter().all(|z| *z == Z::無()));
    }

    #[test]
    fn 反時計回りwrapはlap増() {
        // 甲契約 (機関/環制御 z.rs): 巻は **±π横断** で増減 — 0°交差では数えぬ.
        let zs = 文字列再生(&format!("{}\n{}", tick(170.0, 1.0), tick(190.0, 1.0)), 0.15);
        assert_eq!(zs[1].lap, 1);
    }

    #[test]
    fn 時計回りwrapはlap減() {
        let zs = 文字列再生(&format!("{}\n{}", tick(190.0, 1.0), tick(170.0, 1.0)), 0.15);
        assert_eq!(zs[1].lap, -1);
    }

    #[test]
    fn 零度交差は巻に数えぬ() {
        let zs = 文字列再生(&format!("{}\n{}", tick(350.0, 1.0), tick(10.0, 1.0)), 0.15);
        assert_eq!(zs[1].lap, 0);
    }

    #[test]
    fn 壊れ行は飛ばす() {
        let zs = 文字列再生(
            &format!(
                "TICK L(angle=bad mag=1 house=0)\nTICK L(angle=0 mag=-1 house=0)\n{}",
                tick(90.0, 1.0)
            ),
            0.15,
        );
        assert_eq!(zs.len(), 1);
    }

    #[test]
    fn 角度はradへ変換() {
        let zs = 文字列再生(&tick(180.0, 1.0), 0.15);
        assert!((zs[0].theta - std::f64::consts::PI).abs() < 1e-12);
    }
}
