//! 純粋計算部 (hardware非依存, cargo test対象). 文書/環制御.md §左stick/右stick 参照.

/// stickの生(x,y)を極表現+8扇形snap家番号に変換した結果.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StickReading {
    /// atan2(y,x) 度数, [0,360) に正規化.
    pub angle_deg: f64,
    /// hypot(x,y), 押幅.
    pub magnitude: f64,
    /// 8扇形 (45°刻) の家番号 0-7. 中央deadzone未満なら None (=無).
    pub house: Option<u8>,
}

/// stick生値(x,y)を StickReading に変換する.
///
/// - `deadzone`: 押幅がこれ未満なら house=None (無, 中央春=場への自動帰還域).
/// - house 0 = 角度0° (+x軸, 右) を中心とする ±22.5° 扇形. 以降45°刻で反時計回りに1,2,…7.
pub fn stick_to_polar(x: f32, y: f32, deadzone: f32) -> StickReading {
    let x = x as f64;
    let y = y as f64;
    let magnitude = x.hypot(y);
    let angle_deg = y.atan2(x).to_degrees().rem_euclid(360.0);
    let house = if magnitude < deadzone as f64 {
        None
    } else {
        let idx = ((angle_deg + 22.5) / 45.0).floor() as i64;
        Some((idx.rem_euclid(8)) as u8)
    };
    StickReading {
        angle_deg,
        magnitude,
        house,
    }
}

/// L2/R2等のtrigger圧を [0.0, 1.0] へ clamp する (param越境防御).
pub fn clamp_trigger(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn center_is_house_none() {
        let r = stick_to_polar(0.0, 0.0, 0.15);
        assert_eq!(r.magnitude, 0.0);
        assert_eq!(r.house, None);
    }

    #[test]
    fn below_deadzone_is_house_none() {
        let r = stick_to_polar(0.10, 0.0, 0.15);
        assert!(r.magnitude < 0.15);
        assert_eq!(r.house, None);
    }

    #[test]
    fn at_deadzone_boundary_is_active() {
        // magnitude == deadzone ちょうど → 有効 (>=deadzoneで活性)
        let r = stick_to_polar(0.15, 0.0, 0.15);
        assert!(approx(r.magnitude, 0.15, 1e-6));
        assert_eq!(r.house, Some(0));
    }

    #[test]
    fn right_is_house_0() {
        let r = stick_to_polar(1.0, 0.0, 0.15);
        assert!(approx(r.angle_deg, 0.0, 1e-6));
        assert_eq!(r.house, Some(0));
    }

    #[test]
    fn up_is_house_2() {
        let r = stick_to_polar(0.0, 1.0, 0.15);
        assert!(approx(r.angle_deg, 90.0, 1e-6));
        assert_eq!(r.house, Some(2));
    }

    #[test]
    fn left_is_house_4() {
        let r = stick_to_polar(-1.0, 0.0, 0.15);
        assert!(approx(r.angle_deg, 180.0, 1e-6));
        assert_eq!(r.house, Some(4));
    }

    #[test]
    fn down_is_house_6() {
        let r = stick_to_polar(0.0, -1.0, 0.15);
        assert!(approx(r.angle_deg, 270.0, 1e-6));
        assert_eq!(r.house, Some(6));
    }

    #[test]
    fn all_eight_houses_reachable() {
        let mut seen = [false; 8];
        for i in 0..8 {
            let theta = (i as f64) * 45.0_f64.to_radians();
            let r = stick_to_polar(theta.cos() as f32, theta.sin() as f32, 0.15);
            if let Some(h) = r.house {
                seen[h as usize] = true;
            }
        }
        assert!(seen.iter().all(|&s| s), "houses seen: {seen:?}");
    }

    #[test]
    fn sector_boundary_wraps_to_house_0() {
        // 22.4° は家0側, 22.6° は家1側 — 境界近傍の非対称確認.
        let r0 = stick_to_polar(
            22.4_f64.to_radians().cos() as f32,
            22.4_f64.to_radians().sin() as f32,
            0.15,
        );
        let r1 = stick_to_polar(
            22.6_f64.to_radians().cos() as f32,
            22.6_f64.to_radians().sin() as f32,
            0.15,
        );
        assert_eq!(r0.house, Some(0));
        assert_eq!(r1.house, Some(1));
    }

    #[test]
    fn negative_angle_normalizes_into_0_360() {
        let r = stick_to_polar(0.0, -1.0, 0.15);
        assert!(r.angle_deg >= 0.0 && r.angle_deg < 360.0);
    }

    #[test]
    fn clamp_trigger_bounds() {
        assert_eq!(clamp_trigger(-0.5), 0.0);
        assert_eq!(clamp_trigger(1.5), 1.0);
        assert_eq!(clamp_trigger(0.42), 0.42);
    }
}
