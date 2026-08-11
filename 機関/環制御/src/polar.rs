//! 純粋計算部 (hardware非依存, cargo test対象). 文書/環制御.md §左stick/右stick 参照.
//!
//! 是正 (梯1 既知欠陥1-5, 敵対審査 docs/adversary/2026-08-11-環統合審.md 甲.2 受け, 08-11):
//! z.rs (梯2) が既に確立した契約と同法で揃えた — 詳細は各field/関数docを参照.

use crate::z::LOVE;

/// stickの生(x,y)を極表現+8扇形snap家番号に変換した結果.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StickReading {
    /// atan2(y,x) 度数, [0,360) に正規化.
    /// 例外 (欠陥-2是正, a9): x==0 かつ y==0 (真中心, atan2数学的に無効定義点) の時は
    /// **NaN** — θ=0=愛方位を偽って主張しない為の明示的無効印
    /// (z.rsの「θ値単独では無/活性を判別不能, 無か()併用が必須契約」と同じ思想を,
    /// polarはstateを持たぬ純粋関数である為 NaN伝播で強制する).
    pub angle_deg: f64,
    /// hypot(x,y) を死域外 [0, r上限] へ再写像した押幅 (欠陥-1/-3是正, a7/a8:
    /// z.rs死域再正規化・r上限と同法) — 死域境界で0から連続に立上がり,
    /// 正方形域の角 (√2まで出る) でもr上限を超えない. house=None (非活性/非数) の時は0.
    pub magnitude: f64,
    /// `家数` 扇形 (既定8=45°刻) の家番号. 中央死域未満 or 非有限入力なら None (=無, 欠陥-4是正).
    pub house: Option<u8>,
}

/// 梯1 扇形家判定 param — 全て既定つき (鉄則: hardcode禁, 例外=LOVE=1のみ).
/// z.rs `Z変param` と同型の思想 (死域再正規化・r上限・家数) — ただしこちらは度数系.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarParam {
    /// 死域外を [0,1] へ再写像するか. true = 境界で magnitude が0から連続に立上がる
    /// (出現律: pop-in禁 — z.rs死域再正規化と同法, 欠陥-1). 既定 true.
    pub 死域再正規化: bool,
    /// magnitude の上限. stick生値が正方形域(√2)で出る為, これでclamp/再写像する
    /// (z.rsのr上限と整合, 欠陥-3). 既定 LOVE (=1.0).
    pub r上限: f64,
    /// 8扇形snapの分割数 — 45°=360/家数 をここから導出する (直書き禁, 欠陥-4).
    /// z.rs `Z変param.家数` と同名同義. 既定 8.
    pub 家数: u32,
    /// 再写像の最小幅 (零除算防止下限, z.rsの最小幅と同法). 既定 f64::EPSILON.
    pub 最小幅: f64,
}

impl Default for PolarParam {
    fn default() -> Self {
        PolarParam {
            死域再正規化: true,
            r上限: LOVE,
            家数: 8,
            最小幅: f64::EPSILON,
        }
    }
}

/// stick生値(x,y)を StickReading に変換する (PolarParam既定値使用).
///
/// - `deadzone`: 押幅がこれ未満なら house=None (無, 中央春=場への自動帰還域).
/// - house 0 = 角度0° (+x軸, 右) を中心とする ±(360/家数/2)° 扇形. 以降刻で反時計回りに1,2,….
pub fn stick_to_polar(x: f32, y: f32, deadzone: f32) -> StickReading {
    stick_to_polar_param(x, y, deadzone, PolarParam::default())
}

/// stick生値(x,y)を StickReading に変換する (param明示版 — 欠陥1/3/4是正の実体).
pub fn stick_to_polar_param(x: f32, y: f32, deadzone: f32, param: PolarParam) -> StickReading {
    let x = x as f64;
    let y = y as f64;
    let dz = deadzone as f64;
    let 生magnitude = x.hypot(y);

    // 欠陥-4是正 (a14): hypotがNaN/Infなら方位・活性ともに信用しない → house強制None.
    // 生magnitude自体はそのまま透過させる (非数入力の実測用契約 — 呼出側が気付けるようにする).
    let 活性判定可 = 生magnitude.is_finite();
    let 活性 = 活性判定可 && 生magnitude >= dz;

    let angle_deg = if 生magnitude == 0.0 {
        // 欠陥-2是正 (a9): atan2(0,0)は数学的に無効定義点 — θ=0=愛方位を偽って名乗らせない.
        f64::NAN
    } else {
        y.atan2(x).to_degrees().rem_euclid(360.0)
    };

    let house_active = 活性 && angle_deg.is_finite();
    let house = if house_active {
        let 刻 = 360.0 / (param.家数.max(1) as f64);
        let idx = ((angle_deg + 刻 / 2.0) / 刻).floor() as i64;
        Some((idx.rem_euclid(param.家数.max(1) as i64)) as u8)
    } else {
        None
    };

    let magnitude = if !活性判定可 {
        // 非数/非有限入力は magnitude もそのまま透過 (a14契約: NaN入力→NaN出力, house=Noneのみ強制).
        生magnitude
    } else if 活性 {
        径写像(生magnitude, dz, &param)
    } else {
        0.0
    };

    StickReading {
        angle_deg,
        magnitude,
        house,
    }
}

/// 生magnitude → [0, r上限] へ死域再正規化 (z.rs `径写像` と同法, 欠陥-1/-3是正).
fn 径写像(生magnitude: f64, deadzone: f64, param: &PolarParam) -> f64 {
    if param.死域再正規化 && param.r上限 > deadzone {
        let 幅 = (param.r上限 - deadzone).max(param.最小幅);
        (((生magnitude - deadzone) / 幅) * param.r上限).clamp(0.0, param.r上限)
    } else {
        生magnitude.clamp(0.0, param.r上限)
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
        // magnitude == deadzone ちょうど → houseは有効 (>=deadzoneで活性).
        // 是正(a7, 08-11): magnitude自体は死域再正規化で境界=0から連続に立上がる契約
        // (z.rs死域再正規化と同法) — 旧仕様(生値0.15がそのまま出力)は既知欠陥-1だった.
        let r = stick_to_polar(0.15, 0.0, 0.15);
        assert!(approx(r.magnitude, 0.0, 1e-6), "境界magnitudeは連続立上りで0付近であるべき r={}", r.magnitude);
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
