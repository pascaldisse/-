//! 敵対試験 — 梯1 polar.rs (stick_to_polar) 単体, 独立path検証.
//! 文書=docs/adversary/2026-08-11-環統合審.md §甲.1 攻撃目録A1-A15, §甲.2 既定済欠陥1-5.
//! 法: 既存 src/polar.rs #[cfg(test)] は一切書き換えない (別file). polar.rs実装も無修正.
//! 既知欠陥に対応するtestは「落ちて正しい」— 通す為に緩めていない.
//!
//! **二層構造 (批根丙 2026-08-11)**: 常時赤の門は門では無い (律: cargo test緑必須 —
//! 赤が常態化すれば新規退行が既知赤に紛れて見えなくなる). 故に契約形 (理想) は
//! `#[ignore]` で保存し (`cargo test -- --ignored` で目視可), 同じ欠陥を **現値pinの証跡test**
//! として常時緑でも保持する. 建根甲が polar を是正した瞬間に証跡testが赤化して知らせる
//! → 欠陥の発生も消失も両方検知される. 契約形の assert は一字も緩めていない.

use wa::polar::stick_to_polar;

fn 近(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

// ---------- A1 θ縁 (負ゼロ) ----------
#[test]
fn a1_負ゼロでも角度は単一値() {
    // 度数法[0,360)へのrem_euclidにより ±0 の区別自体が数学的に消える設計 — 実測確認.
    let r_pos0 = stick_to_polar(-1.0, 0.0, 0.15);
    let r_neg0 = stick_to_polar(-1.0, -0.0, 0.15);
    assert!(
        近(r_pos0.angle_deg, r_neg0.angle_deg, 1e-9),
        "負ゼロで角度分裂 pos0={} neg0={}",
        r_pos0.angle_deg,
        r_neg0.angle_deg
    );
    assert!(近(r_pos0.angle_deg, 180.0, 1e-9));
}

// ---------- A6 deadzone境界 (>=固定・片側のみ, 既存testの独立再確認) ----------
#[test]
fn a6_死域境界はgte片側() {
    let 境界 = stick_to_polar(0.15, 0.0, 0.15);
    assert!(境界.house.is_some(), "境界ちょうどは活性であるべき (>=)");
    let 直下 = stick_to_polar(0.15 - 1e-7, 0.0, 0.15);
    assert!(直下.house.is_none(), "境界直下は無であるべき");
}

// ---------- A7 deadzone不連続 (既知欠陥-1 — 落ちて正しい) ----------
#[test]
#[ignore = "既知欠陥-1 (polar=非契約層) — 是正まで赤. 証跡=a7丙"]
fn a7_死域境界でrが不連続に跳躍する_既知欠陥() {
    // 理想契約 (環統合.md出現律): 死域境界直後の r は 0 から連続に立上がるべき.
    // polar.rs は magnitude を無加工返却 → 境界で dz が丸ごと出現 (pop-in).
    let 境界直後 = stick_to_polar(0.1501, 0.0, 0.15);
    assert!(
        境界直後.magnitude < 0.01,
        "既知欠陥-1実測: 境界直後magnitude={} (理想=~0付近であるべきが dz=0.15 がそのまま跳躍出現)",
        境界直後.magnitude
    );
}

// ---------- A8 r>1 (既知欠陥-3 — 落ちて正しい) ----------
#[test]
#[ignore = "既知欠陥-3 — 是正まで赤. 証跡=a8丙"]
fn a8_正方形域の角でr上限を超える_既知欠陥() {
    // 理想契約: r (=magnitude) は amp として下流で使われる為 [0,1] に収まるべき.
    let 角 = stick_to_polar(1.0, 1.0, 0.15);
    assert!(
        角.magnitude <= 1.0,
        "既知欠陥-3実測: 正方形域の角でmagnitude={} (>1, 上限clampが無い)",
        角.magnitude
    );
}

// ---------- A9 r=0=無中心 (既知欠陥-2 — 落ちて正しい) ----------
#[test]
#[ignore = "既知欠陥-2 — 是正まで赤. 証跡=a9丙"]
fn a9_中心でatan2が偽の愛方位を出力する_既知欠陥() {
    let 中心 = stick_to_polar(0.0, 0.0, 0.15);
    assert_eq!(中心.house, None, "house自体は正しく無を示す");
    // 理想契約: r=0時のθは無効印であるべき (環統合.md A9: NaN/Option等, 0.0=有効値は禁).
    assert_ne!(
        中心.angle_deg, 0.0,
        "既知欠陥-2実測: 中心でangle_deg={} — atan2(0,0)=0が θ=0=愛方位 を偽出力 (無効印になっていない)",
        中心.angle_deg
    );
}

// ---------- A12 param網羅 (既知欠陥-4 — 落ちて正しい) ----------
#[test]
#[ignore = "既知欠陥-4 — 是正まで赤. 証跡=a12丙"]
fn a12_家数snap分割が直書きでparam化されていない_既知欠陥() {
    // 理想契約 (環統合.md鉄則): 全定数=param既定つき (例外=LOVE=1のみ).
    // stick_to_polar のsignatureは (x, y, deadzone) のみ — 45°刻·8家がsource直書き.
    let src = include_str!("../src/polar.rs");
    let 家数paramあり =
        src.contains("家数: u32") || src.contains("sectors: u32") || src.contains("houses: u32");
    assert!(
        家数paramあり,
        "既知欠陥-4実測: stick_to_polar に家数/sector数のparamが無い — 22.5/45.0/8がhardcode"
    );
}

// ---------- A14 非有限 (NaN) — polar.rsは検出せず house=Some(0)を偽出力 ----------
#[test]
#[ignore = "既知欠陥 (NaN→house0) — 是正まで赤. 証跡=a14丙"]
fn a14_nan入力がhouse0を偽出力する() {
    let r = stick_to_polar(f32::NAN, 0.0, 0.15);
    assert!(r.magnitude.is_nan(), "magnitude前提確認");
    // 理想契約: 非有限入力はhouse=None (無効印) であるべき·panic禁.
    assert_eq!(
        r.house, None,
        "実測: NaN入力でhouse={:?} (Noneが正しいが, NaNのi64飽和castによりSome(0)=愛方位を偽出力する既知欠陥)",
        r.house
    );
}

// ===== 証跡test (現値pin) — 常時緑. polarが是正された瞬間に赤化して知らせる =====

#[test]
fn a7丙_証跡_境界直後のrは死域分丸々跳ぶ() {
    let z = stick_to_polar(0.1501, 0.0, 0.15);
    assert!(
        z.magnitude > 0.149,
        "欠陥-1が消えている (magnitude={}) → 契約形test a7 の #[ignore] を外せ",
        z.magnitude
    );
}

#[test]
fn a8丙_証跡_方形域の角でmagnitudeは二の平方根() {
    let z = stick_to_polar(1.0, 1.0, 0.15);
    assert!(
        近(z.magnitude, std::f64::consts::SQRT_2, 1e-6),
        "欠陥-3が変化 (magnitude={}) → a8 の #[ignore] を見直せ",
        z.magnitude
    );
}

#[test]
fn a9丙_証跡_中心のangle_degは零を名乗る() {
    let z = stick_to_polar(0.0, 0.0, 0.15);
    assert_eq!(z.house, None);
    assert_eq!(z.angle_deg, 0.0, "欠陥-2が消えている → a9 の #[ignore] を外せ");
}

#[test]
fn a12丙_証跡_家数paramは未存在() {
    let src = include_str!("../src/polar.rs");
    assert!(
        !(src.contains("家数: u32") || src.contains("sectors: u32") || src.contains("houses: u32")),
        "欠陥-4が是正された → a12 の #[ignore] を外せ"
    );
}

#[test]
fn a14丙_証跡_NaNはhouse0へ飽和する() {
    let z = stick_to_polar(f32::NAN, 0.0, 0.15);
    assert!(z.magnitude.is_nan());
    assert_eq!(z.house, Some(0), "NaN振舞いが変わった → a14 の #[ignore] を見直せ");
}

// ---------- A15 八家整合 (独立path一致 — polar.rs単体では自明だが再確認) ----------
#[test]
fn a15_house番号は45度刻みの中心角と対応する() {
    for i in 0u8..8 {
        let center_deg = (i as f64) * 45.0;
        let rad = center_deg.to_radians();
        let r = stick_to_polar(rad.cos() as f32, rad.sin() as f32, 0.15);
        assert_eq!(r.house, Some(i), "家{i}中心角{center_deg}度で不一致 house={:?}", r.house);
    }
}
